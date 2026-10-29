//! Arcana experiments (D-40, P34 method): every complete preset against
//! the default, on paired seat-swapped seeds, with cluster intervals.

use tbg_core::arcana::{BUDGET, MAX_RANK, NODES};
use tbg_core::{ArcanaPreset, Ruleset};

use crate::policy::Policy;
use crate::runner::{run_pairs, Matchup, Setup};
use crate::stats::{self, Estimate};

/// The P34 named fixtures (schema order Guard, Assault, Piercing,
/// Endurance, Resilience, Renewal).
pub const NAMED: [(&str, [u8; NODES]); 8] = [
    ("melee", [2, 2, 2, 0, 0, 0]),
    ("ranged", [2, 0, 2, 2, 0, 0]),
    ("defensive", [2, 0, 0, 2, 2, 0]),
    ("support", [2, 0, 0, 0, 2, 2]),
    ("control", [0, 0, 0, 2, 2, 2]),
    ("hybrid", [1, 1, 1, 1, 1, 1]),
    ("generalist", [2, 0, 2, 0, 2, 0]),
    ("extreme", [0, 2, 2, 0, 0, 2]),
];

/// All complete presets (141), in lexicographic rank order.
pub fn complete() -> Vec<ArcanaPreset> {
    let mut out = Vec::new();
    let base = u32::from(MAX_RANK) + 1;
    for n in 0..base.pow(NODES as u32) {
        let ranks: [u8; NODES] = std::array::from_fn(|i| (n / base.pow(i as u32) % base) as u8);
        if let Ok(p) = ArcanaPreset::complete(ranks) {
            out.push(p);
        }
    }
    debug_assert!(out.iter().all(|p| p.ranks.iter().sum::<u8>() == BUDGET));
    out
}

/// Score of each preset (participant 0) against `opponent`, same policy.
pub fn against(
    rules: &'static Ruleset,
    presets: &[ArcanaPreset],
    opponent: ArcanaPreset,
    policy: Policy,
    seeds: std::ops::Range<u64>,
    reaction: u32,
    threads: usize,
) -> Vec<(ArcanaPreset, Estimate)> {
    let ms: Vec<Matchup> = presets
        .iter()
        .map(|&p| Matchup {
            rules,
            policies: [policy; 2],
            reaction: [reaction; 2],
            setup: Setup::None,
            arcanas: [p, opponent],
        })
        .collect();
    run_pairs(&ms, seeds, threads)
        .into_iter()
        .zip(presets)
        .enumerate()
        .map(|(i, (pairs, &p))| {
            let s: Vec<f64> = pairs
                .iter()
                .map(|g| (g[0].score(0) + g[1].score(0)) / 2.0)
                .collect();
            (p, stats::mean(&s, 2_000, 500 + i as u64))
        })
        .collect()
}
