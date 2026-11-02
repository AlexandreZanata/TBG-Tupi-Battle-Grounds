//! Unit-only finisher, charge, shared-spark and completed-hit striker mechanics.

use super::{cue, nearby, of, scaled, Ability};
use crate::{content::Stats, Event, MatchState, Status, Target, Unit, UnitKind};

pub(crate) fn basic_bonus(state: &mut MatchState, u: &Unit, target: Target, st: &Stats) -> i32 {
    let Target::Unit(id) = target else { return 0 };
    let Some(i) = state.unit_index(id) else {
        return 0;
    };
    let victim = state.units[i];
    if victim.hp <= 0 || victim.seat == u.seat || u.is_summon() {
        return 0;
    }
    let own = match of(state, u) {
        Some(Ability::Finisher { below_bp, bonus_bp }) => {
            if i64::from(victim.hp) * 10_000 < i64::from(victim.max_hp) * i64::from(below_bp) {
                scaled(st.damage, i32::from(bonus_bp))
            } else {
                0
            }
        }
        Some(Ability::Pounce { bonus_bp, .. }) => pounce(state, u, st, bonus_bp),
        Some(Ability::Eclipse {
            high_bp, solar_bp, ..
        }) => solar(u, &victim, st, high_bp, solar_bp),
        Some(Ability::Crusher { max_hp_bp, cap }) => {
            let hp = state.rules().def(victim.kind).hp;
            crate::hits::auxiliary_amount(
                st,
                scaled(hp, i32::from(max_hp_bp)).min(i32::from(cap)) as u16,
            )
        }
        Some(Ability::Opportunity { every, bonus_bp }) => {
            opportunity(state, u, &victim, st, every, bonus_bp)
        }
        _ => 0,
    };
    own.saturating_add(consume_spark(state, u, i))
}

fn pounce(state: &mut MatchState, u: &Unit, st: &Stats, bonus: u16) -> i32 {
    let Some(i) = state.unit_index(u.id) else {
        return 0;
    };
    let fx = &mut state.units[i].fx.completion;
    if !fx.charged {
        return 0;
    }
    fx.charged = false;
    fx.walk_ticks = 0;
    scaled(st.damage, i32::from(bonus))
}

fn solar(u: &Unit, victim: &Unit, st: &Stats, high: u16, bonus: u16) -> i32 {
    if u.fx.attacks.is_multiple_of(2)
        && i64::from(victim.hp) * 10_000 >= i64::from(victim.max_hp) * i64::from(high)
    {
        scaled(st.damage, i32::from(bonus))
    } else {
        0
    }
}

fn opportunity(
    state: &MatchState,
    u: &Unit,
    victim: &Unit,
    st: &Stats,
    every: u8,
    bonus: u16,
) -> i32 {
    if u.fx.attacks.saturating_add(1) < every {
        return 0;
    }
    let Some((target, _)) = victim.windup else {
        return 0;
    };
    let hostile = match target {
        Target::Stronghold(seat) => seat != victim.seat,
        Target::Unit(id) => state
            .unit_index(id)
            .is_some_and(|i| state.units[i].seat != victim.seat),
    };
    if hostile && crate::targeting::alive(state, target) {
        scaled(st.damage, i32::from(bonus))
    } else {
        0
    }
}

fn consume_spark(state: &mut MatchState, u: &Unit, victim: usize) -> i32 {
    let spark = state.units[victim].fx.completion.spark;
    if state.rules().def(u.kind).range > crate::content::completion::SPARK_MELEE_RANGE
        || spark.until <= state.tick
        || spark.source == u.id
        || spark.amount <= 0
    {
        return 0;
    }
    state.units[victim].fx.completion.spark = Default::default();
    spark.amount
}

pub(crate) fn lunar_bp(state: &MatchState, u: &Unit, target: Target) -> i32 {
    let Some(Ability::Eclipse {
        high_bp, lunar_bp, ..
    }) = of(state, u)
    else {
        return 0;
    };
    if u.fx.attacks.is_multiple_of(2) {
        return 0;
    }
    let Target::Unit(id) = target else { return 0 };
    let Some(i) = state.unit_index(id) else {
        return 0;
    };
    let v = state.units[i];
    if v.seat != u.seat
        && !v.is_summon()
        && i64::from(v.hp) * 10_000 < i64::from(v.max_hp) * i64::from(high_bp)
    {
        i32::from(lunar_bp)
    } else {
        0
    }
}

fn counted(state: &mut MatchState, u: &Unit, every: u8) -> bool {
    let Some(i) = state.unit_index(u.id) else {
        return false;
    };
    let n = u.fx.attacks.saturating_add(1);
    state.units[i].fx.attacks = if n >= every { 0 } else { n };
    n >= every
}

pub(crate) fn completed(
    state: &mut MatchState,
    u: &Unit,
    target: Target,
    st: &Stats,
    ev: &mut Vec<Event>,
) {
    let Target::Unit(id) = target else { return };
    let Some(i) = state.unit_index(id) else {
        return;
    };
    let victim = state.units[i];
    if victim.seat == u.seat || u.is_summon() {
        return;
    }
    match of(state, u) {
        Some(Ability::Sweep {
            every,
            damage,
            radius,
            targets,
        }) => {
            if counted(state, u, every) {
                sweep(state, u, victim, st, (damage, radius, targets), ev);
            }
        }
        Some(Ability::Pierce {
            every,
            ratio_bp,
            behind,
        }) => {
            if counted(state, u, every) {
                pierce(state, u, victim, st, ratio_bp, behind, ev);
            }
        }
        Some(Ability::Spark {
            every,
            damage,
            ticks,
        }) => {
            if counted(state, u, every) {
                spark(state, u, i, st, damage, ticks);
            }
        }
        Some(Ability::Eclipse { .. }) => {
            counted(state, u, 2);
        }
        Some(Ability::Opportunity { every, .. }) => {
            counted(state, u, every);
        }
        Some(Ability::Bleeder { damage, pulses }) => bleed(state, u, i, st, damage, pulses),
        _ => {}
    }
}

fn sweep(
    state: &mut MatchState,
    u: &Unit,
    victim: Unit,
    st: &Stats,
    values: (u16, i32, u8),
    ev: &mut Vec<Event>,
) {
    let (damage, radius, cap) = values;
    let amount = crate::hits::auxiliary_amount(st, damage);
    for id in nearby(
        state,
        u.seat.other(),
        victim.x,
        radius,
        cap,
        Some(victim.id),
    ) {
        super::hurt(state, u, id, amount, ev);
    }
    cue(state, u, Some(victim.id), victim.x, ev);
}

fn pierce(
    state: &mut MatchState,
    u: &Unit,
    victim: Unit,
    st: &Stats,
    ratio: u16,
    behind: i32,
    ev: &mut Vec<Event>,
) {
    let candidate = state
        .units
        .iter()
        .filter(|v| {
            v.hp > 0 && v.seat != u.seat && v.id != victim.id && crate::targeting::targetable(v)
        })
        .filter_map(|v| {
            let dx = (v.x - victim.x) * u.seat.dir();
            (dx >= 0 && dx <= behind).then_some((dx, v.id))
        })
        .min();
    if let Some((_, id)) = candidate {
        super::hurt(state, u, id, scaled(st.damage, i32::from(ratio)), ev);
        cue(state, u, Some(id), victim.x, ev);
    }
}

fn spark(state: &mut MatchState, u: &Unit, i: usize, st: &Stats, damage: u16, ticks: u16) {
    let v = state.units[i];
    if v.hp <= 0 || state.tick < v.fx.ward_until {
        return;
    }
    let amount = crate::hits::auxiliary_amount(st, damage);
    let now = state.tick;
    let old = v.fx.completion.spark;
    if now >= old.until || amount >= old.amount {
        state.units[i].fx.completion.spark = super::state::Spark {
            amount,
            until: now.saturating_add(u32::from(ticks)),
            source: u.id,
        };
    }
}

fn bleed(state: &mut MatchState, u: &Unit, i: usize, st: &Stats, damage: u16, pulses: u8) {
    let v = state.units[i];
    if v.hp <= 0 || state.tick < v.fx.ward_until {
        return;
    }
    let damage = crate::hits::auxiliary_amount(st, damage).clamp(0, i32::from(u16::MAX)) as u16;
    let accepted = v.fx.bleed.left == 0 || damage >= v.fx.bleed.value;
    crate::roster_slots::apply_bleed(&mut state.units[i].fx, damage, pulses, u.seat, state.tick);
    if accepted {
        state.units[i].fx.bleed.producer = Some(u.kind);
    }
}

pub(crate) fn movement(state: &mut MatchState, i: usize, before: i32) {
    let u = state.units[i];
    let Some(Ability::Pounce { walk_ticks, .. }) = of(state, &u) else {
        return;
    };
    let fx = &mut state.units[i].fx.completion;
    if u.status != Status::None {
        fx.walk_ticks = 0;
        fx.charged = false;
    } else if u.x != before {
        fx.walk_ticks = fx.walk_ticks.saturating_add(1);
        fx.charged |= fx.walk_ticks >= walk_ticks;
    }
}

pub(crate) fn counter(u: &Unit) -> u8 {
    if matches!(
        u.kind,
        UnitKind::Cragclaw
            | UnitKind::Spinehurler
            | UnitKind::Sunlancer
            | UnitKind::EclipseDancer
            | UnitKind::Needlewing
    ) {
        u.fx.attacks.min(7)
    } else {
        0
    }
}
