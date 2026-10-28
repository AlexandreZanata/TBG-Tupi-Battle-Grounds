//! Shop offers: drawn from each seat's own RNG stream (BR-S05), refilled
//! on purchase, all rotated on one shared clock (BR-S02, BR-S04).

use crate::content::{ROTATION_TICKS, RUNNING_START};
use crate::rng::Pcg32;
use crate::state::MatchState;
use crate::types::{Tick, UnitKind, UNIT_KINDS};

pub(crate) fn draw(rng: &mut Pcg32) -> UnitKind {
    UNIT_KINDS[rng.below(UNIT_KINDS.len() as u32) as usize]
}

pub(crate) fn initial_offers(rng: &mut Pcg32) -> [UnitKind; 3] {
    [draw(rng), draw(rng), draw(rng)]
}

/// Redraw all offers of both seats (A first) on rotation ticks.
pub(crate) fn rotate(state: &mut MatchState) {
    let t = state.tick;
    if !state.is_live() || t <= RUNNING_START || !(t - RUNNING_START).is_multiple_of(ROTATION_TICKS)
    {
        return;
    }
    for p in &mut state.players {
        p.offers = initial_offers(&mut p.rng);
    }
}

/// Ticks until the next rotation.
pub fn next_rotation_in(tick: Tick) -> u32 {
    if tick < RUNNING_START {
        RUNNING_START + ROTATION_TICKS - tick
    } else {
        ROTATION_TICKS - (tick - RUNNING_START) % ROTATION_TICKS
    }
}
