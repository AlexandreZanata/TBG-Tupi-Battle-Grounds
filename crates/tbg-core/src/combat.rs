//! Automatic combat (BR-U01, BR-U04): statuses, movement, windup and
//! hits, splash, heals, Stronghold fire, deaths.

use crate::content::{
    BP, LANE_LEN, STRONGHOLD_DAMAGE, STRONGHOLD_INTERVAL, STRONGHOLD_RANGE, WINDUP,
};
use crate::hits::{damage, resolve_hit};
use crate::state::{stronghold_x, Event, MatchState, Target};
use crate::targeting::{self, Engage};
use crate::types::{Seat, Status};

/// Count down statuses and cooldowns.
pub(crate) fn tick_timers(state: &mut MatchState) {
    for u in &mut state.units {
        u.status = match u.status {
            Status::Slowed { bp, ticks_left } if ticks_left > 1 => Status::Slowed {
                bp,
                ticks_left: ticks_left - 1,
            },
            Status::Rooted { ticks_left } if ticks_left > 1 => Status::Rooted {
                ticks_left: ticks_left - 1,
            },
            _ => Status::None,
        };
        u.cooldown = u.cooldown.saturating_sub(1);
    }
    for p in &mut state.players {
        p.stronghold_cooldown = p.stronghold_cooldown.saturating_sub(1);
    }
}

/// Every living unit acts once, in ascending id order.
pub(crate) fn run_units(state: &mut MatchState, ev: &mut Vec<Event>) {
    for idx in 0..state.units.len() {
        advance_unit(state, idx, ev);
    }
}

fn advance_unit(state: &mut MatchState, idx: usize, ev: &mut Vec<Event>) {
    let u = state.units[idx];
    if u.hp <= 0 {
        return;
    }
    if let Status::Rooted { .. } = u.status {
        state.units[idx].windup = None;
        return;
    }
    let st = state.stats_of(&u);
    if let Some((target, ticks)) = u.windup {
        if ticks > 1 {
            state.units[idx].windup = Some((target, ticks - 1));
            return;
        }
        if targeting::alive(state, target) {
            resolve_hit(state, &u, target, &st, ev);
        }
        let unit = &mut state.units[idx];
        unit.windup = None;
        unit.cooldown = slowed_interval(st.interval, u.status).saturating_sub(WINDUP);
        return;
    }
    match targeting::engage(state, &u, &st) {
        Some(Engage::Act(target)) => {
            if u.cooldown == 0 {
                state.units[idx].windup = Some((target, WINDUP));
                ev.push(Event::AttackStart {
                    from: Target::Unit(u.id),
                    to: target,
                });
            }
        }
        Some(Engage::Hold) => {}
        None => {
            let speed = match u.status {
                Status::Slowed { bp, .. } => scale_down(st.speed, bp),
                _ => st.speed,
            };
            let x = u.x + u.seat.dir() * speed;
            state.units[idx].x = x.clamp(0, LANE_LEN);
        }
    }
}

fn scale_down(value: i32, bp: u16) -> i32 {
    (i64::from(value) * (BP - i64::from(bp)) / BP) as i32
}

fn slowed_interval(interval: u16, status: Status) -> u16 {
    match status {
        Status::Slowed { bp, .. } => {
            (i64::from(interval) * BP / (BP - i64::from(bp))).min(i64::from(u16::MAX)) as u16
        }
        _ => interval,
    }
}

/// Each Stronghold shoots the nearest enemy unit in range.
pub(crate) fn stronghold_fire(state: &mut MatchState, ev: &mut Vec<Event>) {
    for seat in Seat::BOTH {
        if state.player(seat).stronghold_cooldown > 0 {
            continue;
        }
        let sx = stronghold_x(seat);
        let target = state
            .units
            .iter()
            .filter(|u| u.seat != seat && u.hp > 0)
            .map(|u| ((u.x - sx).abs(), u.id))
            .min()
            .filter(|(d, _)| *d <= STRONGHOLD_RANGE);
        if let Some((_, id)) = target {
            let to = Target::Unit(id);
            ev.push(Event::AttackStart {
                from: Target::Stronghold(seat),
                to,
            });
            damage(state, to, STRONGHOLD_DAMAGE, ev);
            state.player_mut(seat).stronghold_cooldown = STRONGHOLD_INTERVAL;
        }
    }
}

pub(crate) fn remove_dead(state: &mut MatchState, ev: &mut Vec<Event>) {
    for u in state.units.iter().filter(|u| u.hp <= 0) {
        ev.push(Event::Death { unit: u.id });
    }
    state.units.retain(|u| u.hp > 0);
}
