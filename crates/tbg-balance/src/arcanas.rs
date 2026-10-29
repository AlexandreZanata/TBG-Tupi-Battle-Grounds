//! Arcana experiments (D-40, P34/P40 method): named family builds and every
//! complete preset against the default, on paired seat-swapped seeds.

use tbg_core::arcana::{BUDGET, MAX, NODES};
use tbg_core::{ArcanaPreset, Ruleset};

use crate::policy::Policy;
use crate::runner::{run_pairs, Matchup, Setup};
use crate::stats::{self, Estimate};

/// The P34/P40 named fixtures (schema order: Guard, Assault, Piercing,
/// Endurance, Resilience, Renewal, Shieldwall, Battle Rush, Combined Arms,
/// Hunter's Focus, Quickdraw, Deadeye, Unshaken, Emergency Care, Purification).
pub const NAMED: [(&str, [u8; NODES]); 10] = [
    ("war-full", [2, 2, 2, 2, 0, 0, 1, 1, 1, 0, 0, 0, 0, 0, 0]),
    ("war-nosup", [2, 2, 2, 2, 0, 0, 1, 1, 0, 1, 1, 0, 0, 0, 0]),
    ("precision", [2, 2, 2, 2, 0, 0, 0, 0, 0, 1, 1, 1, 0, 0, 0]),
    ("vitality", [2, 2, 0, 0, 2, 2, 0, 0, 0, 0, 0, 0, 1, 1, 1]),
    ("hybrid", [2, 2, 2, 2, 2, 2, 0, 0, 0, 0, 0, 0, 0, 0, 0]),
    (
        "hybrid-children",
        [1, 1, 1, 1, 1, 1, 1, 1, 0, 1, 1, 0, 1, 1, 0],
    ),
    ("defensive", [2, 2, 0, 0, 2, 2, 1, 1, 0, 0, 0, 0, 1, 1, 0]),
    ("aggressive", [2, 2, 2, 2, 0, 0, 1, 1, 0, 1, 1, 0, 0, 0, 0]),
    ("control", [0, 0, 2, 2, 2, 2, 0, 0, 0, 1, 1, 0, 1, 1, 0]),
    (
        "ranged-supreme",
        [2, 2, 2, 2, 0, 0, 0, 0, 0, 1, 1, 1, 0, 0, 0],
    ),
];

/// The retired six-point default, kept as a diagnostic reference only.
pub const LEGACY_DEFAULT: [u8; NODES] = [2, 0, 2, 0, 2, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];

/// All complete presets, in mixed-radix rank order.
pub fn complete() -> Vec<ArcanaPreset> {
    let mut out = Vec::new();
    let radix: [u32; NODES] = std::array::from_fn(|i| u32::from(MAX[i]) + 1);
    let total = radix.iter().map(|&r| u64::from(r)).product::<u64>();
    for n in 0..total {
        let mut rest = n;
        let ranks: [u8; NODES] = std::array::from_fn(|i| {
            let rank = (rest % u64::from(radix[i])) as u8;
            rest /= u64::from(radix[i]);
            rank
        });
        if let Ok(p) = ArcanaPreset::complete(ranks) {
            out.push(p);
        }
    }
    debug_assert!(out.iter().all(|p| p.spent() == u32::from(BUDGET)));
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
