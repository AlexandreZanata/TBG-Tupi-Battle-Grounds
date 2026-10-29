//! Deterministic micro-scenarios: exact lane positions, fixed spawn ticks,
//! no Shop, and a fixed horizon. They measure a mechanism (HP + heal,
//! damage + attack speed, root + ranged) before a full match is run.

use tbg_core::content::RUNNING_START;
use tbg_core::{MatchState, Outcome, Phase, Ruleset, Seat, UnitKind};

use super::measure::{Observer, SideMetrics, Snapshot};

#[derive(Clone, Copy, Debug)]
pub struct Spawn {
    /// Tick from the start of the match; `RUNNING_START` is the opening tick.
    pub tick: u32,
    pub seat: Seat,
    pub kind: UnitKind,
    pub tier: u8,
    pub branch: u8,
    pub x: i32,
    /// Optional starting HP; `None` is full HP for the form.
    pub hp: Option<i32>,
}

#[derive(Clone, Debug)]
pub struct Scenario {
    pub name: &'static str,
    pub spawns: Vec<Spawn>,
    /// Ticks after `RUNNING_START`.
    pub horizon: u32,
    pub stronghold_fire: bool,
    pub regen: bool,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ScenarioResult {
    pub name: &'static str,
    pub ticks: u32,
    pub outcome: Option<Outcome>,
    pub metrics: [SideMetrics; 2],
    /// Living unit HP plus Stronghold HP, per seat.
    pub final_hp: [i64; 2],
    /// `final_hp[A] - final_hp[B]`; deterministic and sensitive.
    pub margin: i64,
}

impl Scenario {
    pub fn run(&self, rules: &'static Ruleset) -> ScenarioResult {
        let arena = arena(rules, self.stronghold_fire, self.regen);
        let mut state = MatchState::with_rules(0, arena);
        state.tick = RUNNING_START;
        state.phase = Phase::Running;

        let mut spawns = self.spawns.clone();
        spawns.sort_by_key(|s| s.tick);
        let mut next = 0;
        while next < spawns.len() && spawns[next].tick <= RUNNING_START {
            spawn(&mut state, spawns[next]);
            next += 1;
        }

        let mut observer = Observer::default();
        let end = RUNNING_START + self.horizon;
        while state.tick < end && state.outcome.is_none() {
            while next < spawns.len() && spawns[next].tick <= state.tick {
                spawn(&mut state, spawns[next]);
                next += 1;
            }
            let before = Snapshot::new(&state);
            let events = state.step();
            let after = Snapshot::new(&state);
            observer.observe(&before, &after, &events);
        }

        let mut final_hp = [0i64; 2];
        for u in &state.units {
            if u.hp > 0 {
                final_hp[u.seat.index()] += i64::from(u.hp);
            }
        }
        for seat in Seat::BOTH {
            final_hp[seat.index()] += i64::from(state.player(seat).stronghold_hp);
        }
        ScenarioResult {
            name: self.name,
            ticks: state.tick - RUNNING_START,
            outcome: state.outcome,
            metrics: observer.metrics,
            final_hp,
            margin: final_hp[0] - final_hp[1],
        }
    }
}

fn arena(rules: &Ruleset, stronghold_fire: bool, regen: bool) -> &'static Ruleset {
    let mut r = rules.clone();
    if !stronghold_fire {
        r.stronghold_damage = 0;
    }
    if !regen {
        r.regen_running = 0;
    }
    // ponytail: one small leak per scenario run; the lab is a short-lived
    // process and a lifetime would touch every scenario/test signature.
    Box::leak(Box::new(r))
}

fn spawn(state: &mut MatchState, s: Spawn) {
    let id = state.lab_spawn_at(s.seat, s.kind, s.tier, s.branch, s.x);
    if let Some(hp) = s.hp {
        if let Some(i) = state.unit_index(id) {
            state.units[i].hp = hp.clamp(1, state.units[i].max_hp);
        }
    }
}
