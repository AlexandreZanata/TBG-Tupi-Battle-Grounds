//! Read-only layout/index microexperiment. A sorted snapshot is NOT a
//! valid replacement for sequential live targeting; includes build cost.

#[path = "performance/fixture.rs"]
mod fixture;

use std::{hint::black_box, time::Instant};
use tbg_core::Unit;

#[derive(Clone, Copy)]
struct Hot {
    x: i32,
    id: u32,
    flags: u8,
}

fn record(unit: &Unit, flags: u8) -> Hot {
    Hot {
        x: unit.x,
        id: unit.id,
        flags,
    }
}

fn scan(query: Hot, rows: impl Iterator<Item = Hot>) -> Option<(i32, u32)> {
    rows.filter(|e| e.flags & 1 != query.flags & 1 && e.flags & 2 != 0)
        .map(|e| ((e.x - query.x).abs(), e.id))
        .min()
}

fn ordered(query: Hot, rows: &[Hot]) -> Option<(i32, u32)> {
    let at = rows.partition_point(|e| e.x < query.x);
    let right = rows.get(at).map(|e| ((e.x - query.x).abs(), e.id));
    let left = at.checked_sub(1).map(|index| {
        let x = rows[index].x;
        let first = rows.partition_point(|e| e.x < x);
        ((x - query.x).abs(), rows[first].id)
    });
    match (left, right) {
        (Some(a), Some(b)) => Some(a.min(b)),
        (a, b) => a.or(b),
    }
}

fn timing(queries: &[Hot], f: impl Fn(Hot) -> Option<(i32, u32)>) -> u128 {
    let time = Instant::now();
    for _ in 0..100 {
        for query in queries {
            black_box(f(black_box(*query)));
        }
    }
    time.elapsed().as_nanos() / (100 * queries.len()) as u128
}

fn run(n: usize) {
    let f = fixture::Fixture::new("mixed", n, 1);
    let units = &f.state.units;
    let flags: Vec<_> = units
        .iter()
        .map(|u| {
            u.seat.index() as u8
                | if u.hp > 0 && u.fx.carrier.is_none() {
                    2
                } else {
                    0
                }
        })
        .collect();
    let time = Instant::now();
    let hot: Vec<_> = units
        .iter()
        .zip(&flags)
        .map(|(u, &b)| record(u, b))
        .collect();
    let compact_build = time.elapsed().as_nanos();
    let time = Instant::now();
    let x: Vec<_> = hot.iter().map(|u| u.x).collect();
    let ids: Vec<_> = hot.iter().map(|u| u.id).collect();
    let soa_build = compact_build + time.elapsed().as_nanos();
    let time = Instant::now();
    let mut sorted = [Vec::new(), Vec::new()];
    for u in hot.iter().filter(|u| u.flags & 2 != 0) {
        sorted[usize::from(u.flags & 1)].push(*u);
    }
    for side in &mut sorted {
        side.sort_unstable_by_key(|u| (u.x, u.id));
    }
    let sorted_build = compact_build + time.elapsed().as_nanos();
    let aos = |q| scan(q, units.iter().zip(&flags).map(|(u, &b)| record(u, b)));
    let compact = |q| scan(q, hot.iter().copied());
    let soa = |q| {
        scan(
            q,
            (0..ids.len()).map(|i| Hot {
                x: x[i],
                id: ids[i],
                flags: flags[i],
            }),
        )
    };
    let indexed = |q: Hot| ordered(q, &sorted[usize::from((q.flags & 1) ^ 1)]);
    for q in &hot {
        assert_eq!(aos(*q), compact(*q));
        assert_eq!(aos(*q), soa(*q));
        assert_eq!(aos(*q), indexed(*q));
    }
    // Guard/Stronghold/healing are deliberately outside this nearest-enemy
    // kernel. Report only layout/lookup costs; never claim full tick gains.
    println!(
        "{n},{},{},{},{},{compact_build},{soa_build},{sorted_build}",
        timing(&hot, aos),
        timing(&hot, compact),
        timing(&hot, soa),
        timing(&hot, indexed)
    );
}

fn main() {
    println!("units,aos_ns_query,compact_ns_query,soa_ns_query,sorted_ns_query,compact_build_ns,soa_build_ns,sorted_build_ns");
    for n in [10, 20, 50, 100, 200, 300, 500, 1000] {
        run(n);
    }
}
