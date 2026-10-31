//! The spectator's view: what anyone watching a live match may know. It is
//! a pure function of `MatchState`, so one encode serves every spectator
//! (BR-LV03, ARCHITECTURE §5). Private fields never appear in it: no Energy,
//! regen, Shop offers, `last_seq`, Evolution prices, RNG, or a pending pick.

use serde::{Deserialize, Serialize};

use crate::augment::AugmentCounts;
use crate::content::AUGMENT_OPTIONS;
use crate::sim::phase_ticks_left;
use crate::state::{Event, MatchState, Outcome, Phase};
use crate::types::{Arena, Seat, Tick, FACTION_COUNT};

use super::visibility::is_public;
use super::{StrongholdView, UnitView};

/// The open Augment round as both seats see it. Options are public; each
/// seat's `resolved` flag is public, its choice is not (until resolution).
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct PublicRound {
    pub round: u32,
    pub options: [u8; AUGMENT_OPTIONS],
    pub costs: [u32; AUGMENT_OPTIONS],
    /// Selection-clock steps until the round closes (pauses count).
    pub ticks_left: u32,
    /// Indexed by `Seat::index()`.
    pub resolved: [bool; 2],
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct PublicView {
    /// Combat tick: frozen while an Augment selection pauses the battle.
    pub tick: Tick,
    /// Selection clock: advances every step, pauses included (BR-AG13).
    pub clock: Tick,
    /// Ticks since Running began (the open-ended match clock).
    pub elapsed: u32,
    pub phase: Phase,
    /// Countdown only; 0 while running (no gameplay time limit).
    pub phase_ticks_left: u32,
    /// The arena this match was drawn into (D-55); the spectator scene uses it.
    pub arena: Arena,
    /// Indexed by `Seat::index()`.
    pub stronghold_hp: [i32; 2],
    /// Indexed by `Seat::index()`; public to both seats (BR-SK08).
    pub strongholds: [StrongholdView; 2],
    pub shields: [i32; 2],
    pub units: Vec<UnitView>,
    pub synergy: [[u8; FACTION_COUNT]; 2],
    /// Both seats' Augment copies (public, BR-AG09).
    pub augments: [AugmentCounts; 2],
    /// Present exactly while the battle is paused for a selection.
    pub round: Option<PublicRound>,
    pub events: Vec<Event>,
    pub outcome: Option<Outcome>,
    /// BR-LV05: a count of spectators, never who. Set by the server.
    pub spectators: u16,
}

impl MatchState {
    /// The spectator view of this match at this moment. `events` are the
    /// events since the last view; only the public ones are kept.
    pub fn public_view(&self, events: &[Event], spectators: u16) -> PublicView {
        let round = (self.is_live() && self.augments.selecting()).then(|| PublicRound {
            round: self.augments.round,
            options: self.augments.options,
            costs: self.round_costs(),
            ticks_left: self.augments.deadline.saturating_sub(self.clock),
            resolved: Seat::BOTH.map(|s| self.player(s).augment_resolved >= self.augments.round),
        });
        PublicView {
            tick: self.tick,
            clock: self.clock,
            elapsed: self.tick.saturating_sub(crate::content::RUNNING_START),
            phase: self.phase,
            phase_ticks_left: phase_ticks_left(self),
            arena: self.arena,
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
                .map(|u| super::unit_view(u, self.tick))
                .collect(),
            synergy: [0, 1].map(|i| self.players[i].synergy),
            augments: [self.players[0].augments, self.players[1].augments],
            round,
            events: events.iter().filter(|e| is_public(e)).cloned().collect(),
            outcome: self.outcome,
            spectators,
        }
    }
}
