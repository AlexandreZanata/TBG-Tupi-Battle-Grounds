//! Offline balance-lab hooks (D-33), compiled only with the `lab` feature.
//! The server and the client never enable it (contract C10): online, units
//! enter the lane only through a validated purchase (BR-B02).

use crate::content::LANE_LEN;
use crate::state::MatchState;
use crate::types::{Seat, UnitId, UnitKind};

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
