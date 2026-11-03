//! Per-player Shop with the 1v1 rules: five distinct offers from the shared
//! champion pool (BR-S01), a full redraw after every purchase (BR-S04),
//! locked slots kept across redraws, and a paid reroll (M-19).

use crate::content::OFFERS;
use crate::rng::Rng;
use crate::types::{Champion, POOL};

/// Redraws every unlocked slot with champions not held by a locked slot;
/// a partial Fisher-Yates keeps the draw distinct and seeded.
pub fn redraw(rng: &mut Rng, offers: &[Champion; OFFERS], locked: u8) -> [Champion; OFFERS] {
    let held = |i: usize| locked & (1 << i) != 0;
    let mut candidates: Vec<Champion> = POOL
        .iter()
        .copied()
        .filter(|k| !(0..OFFERS).any(|i| held(i) && offers[i] == *k))
        .collect();
    let mut result = *offers;
    let mut next = 0;
    for (i, slot) in result.iter_mut().enumerate() {
        if held(i) {
            continue;
        }
        let j = next + rng.below((candidates.len() - next) as u32) as usize;
        candidates.swap(next, j);
        *slot = candidates[next];
        next += 1;
    }
    result
}

/// The first offers of a seat: nothing locked yet.
pub fn opening(rng: &mut Rng) -> [Champion; OFFERS] {
    redraw(rng, &[POOL[0]; OFFERS], 0)
}
