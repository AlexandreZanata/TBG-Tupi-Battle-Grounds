//! Offline balance-lab hooks (D-33), compiled only with the `lab` feature.
//! The server and the client never enable it (contract C10): online, units
//! enter the lane only through a validated purchase (BR-B02).

use crate::state::MatchState;
use crate::types::{Seat, UnitId, UnitKind};

impl MatchState {
    /// Place a unit of any form without the Shop or Energy, for unit duels.
    pub fn lab_spawn(&mut self, seat: Seat, kind: UnitKind, tier: u8, branch: u8) -> UnitId {
        self.spawn(seat, kind, tier.min(2), branch.min(1))
    }
}
