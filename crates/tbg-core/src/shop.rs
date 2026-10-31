//! Shop offers: drawn from each seat's own RNG stream (BR-S05) over the
//! ruleset pool (ROSTER-EXPANSION §F.5). A purchase redraws all five offers
//! at random, the bought unit included (BR-S04); there is no timed rotation
//! (owner revision 2026-10-29).

use crate::content::OFFERS;
use crate::rng::Pcg32;
use crate::types::UnitKind;

/// Five distinct units drawn uniformly from the pool (BR-S01): a partial
/// Fisher-Yates over indices. Used for the first offers and for the full
/// redraw after every purchase, so the unit just bought may come back.
pub(crate) fn draw_offers(rng: &mut Pcg32, pool: &[UnitKind]) -> [UnitKind; OFFERS] {
    debug_assert!(pool.len() >= OFFERS);
    // One small index vector per draw: purchases are player-paced, never hot.
    let mut idx: Vec<usize> = (0..pool.len()).collect();
    for i in 0..OFFERS {
        let j = i + rng.below((pool.len() - i) as u32) as usize;
        idx.swap(i, j);
    }
    std::array::from_fn(|i| pool[idx[i]])
}
