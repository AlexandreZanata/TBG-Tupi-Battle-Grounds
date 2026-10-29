//! Paired counterfactuals, the Energy shadow price, and PVE. The canonical
//! scalar is match-based: micro-scenario metrics are features and diagnostic
//! dimensions, never silently summed into PVE.

use std::ops::Range;

use tbg_core::content::ENERGY_SCALE;
use tbg_core::{ArcanaPreset, Ruleset, Seat};

use crate::metrics::MatchResult;
use crate::policy::Policy;
use crate::runner::{run_pairs, Matchup, Setup};
use crate::stats::{self, Estimate};

use super::intervention::Intervention;

#[derive(Clone, Debug)]
pub struct Compare {
    pub base: Vec<[MatchResult; 2]>,
    pub variant: Vec<[MatchResult; 2]>,
    /// Variant minus baseline for participant 0, paired by seed.
    pub delta: Estimate,
    pub base_score: f64,
    pub variant_score: f64,
    pub seed_pairs: usize,
}

pub fn compare(
    intervention: Intervention,
    policies: [Policy; 2],
    reaction: u32,
    seeds: Range<u64>,
    threads: usize,
) -> Result<Compare, String> {
    compare_with(
        Ruleset::canon(),
        intervention,
        policies,
        reaction,
        seeds,
        threads,
    )
}

pub fn compare_with(
    base_rules: &'static Ruleset,
    intervention: Intervention,
    policies: [Policy; 2],
    reaction: u32,
    seeds: Range<u64>,
    threads: usize,
) -> Result<Compare, String> {
    let variant_rules = intervention.rules_from(base_rules)?.unwrap_or(base_rules);
    let base = Matchup {
        rules: base_rules,
        policies,
        reaction: [reaction; 2],
        setup: Setup::None,
        arcanas: [ArcanaPreset::NEUTRAL; 2],
    };
    let variant = Matchup {
        rules: variant_rules,
        policies,
        reaction: [reaction; 2],
        setup: intervention.setup(),
        arcanas: [ArcanaPreset::NEUTRAL; 2],
    };
    let seed_pairs = (seeds.end - seeds.start) as usize;
    let b = run_pairs(&[base], seeds.clone(), threads)
        .into_iter()
        .next()
        .ok_or("baseline produced no row")?;
    let v = run_pairs(&[variant], seeds, threads)
        .into_iter()
        .next()
        .ok_or("variant produced no row")?;
    let delta = paired_delta(&b, &v, 0);
    Ok(Compare {
        base_score: mean_score(&b, 0),
        variant_score: mean_score(&v, 0),
        base: b,
        variant: v,
        delta,
        seed_pairs,
    })
}

pub fn paired_delta(
    base: &[[MatchResult; 2]],
    variant: &[[MatchResult; 2]],
    participant: usize,
) -> Estimate {
    let values: Vec<f64> = base
        .iter()
        .zip(variant)
        .map(|(b, v)| {
            (v[0].score(participant) + v[1].score(participant)
                - b[0].score(participant)
                - b[1].score(participant))
                / 2.0
        })
        .collect();
    stats::mean(&values, stats::RESAMPLES, 0x0051_ad0b)
}

pub fn mean_score(games: &[[MatchResult; 2]], participant: usize) -> f64 {
    let total: f64 = games
        .iter()
        .map(|p| p[0].score(participant) + p[1].score(participant))
        .sum();
    total / (2 * games.len().max(1)) as f64
}

#[derive(Clone, Copy, Debug)]
pub struct ShadowPoint {
    pub tick: u32,
    /// Score change for participant 0 per 1 Energy.
    pub lambda: f64,
    pub lo: f64,
    pub hi: f64,
}

/// Price one Energy at each tick by granting it to participant 0 only.
/// The policies are the shipped bot, so this is a lower bound on an expert
/// player's value; the report says so.
pub fn shadow(
    ticks: &[u32],
    reaction: u32,
    seeds: Range<u64>,
    threads: usize,
) -> Result<Vec<ShadowPoint>, String> {
    shadow_with(Ruleset::canon(), ticks, reaction, seeds, threads)
}

pub fn shadow_with(
    base: &'static Ruleset,
    ticks: &[u32],
    reaction: u32,
    seeds: Range<u64>,
    threads: usize,
) -> Result<Vec<ShadowPoint>, String> {
    let mut out = Vec::with_capacity(ticks.len());
    for &tick in ticks {
        let c = compare_with(
            base,
            Intervention::Energy {
                seat: 0,
                tick,
                sub_energy: ENERGY_SCALE,
            },
            [Policy::Bot, Policy::Bot],
            reaction,
            seeds.clone(),
            threads,
        )?;
        out.push(ShadowPoint {
            tick,
            lambda: c.delta.mean,
            lo: c.delta.lo,
            hi: c.delta.hi,
        });
    }
    Ok(out)
}

/// Median shadow price over the measured ticks. One Energy at this reference
/// is displayed as 100 PVE; that is a scale, not a universal exchange rate.
pub fn reference_lambda(points: &[ShadowPoint]) -> Result<f64, String> {
    let mut values: Vec<f64> = points.iter().map(|p| p.lambda).collect();
    values.sort_by(f64::total_cmp);
    let mid = values
        .get(values.len() / 2)
        .copied()
        .ok_or("no shadow points")?;
    if mid <= 0.0 {
        return Err(format!("reference Energy price is not positive: {mid}"));
    }
    Ok(mid)
}

#[derive(Clone, Copy, Debug)]
pub struct Pve {
    pub value: f64,
    pub lo: f64,
    pub hi: f64,
}

pub fn pve(delta: &Estimate, lambda_ref: f64) -> Pve {
    let scale = 100.0 / lambda_ref;
    Pve {
        value: delta.mean * scale,
        lo: delta.lo * scale,
        hi: delta.hi * scale,
    }
}

/// Spearman rank correlation with average ranks for ties.
pub fn spearman(a: &[f64], b: &[f64]) -> Option<f64> {
    if a.len() != b.len() || a.len() < 2 {
        return None;
    }
    let ra = ranks(a);
    let rb = ranks(b);
    let n = a.len() as f64;
    let mean = (n + 1.0) / 2.0;
    let mut num = 0.0;
    let mut da = 0.0;
    let mut db = 0.0;
    for (x, y) in ra.iter().zip(&rb) {
        num += (x - mean) * (y - mean);
        da += (x - mean).powi(2);
        db += (y - mean).powi(2);
    }
    (da > 0.0 && db > 0.0).then(|| num / (da.sqrt() * db.sqrt()))
}

fn ranks(values: &[f64]) -> Vec<f64> {
    let mut order: Vec<usize> = (0..values.len()).collect();
    order.sort_by(|&i, &j| values[i].total_cmp(&values[j]));
    let mut out = vec![0.0; values.len()];
    let mut i = 0;
    while i < order.len() {
        let mut j = i + 1;
        while j < order.len() && values[order[j]] == values[order[i]] {
            j += 1;
        }
        let rank = (i + 1 + j) as f64 / 2.0;
        for &idx in &order[i..j] {
            out[idx] = rank;
        }
        i = j;
    }
    out
}

/// A static, cost-aware baseline used to check whether full-match PVE adds
/// information: `HP * damage / interval / cost` (or the matching trade for a
/// support/utility unit). It is deliberately crude.
pub fn static_power(rules: &Ruleset, kind: tbg_core::UnitKind) -> f64 {
    let d = rules.def(kind);
    let cost = f64::from(d.cost.max(1));
    let hp = f64::from(d.hp.max(1));
    let damage = f64::from(d.damage.max(1));
    let interval = f64::from(d.interval.max(1));
    let heal = f64::from(d.heal.max(0));
    (hp * (damage + heal) / interval) / cost
}

/// Participant 0's final score with a seeded `Setup`, for tests and ad-hoc
/// checks that do not need the full `Compare`.
pub fn score_with_setup(
    setup: Setup,
    policies: [Policy; 2],
    reaction: u32,
    seeds: Range<u64>,
    threads: usize,
) -> f64 {
    let m = Matchup {
        rules: Ruleset::canon(),
        policies,
        reaction: [reaction; 2],
        setup,
        arcanas: [ArcanaPreset::NEUTRAL; 2],
    };
    let r = run_pairs(&[m], seeds, threads).remove(0);
    mean_score(&r, 0)
}

/// Convenience for a seat-independent score of a physical seat.
pub fn seat_score(game: &MatchResult, seat: Seat) -> f64 {
    game.score(crate::metrics::participant(seat, game.seat_a))
}
