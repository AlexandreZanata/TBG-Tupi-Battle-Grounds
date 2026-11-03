//! Three deterministic role-locked Shop slots, purchase redraws, locks and rerolls (M-19/M-20).

use crate::content::OFFERS;
use crate::rng::Rng;
use crate::types::{Champion, POOL};
use tbg_core::types::Archetype;

/// Partial Fisher-Yates in slot order, excluding held champions and earlier draws.
pub fn redraw(
    rng: &mut Rng,
    offers: &[Champion; OFFERS],
    locked: u8,
    rules: &[&[Archetype]; OFFERS],
) -> [Champion; OFFERS] {
    let held = |i: usize| locked & (1 << i) != 0;
    let mut result = *offers;
    for i in 0..OFFERS {
        if held(i) {
            continue;
        }
        let mut candidates: Vec<Champion> = POOL
            .iter()
            .copied()
            .filter(|kind| {
                rules[i].contains(&tbg_core::content::def(*kind).archetype)
                    && !(0..OFFERS).any(|j| (held(j) || j < i) && result[j] == *kind)
            })
            .collect();
        // M-20's fixed sets always contain more than three champions.
        let j = rng.below(candidates.len() as u32) as usize;
        candidates.swap(0, j);
        result[i] = candidates[0];
    }
    result
}

pub fn opening(rng: &mut Rng, rules: &[&[Archetype]; OFFERS]) -> [Champion; OFFERS] {
    redraw(rng, &[POOL[0]; OFFERS], 0, rules)
}
