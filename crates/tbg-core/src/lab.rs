//! Offline balance-lab hooks (D-33), compiled only with the `lab` feature.
//! The server and the client never enable it (contract C10): online, units
//! enter the lane only through a validated purchase (BR-B02).

use crate::content::{LANE_LEN, RUNNING_START, UNIT_KIND_COUNT};
use crate::state::{MatchState, Unit};
use crate::types::{Seat, Tick, UnitId, UnitKind};

impl MatchState {
    /// Place a unit of any form without the Shop or Energy, for unit duels.
    pub fn lab_spawn(&mut self, seat: Seat, kind: UnitKind, tier: u8, branch: u8) -> UnitId {
        self.spawn(seat, kind, tier.min(2), branch.min(1))
    }

    /// Place a unit at an exact lane position for an offline value scenario.
    /// Reuses `spawn`, so max HP, Synergy, and Hatch initialization are the
    /// same as a purchase; only `x` is overridden.
    pub fn lab_spawn_at(
        &mut self,
        seat: Seat,
        kind: UnitKind,
        tier: u8,
        branch: u8,
        x: i32,
    ) -> UnitId {
        let id = self.lab_spawn(seat, kind, tier, branch);
        if let Some(i) = self.unit_index(id) {
            self.units[i].x = x.clamp(0, LANE_LEN);
        }
        id
    }
}

/// Per-seat totals for every registered kind, including inactive prototypes. Never
/// serialized, so it is not part of `hash()`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LabStats {
    /// HP removed by hits of this seat's units of each kind (primary and splash).
    pub dealt: [[u64; UNIT_KIND_COUNT]; 2],
    /// HP removed from this seat's units of each kind, by any source.
    pub taken: [[u64; UNIT_KIND_COUNT]; 2],
    /// HP restored by this seat's units of each kind (direct, splash, lifesteal).
    pub healed: [[u64; UNIT_KIND_COUNT]; 2],
    /// Status ticks (slow or root) applied by this seat's units of each kind.
    pub status_ticks: [[u64; UNIT_KIND_COUNT]; 2],
    /// HP removed from units by each seat's Stronghold.
    pub stronghold_dealt: [u64; 2],
    /// Last combat tick with Stronghold damage or a conventional death.
    pub last_progress: Tick,
    /// Longest gap, in combat ticks, between two such ticks so far.
    pub longest_stall: Tick,
}

impl Default for LabStats {
    fn default() -> LabStats {
        LabStats {
            dealt: [[0; UNIT_KIND_COUNT]; 2],
            taken: [[0; UNIT_KIND_COUNT]; 2],
            healed: [[0; UNIT_KIND_COUNT]; 2],
            status_ticks: [[0; UNIT_KIND_COUNT]; 2],
            stronghold_dealt: [0; 2],
            last_progress: RUNNING_START,
            longest_stall: 0,
        }
    }
}

impl LabStats {
    /// Longest stall including the tail after the last progress tick.
    pub fn stall_at_end(&self, end: Tick) -> Tick {
        self.longest_stall
            .max(end.saturating_sub(self.last_progress))
    }
}

pub(crate) fn dealt(state: &mut MatchState, attacker: &Unit, amount: i32) {
    if amount > 0 {
        state.lab.dealt[attacker.seat.index()][attacker.kind as usize] += amount as u64;
    }
}

pub(crate) fn taken(state: &mut MatchState, owner: Seat, kind: UnitKind, amount: i32) {
    if amount > 0 {
        state.lab.taken[owner.index()][kind as usize] += amount as u64;
    }
}

pub(crate) fn healed(state: &mut MatchState, healer: &Unit, amount: i32) {
    if amount > 0 {
        state.lab.healed[healer.seat.index()][healer.kind as usize] += amount as u64;
    }
}

pub(crate) fn status(state: &mut MatchState, attacker: &Unit, ticks: u16) {
    if ticks > 0 {
        state.lab.status_ticks[attacker.seat.index()][attacker.kind as usize] += u64::from(ticks);
    }
}

pub(crate) fn stronghold(state: &mut MatchState, seat: Seat, amount: i32) {
    if amount > 0 {
        state.lab.stronghold_dealt[seat.index()] += amount as u64;
    }
}

pub(crate) fn progress(state: &mut MatchState) {
    let gap = state.tick.saturating_sub(state.lab.last_progress);
    state.lab.longest_stall = state.lab.longest_stall.max(gap);
    state.lab.last_progress = state.tick;
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{hits, ArcanaPreset, Target};

    #[test]
    fn staged_kinds_collect_lab_damage_without_public_pool_activation() {
        let mut s = MatchState::with_arcanas(1, [ArcanaPreset::NEUTRAL; 2]);
        let a = s.lab_spawn_at(Seat::A, UnitKind::Bonehorn, 0, 0, 4_000);
        let b = s.lab_spawn_at(Seat::B, UnitKind::Archer, 0, 0, 4_500);
        let attacker = s.units[s.unit_index(a).unwrap()];
        let stats = s.stats_of(&attacker);
        hits::resolve_hit(&mut s, &attacker, Target::Unit(b), &stats, &mut Vec::new());
        assert_eq!(
            s.lab.dealt[Seat::A.index()][UnitKind::Bonehorn as usize],
            40
        );
        hits::damage(&mut s, Target::Unit(a), 100, &mut Vec::new());
        assert_eq!(
            s.lab.taken[Seat::A.index()][UnitKind::Bonehorn as usize],
            100
        );
        assert!(!s.rules().pool.contains(&UnitKind::Bonehorn));
    }
}
