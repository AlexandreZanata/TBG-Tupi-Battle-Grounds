//! Authoritative hooks shared only by the seventeen champion-completion kits.

mod specials;
pub mod state;
mod strikers;
mod supports;
#[cfg(test)]
mod tests;

pub(crate) use specials::{
    death_bursts, special_period, special_primary, special_resolve, special_start, special_windup,
};
pub(crate) use strikers::{basic_bonus, completed, counter, lunar_bp, movement};
pub(crate) use supports::{after_damage, channeling, heal_action, prepare_heal, tick};

use crate::{content, Event, MatchState, Ruleset, Seat, Unit, UnitId};
pub(crate) use content::completion::Ability;

pub(crate) fn parameters(
    rules: &Ruleset,
    kind: crate::UnitKind,
    tier: u8,
    branch: u8,
) -> Option<Ability> {
    rules
        .def(kind)
        .ability(tier, branch)
        .into_iter()
        .find_map(|ability| match ability {
            content::Ability::Completion(value) => Some(value),
            _ => None,
        })
}

pub(crate) fn of(state: &MatchState, u: &Unit) -> Option<Ability> {
    parameters(state.rules(), u.kind, u.tier, u.branch)
}

pub(crate) fn cue(
    state: &MatchState,
    u: &Unit,
    target: Option<UnitId>,
    x: i32,
    ev: &mut Vec<Event>,
) {
    ev.push(Event::AbilityResolved {
        unit: u.id,
        kind: u.kind,
        primary: target,
        center: x,
        tick: state.tick,
    });
}

pub(crate) fn nearby(
    state: &MatchState,
    enemy: Seat,
    center: i32,
    radius: i32,
    cap: u8,
    exclude: Option<UnitId>,
) -> Vec<UnitId> {
    let mut targets: Vec<_> = state
        .units
        .iter()
        .filter(|v| {
            v.hp > 0
                && v.seat == enemy
                && Some(v.id) != exclude
                && crate::targeting::targetable(v)
                && (v.x - center).abs() <= radius
        })
        .map(|v| ((v.x - center).abs(), v.id))
        .collect();
    targets.sort_unstable();
    targets
        .into_iter()
        .take(usize::from(cap))
        .map(|(_, id)| id)
        .collect()
}

pub(crate) fn scaled(value: i32, bp: i32) -> i32 {
    (i64::from(value).saturating_mul(i64::from(bp)) / content::BP).clamp(0, i64::from(i32::MAX))
        as i32
}

pub(crate) fn hurt(state: &mut MatchState, u: &Unit, id: UnitId, amount: i32, ev: &mut Vec<Event>) {
    crate::training_hooks::scoped(
        state,
        crate::training_hooks::Source::Unit(u.id),
        crate::training_hooks::Effect::Splash,
        |state| crate::hits::auxiliary_hit(state, u, id, amount, ev),
    );
}
