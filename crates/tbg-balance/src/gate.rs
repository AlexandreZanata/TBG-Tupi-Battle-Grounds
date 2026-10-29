//! The balance gate (P16-C step 7, GAME-DESIGN §6 "Balance lab"): which
//! matchups to play and the rows they are judged by, with targets in one
//! table. Tuning a threshold is a product decision, never a test fix.

use std::collections::BTreeMap;
use std::ops::Range;

use tbg_core::content::TICK_HZ;
use tbg_core::ArcanaPreset;
use tbg_core::{EndReason, Ruleset, FACTIONS, UNIT_KINDS};

use crate::duels::Matrix;
use crate::meta;
use crate::metrics::MatchResult;
use crate::policy::Policy;
use crate::runner::{run_pairs, Matchup, Setup};
use crate::stats::{self, Estimate};

/// Default views between decisions for every non-`Bot` policy (500 ms).
pub const REACTION: u32 = 5;
pub const NASH_ITERATIONS: u32 = 200_000;
/// Band half-width around the median `Favor` score (relative unit strength).
const MONO_BAND: f64 = 0.10;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Scope {
    /// What the search optimizes: strategic round robin, spam and favor
    /// probes against `Bot`.
    Suite,
    /// The whole pool round robin (adds the meta-game rows).
    Full,
}

pub type Results = BTreeMap<(Policy, Policy), Vec<[MatchResult; 2]>>;

pub fn matchups(scope: Scope) -> Vec<(Policy, Policy)> {
    let list = match scope {
        Scope::Suite => Policy::strategic(),
        Scope::Full => Policy::pool(),
    };
    let mut v = vec![(Policy::Bot, Policy::Bot)];
    for (i, &a) in list.iter().enumerate() {
        v.extend(list[i + 1..].iter().map(|&b| (a, b)));
    }
    if scope == Scope::Suite {
        v.extend([Policy::Heavy, Policy::Cheap].map(|p| (p, Policy::Bot)));
        v.extend(UNIT_KINDS.map(|k| (Policy::Favor(k), Policy::Bot)));
    }
    v
}

pub fn run(
    rules: &'static Ruleset,
    scope: Scope,
    seeds: Range<u64>,
    reaction: u32,
    threads: usize,
) -> Results {
    let list = matchups(scope);
    let ms: Vec<Matchup> = list
        .iter()
        .map(|&(a, b)| Matchup {
            rules,
            policies: [a, b],
            reaction: [reaction; 2],
            setup: Setup::None,
            arcanas: [ArcanaPreset::NEUTRAL; 2],
        })
        .collect();
    list.into_iter()
        .zip(run_pairs(&ms, seeds, threads))
        .collect()
}

/// Per-pair scores of `a` against `b`, whichever way the matchup was stored.
pub fn scores(results: &Results, a: Policy, b: Policy) -> Option<Vec<f64>> {
    if let Some(pairs) = results.get(&(a, b)) {
        return Some(
            pairs
                .iter()
                .map(|p| (p[0].score(0) + p[1].score(0)) / 2.0)
                .collect(),
        );
    }
    let pairs = results.get(&(b, a))?;
    Some(
        pairs
            .iter()
            .map(|p| (p[0].score(1) + p[1].score(1)) / 2.0)
            .collect(),
    )
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Unit {
    Percent,
    Seconds,
    Count,
}

#[derive(Clone, Debug)]
pub struct Row {
    pub name: String,
    pub value: f64,
    pub interval: Option<(f64, f64)>,
    pub band: (f64, f64),
    pub unit: Unit,
    /// Part of the search loss (meta-game rows are gate-only).
    pub searched: bool,
}

impl Row {
    pub fn pass(&self) -> bool {
        self.value.is_finite() && self.value >= self.band.0 && self.value <= self.band.1
    }

    /// Distance outside the band in points: percentage points, seconds / 3,
    /// or 5 per flagged item.
    pub fn distance(&self) -> f64 {
        if !self.value.is_finite() {
            return 0.0;
        }
        let d = (self.band.0 - self.value)
            .max(self.value - self.band.1)
            .max(0.0);
        match self.unit {
            Unit::Percent => d * 100.0,
            Unit::Seconds => d / 3.0,
            Unit::Count => d * 5.0,
        }
    }
}

fn pct(name: impl Into<String>, e: Estimate, band: (f64, f64)) -> Row {
    Row {
        name: name.into(),
        value: e.mean,
        interval: Some((e.lo, e.hi)),
        band,
        unit: Unit::Percent,
        searched: true,
    }
}

fn count(name: &str, n: usize, max: usize, searched: bool) -> Row {
    Row {
        name: name.into(),
        value: n as f64,
        interval: None,
        band: (0.0, max as f64),
        unit: Unit::Count,
        searched,
    }
}

/// Games between two strategic policies, as seed-pair clusters.
fn strategic(results: &Results) -> Vec<&[MatchResult; 2]> {
    let s = Policy::strategic();
    results
        .iter()
        .filter(|((a, b), _)| s.contains(a) && s.contains(b))
        .flat_map(|(_, pairs)| pairs.iter())
        .collect()
}

fn cluster(pairs: &[&[MatchResult; 2]], f: impl Fn(&MatchResult) -> (f64, f64)) -> Vec<(f64, f64)> {
    pairs
        .iter()
        .map(|p| {
            let (a, b) = (f(&p[0]), f(&p[1]));
            (a.0 + b.0, a.1 + b.1)
        })
        .collect()
}

/// Every gate row; `resamples` 0 skips intervals (the search's fast path).
pub fn rows(results: &Results, duels: Option<(&Matrix, &Ruleset)>, resamples: u32) -> Vec<Row> {
    let mut rows = Vec::new();
    if let Some(pairs) = results.get(&(Policy::Bot, Policy::Bot)) {
        let a_wins: Vec<f64> = pairs
            .iter()
            .map(|p| (p[0].score(p[0].seat_a) + p[1].score(p[1].seat_a)) / 2.0)
            .collect();
        rows.push(pct(
            "seat A score, bot vs bot",
            stats::mean(&a_wins, resamples, 1),
            (0.485, 0.515),
        ));
    }
    rows.extend(fun_rows(results, resamples));
    rows.extend(dominant_style(results));
    for (p, stream) in [(Policy::Heavy, 30), (Policy::Cheap, 31)] {
        if let Some(s) = scores(results, p, Policy::Bot) {
            rows.push(pct(
                format!("{} vs bot", p.name()),
                stats::mean(&s, resamples, stream),
                (0.0, 0.55),
            ));
        }
    }
    let favor: Vec<(Policy, Vec<f64>)> = UNIT_KINDS
        .iter()
        .filter_map(|&k| {
            Some((
                Policy::Favor(k),
                scores(results, Policy::Favor(k), Policy::Bot)?,
            ))
        })
        .collect();
    let mut means: Vec<f64> = favor
        .iter()
        .map(|(_, s)| s.iter().sum::<f64>() / s.len().max(1) as f64)
        .collect();
    means.sort_by(f64::total_cmp);
    if let Some(&median) = means.get(means.len() / 2) {
        for (i, (p, s)) in favor.iter().enumerate() {
            let band = (median - MONO_BAND, median + MONO_BAND);
            rows.push(pct(
                format!("{} vs bot (median ±10 pp)", p.name()),
                stats::mean(s, resamples, 40 + i as u64),
                band,
            ));
        }
    }
    if let Some((m, rules)) = duels {
        let f = m.flags(rules);
        rows.push(count(
            "duel base forms weak everywhere",
            f.weak.len(),
            0,
            true,
        ));
        rows.push(count(
            "duel base forms oppressive everywhere",
            f.oppressive.len(),
            0,
            true,
        ));
        rows.push(count(
            "duel forms without a counter",
            f.counterless.len(),
            0,
            true,
        ));
        let evolved = m.forms.iter().filter(|f| f.tier > 0).count().max(1);
        rows.push(Row {
            name: "duel Evolutions never worth their Energy".into(),
            value: f.evolution_not_worth.len() as f64 / evolved as f64,
            interval: None,
            band: (0.0, 0.25),
            unit: Unit::Percent,
            searched: true,
        });
        rows.push(count(
            "duel Evolution branches without a niche",
            f.branch_without_niche.len(),
            0,
            true,
        ));
    }
    rows.extend(meta_rows(results));
    if resamples == 0 {
        // No bootstrap: do not print a zero-width interval.
        rows.iter_mut().for_each(|r| r.interval = None);
    }
    rows
}

/// The strongest strategic style's average against the other strategic
/// styles: one style winning most matchups is a solved game.
fn dominant_style(results: &Results) -> Option<Row> {
    let ps = Policy::strategic();
    let avg = |p: Policy| -> Option<f64> {
        let v: Vec<f64> = ps
            .iter()
            .filter(|&&q| q != p)
            .map(|&q| scores(results, p, q).map(|s| s.iter().sum::<f64>() / s.len().max(1) as f64))
            .collect::<Option<_>>()?;
        Some(v.iter().sum::<f64>() / v.len().max(1) as f64)
    };
    let all: Vec<(Policy, f64)> = ps
        .iter()
        .map(|&p| Some((p, avg(p)?)))
        .collect::<Option<_>>()?;
    let (p, best) = all.into_iter().max_by(|a, b| a.1.total_cmp(&b.1))?;
    Some(Row {
        name: format!("strongest strategic style ({})", p.name()),
        value: best,
        interval: None,
        band: (0.0, 0.60),
        unit: Unit::Percent,
        searched: true,
    })
}

/// GAME-DESIGN §6 harness rows over strategic games.
fn fun_rows(results: &Results, resamples: u32) -> Vec<Row> {
    let games = strategic(results);
    if games.is_empty() {
        return Vec::new();
    }
    let est = |c: &[(f64, f64)], stream: u64| stats::ratio(c, resamples, stream);
    let mut ticks: Vec<u32> = games
        .iter()
        .flat_map(|p| p.iter().map(|r| r.ticks))
        .collect();
    ticks.sort_unstable();
    let median = f64::from(ticks[ticks.len() / 2]) / f64::from(TICK_HZ);
    let mut rows = vec![Row {
        name: "median duration (s)".into(),
        value: median,
        interval: None,
        band: (150.0, 240.0),
        unit: Unit::Seconds,
        searched: true,
    }];
    let share = |reason: EndReason| {
        cluster(&games, move |r| {
            (f64::from(u8::from(r.reason == reason)), 1.0)
        })
    };
    rows.push(pct(
        "ended by Stronghold kill",
        est(&share(EndReason::StrongholdDestroyed), 2),
        (0.60, 1.0),
    ));
    rows.push(pct(
        "ended by time",
        est(&share(EndReason::TimeUp), 3),
        (0.0, 0.30),
    ));
    let draws = cluster(&games, |r| (f64::from(u8::from(r.winner.is_none())), 1.0));
    rows.push(pct("draws", est(&draws, 4), (0.0, 0.02)));
    for f in FACTIONS {
        let c = cluster(&games, |r| {
            (0..2)
                .filter(|&p| r.sides[p].majority() == Some(f.index()))
                .fold((0.0, 0.0), |(n, d), p| (n + r.score(p), d + 1.0))
        });
        rows.push(pct(
            format!("faction-majority {f:?} score"),
            est(&c, 10 + f.index() as u64),
            (0.45, 0.55),
        ));
    }
    for k in UNIT_KINDS {
        let c = cluster(&games, |r| {
            let all: u32 = r
                .sides
                .iter()
                .flat_map(|s| s.purchases.iter())
                .map(|&n| u32::from(n))
                .sum();
            let mine: u32 = r
                .sides
                .iter()
                .map(|s| u32::from(s.purchases[k as usize]))
                .sum();
            (f64::from(mine), f64::from(all))
        });
        rows.push(pct(
            format!("purchase share {k:?}"),
            est(&c, 20 + k as u64),
            (0.05, 1.0),
        ));
    }
    let comebacks = cluster(&games, |r| match r.comeback() {
        Some(c) => (f64::from(u8::from(c)), 1.0),
        None => (0.0, 0.0),
    });
    rows.push(pct(
        "comebacks (behind at 2:00, won)",
        est(&comebacks, 5),
        (0.20, 1.0),
    ));
    rows
}

/// Payoff matrix and equilibrium over every policy present in `results`.
pub fn payoff(results: &Results) -> (Vec<Policy>, Vec<Vec<f64>>) {
    let pool = Policy::pool();
    let ps: Vec<Policy> = pool
        .iter()
        .copied()
        .filter(|&p| {
            pool.iter()
                .any(|&q| p != q && scores(results, p, q).is_some())
        })
        .collect();
    let m = ps
        .iter()
        .map(|&a| {
            ps.iter()
                .map(|&b| {
                    if a == b {
                        return 0.5;
                    }
                    scores(results, a, b)
                        .map_or(0.5, |s| s.iter().sum::<f64>() / s.len().max(1) as f64)
                })
                .collect()
        })
        .collect();
    (ps, m)
}

fn meta_rows(results: &Results) -> Vec<Row> {
    let (ps, m) = payoff(results);
    // Only meaningful when every pair of the pool was played.
    let full = ps.len() == Policy::pool().len()
        && ps.iter().all(|&a| {
            ps.iter()
                .all(|&b| a == b || scores(results, a, b).is_some())
        });
    if !full {
        return Vec::new();
    }
    let mix = meta::nash(&m, NASH_ITERATIONS);
    let top = mix.iter().copied().fold(0.0, f64::max);
    vec![
        Row {
            name: "Nash support (policies ≥ 1 %)".into(),
            value: meta::support(&mix) as f64,
            interval: None,
            band: (3.0, f64::INFINITY),
            unit: Unit::Count,
            searched: false,
        },
        Row {
            name: "Nash largest weight".into(),
            value: top,
            interval: None,
            band: (0.0, 0.50),
            unit: Unit::Percent,
            searched: false,
        },
    ]
}

/// Search loss: squared distance outside every searched band.
pub fn loss(rows: &[Row]) -> f64 {
    rows.iter()
        .filter(|r| r.searched)
        .map(|r| r.distance().powi(2))
        .sum()
}
