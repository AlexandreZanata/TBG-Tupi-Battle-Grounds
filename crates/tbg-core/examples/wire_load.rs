//! Native full-snapshot versus lossless-delta experiment. No live
//! protocol/transport changes; verify exact reconstruction on every view.

#[path = "performance/delta.rs"]
mod delta;
#[path = "performance/fixture.rs"]
mod fixture;

use std::hint::black_box;
use std::time::Instant;
use tbg_core::{protocol, MatchView, Seat};

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let get = |key: &str, fallback: &str| {
        args.windows(2)
            .find(|p| p[0] == key)
            .map_or(fallback.to_owned(), |p| p[1].clone())
    };
    let seed = get("--seed", "1").parse().unwrap();
    let ticks = get("--ticks", "1400").parse().unwrap();
    println!("scene,target,seed,frames,full_bytes,delta_bytes,full_encode_ns,delta_build_encode_ns,reconstruct_ns,full_resets");
    for scene in get("--scenes", "melee,healing,mixed,formation,cargo").split(',') {
        for target in get("--sizes", "20,200,1000").split(',') {
            run(scene, target.parse().unwrap(), seed, ticks);
        }
    }
}

fn run(scene: &str, target: usize, seed: u64, ticks: usize) {
    let mut fixture = fixture::Fixture::new(scene, target, seed);
    let mut previous: [Option<MatchView>; 2] = [None, None];
    let mut events = Vec::new();
    let (
        mut full_bytes,
        mut delta_bytes,
        mut full_ns,
        mut delta_ns,
        mut decode_ns,
        mut frames,
        mut resets,
    ) = (0, 0, 0, 0, 0, 0, 0);
    for step in 0..ticks {
        fixture.prepare();
        events.extend(fixture.state.step());
        if step % 2 == 0 {
            continue;
        }
        for seat in Seat::BOTH {
            let current = fixture.state.view_for(seat, &events);
            let full = protocol::ServerMsg::View(Box::new(current.clone()));
            let time = Instant::now();
            let bytes = black_box(protocol::encode(&full));
            let full_elapsed = time.elapsed().as_nanos();
            full_ns += full_elapsed;
            full_bytes += bytes.len();
            // Deliberate baseline loss/reconnect every 128 view frames;
            // this is an experimental recovery case, not a gameplay rule.
            if (step / 2) % 128 == 0 {
                previous[seat.index()] = None;
            }
            if let Some(old) = &previous[seat.index()] {
                let time = Instant::now();
                let packet = delta::build(step as u64 - 2, old, &current);
                let encoded = black_box(postcard::to_allocvec(&packet).unwrap());
                delta_ns += time.elapsed().as_nanos();
                delta_bytes += encoded.len() + 1;
                assert!(
                    delta::apply(u64::MAX, old, postcard::from_bytes(&encoded).unwrap()).is_none()
                );
                let time = Instant::now();
                let decoded = delta::apply(
                    step as u64 - 2,
                    old,
                    postcard::from_bytes(&encoded).unwrap(),
                )
                .unwrap();
                decode_ns += time.elapsed().as_nanos();
                assert_eq!(decoded, current, "lossy delta in {scene}");
            } else {
                delta_bytes += bytes.len();
                delta_ns += full_elapsed;
                resets += 1;
            }
            previous[seat.index()] = Some(current);
            frames += 1;
        }
        events.clear();
    }
    println!("{scene},{target},{seed},{frames},{full_bytes},{delta_bytes},{full_ns},{delta_ns},{decode_ns},{resets}");
}
