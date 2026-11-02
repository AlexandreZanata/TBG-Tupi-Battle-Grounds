//! TW-01: Bastion armor and Artillery collateral, with authoritative kill attribution.
use crate::{content, Event, MatchState, Seat, StrongholdKind, Target, UnitId};
pub fn armor(kind: StrongholdKind, tier: u8, branch: u8) -> i32 {
    if kind != StrongholdKind::Bastion {
        0
    } else {
        content::BASTION_ARMOR_BP
            + if branch == 0 {
                i32::from(tier.min(2)) * content::BASTION_ARMOR_STEP_BP
            } else {
                0
            }
    }
}
pub fn blast(kind: StrongholdKind, tier: u8, branch: u8) -> (i32, i32) {
    if kind != StrongholdKind::Artillery {
        (0, 0)
    } else {
        (
            content::ARTILLERY_SPLASH_BP
                + if branch == 0 {
                    i32::from(tier.min(2)) * content::ARTILLERY_SPLASH_STEP_BP
                } else {
                    0
                },
            content::ARTILLERY_SPLASH_RADIUS,
        )
    }
}
pub(crate) fn collateral(
    s: &mut MatchState,
    seat: Seat,
    id: UnitId,
    amount: i32,
    ev: &mut Vec<Event>,
) {
    let p = s.player(seat);
    let (bp, radius) = blast(p.stronghold, p.fort_tier, p.fort_branch);
    if bp == 0 {
        return;
    }
    let Some(center) = s.unit_index(id).map(|i| s.units[i].x) else {
        return;
    };
    let mut ids: Vec<_> = s
        .units
        .iter()
        .filter(|u| u.hp > 0 && u.seat != seat && u.id != id && (u.x - center).abs() <= radius)
        .map(|u| ((u.x - center).abs(), u.id))
        .collect();
    ids.sort_unstable();
    for (_, id) in ids.into_iter().take(content::ARTILLERY_SPLASH_TARGETS) {
        let conventional = s.unit_index(id).is_some_and(|i| !s.units[i].is_summon());
        let removed = crate::hits::damage(
            s,
            Target::Unit(id),
            ((i64::from(amount) * i64::from(bp)) / content::BP) as i32,
            ev,
        );
        #[cfg(feature = "lab")]
        crate::lab::stronghold(s, seat, removed);
        if conventional && removed > 0 && s.unit_index(id).is_some_and(|i| s.units[i].hp <= 0) {
            killed(s, seat);
        }
        let _ = removed;
    }
}

pub(crate) fn killed(s: &mut MatchState, seat: Seat) {
    if let Some((max, ticks)) = crate::augment::tower_fury_rule(&s.player(seat).augments) {
        let until = s.tick.saturating_add(u32::from(ticks));
        let p = s.player_mut(seat);
        p.fury_stacks = p.fury_stacks.saturating_add(1).min(max);
        p.fury_until = until;
    }
}

#[cfg(all(test, feature = "lab"))]
mod tests;
