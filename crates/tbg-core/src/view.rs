//! What one seat may see of a match. Opponent Energy, offers, and every
//! RNG stay inside `MatchState` (BR-S05).

use serde::{Deserialize, Serialize};

use crate::shop::next_rotation_in;
use crate::sim::phase_ticks_left;
use crate::state::{Event, MatchState, Outcome, Phase};
use crate::types::{Seat, Status, Tick, UnitId, UnitKind};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct UnitView {
    pub id: UnitId,
    pub seat: Seat,
    pub kind: UnitKind,
    pub tier: u8,
    pub branch: u8,
    pub x: i32,
    pub hp: i32,
    pub max_hp: i32,
    /// 0 none, 1 slowed, 2 rooted.
    pub status: u8,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct MatchView {
    pub tick: Tick,
    pub phase: Phase,
    pub phase_ticks_left: u32,
    pub you: Seat,
    pub energy: u32,
    pub offers: [UnitKind; 3],
    pub next_rotation_in: u32,
    pub slots_used: u8,
    /// Indexed by `Seat::index()`.
    pub stronghold_hp: [i32; 2],
    pub units: Vec<UnitView>,
    pub synergy: [[u8; 4]; 2],
    pub last_seq: u32,
    pub events: Vec<Event>,
    pub outcome: Option<Outcome>,
}

impl MatchState {
    pub fn view_for(&self, seat: Seat, events: &[Event]) -> MatchView {
        let me = self.player(seat);
        MatchView {
            tick: self.tick,
            phase: self.phase,
            phase_ticks_left: phase_ticks_left(self),
            you: seat,
            energy: me.energy,
            offers: me.offers,
            next_rotation_in: next_rotation_in(self.tick),
            slots_used: self.slots_used(seat),
            stronghold_hp: [0, 1].map(|i| self.players[i].stronghold_hp),
            units: self
                .units
                .iter()
                .filter(|u| u.hp > 0)
                .map(unit_view)
                .collect(),
            synergy: [0, 1].map(|i| self.players[i].synergy),
            last_seq: me.last_seq,
            events: events.to_vec(),
            outcome: self.outcome,
        }
    }
}

fn unit_view(u: &crate::state::Unit) -> UnitView {
    UnitView {
        id: u.id,
        seat: u.seat,
        kind: u.kind,
        tier: u.tier,
        branch: u.branch,
        x: u.x,
        hp: u.hp,
        max_hp: u.max_hp,
        status: match u.status {
            Status::None => 0,
            Status::Slowed { .. } => 1,
            Status::Rooted { .. } => 2,
        },
    }
}
