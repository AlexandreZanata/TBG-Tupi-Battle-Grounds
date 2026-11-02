//! GD-04 Astreon's Verdict Lance: consecutive same-enemy hits end in a capped max-HP pierce.
use crate::{
    content::{Ability, Stats},
    state::{Event, MatchState, Target, Unit},
    types::UnitKind,
};

#[derive(Clone, Copy)]
struct Lance {
    every: u8,
    max_hp_bp: u16,
    cap: u16,
    low_hp_bp: u16,
    execute_bp: u16,
}

fn parameters(s: &MatchState, u: &Unit) -> Option<Lance> {
    if u.kind != UnitKind::Astreon || u.is_summon() {
        return None;
    }
    s.rules()
        .def(u.kind)
        .ability(u.tier, u.branch)
        .into_iter()
        .find_map(|a| match a {
            Ability::VerdictLance {
                every,
                max_hp_bp,
                cap,
                low_hp_bp,
                execute_bp,
            } => Some(Lance {
                every: every.max(1),
                max_hp_bp,
                cap,
                low_hp_bp,
                execute_bp,
            }),
            _ => None,
        })
}

pub(crate) fn counter(u: &Unit) -> u8 {
    if u.kind == UnitKind::Astreon && !u.is_summon() && u.fx.mark_target.is_some() {
        u.fx.attacks.min(3)
    } else {
        0
    }
}

fn clear(u: &mut Unit) {
    u.fx.attacks = 0;
    u.fx.mark_target = None;
    u.fx.mark_until = 0;
}

/// A new windup on anything but the marked enemy restarts the sequence.
pub(crate) fn retarget(s: &mut MatchState, i: usize, target: Target) {
    let u = s.units[i];
    if parameters(s, &u).is_none() {
        return;
    }
    if !matches!(target, Target::Unit(id) if u.fx.mark_target == Some(id)) {
        clear(&mut s.units[i]);
    }
}

/// The pierce's victim index when this hit completes the sequence.
fn piercing(s: &MatchState, u: &Unit, target: Target, lance: &Lance) -> Option<usize> {
    let Target::Unit(id) = target else {
        return None;
    };
    let n = if u.fx.mark_target == Some(id) {
        counter(u).saturating_add(1)
    } else {
        1
    };
    s.unit_index(id)
        .filter(|&t| n >= lance.every && s.units[t].hp > 0 && s.units[t].seat != u.seat)
}

/// B1/B2 execution: a share of static basic damage on the pierce against a
/// target below the HP threshold, read at hit resolution (BR-X11).
pub(crate) fn bonus(s: &MatchState, u: &Unit, target: Target, st: &Stats) -> i32 {
    let Some(lance) = parameters(s, u) else {
        return 0;
    };
    let Some(t) = piercing(s, u, target, &lance) else {
        return 0;
    };
    let victim = &s.units[t];
    let low = i64::from(victim.hp) * 10_000 < i64::from(victim.max_hp) * i64::from(lance.low_hp_bp);
    if lance.execute_bp == 0 || !low {
        return 0;
    }
    (i64::from(st.damage) * i64::from(lance.execute_bp) / 10_000).clamp(0, i64::from(i32::MAX))
        as i32
}

/// The max-HP pierce, already capped: added after offensive percentages so the
/// cap holds exactly before damage reduction and shields apply.
pub(crate) fn pierce(s: &MatchState, u: &Unit, target: Target) -> i32 {
    let Some(lance) = parameters(s, u) else {
        return 0;
    };
    let Some(t) = piercing(s, u, target, &lance) else {
        return 0;
    };
    let share = i64::from(s.units[t].max_hp) * i64::from(lance.max_hp_bp) / 10_000;
    share.clamp(0, i64::from(lance.cap)) as i32
}

pub(crate) fn completed(s: &mut MatchState, u: &Unit, target: Target, ev: &mut Vec<Event>) {
    let Some(lance) = parameters(s, u) else {
        return;
    };
    let Some(i) = s.unit_index(u.id) else {
        return;
    };
    // Strongholds never build the pierce, and they end any sequence in progress.
    let Target::Unit(id) = target else {
        clear(&mut s.units[i]);
        return;
    };
    let Some(t) = s.unit_index(id).filter(|&t| s.units[t].seat != u.seat) else {
        return;
    };
    let n = if u.fx.mark_target == Some(id) {
        counter(u).saturating_add(1)
    } else {
        1
    };
    let center = s.units[t].x;
    if n >= lance.every || s.units[t].hp <= 0 {
        clear(&mut s.units[i]);
    } else {
        s.units[i].fx.attacks = n;
        s.units[i].fx.mark_target = Some(id);
        s.units[i].fx.mark_until = 0;
    }
    if n >= lance.every {
        ev.push(Event::AbilityResolved {
            unit: u.id,
            kind: u.kind,
            primary: Some(id),
            center,
            tick: s.tick,
        });
    }
}

#[cfg(all(test, feature = "lab"))]
mod tests;
