//! Staged search over knobs (P16-C step 8): a sensitivity screen, then a
//! (1+λ) evolution strategy with common seeds and successive halving.
//! Everything is deterministic from the search seed; small, readable
//! proposals win through a penalty per changed number.

use std::collections::BTreeMap;
use std::ops::Range;

use tbg_core::{Pcg32, Ruleset};

use crate::duels;
use crate::gate::{self, Row, Scope};
use crate::knobs::{self, Knob};

/// Seed blocks (P16-C step 4): confirmation seeds never choose a value.
pub const DISCOVERY: u64 = 0;
pub const SCREENING: u64 = 200_000;
pub const CONFIRMATION: u64 = 1_000_000;
/// A proposal changes at most this many numbers.
pub const MAX_CHANGES: usize = 8;

#[derive(Clone, Copy, Debug)]
pub struct Budget {
    pub threads: usize,
    pub reaction: u32,
}

/// A candidate: changes from canon, in steps per knob.
pub type Changes = BTreeMap<Knob, i64>;

pub fn build(canon: &Ruleset, changes: &Changes) -> Ruleset {
    let mut r = canon.clone();
    for (&k, &steps) in changes {
        knobs::set(&mut r, k, knobs::moved(k, knobs::get(canon, k), steps));
    }
    r
}

/// 0.5 per changed number plus 5 per 100 % of relative change.
pub fn penalty(canon: &Ruleset, changes: &Changes) -> f64 {
    let changed = build(canon, changes);
    changes
        .keys()
        .map(|&k| {
            let (a, b) = (knobs::get(canon, k), knobs::get(&changed, k));
            0.5 + 5.0 * ((b - a) as f64 / a.abs().max(1) as f64).abs()
        })
        .sum()
}

/// Suite rows (with duels) for a ruleset on `seeds`; intervals skipped.
pub fn evaluate(rules: &Ruleset, seeds: Range<u64>, b: Budget) -> Vec<Row> {
    let leaked: &'static Ruleset = Box::leak(Box::new(rules.clone()));
    let results = gate::run(leaked, Scope::Suite, seeds, b.reaction, b.threads);
    let matrix = duels::matrix(rules, b.threads);
    gate::rows(&results, Some((&matrix, rules)), 0)
}

pub struct Screened {
    pub knob: Knob,
    pub steps: i64,
    pub loss: f64,
}

/// Every knob at −2 and +2 steps on the screening block; sorted by loss.
pub fn screen(
    canon: &Ruleset,
    pairs: u64,
    b: Budget,
    mut log: impl FnMut(&str),
) -> (f64, Vec<Screened>) {
    let seeds = SCREENING..SCREENING + pairs;
    let base = gate::loss(&evaluate(canon, seeds.clone(), b));
    log(&format!("canon loss {base:.1}"));
    let mut out = Vec::new();
    for k in knobs::all(canon) {
        for steps in [-2, 2] {
            let changes = Changes::from([(k, steps)]);
            let r = build(canon, &changes);
            if r == *canon || knobs::valid(&r).is_err() {
                continue;
            }
            let loss = gate::loss(&evaluate(&r, seeds.clone(), b));
            log(&format!("{} {steps:+} -> {loss:.1}", knobs::name(canon, k)));
            out.push(Screened {
                knob: k,
                steps,
                loss,
            });
        }
    }
    out.sort_by(|a, b| a.loss.total_cmp(&b.loss));
    (base, out)
}

/// Knobs ordered by their best screened loss, without repeats.
pub fn ranked(screened: &[Screened], top: usize) -> Vec<Knob> {
    let mut v: Vec<Knob> = Vec::new();
    for s in screened {
        if !v.contains(&s.knob) {
            v.push(s.knob);
        }
    }
    v.truncate(top);
    v
}

pub struct Settings {
    pub generations: u32,
    pub lambda: usize,
    /// Final stage pairs; earlier stages use 1/16 and 1/4 of it.
    pub pairs: u64,
    pub seed: u64,
}

pub struct Found {
    pub changes: Changes,
    pub loss: f64,
    pub penalty: f64,
    pub rows: Vec<Row>,
}

fn mutate(canon: &Ruleset, parent: &Changes, knobs_: &[Knob], rng: &mut Pcg32) -> Option<Changes> {
    for _ in 0..20 {
        let mut c = parent.clone();
        for _ in 0..1 + rng.below(3) {
            let k = knobs_[rng.below(knobs_.len() as u32) as usize];
            let delta = [-2, -1, 1, 2][rng.below(4) as usize];
            let steps = c.get(&k).copied().unwrap_or(0) + delta;
            let (_, max) = knobs::step(k, knobs::get(canon, k));
            if steps == 0 || steps.abs() > max {
                c.remove(&k);
            } else {
                c.insert(k, steps);
            }
        }
        if c != *parent && c.len() <= MAX_CHANGES && knobs::valid(&build(canon, &c)).is_ok() {
            return Some(c);
        }
    }
    None
}

/// Total of `changes` at `pairs` discovery seeds: (loss, penalty, rows).
fn score(canon: &Ruleset, changes: &Changes, pairs: u64, b: Budget) -> (f64, f64, Vec<Row>) {
    let rows = evaluate(&build(canon, changes), DISCOVERY..DISCOVERY + pairs, b);
    (gate::loss(&rows), penalty(canon, changes), rows)
}

pub fn search(
    canon: &Ruleset,
    knobs_: &[Knob],
    s: &Settings,
    b: Budget,
    mut log: impl FnMut(&str),
) -> Found {
    let mut rng = Pcg32::new(s.seed, 23);
    let stages = [(s.pairs / 16).max(8), (s.pairs / 4).max(16), s.pairs];
    let mut parent = Changes::new();
    let (mut loss, mut pen, mut rows) = score(canon, &parent, s.pairs, b);
    log(&format!("gen 0: canon loss {loss:.1}"));
    for generation in 1..=s.generations {
        let mut children: Vec<Changes> = Vec::new();
        for _ in 0..s.lambda {
            if let Some(c) = mutate(canon, &parent, knobs_, &mut rng) {
                if !children.contains(&c) {
                    children.push(c);
                }
            }
        }
        // Successive halving on common seeds: all children cheaply, the
        // best quarter longer, the best one at full size.
        let mut scored: Vec<(f64, Changes)> = children
            .into_iter()
            .map(|c| {
                let (l, p, _) = score(canon, &c, stages[0], b);
                (l + p, c)
            })
            .collect();
        scored.sort_by(|a, b| a.0.total_cmp(&b.0));
        scored.truncate(4);
        for entry in &mut scored {
            let (l, p, _) = score(canon, &entry.1, stages[1], b);
            entry.0 = l + p;
        }
        let Some((_, best)) = scored.into_iter().min_by(|a, b| a.0.total_cmp(&b.0)) else {
            continue;
        };
        let (l, p, r) = score(canon, &best, stages[2], b);
        let accepted = l + p < loss + pen;
        if accepted {
            (parent, loss, pen, rows) = (best, l, p, r);
        }
        let names: Vec<String> = parent
            .iter()
            .map(|(&k, &st)| format!("{}{:+}", knobs::name(canon, k), st))
            .collect();
        log(&format!(
            "gen {generation}: {} loss {loss:.1} penalty {pen:.1} [{}]",
            if accepted { "improved" } else { "kept" },
            names.join(", ")
        ));
        if loss == 0.0 {
            break;
        }
    }
    Found {
        changes: parent,
        loss,
        penalty: pen,
        rows,
    }
}

/// `name=value` list of a candidate, accepted by `--apply`.
pub fn apply_spec(canon: &Ruleset, changes: &Changes) -> String {
    let r = build(canon, changes);
    changes
        .keys()
        .map(|&k| format!("{}={}", knobs::name(canon, k), knobs::get(&r, k)))
        .collect::<Vec<_>>()
        .join(",")
}
