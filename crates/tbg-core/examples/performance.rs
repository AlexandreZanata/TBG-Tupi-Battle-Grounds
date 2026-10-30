//! P30-D native experiment driver: synthetic fixtures, release timing,
//! allocation counts, and exact state/event/view traces. Never deployed.

#[cfg(feature = "bench-alloc")]
#[path = "performance/allocations.rs"]
mod allocations;
#[path = "performance/fixture.rs"]
mod fixture;
#[path = "performance/measure.rs"]
mod measure;

use std::io::{self, Write};
use std::mem::size_of;

use tbg_core::{state::UnitFx, Event, MatchState, MatchView, Unit, UnitView};

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let get = |key: &str, default: &str| {
        args.windows(2)
            .find(|p| p[0] == key)
            .map_or_else(|| default.to_owned(), |p| p[1].clone())
    };
    let ticks: usize = get("--ticks", "240").parse().unwrap();
    let seed: u64 = get("--seed", "1").parse().unwrap();
    let sizes = get("--sizes", "10,20,50,100,200,300,500,1000");
    let scenes = get(
        "--scenes",
        "melee,ranged,healing,mixed,formation,puppet,cargo",
    );
    let trace = args.iter().any(|a| a == "--trace");
    let commands = args.iter().any(|a| a == "--commands");
    let mut out = io::BufWriter::new(io::stdout().lock());
    if commands {
        writeln!(
            out,
            "scene,target,seed,commands,total_ns,max_ns,allocs,allocated_bytes"
        )
        .unwrap();
    } else if !trace {
        writeln!(out, "meta,unit_bytes,{},fx_bytes,{},unit_view_bytes,{},event_bytes,{},match_bytes,{},view_bytes,{},allocations,{}", size_of::<Unit>(), size_of::<UnitFx>(), size_of::<UnitView>(), size_of::<Event>(), size_of::<MatchState>(), size_of::<MatchView>(), cfg!(feature = "bench-alloc")).unwrap();
        writeln!(out, "scene,target,seed,ticks,live_mean,step_mean_ns,p50_ns,p95_ns,p99_ns,max_ns,view_mean_ns,encode_mean_ns,bytes_per_view,step_allocs,step_alloc_bytes,view_allocs,encode_allocs,peak_live_alloc_bytes,id_searches,stats_calls,target_candidates,formation_candidates").unwrap();
    }
    for scene in scenes.split(',') {
        for n in sizes.split(',').map(|n| n.parse::<usize>().unwrap()) {
            let mut fixture = fixture::Fixture::new(scene, n, seed);
            if commands {
                measure::commands(&mut fixture, seed, &mut out);
            } else if trace {
                measure::trace(&mut fixture, ticks, &mut out);
            } else {
                measure::run(&mut fixture, ticks, seed, &mut out);
            }
            out.flush().unwrap();
        }
    }
}
