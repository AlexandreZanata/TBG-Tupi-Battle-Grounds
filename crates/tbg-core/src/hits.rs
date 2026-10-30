//! Resolving one hit: damage or heal, splash, situational Augment damage,
//! damage reduction, lifesteal, heal cut, Second Wind, shields and on-hit
//! statuses (GAME-DESIGN §4, §10.7).

use crate::arcana;
use crate::augment;
use crate::content::{Stats, BP};
use crate::state::{stronghold_x, Event, MatchState, Target, Unit};
use crate::targeting;
use crate::types::{OnHit, Seat, Status};

pub(crate) fn resolve_hit(
    state: &mut MatchState,
    attacker: &Unit,
    target: Target,
    st: &Stats,
    ev: &mut Vec<Event>,
) {
    let tx = targeting::target_x(state, target);
    let healing = matches!(target, Target::Unit(id)
        if state.unit_index(id).is_some_and(|i| state.units[i].seat == attacker.seat));
    if healing {
        let Target::Unit(id) = target else { return };
        heal(state, attacker.seat, id, st.heal, true, ev);
        if st.splash > 0 {
            // Index loop, no clone: heals never remove units mid-loop.
            for i in 0..state.units.len() {
                let a = state.units[i];
                let near = (a.x - tx).abs() <= st.splash;
                if a.seat == attacker.seat && a.id != id && a.hp > 0 && a.hp < a.max_hp && near {
                    heal(state, attacker.seat, a.id, st.heal, true, ev);
                }
            }
        }
        return;
    }
    let mut amount = st.damage;
    if !attacker.kind.is_summon() {
        let target_status = match target {
            Target::Unit(id) => state
                .unit_index(id)
                .map_or(Status::None, |i| state.units[i].status),
            Target::Stronghold(_) => Status::None,
        };
        let army = state.army_of(attacker.seat);
        let arcana_bp = arcana::damage_bonus_bp(
            &state.player(attacker.seat).arcanas,
            &state.rules().arcana,
            state.rules().def(attacker.kind).archetype,
            target_status,
            army.tanks > 0,
        );
        amount = scale(
            amount,
            augment::situational_bp(state, attacker, target)
                + arcana_bp
                + crate::stronghold::aura_bp(state, attacker),
        );
    }
    let removed = damage(state, target, amount, ev);
    if let Target::Unit(id) = target {
        if !attacker.kind.is_summon() {
            apply_heal_cut(state, attacker.seat, id);
        }
    }
    // Lifesteal: primary hit on an enemy conventional unit only (BR-X06).
    if !attacker.kind.is_summon() && removed > 0 {
        if let Target::Unit(id) = target {
            let conventional = state.unit_index(id).is_some_and(|i| {
                !state.units[i].kind.is_summon() && state.units[i].seat != attacker.seat
            });
            let bp = augment::lifesteal_bp(&state.player(attacker.seat).augments);
            if conventional && bp > 0 {
                let healed = (i64::from(removed) * i64::from(bp) / BP) as i32;
                heal(state, attacker.seat, attacker.id, healed, false, ev);
            }
        }
    }
    if let (Target::Unit(id), Some(on_hit)) = (target, st.on_hit) {
        apply_on_hit(state, id, on_hit, attacker.seat, ev);
    }
    if st.splash > 0 {
        splash(state, attacker, target, tx, st, ev);
    }
}

fn scale(value: i32, bp: i32) -> i32 {
    if bp == 0 {
        return value;
    }
    let out = i64::from(value) * (BP + i64::from(bp)) / BP;
    out.clamp(0, i64::from(i32::MAX)) as i32
}

fn apply_heal_cut(state: &mut MatchState, attacker: Seat, id: u32) {
    let bp = augment::heal_cut_bp(state, attacker);
    if bp <= 0 {
        return;
    }
    let ticks = augment::heal_cut_ticks(state, attacker);
    let until = state.tick.saturating_add(u32::from(ticks));
    if let Some(i) = state.unit_index(id) {
        state.units[i].heal_cut_until = until;
        state.units[i].heal_cut_bp = bp;
    }
}

/// Damage every other enemy (units and Stronghold) within the radius.
fn splash(
    state: &mut MatchState,
    attacker: &Unit,
    primary: Target,
    tx: i32,
    st: &Stats,
    ev: &mut Vec<Event>,
) {
    let enemy = attacker.seat.other();
    // Index loop, no clone: dead units are only removed in `remove_dead`.
    for i in 0..state.units.len() {
        let e = state.units[i];
        let near = (e.x - tx).abs() <= st.splash;
        if e.seat == enemy && e.hp > 0 && near && primary != Target::Unit(e.id) {
            damage(state, Target::Unit(e.id), st.damage, ev);
            if !attacker.kind.is_summon() {
                apply_heal_cut(state, attacker.seat, e.id);
            }
        }
    }
    let base = Target::Stronghold(enemy);
    if primary != base && (stronghold_x(enemy) - tx).abs() <= st.splash {
        damage(state, base, st.damage, ev);
    }
}

/// Apply damage after the target owner's Augment reduction and shield;
/// returns the HP actually removed.
pub(crate) fn damage(
    state: &mut MatchState,
    target: Target,
    amount: i32,
    ev: &mut Vec<Event>,
) -> i32 {
    let amount = amount.max(0);
    match target {
        Target::Unit(id) => {
            let Some(i) = state.unit_index(id) else {
                return 0;
            };
            let victim = state.units[i];
            let owner = victim.seat;
            let conventional = !victim.kind.is_summon();
            let archetype = state.rules().def(victim.kind).archetype;
            let army = state.army_of(owner);
            let arcana_dr = if conventional {
                arcana::damage_reduction_bp(
                    &state.player(owner).arcanas,
                    &state.rules().arcana,
                    archetype,
                    army.tanks > 0,
                    army.dps > 0,
                )
            } else {
                0
            };
            let dr = (augment::damage_reduction_bp(state, &victim) + arcana_dr).min(2_000);
            let amount = if dr > 0 {
                let reduced = i64::from(amount) * (BP - i64::from(dr)) / BP;
                if amount >= 1 {
                    (reduced as i32).max(1)
                } else {
                    0
                }
            } else {
                amount
            };
            let copies = augment::second_wind_charges(&state.player(owner).augments);
            let u = &mut state.units[i];
            if u.hp <= 0 {
                return 0;
            }
            let before = u.hp;
            if !u.kind.is_summon() && amount >= before && u.second_wind_used < copies {
                u.hp = 1;
                u.second_wind_used = u.second_wind_used.saturating_add(1);
                ev.push(Event::SecondWind { unit: id });
            } else {
                u.hp = (u.hp - amount).max(0);
            }
            let removed = before - u.hp;
            ev.push(Event::Hit { to: target, amount });
            if removed > 0 && u.hp <= 0 && conventional {
                state.army[owner.index()].remove(archetype);
            }
            removed
        }
        Target::Stronghold(seat) => {
            let tick = state.tick;
            let p = state.player_mut(seat);
            let between = p.stronghold_hp;
            let mut incoming = amount;
            if tick < p.shield_until && p.shield > 0 {
                let absorbed = p.shield.min(incoming);
                p.shield -= absorbed;
                incoming -= absorbed;
            }
            p.stronghold_hp = (p.stronghold_hp - incoming).max(0);
            let removed = between - p.stronghold_hp;
            ev.push(Event::Hit { to: target, amount });
            if removed > 0 {
                augment::after_stronghold_damage(state, seat, ev);
            }
            removed
        }
    }
}

fn heal(
    state: &mut MatchState,
    healer: Seat,
    id: u32,
    amount: i32,
    direct: bool,
    ev: &mut Vec<Event>,
) {
    let Some(i) = state.unit_index(id) else {
        return;
    };
    let target = state.units[i];
    let effective = augment::heal_amount(state, healer, &target, amount, direct);
    if effective <= 0 {
        return;
    }
    let u = &mut state.units[i];
    u.hp = (u.hp + effective).min(u.max_hp);
    ev.push(Event::Heal {
        to: id,
        amount: effective,
    });
    if !target.kind.is_summon() {
        if let Some(status) = arcana::purify_status(
            &state.player(healer).arcanas,
            &state.rules().arcana,
            target.status,
        ) {
            state.units[i].status = status;
        }
    }
}

fn status_ticks(base: u16, bp: i32) -> u16 {
    if bp <= 0 {
        return base;
    }
    (i64::from(base) * (BP + i64::from(bp)) / BP).min(i64::from(u16::MAX)) as u16
}

/// Root replaces anything (keeping the longer root); a stronger slow
/// replaces a slow, an equal or weaker one refreshes it; slow never
/// replaces root. Only duration is modified by Augments (BR-X07).
fn apply_on_hit(
    state: &mut MatchState,
    id: u32,
    on_hit: OnHit,
    attacker: Seat,
    ev: &mut Vec<Event>,
) {
    let Some(i) = state.unit_index(id) else {
        return;
    };
    let bonus = augment::status_duration_bp(&state.player(attacker).augments);
    let target = state.units[i];
    let archetype = state.rules().def(target.kind).archetype;
    let shorten = |ticks| {
        if target.kind.is_summon() {
            ticks
        } else {
            arcana::shorten_status(
                &state.player(target.seat).arcanas,
                &state.rules().arcana,
                archetype,
                ticks,
            )
        }
    };
    let on_hit = match on_hit {
        OnHit::Slow { bp, ticks } => OnHit::Slow {
            bp,
            ticks: shorten(status_ticks(ticks, bonus)),
        },
        OnHit::Root { ticks } => OnHit::Root {
            ticks: shorten(status_ticks(ticks, bonus)),
        },
    };
    let u = &mut state.units[i];
    if u.hp <= 0 {
        return;
    }
    u.status = match (on_hit, u.status) {
        (OnHit::Root { ticks }, Status::Rooted { ticks_left }) => Status::Rooted {
            ticks_left: ticks.max(ticks_left),
        },
        (OnHit::Root { ticks }, _) => Status::Rooted { ticks_left: ticks },
        (OnHit::Slow { .. }, rooted @ Status::Rooted { .. }) => rooted,
        (
            OnHit::Slow { bp, ticks },
            Status::Slowed {
                bp: old,
                ticks_left,
            },
        ) if bp <= old => Status::Slowed {
            bp: old,
            ticks_left: ticks_left.max(ticks),
        },
        (OnHit::Slow { bp, ticks }, _) => Status::Slowed {
            bp,
            ticks_left: ticks,
        },
    };
    ev.push(Event::StatusApplied { unit: id });
}
