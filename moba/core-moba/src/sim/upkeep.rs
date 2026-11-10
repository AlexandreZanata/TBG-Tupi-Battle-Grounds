//! Passive per-Seat Gold and Essence income.

use crate::content::{self, TICK_HZ};
use crate::state::MobaMatch;

pub(super) fn income(m: &mut MobaMatch) {
    let gold = if m.tick.is_multiple_of(TICK_HZ) {
        content::GOLD_PER_SECOND
    } else {
        0
    };
    let essence = u32::from(m.tick.is_multiple_of(content::ESSENCE_EVERY_TICKS));
    if gold == 0 && essence == 0 {
        return;
    }
    for p in &mut m.players {
        p.gold += gold;
        p.gold_earned += gold;
        p.essence += essence;
    }
}
