//! Shop offers: drawn from each seat's own RNG stream (BR-S05), refilled
//! on purchase, all rotated on one shared clock (BR-S02, BR-S04).

use crate::content::{OFFERS, ROTATION_TICKS, RUNNING_START};
use crate::rng::Pcg32;
use crate::state::MatchState;
use crate::types::{Tick, UnitKind, UNIT_KINDS};

/// Five distinct units (BR-S01): a partial Fisher-Yates over the pool.
pub(crate) fn initial_offers(rng: &mut Pcg32) -> [UnitKind; OFFERS] {
    let mut pool = UNIT_KINDS;
    for i in 0..OFFERS {
        let j = i + rng.below((pool.len() - i) as u32) as usize;
        pool.swap(i, j);
    }
    std::array::from_fn(|i| pool[i])
}

/// Refill one offer with a unit not shown in the other offers.
pub(crate) fn refill(offers: &mut [UnitKind; OFFERS], slot: usize, rng: &mut Pcg32) {
    let others = |k: &UnitKind| !offers.iter().enumerate().any(|(i, o)| i != slot && o == k);
    let count = UNIT_KINDS.iter().filter(|k| others(k)).count();
    let pick = rng.below(count as u32) as usize;
    if let Some(&kind) = UNIT_KINDS.iter().filter(|k| others(k)).nth(pick) {
        offers[slot] = kind;
    }
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
