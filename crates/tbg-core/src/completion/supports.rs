//! Snapshot healing, non-stacking care effects and interruptible Nectar channels.

use super::{
    of, scaled,
    state::{Care, Channel, Regrowth},
    Ability,
};
use crate::{content::Stats, Event, MatchState, Status, Target, Unit, UnitKind};

pub(crate) fn prepare_heal(state: &mut MatchState, i: usize, target: Target, st: &Stats) {
    let u = state.units[i];
    let Some(Ability::Courage {
        heal_per_enemy,
        enemies,
        radius,
    }) = of(state, &u)
    else {
        return;
    };
    let Target::Unit(id) = target else { return };
    let Some(patient) = state.unit_index(id).map(|i| state.units[i]) else {
        return;
    };
    if patient.seat != u.seat {
        return;
    }
    let count = state
        .units
        .iter()
        .filter(|v| v.hp > 0 && v.seat != u.seat && (v.x - patient.x).abs() <= radius)
        .take(usize::from(enemies))
        .count();
    let bonus = healing_scaled(state, &u, st, i32::from(heal_per_enemy) * count as i32);
    state.units[i].fx.completion.prepared_heal = bonus;
}

fn healing_scaled(state: &MatchState, u: &Unit, st: &Stats, amount: i32) -> i32 {
    let base = state.rules().def(u.kind).heal.max(1);
    (i64::from(amount) * i64::from(st.heal) / i64::from(base)).clamp(0, i64::from(i32::MAX)) as i32
}

fn restore(state: &mut MatchState, u: &Unit, id: u32, amount: i32, ev: &mut Vec<Event>) {
    let _actual = crate::hits::heal(state, u.seat, id, amount, true, ev);
    #[cfg(feature = "lab")]
    crate::lab::healed(state, u, _actual);
}

pub(crate) fn heal_action(
    state: &mut MatchState,
    u: &Unit,
    id: u32,
    st: &Stats,
    ev: &mut Vec<Event>,
) -> bool {
    let Some(ability) = of(state, u) else {
        return false;
    };
    let Some(i) = state.unit_index(id) else {
        return true;
    };
    if state.units[i].hp <= 0 || state.units[i].seat != u.seat {
        return true;
    }
    match ability {
        Ability::Echo { heal, ticks } => {
            restore(state, u, id, st.heal, ev);
            let amount = healing_scaled(state, u, st, i32::from(heal));
            let old = state.units[i].fx.completion.echo;
            if state.tick >= old.until || amount >= old.amount {
                state.units[i].fx.completion.echo = Care {
                    amount,
                    until: state.tick.saturating_add(u32::from(ticks)),
                    seat: Some(u.seat),
                    producer: Some(u.kind),
                };
            }
        }
        Ability::Regrowth { heal, pulses, gap } => {
            restore(state, u, id, st.heal, ev);
            let amount = healing_scaled(state, u, st, i32::from(heal));
            let old = state.units[i].fx.completion.regrowth;
            if old.left == 0 || amount >= old.amount {
                state.units[i].fx.completion.regrowth = Regrowth {
                    amount,
                    next: if old.left == 0 {
                        state.tick.saturating_add(u32::from(gap))
                    } else {
                        old.next
                    },
                    left: pulses,
                    seat: Some(u.seat),
                    producer: Some(u.kind),
                    gap,
                };
            }
        }
        Ability::Courage { .. } => restore(
            state,
            u,
            id,
            st.heal.saturating_add(u.fx.completion.prepared_heal),
            ev,
        ),
        Ability::Channel {
            pulses,
            gap,
            first_bp,
        } => {
            start_channel(state, u, id, st.heal, (pulses, gap, first_bp), ev);
        }
        _ => return false,
    }
    true
}

fn start_channel(
    state: &mut MatchState,
    u: &Unit,
    id: u32,
    amount: i32,
    values: (u8, u16, u16),
    ev: &mut Vec<Event>,
) {
    let Some(i) = state.unit_index(u.id) else {
        return;
    };
    if state.units[i].fx.completion.channel.is_some() {
        return;
    }
    let (pulses, gap, first_bp) = values;
    restore(state, u, id, scaled(amount, i32::from(first_bp)), ev);
    if pulses > 1 {
        state.units[i].fx.completion.channel = Some(Channel {
            target: id,
            amount,
            next: state.tick.saturating_add(u32::from(gap)),
            left: pulses - 1,
            gap,
        });
    }
}

pub(crate) fn channeling(u: &Unit) -> bool {
    u.fx.completion.channel.is_some()
}

pub(crate) fn after_damage(state: &mut MatchState, id: u32, removed: i32, ev: &mut Vec<Event>) {
    if removed <= 0 {
        return;
    }
    let Some(i) = state.unit_index(id) else {
        return;
    };
    let u = state.units[i];
    state.units[i].fx.completion.regrowth = Default::default();
    if u.hp <= 0 {
        if u.kind == UnitKind::Blastbeetle {
            state.units[i].fx.completion.combat_death = true;
        }
        state.units[i].fx.completion.channel = None;
        return;
    }
    let echo = u.fx.completion.echo;
    if state.tick >= echo.until || echo.amount <= 0 {
        return;
    }
    state.units[i].fx.completion.echo = Default::default();
    if let Some(seat) = echo.seat {
        let _actual = crate::hits::heal(state, seat, id, echo.amount, true, ev);
        #[cfg(feature = "lab")]
        if let Some(kind) = echo.producer {
            crate::lab::healed_kind(state, seat, kind, _actual);
        }
    }
}

pub(crate) fn tick(state: &mut MatchState, ev: &mut Vec<Event>) {
    for i in 0..state.units.len() {
        let u = state.units[i];
        if u.hp <= 0 {
            state.units[i].fx.completion.channel = None;
            continue;
        }
        regrowth_tick(state, i, &u, ev);
        channel_tick(state, i, &u, ev);
        if u.status != Status::None && u.kind == UnitKind::Hushfang {
            state.units[i].fx.completion.charged = false;
            state.units[i].fx.completion.walk_ticks = 0;
        }
    }
}

fn regrowth_tick(state: &mut MatchState, i: usize, u: &Unit, ev: &mut Vec<Event>) {
    let mut regen = u.fx.completion.regrowth;
    if regen.left == 0 || state.tick < regen.next {
        return;
    }
    if let Some(seat) = regen.seat {
        let _actual = crate::hits::heal(state, seat, u.id, regen.amount, true, ev);
        #[cfg(feature = "lab")]
        if let Some(kind) = regen.producer {
            crate::lab::healed_kind(state, seat, kind, _actual);
        }
    }
    regen.left -= 1;
    regen.next = regen.next.saturating_add(u32::from(regen.gap));
    state.units[i].fx.completion.regrowth = if regen.left == 0 {
        Regrowth::default()
    } else {
        regen
    };
}

fn channel_tick(state: &mut MatchState, i: usize, u: &Unit, ev: &mut Vec<Event>) {
    let Some(mut channel) = u.fx.completion.channel else {
        return;
    };
    let valid = state.unit_index(channel.target).is_some_and(|j| {
        let v = state.units[j];
        v.hp > 0 && v.seat == u.seat && (v.x - u.x).abs() <= state.stats_of(u).range
    });
    if !valid || matches!(u.status, Status::Rooted { .. }) {
        state.units[i].fx.completion.channel = None;
        return;
    }
    if state.tick < channel.next {
        return;
    }
    restore(state, u, channel.target, channel.amount, ev);
    channel.left = channel.left.saturating_sub(1);
    channel.next = channel.next.saturating_add(u32::from(channel.gap));
    state.units[i].fx.completion.channel = (channel.left > 0).then_some(channel);
}
