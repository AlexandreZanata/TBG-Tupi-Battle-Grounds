//! Large-army load (P16-B step 8):
//! `cargo run --release -p tbg-core --example army_load`.
//! Each seat is topped up to N living conventional units every tick (the
//! roster in turn, Nesters included, so the Beetling limit is reached), and
//! the armies fight continuously. Prints step time mean/p95/p99/max, the
//! encoded view size of a busy tick, and the resulting bandwidth per player.

use std::time::Instant;

use tbg_core::protocol::{encode, ServerMsg};
use tbg_core::*;

/// Views per second the server sends to each player (ARCHITECTURE §5).
const VIEWS_PER_SECOND: f64 = 10.0;
const TICKS: u32 = 1_200;

fn main() {
    println!("| units/side | live units (avg) | step mean µs | p95 | p99 | max | view bytes avg / max | KB/s per player |");
    println!("|---|---|---|---|---|---|---|---|");
    for n in [10, 20, 40, 60, 100] {
        run(n);
    }
}

fn top_up(s: &mut MatchState, seat: Seat, n: usize, next: &mut usize) {
    while s.army_size(seat) < n {
        let kind = UNIT_KINDS[*next % UNIT_KINDS.len()];
        *next += 1;
        s.players[seat.index()].offers[0] = kind;
        s.players[seat.index()].energy = 6_000;
        if s.apply_intention(seat, 0, PlayerIntention::Purchase { offer: 0 })
            .is_err()
        {
            return;
        }
    }
}

fn run(n: usize) {
    let mut s = MatchState::new(n as u64);
    while s.phase != Phase::Running {
        s.step();
    }
    // Strongholds out of reach: the load test must not end the match.
    let mut steps = Vec::with_capacity(TICKS as usize);
    let (mut next, mut live, mut bytes, mut max_bytes, mut views) = ([0, 0], 0, 0, 0, 0);
    let mut previous = Vec::new();
    for tick in 0..TICKS {
        for seat in Seat::BOTH {
            top_up(&mut s, seat, n, &mut next[seat.index()]);
            s.players[seat.index()].stronghold_hp = content::STRONGHOLD_HP;
        }
        let start = Instant::now();
        let events = s.step();
        steps.push(start.elapsed().as_nanos() as f64 / 1_000.0);
        live += s.units.len();
        if tick % 2 == 1 {
            let mut both = std::mem::take(&mut previous);
            both.extend(events);
            let size = encode(&ServerMsg::View(Box::new(s.view_for(Seat::A, &both)))).len();
            bytes += size;
            max_bytes = max_bytes.max(size);
            views += 1;
        } else {
            previous = events;
        }
    }
    steps.sort_by(f64::total_cmp);
    let at = |q: f64| steps[((steps.len() - 1) as f64 * q) as usize];
    let mean = steps.iter().sum::<f64>() / steps.len() as f64;
    let avg_bytes = bytes as f64 / f64::from(views);
    println!(
        "| {n} | {} | {mean:.1} | {:.1} | {:.1} | {:.1} | {avg_bytes:.0} / {max_bytes} | {:.1} |",
        live / TICKS as usize,
        at(0.95),
        at(0.99),
        at(1.0),
        avg_bytes * VIEWS_PER_SECOND / 1_000.0
    );
}
