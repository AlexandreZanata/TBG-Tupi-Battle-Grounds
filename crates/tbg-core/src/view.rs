//! What one seat may see of a match. Opponent Energy, offers, and every
//! RNG stay inside `MatchState` (BR-S05).

use serde::{Deserialize, Serialize};

use crate::augment::AugmentCounts;
use crate::content::{AUGMENT_OPTIONS, OFFERS, RUNNING_START};
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

/// One open Augment round, identical options for both seats.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct RoundView {
    pub round: u32,
    pub options: [u8; AUGMENT_OPTIONS],
    pub costs: [u32; AUGMENT_OPTIONS],
    /// Selection-clock steps until the round closes (pauses count).
    pub ticks_left: u32,
    /// This seat confirmed (or forwent); never the other seat's state.
    pub resolved: bool,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct MatchView {
    /// Combat tick: frozen while an Augment selection pauses the battle.
    pub tick: Tick,
    /// Selection clock: advances every step, pauses included (BR-AG13).
    pub clock: Tick,
    /// Ticks since Running began (the open-ended match clock).
    pub elapsed: u32,
    pub phase: Phase,
    /// Countdown only; 0 while running (no gameplay time limit).
    pub phase_ticks_left: u32,
    pub you: Seat,
    pub energy: u32,
    /// Energy regeneration now, in basis points of the base rate: 10 000
    /// plus 2 500 per full 5 Energy held, at most 22 500 (BR-E05).
    pub regen_bp: u32,
    pub offers: [UnitKind; OFFERS],
    /// Indexed by `Seat::index()`.
    pub stronghold_hp: [i32; 2],
    pub shields: [i32; 2],
    pub units: Vec<UnitView>,
    pub synergy: [[u8; 4]; 2],
    /// Both seats' Augment copies (public, BR-AG09).
    pub augments: [AugmentCounts; 2],
    /// Present exactly while the battle is paused for a selection; both
    /// seats get the same round, deadline and options (BR-AG12).
    pub augment_round: Option<RoundView>,
    pub last_seq: u32,
    pub events: Vec<Event>,
    pub outcome: Option<Outcome>,
}

impl MatchState {
    pub fn view_for(&self, seat: Seat, events: &[Event]) -> MatchView {
        let me = self.player(seat);
        let round = (self.is_live() && self.augments.selecting()).then(|| {
            let costs = self.augments.options.map(|id| {
                crate::content::augment_def(id)
                    .map(|def| self.rules.augment.costs[def.rarity.index()])
                    .unwrap_or(0)
            });
            RoundView {
                round: self.augments.round,
                options: self.augments.options,
                costs,
                ticks_left: self.augments.deadline.saturating_sub(self.clock),
                resolved: me.augment_resolved >= self.augments.round,
            }
        });
        MatchView {
            tick: self.tick,
            clock: self.clock,
            elapsed: self.tick.saturating_sub(RUNNING_START),
            phase: self.phase,
            phase_ticks_left: phase_ticks_left(self),
            you: seat,
            energy: me.energy,
            regen_bp: self.rules.interest.regen_bp(me.energy),
            offers: me.offers,
            stronghold_hp: [0, 1].map(|i| self.players[i].stronghold_hp),
            shields: [0, 1].map(|i| self.players[i].shield),
            units: self
                .units
                .iter()
                .filter(|u| u.hp > 0)
                .map(unit_view)
                .collect(),
            synergy: [0, 1].map(|i| self.players[i].synergy),
            augments: [self.players[0].augments, self.players[1].augments],
            augment_round: round,
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
