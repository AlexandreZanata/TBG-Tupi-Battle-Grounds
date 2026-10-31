//! Wooden training dummies (P62 §3.4): an inert Crate carrier tagged in the
//! session runtime, with its own HP. The tag makes the Crate never open,
//! never act and never rescale (`training_hooks::is_dummy` call sites);
//! summon rules already keep it out of Synergy, Bounty, lifesteal,
//! Essence, Arcanas and Augments.

use crate::state::MatchState;
use crate::types::{Seat, UnitId, UnitKind};

/// Finite targets are removed for good; Renewable ones respawn with a new
/// id on the next simulation tick and keep the trial running.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DummyMode {
    Finite,
    Renewable,
}

/// HP presets (GAME-DESIGN §16, **Proposed**, owner decision O02).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DummyHp {
    Light,
    Heavy,
}

impl DummyHp {
    pub fn hp(self) -> i32 {
        match self {
            DummyHp::Light => 1_000,
            DummyHp::Heavy => 10_000,
        }
    }
}

/// Place one tagged dummy on the target seat (B) at `x`.
pub(crate) fn spawn(state: &mut MatchState, x: i32, hp: i32) -> UnitId {
    let id = state.spawn(Seat::B, UnitKind::Crate, 0, 0);
    if let Some(rt) = state.training.as_mut() {
        rt.dummies.push(id);
    }
    if let Some(i) = state.unit_index(id) {
        let u = &mut state.units[i];
        u.x = x;
        u.hp = hp;
        u.max_hp = hp;
    }
    id
}
