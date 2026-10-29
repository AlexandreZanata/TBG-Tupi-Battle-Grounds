//! Shop offers: drawn from each seat's own RNG stream (BR-S05) and
//! changed only by a purchase, which refills that slot (BR-S04). There is
//! no timed rotation (owner revision 2026-10-29).

use crate::content::OFFERS;
use crate::rng::Pcg32;
use crate::types::{UnitKind, UNIT_KINDS};

/// Five distinct units (BR-S01): a partial Fisher-Yates over the pool.
pub(crate) fn initial_offers(rng: &mut Pcg32) -> [UnitKind; OFFERS] {
    let mut pool = UNIT_KINDS;
    for i in 0..OFFERS {
        let j = i + rng.below((pool.len() - i) as u32) as usize;
        pool.swap(i, j);
    }
    std::array::from_fn(|i| pool[i])
}

/// Refill a bought slot uniformly with a unit not on screen: neither the
/// four other offers nor the unit just bought (BR-S04).
pub(crate) fn refill(offers: &mut [UnitKind; OFFERS], slot: usize, rng: &mut Pcg32) {
    let hidden = |k: &&UnitKind| !offers.contains(k);
    let count = UNIT_KINDS.iter().filter(hidden).count();
    let pick = rng.below(count as u32) as usize;
    if let Some(&kind) = UNIT_KINDS.iter().filter(hidden).nth(pick) {
        offers[slot] = kind;
    }
}
