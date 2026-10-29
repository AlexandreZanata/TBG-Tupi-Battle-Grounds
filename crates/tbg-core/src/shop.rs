//! Shop offers: drawn from each seat's own RNG stream (BR-S05). A purchase
//! redraws all five offers at random, the bought unit included (BR-S04);
//! there is no timed rotation (owner revision 2026-10-29).

use crate::content::OFFERS;
use crate::rng::Pcg32;
use crate::types::{UnitKind, UNIT_KINDS};

/// Five distinct units drawn uniformly from the whole roster (BR-S01): a
/// partial Fisher-Yates over the pool. Used for the first offers and for the
/// full redraw after every purchase, so the unit just bought may come back.
pub(crate) fn draw_offers(rng: &mut Pcg32) -> [UnitKind; OFFERS] {
    let mut pool = UNIT_KINDS;
    for i in 0..OFFERS {
        let j = i + rng.below((pool.len() - i) as u32) as usize;
        pool.swap(i, j);
    }
    std::array::from_fn(|i| pool[i])
}
