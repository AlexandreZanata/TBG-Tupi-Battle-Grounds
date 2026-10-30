//! Tick, view, and encoding measurements, plus exact binary traces of all
//! serialized state and public events/views for differential experiments.

use std::hint::black_box;
use std::io::Write;
use std::time::Instant;

use tbg_core::protocol::{encode, ServerMsg};
use tbg_core::Seat;

use super::fixture::Fixture;

#[derive(Default, Clone, Copy)]
pub struct Counts {
    pub calls: u64,
    pub bytes: u64,
    pub peak: u64,
    pub operations: [u64; 4],
}

fn start() {
    #[cfg(feature = "bench-profile")]
    tbg_core::profiling::reset();
    #[cfg(feature = "bench-alloc")]
    super::allocations::start();
}

fn stop() -> Counts {
    #[cfg(feature = "bench-alloc")]
    {
        super::allocations::stop()
    }
    #[cfg(not(feature = "bench-alloc"))]
    {
        Counts {
            #[cfg(feature = "bench-profile")]
            operations: tbg_core::profiling::snapshot(),
            ..Counts::default()
        }
    }
}

fn time<T>(f: impl FnOnce() -> T) -> (T, u128, Counts) {
    start();
    let t = Instant::now();
    let value = black_box(f());
    let ns = t.elapsed().as_nanos();
    let counts = stop();
    (value, ns, counts)
}

pub fn run(f: &mut Fixture, ticks: usize, seed: u64, out: &mut impl Write) {
    let mut step = Vec::with_capacity(ticks);
    let (mut view_ns, mut enc_ns, mut bytes, mut live) = (0, 0, 0, 0);
    let (mut step_alloc, mut step_bytes, mut view_alloc, mut enc_alloc, mut peak) = (0, 0, 0, 0, 0);
    let mut previous = Vec::new();
    let mut operations = [0; 4];
    for tick in 0..ticks {
        f.prepare();
        let (events, ns, a) = time(|| f.state.step());
        step.push(ns);
        step_alloc += a.calls;
        step_bytes += a.bytes;
        for (total, value) in operations.iter_mut().zip(a.operations) {
            *total += value;
        }
        peak = peak.max(a.peak);
        live += f.state.units.len();
        previous.extend(events);
        if tick % 2 == 1 {
            let (view, ns, a) = time(|| f.state.view_for(Seat::A, &previous));
            view_ns += ns;
            view_alloc += a.calls;
            let message = ServerMsg::View(Box::new(view));
            let (encoded, ns, a) = time(|| encode(&message));
            enc_ns += ns;
            enc_alloc += a.calls;
            bytes += encoded.len();
            previous.clear();
        }
    }
    step.sort_unstable();
    let avg = step.iter().sum::<u128>() / ticks as u128;
    let at = |percent: usize| step[(step.len() - 1) * percent / 100];
    let views = (ticks / 2).max(1) as u128;
    let ops = operations.map(|value| value as f64 / ticks as f64);
    writeln!(
        out,
        "{},{},{seed},{ticks},{:.2},{avg},{},{},{},{},{},{},{},{:.3},{:.3},{:.3},{:.3},{peak},{:.2},{:.2},{:.2},{:.2}",
        f.scene,
        f.target,
        live as f64 / ticks as f64,
        at(50),
        at(95),
        at(99),
        at(100),
        view_ns / views,
        enc_ns / views,
        bytes as u128 / views,
        step_alloc as f64 / ticks as f64,
        step_bytes as f64 / ticks as f64,
        view_alloc as f64 / views as f64,
        enc_alloc as f64 / views as f64,
        ops[0], ops[1], ops[2], ops[3]
    )
    .unwrap();
}

pub fn trace(f: &mut Fixture, ticks: usize, out: &mut impl Write) {
    for _ in 0..ticks {
        f.prepare();
        let events = f.state.step();
        // Include derived conditional-Arcana counters and observable stats:
        // serde intentionally skips runtime-only fields in MatchState.
        let stats: Vec<_> = f
            .state
            .units
            .iter()
            .map(|u| format!("{:?}", f.state.stats_of(u)))
            .collect();
        let payload = postcard::to_allocvec(&(
            &f.state,
            &events,
            stats,
            f.state.view_for(Seat::A, &events),
            f.state.view_for(Seat::B, &events),
            [Seat::A, Seat::B].map(|s| (f.state.army_of(s).tanks, f.state.army_of(s).dps)),
        ))
        .unwrap();
        out.write_all(&(payload.len() as u64).to_le_bytes())
            .unwrap();
        out.write_all(&payload).unwrap();
    }
}

pub fn commands(f: &mut Fixture, seed: u64, out: &mut impl Write) {
    let (mut total, mut max, mut calls, mut bytes) = (0, 0, 0, 0);
    for sequence in 1..=40 {
        for seat in Seat::BOTH {
            let (result, ns, counts) = time(|| {
                f.state.apply_intention(
                    seat,
                    sequence,
                    tbg_core::PlayerIntention::Purchase { offer: 0 },
                )
            });
            result.unwrap();
            total += ns;
            max = max.max(ns);
            calls += counts.calls;
            bytes += counts.bytes;
        }
    }
    writeln!(
        out,
        "{},{},{seed},80,{total},{max},{calls},{bytes}",
        f.scene, f.target
    )
    .unwrap();
}
