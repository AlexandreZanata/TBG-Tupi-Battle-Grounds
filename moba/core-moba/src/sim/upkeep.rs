//! Passive income and fountain healing.

use crate::content::{self, TICK_HZ};
use crate::map;
use crate::state::MobaMatch;
use crate::types::{Side, UnitKind};

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

/// Commanders near their own Stronghold heal quickly (M-22).
pub(super) fn fountain(m: &mut MobaMatch) {
    for u in &mut m.units {
        let (UnitKind::Commander(_), Side::Team(team)) = (u.kind, u.side) else {
            continue;
        };
        if u.alive()
            && u.hp < u.max_hp
            && u.pos.within(map::stronghold(team), content::FOUNTAIN_RANGE)
        {
            u.hp = (u.hp + (u.max_hp / content::FOUNTAIN_DIVISOR).max(1)).min(u.max_hp);
        }
    }
}
