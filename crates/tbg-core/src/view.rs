//! What one seat may see of a match. Opponent Energy, offers, and every
//! RNG stay inside `MatchState` (BR-S05).

use serde::{Deserialize, Serialize};

use crate::augment::AugmentCounts;
use crate::content::{AUGMENT_OPTIONS, OFFERS, RUNNING_START};
use crate::sim::phase_ticks_left;
use crate::state::{Event, MatchState, Outcome, Phase};
use crate::types::{Seat, Status, StrongholdKind, Tick, UnitId, UnitKind, FACTION_COUNT};

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
    /// Packed ability state (ROSTER-EXPANSION §F.4): poison 3 bits, fish 2,
    /// confused 1, ward 1, bonded 1, plates 3, growth 3, carried 1, 1 spare.
    /// Zero until the waves land, except the spare top bit: `FX_SUMMON`.
    pub fx: u16,
    /// The carrier this unit rides (D-61), set only while `FX_CARRIED` is:
    /// the client seats the passenger on that unit, not on a lane guess.
    pub carrier: Option<UnitId>,
    /// The unit shield still standing at the view's tick (D-63); zero once expired.
    pub shield: u16,
}

/// `UnitView.fx` bit for a summon by unit, e.g. Smuggler cargo (BR-U08):
/// its kind alone cannot tell, and the server refuses to evolve it.
pub const FX_SUMMON: u16 = 1 << 15;

/// `UnitView.fx` bit for a carried passenger, e.g. riding a Cargo Crab (D-56, §C.15).
pub const FX_CARRIED: u16 = 1 << 14;

impl UnitView {
    /// Summoned by kind (Beetling, Crate) or by unit (Smuggler cargo).
    pub fn is_summon(&self) -> bool {
        self.kind.is_summon() || self.fx & FX_SUMMON != 0
    }

    /// Carried by a carrier (e.g. riding a Cargo Crab).
    pub fn is_carried(&self) -> bool {
        self.fx & FX_CARRIED != 0
    }
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

/// One seat's tower as both seats see it (BR-SK08): kind, Fortification
/// and the current maximum HP.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct StrongholdView {
    pub kind: StrongholdKind,
    pub tier: u8,
    pub branch: u8,
    pub max_hp: i32,
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
    /// v15 (ARENAS-G): what evolving a tier-0 and a tier-1 unit costs this
    /// seat now (Augment and arena discounts applied).
    pub evolve_costs: [u32; 2],
    pub offers: [UnitKind; OFFERS],
    /// Indexed by `Seat::index()`.
    pub stronghold_hp: [i32; 2],
    /// Indexed by `Seat::index()`; public to both seats (BR-SK08).
    pub strongholds: [StrongholdView; 2],
    pub shields: [i32; 2],
    pub units: Vec<UnitView>,
    pub synergy: [[u8; FACTION_COUNT]; 2],
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
            evolve_costs: [0, 1].map(|tier| self.evolve_price(seat, tier)),
            offers: me.offers,
            stronghold_hp: [0, 1].map(|i| self.players[i].stronghold_hp),
            strongholds: Seat::BOTH.map(|s| StrongholdView {
                kind: self.player(s).stronghold,
                tier: self.player(s).fort_tier,
                branch: self.player(s).fort_branch,
                max_hp: self.stronghold(s).max_hp,
            }),
            shields: [0, 1].map(|i| self.players[i].shield),
            units: self
                .units
                .iter()
                .filter(|u| u.hp > 0)
                .map(|u| unit_view(u, self.tick))
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

fn unit_view(u: &crate::state::Unit, tick: Tick) -> UnitView {
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
        // D-56: carried bit packed for passenger presentation; other affliction bits pending.
        fx: {
            let mut fx = 0;
            if u.is_summon() {
                fx |= FX_SUMMON;
            }
            if u.fx.carrier.is_some() {
                fx |= FX_CARRIED;
            }
            fx
        },
        carrier: u.fx.carrier,
        shield: crate::roster_slots::shield_now(&u.fx, tick),
    }
}
