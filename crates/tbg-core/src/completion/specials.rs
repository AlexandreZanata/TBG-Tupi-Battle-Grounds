//! Locked interrupt/beam/rhythm casts and finite combat-only death bursts.

use super::{cue, nearby, of, parameters, Ability};
use crate::{
    content::{Stats, WINDUP},
    Event, MatchState, OnHit, Ruleset, Target, Unit, UnitId, UnitKind,
};

pub(crate) fn special_period(rules: &Ruleset, kind: UnitKind, tier: u8, branch: u8) -> Option<u16> {
    match parameters(rules, kind, tier, branch) {
        Some(
            Ability::Interrupt { period, .. }
            | Ability::Beam { period, .. }
            | Ability::Rhythm { period, .. },
        ) => Some(period),
        _ => None,
    }
}

pub(crate) fn special_windup(rules: &Ruleset, u: &Unit) -> u16 {
    match parameters(rules, u.kind, u.tier, u.branch) {
        Some(Ability::Interrupt { windup, .. }) => windup,
        _ => WINDUP,
    }
}

pub(crate) fn special_primary(state: &MatchState, u: &Unit) -> Option<Option<UnitId>> {
    if !matches!(of(state, u), Some(Ability::Interrupt { .. })) {
        return None;
    }
    let target = crate::targeting::enemy_in_range(state, u, state.stats_of(u).range);
    Some(match target {
        Some(Target::Unit(id)) => state
            .unit_index(id)
            .filter(|&i| interruptible(state, &state.units[i]))
            .map(|_| id),
        _ => None,
    })
}

fn interruptible(state: &MatchState, v: &Unit) -> bool {
    v.hp > 0
        && v.fx.special.is_some()
        && state.tick >= v.fx.completion.interrupt_until
        && state.tick >= v.fx.ward_until
        && !crate::afflictions::is_immune_to_control(v)
}

pub(crate) fn special_start(state: &mut MatchState, i: usize, center: i32) {
    if matches!(of(state, &state.units[i]), Some(Ability::Beam { .. })) {
        state.units[i].fx.completion.beam_center = center;
    }
}

pub(crate) fn special_resolve(
    state: &mut MatchState,
    u: &Unit,
    target: UnitId,
    ev: &mut Vec<Event>,
) {
    match of(state, u) {
        Some(Ability::Interrupt { lock_ticks, .. }) => {
            if let Some(i) = state
                .unit_index(target)
                .filter(|&i| interruptible(state, &state.units[i]))
            {
                state.units[i].fx.special = None;
                state.units[i].fx.completion.interrupt_until =
                    state.tick.saturating_add(u32::from(lock_ticks));
            }
        }
        Some(Ability::Beam {
            damage,
            radius,
            targets,
            ..
        }) => {
            let center = u.fx.completion.beam_center;
            let st = state.stats_of(u);
            let amount = crate::hits::auxiliary_amount(&st, damage);
            for id in nearby(state, u.seat.other(), center, radius, targets, None) {
                super::hurt(state, u, id, amount, ev);
            }
        }
        Some(Ability::Rhythm {
            bp,
            ticks,
            radius,
            targets,
            ..
        }) => {
            let center = state.unit_index(target).map_or(u.x, |i| state.units[i].x);
            for id in nearby(state, u.seat.other(), center, radius, targets, None) {
                let _duration =
                    crate::hits::apply_on_hit(state, id, OnHit::Slow { bp, ticks }, u.seat, ev);
                #[cfg(feature = "lab")]
                crate::lab::status(state, u, _duration);
            }
        }
        _ => {}
    }
}

pub(crate) fn death_bursts(state: &mut MatchState, ev: &mut Vec<Event>) -> bool {
    let dead: Vec<_> = state
        .units
        .iter()
        .filter(|u| {
            u.hp <= 0
                && u.kind == UnitKind::Blastbeetle
                && u.fx.completion.combat_death
                && !u.fx.completion.burst_suppressed
        })
        .copied()
        .collect();
    for u in &dead {
        if let Some(i) = state.unit_index(u.id) {
            state.units[i].fx.completion.combat_death = false;
        }
        pop(state, u, ev);
    }
    !dead.is_empty()
}

fn pop(state: &mut MatchState, u: &Unit, ev: &mut Vec<Event>) {
    let Some(Ability::DeathPop {
        damage,
        radius,
        targets,
    }) = of(state, u)
    else {
        return;
    };
    let st: Stats = state.stats_of(u);
    let amount = crate::hits::auxiliary_amount(&st, damage);
    for id in nearby(state, u.seat.other(), u.x, radius, targets, None) {
        super::hurt(state, u, id, amount, ev);
        if let Some(i) = state.unit_index(id) {
            if state.units[i].kind == UnitKind::Blastbeetle && state.units[i].hp <= 0 {
                state.units[i].fx.completion.burst_suppressed = true;
            }
        }
    }
    cue(state, u, None, u.x, ev);
}
