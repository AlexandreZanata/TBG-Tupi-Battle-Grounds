//! Resolving one hit: damage or heal, splash, and on-hit statuses
//! (GAME-DESIGN §4: roles, splash, status rules).

use crate::content::Stats;
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
        heal(state, id, st.heal, ev);
        if st.splash > 0 {
            for a in state.units.clone() {
                let near = (a.x - tx).abs() <= st.splash;
                if a.seat == attacker.seat && a.id != id && a.hp > 0 && a.hp < a.max_hp && near {
                    heal(state, a.id, st.heal, ev);
                }
            }
        }
        return;
    }
    damage(state, target, st.damage, ev);
    if let (Target::Unit(id), Some(on_hit)) = (target, st.on_hit) {
        apply_on_hit(state, id, on_hit, ev);
    }
    if st.splash > 0 {
        splash(state, attacker.seat.other(), target, tx, st, ev);
    }
}

/// Damage every other enemy (units and Stronghold) within the radius.
fn splash(
    state: &mut MatchState,
    enemy: Seat,
    primary: Target,
    tx: i32,
    st: &Stats,
    ev: &mut Vec<Event>,
) {
    for e in state.units.clone() {
        let near = (e.x - tx).abs() <= st.splash;
        if e.seat == enemy && e.hp > 0 && near && primary != Target::Unit(e.id) {
            damage(state, Target::Unit(e.id), st.damage, ev);
        }
    }
    let base = Target::Stronghold(enemy);
    if primary != base && (stronghold_x(enemy) - tx).abs() <= st.splash {
        damage(state, base, st.damage, ev);
    }
}

pub(crate) fn damage(state: &mut MatchState, target: Target, amount: i32, ev: &mut Vec<Event>) {
    match target {
        Target::Unit(id) => {
            let Some(i) = state.unit_index(id) else {
                return;
            };
            let u = &mut state.units[i];
            u.hp = (u.hp - amount).max(0);
        }
        Target::Stronghold(seat) => {
            let p = state.player_mut(seat);
            p.stronghold_hp = (p.stronghold_hp - amount).max(0);
        }
    }
    ev.push(Event::Hit { to: target, amount });
}

fn heal(state: &mut MatchState, id: u32, amount: i32, ev: &mut Vec<Event>) {
    let Some(i) = state.unit_index(id) else {
        return;
    };
    let u = &mut state.units[i];
    let healed = (u.hp + amount).min(u.max_hp) - u.hp;
    u.hp += healed;
    ev.push(Event::Heal {
        to: id,
        amount: healed,
    });
}

/// Root replaces anything (keeping the longer root); a stronger slow
/// replaces a slow, an equal or weaker one refreshes it; slow never
/// replaces root.
fn apply_on_hit(state: &mut MatchState, id: u32, on_hit: OnHit, ev: &mut Vec<Event>) {
    let Some(i) = state.unit_index(id) else {
        return;
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
