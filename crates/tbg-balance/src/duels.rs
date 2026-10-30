//! Energy-equal unit duels (P16-C step 6): every form of every shop unit
//! against every other, with the same Energy budget (Evolution included),
//! no Shop, no regeneration, and Strongholds that do not fire. Exact and
//! cheap: it shows cost-inefficient forms, oppressive forms, and whether
//! every form has a counter (GAME-DESIGN §6 "Counterplay").

use tbg_core::content::{ENERGY_SCALE, RUNNING_START, TICK_HZ};
use tbg_core::{Archetype, MatchState, Phase, Ruleset, Seat, UnitKind};

use crate::runner::parallel_map;

/// Army-sized budgets: large enough that integer unit counts waste little.
pub const BUDGETS: [u32; 3] = [20, 40, 60];
/// Fight time after the first spawn before HP decides.
const LIMIT: u32 = 90 * TICK_HZ;
/// One unit per side per second.
const SPAWN_EVERY: u32 = TICK_HZ;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Form {
    pub kind: UnitKind,
    pub tier: u8,
    pub branch: u8,
}

impl Form {
    pub fn name(&self, rules: &Ruleset) -> String {
        match rules.form_of(self.kind, self.tier, self.branch) {
            Some(f) => format!("{:?}/{}", self.kind, f.name.replace(' ', "")),
            None => format!("{:?}", self.kind),
        }
    }

    /// Unit cost plus the Evolutions up to its tier, in sub-energy.
    pub fn cost(&self, rules: &Ruleset) -> u32 {
        rules.def(self.kind).cost
            + rules.evolve_cost[..usize::from(self.tier)]
                .iter()
                .sum::<u32>()
    }
}

/// Base, A1, A2, B1, B2 of every conventional unit (summons excluded).
pub fn forms() -> Vec<Form> {
    tbg_core::ALL_KINDS
        .iter()
        .copied()
        .filter(|k| k.is_conventional())
        .flat_map(|kind| {
            [(0, 0), (1, 0), (2, 0), (1, 1), (2, 1)].map(|(tier, branch)| Form {
                kind,
                tier,
                branch,
            })
        })
        .collect()
}

/// The duel arena: the ruleset under test without Stronghold fire or regen.
pub fn arena(rules: &Ruleset) -> &'static Ruleset {
    let mut r = rules.clone();
    r.stronghold_damage = 0;
    r.regen_running = 0;
    r.interest.max_bands = 0;
    r.augment.round_ticks = 0;
    // ponytail: leaked once per tested ruleset (a few KB, freed at exit);
    // a lifetime on MatchState would touch every online caller.
    Box::leak(Box::new(r))
}

/// Score of `a` (1 win, ½ draw, 0 loss) with `a` on seat A, or `None` if
/// either side cannot field one unit with the budget.
pub fn duel(arena: &'static Ruleset, a: Form, b: Form, budget: u32) -> Option<f64> {
    let energy = budget * ENERGY_SCALE;
    let counts = [a, b].map(|f| energy / f.cost(arena));
    if counts.contains(&0) {
        return None;
    }
    let mut s = MatchState::with_rules(0, arena);
    while s.tick < RUNNING_START {
        s.step();
    }
    let mut spawned = [0u32; 2];
    let start = s.tick;
    loop {
        let t = s.tick - start;
        if t.is_multiple_of(SPAWN_EVERY) {
            for (i, f) in [a, b].iter().enumerate() {
                if spawned[i] < counts[i] {
                    s.lab_spawn(Seat::BOTH[i], f.kind, f.tier, f.branch);
                    spawned[i] += 1;
                }
            }
        }
        s.step();
        if let Some(o) = s.outcome {
            return Some(match o.winner {
                Some(Seat::A) => 1.0,
                Some(Seat::B) => 0.0,
                None => 0.5,
            });
        }
        let hp = Seat::BOTH.map(|seat| {
            s.units
                .iter()
                .filter(|u| u.seat == seat && u.hp > 0)
                .map(|u| i64::from(u.hp))
                .sum::<i64>()
        });
        let done = spawned == counts;
        if done && (hp[0] == 0 || hp[1] == 0) || t >= LIMIT || s.phase == Phase::Ended {
            return Some(match hp[0].cmp(&hp[1]) {
                std::cmp::Ordering::Greater => 1.0,
                std::cmp::Ordering::Less => 0.0,
                std::cmp::Ordering::Equal => 0.5,
            });
        }
    }
}

/// `scores[budget][i][j]`: score of form `i` against `j`, both seat orders
/// averaged; `None` where a side cannot afford one unit.
pub struct Matrix {
    pub forms: Vec<Form>,
    pub scores: Vec<Vec<Vec<Option<f64>>>>,
}

pub fn matrix(rules: &Ruleset, threads: usize) -> Matrix {
    matrix_of(rules, forms(), threads)
}

/// The duel matrix over `forms` only (contract tests use base forms).
pub fn matrix_of(rules: &Ruleset, forms: Vec<Form>, threads: usize) -> Matrix {
    let arena = arena(rules);
    let n = forms.len();
    let jobs: Vec<(usize, usize, usize)> = (0..BUDGETS.len())
        .flat_map(|b| (0..n).flat_map(move |i| (i + 1..n).map(move |j| (b, i, j))))
        .collect();
    let results = parallel_map(&jobs, threads, |&(b, i, j)| {
        let (fa, fb) = (forms[i], forms[j]);
        let one = duel(arena, fa, fb, BUDGETS[b])?;
        let two = duel(arena, fb, fa, BUDGETS[b])?;
        Some((one + 1.0 - two) / 2.0)
    });
    let mut scores = vec![vec![vec![None; n]; n]; BUDGETS.len()];
    for (&(b, i, j), r) in jobs.iter().zip(results) {
        scores[b][i][j] = r;
        scores[b][j][i] = r.map(|v| 1.0 - v);
    }
    for (b, budget) in scores.iter_mut().enumerate() {
        for (i, row) in budget.iter_mut().enumerate() {
            row[i] = (forms[i].cost(rules) <= BUDGETS[b] * ENERGY_SCALE).then_some(0.5);
        }
    }
    Matrix { forms, scores }
}

/// Forms flagged by the gate (P16-C step 7).
#[derive(Debug, Default)]
pub struct Flags {
    /// Base form averaging ≤ 30 % against the other base forms at every
    /// budget. Support is exempt: it multiplies an army, it cannot be one.
    pub weak: Vec<usize>,
    /// Base form averaging ≥ 70 % against the other base forms everywhere.
    pub oppressive: Vec<usize>,
    /// Wins on average somewhere, yet no opponent scores ≥ 60 % against
    /// it at any budget.
    pub counterless: Vec<usize>,
    /// Evolved form losing to its own base form, Energy for Energy, at
    /// every budget: that Evolution is never worth buying in a fight.
    pub evolution_not_worth: Vec<usize>,
    /// `(kind, tier)` where one branch is never better than the other.
    pub branch_without_niche: Vec<(UnitKind, u8)>,
}

impl Matrix {
    /// Average score of form `i` at budget `b` over fieldable opponents
    /// accepted by `among`.
    pub fn average_among(&self, b: usize, i: usize, among: impl Fn(&Form) -> bool) -> Option<f64> {
        let v: Vec<f64> = (0..self.forms.len())
            .filter(|&j| j != i && among(&self.forms[j]))
            .filter_map(|j| self.scores[b][i][j])
            .collect();
        (!v.is_empty()).then(|| v.iter().sum::<f64>() / v.len() as f64)
    }

    /// Average score of form `i` at budget `b` over every fieldable form.
    pub fn average(&self, b: usize, i: usize) -> Option<f64> {
        self.average_among(b, i, |_| true)
    }

    fn base_of(&self, i: usize) -> Option<usize> {
        let k = self.forms[i].kind;
        self.forms.iter().position(|f| f.kind == k && f.tier == 0)
    }

    pub fn flags(&self, rules: &Ruleset) -> Flags {
        let mut f = Flags::default();
        let n = self.forms.len();
        let budgets = 0..BUDGETS.len();
        for i in 0..n {
            let form = self.forms[i];
            if form.tier == 0 {
                let avgs: Vec<f64> = budgets
                    .clone()
                    .filter_map(|b| self.average_among(b, i, |o| o.tier == 0))
                    .collect();
                let support = rules.def(form.kind).archetype == Archetype::Support;
                if !support && !avgs.is_empty() && avgs.iter().all(|&a| a <= 0.30) {
                    f.weak.push(i);
                }
                if !avgs.is_empty() && avgs.iter().all(|&a| a >= 0.70) {
                    f.oppressive.push(i);
                }
            } else if let Some(base) = self.base_of(i) {
                let vs: Vec<f64> = budgets
                    .clone()
                    .filter_map(|b| self.scores[b][i][base])
                    .collect();
                if !vs.is_empty() && vs.iter().all(|&s| s < 0.5) {
                    f.evolution_not_worth.push(i);
                }
            }
            let fieldable = budgets.clone().any(|b| self.scores[b][i][i].is_some());
            let countered = budgets
                .clone()
                .any(|b| (0..n).any(|j| j != i && self.scores[b][j][i].is_some_and(|s| s >= 0.6)));
            // Only a form that wins on average needs a counter.
            let wins = budgets
                .clone()
                .filter_map(|b| self.average(b, i))
                .any(|a| a > 0.5);
            if fieldable && wins && !countered {
                f.counterless.push(i);
            }
        }
        for (i, a) in self.forms.iter().enumerate() {
            if a.tier == 0 || a.branch == 1 {
                continue;
            }
            let Some(j) = self
                .forms
                .iter()
                .position(|b| b.kind == a.kind && b.tier == a.tier && b.branch == 1)
            else {
                continue;
            };
            let better = |x: usize, y: usize| {
                budgets.clone().any(|b| {
                    (0..n).any(|o| match (self.scores[b][x][o], self.scores[b][y][o]) {
                        (Some(sx), Some(sy)) => sx > sy,
                        _ => false,
                    })
                })
            };
            if !better(i, j) || !better(j, i) {
                f.branch_without_niche.push((a.kind, a.tier));
            }
        }
        f
    }
}
