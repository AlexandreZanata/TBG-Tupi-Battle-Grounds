//! Bloodweaver's primary-hit healing of the lowest-percent conventional allies.

use crate::afflictions::abilities_of;
use crate::content::{Ability, BP};
use crate::state::{Event, MatchState, Unit};
use crate::types::UnitId;

pub(crate) fn apply(
    state: &mut MatchState,
    attacker: &Unit,
    enemy: UnitId,
    removed: i32,
    ev: &mut Vec<Event>,
) {
    if removed <= 0 {
        return;
    }
    let params = abilities_of(state, attacker)
        .into_iter()
        .find_map(|a| match a {
            Ability::Siphon {
                heal_bp,
                low_bp,
                split,
            } => Some((heal_bp, low_bp, split)),
            _ => None,
        });
    let Some((heal_bp, low_bp, split)) = params else {
        return;
    };
    let first = receiver(state, attacker, None);
    let second = if split {
        receiver(state, attacker, first)
    } else {
        None
    };
    let count = 1 + i32::from(second.is_some());
    let amount = (i64::from(removed) * i64::from(heal_bp) / BP) as i32 / count;
    for id in [first, second].into_iter().flatten() {
        let Some(i) = state.unit_index(id) else {
            continue;
        };
        let target = state.units[i];
        let double = i64::from(target.hp) * BP < i64::from(target.max_hp) * i64::from(low_bp);
        let requested = amount.saturating_mul(if double { 2 } else { 1 });
        crate::hits::heal(state, attacker.seat, id, requested, true, ev);
        let restored = state.units[i].hp - target.hp;
        if restored > 0 {
            ev.push(Event::Transfer {
                from: enemy,
                to: id,
                amount: restored,
            });
            #[cfg(feature = "lab")]
            crate::lab::healed(state, attacker, restored);
        }
    }
}

fn receiver(state: &MatchState, attacker: &Unit, exclude: Option<UnitId>) -> Option<UnitId> {
    state
        .units
        .iter()
        .filter(|u| {
            u.seat == attacker.seat
                && u.hp > 0
                && u.hp < u.max_hp
                && !u.is_summon()
                && Some(u.id) != exclude
        })
        .min_by(|a, b| {
            // Cross-products preserve exact HP fractions; rounded percentages can tie incorrectly.
            (i64::from(a.hp) * i64::from(b.max_hp))
                .cmp(&(i64::from(b.hp) * i64::from(a.max_hp)))
                .then_with(|| (a.x - attacker.x).abs().cmp(&(b.x - attacker.x).abs()))
                .then(a.id.cmp(&b.id))
        })
        .map(|u| u.id)
}
