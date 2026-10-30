//! Stronghold kind matrix (D-46): every kind against every kind and the
//! Classic reference, both seats per seed pair, one policy for both sides.

use tbg_core::{ArcanaPreset, Ruleset, StrongholdKind};

use crate::policy::Policy;
use crate::runner::{run_pairs, Matchup, Setup};
use crate::stats::{self, Estimate};

/// The rows and columns: the four player kinds, then Classic.
pub const KINDS: [StrongholdKind; 5] = [
    StrongholdKind::Treasury,
    StrongholdKind::Bastion,
    StrongholdKind::Artillery,
    StrongholdKind::Command,
    StrongholdKind::Classic,
];

/// Score of the row kind (participant 0) against each column kind.
pub fn matrix(
    rules: &'static Ruleset,
    policy: Policy,
    seeds: std::ops::Range<u64>,
    reaction: u32,
    threads: usize,
) -> Vec<(StrongholdKind, StrongholdKind, Estimate)> {
    let cells: Vec<(StrongholdKind, StrongholdKind)> = KINDS
        .iter()
        .flat_map(|&row| KINDS.iter().map(move |&col| (row, col)))
        .collect();
    let ms: Vec<Matchup> = cells
        .iter()
        .map(|&(row, col)| Matchup {
            rules,
            policies: [policy; 2],
            reaction: [reaction; 2],
            setup: Setup::Strongholds([row, col]),
            arcanas: [ArcanaPreset::NEUTRAL; 2],
        })
        .collect();
    run_pairs(&ms, seeds, threads)
        .into_iter()
        .zip(cells)
        .enumerate()
        .map(|(i, (pairs, (row, col)))| {
            let s: Vec<f64> = pairs
                .iter()
                .map(|g| (g[0].score(0) + g[1].score(0)) / 2.0)
                .collect();
            (row, col, stats::mean(&s, 2_000, 900 + i as u64))
        })
        .collect()
}
