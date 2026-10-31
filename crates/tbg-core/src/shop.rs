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

/// Preserve held slots; an unused feature consumes exactly the original RNG.
pub(crate) fn redraw(
    rng: &mut Pcg32,
    pool: &[UnitKind],
    offers: &[UnitKind; OFFERS],
    keep: u8,
) -> [UnitKind; OFFERS] {
    if keep == 0 {
        return draw_offers(rng, pool);
    }
    let held = |i: usize| keep & (1 << i) != 0;
    let mut candidates: Vec<_> = pool
        .iter()
        .copied()
        .filter(|k| !(0..OFFERS).any(|i| held(i) && offers[i] == *k))
        .collect();
    let mut result = *offers;
    let mut drawn = 0;
    for (i, result) in result.iter_mut().enumerate() {
        if held(i) {
            continue;
        }
        let j = drawn + rng.below((candidates.len() - drawn) as u32) as usize;
        candidates.swap(drawn, j);
        *result = candidates[drawn];
        drawn += 1;
    }
    result
}

pub(crate) fn new_shop(
    p: &mut crate::state::PlayerState,
    pool: &[UnitKind],
    bought: Option<usize>,
) {
    let keep = (0..OFFERS)
        .filter(|&i| p.reserve[i] > 0 && Some(i) != bought)
        .fold(0, |mask, i| mask | 1 << i);
    p.offers = redraw(&mut p.rng, pool, &p.offers, keep);
    for i in 0..OFFERS {
        if keep & (1 << i) != 0 {
            p.reserve[i] -= 1;
        } else {
            p.reserve[i] = 0;
            p.held_mask &= !(1 << i);
        }
    }
}

use crate::{MatchState, Rejection, Seat};

pub(crate) fn reserve(
    s: &mut MatchState,
    seat: Seat,
    offer: u8,
    kind: UnitKind,
) -> Result<(), Rejection> {
    let rule = s.rules().shop.ok_or(Rejection::Unavailable)?;
    let i = slot(offer)?;
    let p = &mut s.players[seat.index()];
    if p.offers[i] != kind {
        return Err(Rejection::StaleOffer);
    }
    if p.held_mask & (1 << i) != 0 {
        return Err(Rejection::AlreadyHeld);
    }
    if p.reserve.iter().filter(|&&n| n > 0).count() >= usize::from(rule.reserve_slots) {
        return Err(Rejection::ReserveFull);
    }
    p.reserve[i] = rule.reserve_shops;
    p.held_mask |= 1 << i;
    Ok(())
}

pub(crate) fn release(s: &mut MatchState, seat: Seat, offer: u8) -> Result<(), Rejection> {
    s.rules().shop.ok_or(Rejection::Unavailable)?;
    let i = slot(offer)?;
    let p = &mut s.players[seat.index()];
    if p.reserve[i] == 0 {
        return Err(Rejection::NoReserve);
    }
    p.reserve[i] = 0;
    Ok(())
}

fn slot(offer: u8) -> Result<usize, Rejection> {
    let i = usize::from(offer);
    if i >= OFFERS {
        Err(Rejection::BadOffer)
    } else {
        Ok(i)
    }
}

pub(crate) fn reroll_cost(s: &MatchState, seat: Seat) -> Option<u32> {
    let rule = s.rules().shop?;
    let cost = rule.reroll_base.saturating_add(
        rule.reroll_step
            .saturating_mul(u32::from(s.player(seat).reroll_streak)),
    );
    Some(rule.reroll_max.map_or(cost, |max| max.min(cost)))
}

pub(crate) fn reroll(s: &mut MatchState, seat: Seat, cost: u32) -> Result<(), Rejection> {
    let actual = reroll_cost(s, seat).ok_or(Rejection::Unavailable)?;
    if cost != actual {
        return Err(Rejection::StalePrice);
    }
    let pool = s.rules().pool;
    let p = &mut s.players[seat.index()];
    if !p.spend(cost) {
        return Err(Rejection::NotEnoughEnergy);
    }
    new_shop(p, pool, None);
    p.reroll_streak = p.reroll_streak.saturating_add(1);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::FULL_POOL;
    #[test]
    fn no_keep_preserves_rng_and_kept_slots_remain_distinct() {
        let mut a = Pcg32::new(1, 1);
        let mut b = a.clone();
        let offers = draw_offers(&mut a, &FULL_POOL);
        assert_eq!(offers, redraw(&mut b, &FULL_POOL, &offers, 0));
        assert_eq!(a, b);
        for keep in [1, 3] {
            for _ in 0..10_000 {
                let next = redraw(&mut a, &FULL_POOL, &offers, keep);
                for i in 0..OFFERS {
                    if keep & (1 << i) != 0 {
                        assert_eq!(next[i], offers[i]);
                    }
                    assert!(!next[i + 1..].contains(&next[i]));
                }
            }
        }
    }
}
