//! Match state, units, outcomes, and the events a tick produces.

use serde::{Deserialize, Serialize};

use crate::content::{self, Stats};
use crate::rng::Pcg32;
use crate::types::{Faction, Seat, Status, Tick, UnitId, UnitKind};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Phase {
    Countdown,
    Running,
    Overtime,
    Ended,
}

/// What an attack, heal, or event points at.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Target {
    Unit(UnitId),
    Stronghold(Seat),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Unit {
    pub id: UnitId,
    pub seat: Seat,
    pub kind: UnitKind,
    pub tier: u8,
    pub branch: u8,
    pub x: i32,
    pub hp: i32,
    pub max_hp: i32,
    pub cooldown: u16,
    pub windup: Option<(Target, u16)>,
    pub status: Status,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct PlayerState {
    pub energy: u32,
    pub offers: [UnitKind; 3],
    pub stronghold_hp: i32,
    pub stronghold_cooldown: u16,
    /// Synergy level (0..=2) per `Faction` index.
    pub synergy: [u8; 4],
    pub last_seq: u32,
    pub rng: Pcg32,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum EndReason {
    StrongholdDestroyed,
    TimeUp,
    Forfeit,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Outcome {
    /// `None` is a draw.
    pub winner: Option<Seat>,
    pub reason: EndReason,
}

/// Everything that happened in a tick; clients animate from these.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Event {
    Spawn {
        unit: UnitId,
        seat: Seat,
        kind: UnitKind,
    },
    AttackStart {
        from: Target,
        to: Target,
    },
    Hit {
        to: Target,
        amount: i32,
    },
    Heal {
        to: UnitId,
        amount: i32,
    },
    StatusApplied {
        unit: UnitId,
    },
    Death {
        unit: UnitId,
    },
    Evolved {
        unit: UnitId,
        tier: u8,
        branch: u8,
    },
    SynergyChanged {
        seat: Seat,
        faction: Faction,
        level: u8,
    },
    PhaseChanged {
        phase: Phase,
    },
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct MatchState {
    pub tick: Tick,
    pub phase: Phase,
    pub players: [PlayerState; 2],
    /// Always sorted by id (new units get the next, larger id).
    pub units: Vec<Unit>,
    pub next_unit_id: UnitId,
    pub outcome: Option<Outcome>,
    /// Events created by intentions, returned by the next `step()`.
    #[serde(skip)]
    pub(crate) pending: Vec<Event>,
}

/// x position of a seat's Stronghold.
pub fn stronghold_x(seat: Seat) -> i32 {
    match seat {
        Seat::A => 0,
        Seat::B => content::LANE_LEN,
    }
}

impl MatchState {
    pub fn player(&self, seat: Seat) -> &PlayerState {
        &self.players[seat.index()]
    }

    pub(crate) fn player_mut(&mut self, seat: Seat) -> &mut PlayerState {
        &mut self.players[seat.index()]
    }

    pub fn unit_index(&self, id: UnitId) -> Option<usize> {
        self.units.binary_search_by_key(&id, |u| u.id).ok()
    }

    pub fn is_live(&self) -> bool {
        matches!(self.phase, Phase::Running | Phase::Overtime)
    }

    /// Slots occupied by a seat's living units.
    pub fn slots_used(&self, seat: Seat) -> u8 {
        self.units
            .iter()
            .filter(|u| u.seat == seat && u.hp > 0)
            .map(|u| content::def(u.kind).slots.count())
            .sum()
    }

    pub fn stats_of(&self, u: &Unit) -> Stats {
        let faction = content::def(u.kind).faction.index();
        let level = self.player(u.seat).synergy[faction];
        content::effective_stats(u.kind, u.tier, u.branch, level)
    }

    /// Recompute a unit's max HP, keeping its HP ratio (GAME-DESIGN §4).
    pub(crate) fn refresh_max_hp(&mut self, idx: usize) {
        let new_max = self.stats_of(&self.units[idx]).max_hp;
        let u = &mut self.units[idx];
        if new_max != u.max_hp && u.hp > 0 {
            let hp = i64::from(u.hp) * i64::from(new_max) / i64::from(u.max_hp);
            u.hp = (hp as i32).max(1);
            u.max_hp = new_max;
        }
    }
}
