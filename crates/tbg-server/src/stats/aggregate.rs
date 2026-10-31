//! P61 §6 exact definitions over eligible sides (`S = 2M`): presence
//! `N_X/S`, binary win rate `W_X/N_X` (draws are non-wins), draw and score
//! rates, complement delta in percentage points, expected-score residual
//! from settlement-time ratings, and the §7 publication gates. Association
//! only: nothing here isolates causal strength.

use std::collections::{BTreeMap, BTreeSet};

use tbg_core::rating::expected;

use super::{Catalog, MatchObs, SideData, UnitRow};

/// §6 primary internal unit investigation: purchase by 60 combat seconds.
pub const LANDMARK_SECS: u32 = 60;

/// §7 public gates per exposure (**Proposed**, O4).
pub const GATE_SIDES: u64 = 50;
pub const GATE_ACCOUNTS: usize = 5;
pub const GATE_PAIRS: usize = 3;
pub const GATE_MAX_SHARE_PCT: u64 = 40;

/// Outcomes and concentration of one exposure group.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Tally {
    pub n: u64,
    pub w: u64,
    pub d: u64,
    pub l: u64,
    pub residual_sum: f64,
    /// Exposure per account (exact; offline CLI, bounded by `COHORT_CAP`).
    accounts: BTreeMap<i64, u64>,
    pairs: BTreeSet<(i64, i64)>,
}

impl Tally {
    fn add(&mut self, side: &SideRef) {
        self.n += 1;
        match side.score {
            Outcome::Win => self.w += 1,
            Outcome::Draw => self.d += 1,
            Outcome::Loss => self.l += 1,
        }
        self.residual_sum += side.score.value() - side.expected;
        *self.accounts.entry(side.account).or_default() += 1;
        let pair = (
            side.account.min(side.opponent),
            side.account.max(side.opponent),
        );
        self.pairs.insert(pair);
    }

    pub fn win_rate(&self) -> Option<f64> {
        (self.n > 0).then(|| self.w as f64 / self.n as f64)
    }

    pub fn draw_rate(&self) -> Option<f64> {
        (self.n > 0).then(|| self.d as f64 / self.n as f64)
    }

    pub fn score_rate(&self) -> Option<f64> {
        (self.n > 0).then(|| (self.w as f64 + 0.5 * self.d as f64) / self.n as f64)
    }

    pub fn residual(&self) -> Option<f64> {
        (self.n > 0).then(|| self.residual_sum / self.n as f64)
    }

    pub fn distinct_accounts(&self) -> usize {
        self.accounts.len()
    }

    pub fn distinct_pairs(&self) -> usize {
        self.pairs.len()
    }

    /// Largest single account's share of this exposure, in percent.
    pub fn max_share_pct(&self) -> u64 {
        let max = self.accounts.values().copied().max().unwrap_or(0);
        (max * 100).checked_div(self.n).unwrap_or(0)
    }

    /// §7: whether this group alone could be published.
    pub fn passes_gate(&self) -> bool {
        self.n >= GATE_SIDES
            && self.distinct_accounts() >= GATE_ACCOUNTS
            && self.distinct_pairs() >= GATE_PAIRS
            && self.max_share_pct() <= GATE_MAX_SHARE_PCT
    }
}

/// Exposed sides and their complement, each gated on its own (§7).
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Exposure {
    pub label: String,
    pub yes: Tally,
    pub no: Tally,
    /// Copies (units) or opportunities (Augments) where meaningful.
    pub amount: u64,
    pub extra: Vec<(String, f64)>,
}

impl Exposure {
    /// Buyer delta in percentage points; `None` unless both sides have data.
    pub fn delta_pp(&self) -> Option<f64> {
        Some((self.yes.win_rate()? - self.no.win_rate()?) * 100.0)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Outcome {
    Win,
    Draw,
    Loss,
}

impl Outcome {
    fn value(self) -> f64 {
        match self {
            Outcome::Win => 1.0,
            Outcome::Draw => 0.5,
            Outcome::Loss => 0.0,
        }
    }
}

struct SideRef<'a> {
    data: &'a SideData,
    account: i64,
    opponent: i64,
    score: Outcome,
    expected: f64,
    combat_ticks: u32,
}

fn sides(m: &MatchObs) -> [SideRef<'_>; 2] {
    std::array::from_fn(|i| SideRef {
        data: &m.sides[i],
        account: m.accounts[i],
        opponent: m.accounts[1 - i],
        score: match m.winner {
            None => Outcome::Draw,
            Some(w) if w == i => Outcome::Win,
            Some(_) => Outcome::Loss,
        },
        expected: expected(m.before[i], m.before[1 - i]),
        combat_ticks: m.combat_ticks,
    })
}

/// Every dimension of §6 for one revision's eligible matches.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Report {
    pub matches: u64,
    pub sides: u64,
    pub overall: Tally,
    pub seat_a: Tally,
    pub units: Vec<Exposure>,
    pub landmark_units: Vec<Exposure>,
    pub landmark_matches: u64,
    pub landmark_excluded: u64,
    pub evolution: Vec<Exposure>,
    pub factions: Vec<Exposure>,
    pub archetypes: Vec<Exposure>,
    pub synergy: Vec<Exposure>,
    pub augments: Vec<Exposure>,
    pub arcana: Vec<Exposure>,
    pub strongholds: Vec<Exposure>,
    pub arenas: Vec<ArenaRow>,
    pub general: General,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct ArenaRow {
    pub label: String,
    pub matches: u64,
    pub seat_a: Tally,
    pub forfeits: u64,
    pub wall_ms_sum: u64,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct General {
    pub wall_ms_sum: u64,
    pub combat_ticks_sum: u64,
    pub surrenders: u64,
    pub disconnects: u64,
    pub draws: u64,
    /// Matches per UTC completion day (`days since epoch`, count).
    pub per_day: BTreeMap<i64, u64>,
}

/// Group `matches` side by side for each exposure predicate.
fn exposures<F>(matches: &[MatchObs], labels: &[String], mut on: F) -> Vec<Exposure>
where
    F: FnMut(&SideRef, usize) -> Option<u64>,
{
    let mut rows: Vec<Exposure> = labels
        .iter()
        .map(|l| Exposure {
            label: l.clone(),
            ..Exposure::default()
        })
        .collect();
    for m in matches {
        for side in sides(m) {
            for (i, row) in rows.iter_mut().enumerate() {
                match on(&side, i) {
                    Some(amount) => {
                        row.yes.add(&side);
                        row.amount += amount;
                    }
                    None => row.no.add(&side),
                }
            }
        }
    }
    rows
}

fn bought(side: &SideData, kind: usize) -> Option<&UnitRow> {
    side.units.iter().find(|u| u.kind == kind && u.bought > 0)
}

pub fn build(matches: &[MatchObs], catalog: &Catalog) -> Report {
    let mut r = Report {
        matches: matches.len() as u64,
        sides: 2 * matches.len() as u64,
        ..Report::default()
    };
    let kinds: Vec<usize> = (0..catalog.kinds.len())
        .filter(|&k| catalog.kinds[k].purchasable)
        .collect();
    let kind_labels: Vec<String> = kinds
        .iter()
        .map(|&k| catalog.kinds[k].name.clone())
        .collect();
    for m in matches {
        let [a, b] = sides(m);
        r.overall.add(&a);
        r.overall.add(&b);
        r.seat_a.add(&a);
    }
    // Unit presence: at least one successful purchase; amount = copies.
    r.units = exposures(matches, &kind_labels, |s, i| {
        bought(s.data, kinds[i]).map(|u| u64::from(u.bought))
    });
    // Landmark: matches reaching 60 combat seconds; early = bought by then.
    let landmark = LANDMARK_SECS * catalog.tick_hz.max(1);
    let reached: Vec<MatchObs> = matches
        .iter()
        .filter(|m| m.combat_ticks >= landmark)
        .cloned()
        .collect();
    r.landmark_matches = reached.len() as u64;
    r.landmark_excluded = r.matches - r.landmark_matches;
    r.landmark_units = exposures(&reached, &kind_labels, |s, i| {
        bought(s.data, kinds[i])
            .filter(|u| u.first_tick.is_some_and(|t| t <= landmark))
            .map(|_| 0)
    });
    // Evolution among buyers: reach tier >= 1 / >= 2, plus copy conversion.
    r.evolution = evolution(matches, &kinds, &kind_labels);
    let faction_labels: Vec<String> = ["Humans", "Undead", "Machines", "Plants"]
        .map(String::from)
        .to_vec();
    r.factions = exposures(matches, &faction_labels, |s, f| {
        let copies: u64 = s
            .data
            .units
            .iter()
            .filter(|u| catalog.kinds[u.kind].faction == f && catalog.kinds[u.kind].purchasable)
            .map(|u| u64::from(u.bought))
            .sum();
        (copies > 0).then_some(copies)
    });
    let mut archetypes: Vec<String> = kinds
        .iter()
        .map(|&k| catalog.kinds[k].archetype.clone())
        .collect();
    archetypes.sort();
    archetypes.dedup();
    r.archetypes = exposures(matches, &archetypes, |s, a| {
        let copies: u64 = s
            .data
            .units
            .iter()
            .filter(|u| {
                catalog.kinds[u.kind].archetype == archetypes[a]
                    && catalog.kinds[u.kind].purchasable
            })
            .map(|u| u64::from(u.bought))
            .sum();
        (copies > 0).then_some(copies)
    });
    r.synergy = synergy(matches, &faction_labels);
    r.augments = augments(matches, catalog);
    let nodes: Vec<String> = (0..tbg_core::arcana::NODES)
        .map(|n| format!("node {n}"))
        .collect();
    r.arcana = exposures(matches, &nodes, |s, n| {
        s.data.arcanas.get(n).filter(|&&rank| rank > 0).map(|_| 0)
    });
    let kinds_sh = ["Classic", "Treasury", "Bastion", "Artillery", "Command"]
        .map(String::from)
        .to_vec();
    r.strongholds = exposures(matches, &kinds_sh, |s, k| {
        (s.data.stronghold == k).then_some(0)
    });
    r.strongholds
        .retain(|e| e.label != "Classic" || e.yes.n > 0);
    r.arenas = arenas(matches, catalog);
    r.general = general(matches);
    r
}

fn evolution(matches: &[MatchObs], kinds: &[usize], labels: &[String]) -> Vec<Exposure> {
    let mut rows = Vec::new();
    for (i, &k) in kinds.iter().enumerate() {
        let mut row = Exposure {
            label: labels[i].clone(),
            ..Exposure::default()
        };
        let (mut bought_sum, mut t1, mut t2, mut reach2) = (0u64, 0u64, 0u64, 0u64);
        for m in matches {
            for side in sides(m) {
                let Some(u) = bought(side.data, k) else {
                    continue;
                };
                bought_sum += u64::from(u.bought);
                t1 += u64::from(u.tier1);
                t2 += u64::from(u.tier2);
                // Exposure: the buying side ever reached tier >= 1.
                if u.max_tier >= 1 {
                    row.yes.add(&side);
                } else {
                    row.no.add(&side);
                }
                if u.max_tier >= 2 {
                    reach2 += 1;
                }
            }
        }
        if bought_sum > 0 {
            row.amount = bought_sum;
            row.extra = vec![
                ("reach_tier2_sides".into(), reach2 as f64),
                (
                    "tier1_copy_conversion".into(),
                    t1 as f64 / bought_sum as f64,
                ),
                (
                    "tier2_copy_conversion".into(),
                    t2 as f64 / bought_sum as f64,
                ),
            ];
        }
        rows.push(row);
    }
    rows
}

fn synergy(matches: &[MatchObs], labels: &[String]) -> Vec<Exposure> {
    let mut rows = exposures(matches, labels, |s, f| {
        (s.data.synergy_max[f] >= 1).then(|| u64::from(s.data.synergy_ticks[f][0]))
    });
    // Uptime over exposed sides: active ticks / their combat ticks.
    for (f, row) in rows.iter_mut().enumerate() {
        let mut combat: u64 = 0;
        let mut level2 = 0u64;
        for m in matches {
            for side in sides(m) {
                if side.data.synergy_max[f] >= 1 {
                    combat += u64::from(side.combat_ticks);
                }
                if side.data.synergy_max[f] >= 2 {
                    level2 += 1;
                }
            }
        }
        if combat > 0 {
            row.extra = vec![
                ("uptime_level1".into(), row.amount as f64 / combat as f64),
                ("sides_level2".into(), level2 as f64),
            ];
        }
    }
    rows
}

fn augments(matches: &[MatchObs], catalog: &Catalog) -> Vec<Exposure> {
    let ids = catalog.augments.len();
    let mut rows = exposures(matches, &catalog.augments, |s, id| {
        s.data
            .chosen
            .iter()
            .find(|c| c.0 == id && c.1 > 0)
            .map(|c| u64::from(c.1))
    });
    for (id, row) in rows.iter_mut().enumerate().take(ids) {
        let mut offered = 0u64;
        let mut offered_matches = 0u64;
        for m in matches {
            let in_match: u64 = m
                .sides
                .iter()
                .flat_map(|s| s.offered.iter())
                .filter(|o| o.0 == id)
                .map(|o| u64::from(o.1))
                .sum();
            offered += in_match;
            offered_matches += u64::from(in_match > 0);
        }
        if offered > 0 {
            row.extra = vec![
                ("offered_opportunities".into(), offered as f64),
                ("offered_matches".into(), offered_matches as f64),
                (
                    "conditional_pick".into(),
                    row.amount as f64 / offered as f64,
                ),
            ];
        }
    }
    rows
}

fn arenas(matches: &[MatchObs], catalog: &Catalog) -> Vec<ArenaRow> {
    let mut rows: Vec<ArenaRow> = catalog
        .arenas
        .iter()
        .map(|a| ArenaRow {
            label: a.clone(),
            ..ArenaRow::default()
        })
        .collect();
    for m in matches {
        let Some(row) = rows.get_mut(m.arena) else {
            continue;
        };
        let [a, _] = sides(m);
        row.matches += 1;
        row.seat_a.add(&a);
        row.wall_ms_sum += m.wall_ms;
        row.forfeits += u64::from(m.end_detail != "stronghold");
    }
    rows
}

fn general(matches: &[MatchObs]) -> General {
    let mut g = General::default();
    for m in matches {
        g.wall_ms_sum += m.wall_ms;
        g.combat_ticks_sum += u64::from(m.combat_ticks);
        g.surrenders += u64::from(m.end_detail == "surrender");
        g.disconnects += u64::from(m.end_detail == "disconnect");
        g.draws += u64::from(m.winner.is_none());
        *g.per_day
            .entry(m.ended_unix.div_euclid(86_400))
            .or_default() += 1;
    }
    g
}
