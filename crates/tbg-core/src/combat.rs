//! Automatic combat (BR-U01, BR-U04): statuses, movement, windup and
//! hits, splash, heals, Stronghold fire, deaths.

use crate::arcana;
use crate::augment;
use crate::content::{Ability, BP, LANE_LEN, MIN_INTERVAL, WINDUP};
use crate::hits::{damage, resolve_hit};
use crate::state::{stronghold_x, Event, MatchState, Target, Unit};
use crate::targeting::{self, Engage};
use crate::types::{Seat, Status, UnitId, UnitKind};

mod formation;
use formation::FormationIndex;

/// Count down statuses and cooldowns.
pub(crate) fn tick_timers(state: &mut MatchState) {
    let rules = state.rules();
    for u in &mut state.units {
        crate::jawguard::tick(rules, u, state.tick);
        let before = u.status;
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
        crate::roster_slots::release_queued_slow(u, before);
        u.cooldown = u.cooldown.saturating_sub(1);
        if !u.is_summon() && crate::specials::period(rules, u.kind, u.tier, u.branch).is_some() {
            u.action_cd = u.action_cd.saturating_sub(1);
        }
    }
    for p in &mut state.players {
        p.stronghold_cooldown = p.stronghold_cooldown.saturating_sub(1);
    }
}

/// Every living unit acts once, in ascending id order.
pub(crate) fn run_units(state: &mut MatchState, ev: &mut Vec<Event>) {
    state.refresh_army();
    // Carried passengers ride at the crab's x before anyone acts.
    sync_passengers(state);
    // Unit indices and form parameters stay fixed during this pass: all
    // spawns happen before it and removal happens afterwards.
    let mut formations = FormationIndex::new();
    for idx in 0..state.units.len() {
        advance_unit(state, idx, ev, &mut formations);
        crate::specials::advance(state, idx, ev);
    }
    // ...and again after everyone moved.
    sync_passengers(state);
}

/// Pin every carried passenger's x to its crab's x (P48-A, §C.15).
fn sync_passengers(state: &mut MatchState) {
    let pairs: Vec<(usize, i32)> = state
        .units
        .iter()
        .enumerate()
        .filter_map(|(i, u)| {
            u.fx.carrier
                .and_then(|c| state.unit_index(c))
                .map(|ci| (i, state.units[ci].x))
        })
        .collect();
    for (i, x) in pairs {
        state.units[i].x = x;
    }
}

fn advance_unit(
    state: &mut MatchState,
    idx: usize,
    ev: &mut Vec<Event>,
    formations: &mut FormationIndex,
) {
    let u = state.units[idx];
    if u.hp <= 0 {
        return;
    }
    if let Status::Rooted { .. } = u.status {
        // Root cancels the windup and any betrayal with it (§C.10).
        state.units[idx].windup = None;
        state.units[idx].fx.betray = false;
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
        let distance = (targeting::target_x(state, target) - u.x).abs();
        let quickdraw = if u.is_summon() {
            0
        } else {
            arcana::attack_speed_bp(
                &state.player(u.seat).arcanas,
                &state.rules().arcana,
                state.rules().def(u.kind).archetype,
                distance,
            )
        };
        let interval = if quickdraw > 0 {
            let base = i64::from(state.rules().def(u.kind).interval);
            // BR-X02: every attack-speed source together stays within +100%.
            let bp = (i64::from(st.atk_bp) + i64::from(quickdraw)).min(10_000);
            (base * BP / (BP + bp)) as u16
        } else {
            st.interval
        }
        .max(MIN_INTERVAL);
        let unit = &mut state.units[idx];
        unit.windup = None;
        unit.cooldown = slowed_interval(interval, u.status).saturating_sub(WINDUP);
        return;
    }
    match targeting::engage(state, &u, &st) {
        Some(Engage::Act(target)) => {
            if u.cooldown == 0 {
                // Confused attackers betray their own ally instead (§C.10):
                // decided at windup start, so AttackStart already shows it.
                // Root cancels the windup and the betrayal with it.
                let (target, betrayed) = crate::support::betray_windup(state, &u, st.range, target);
                crate::ravager::retarget(state, idx, target);
                crate::mantis::retarget(state, idx, target);
                state.units[idx].windup = Some((target, WINDUP));
                state.units[idx].fx.betray = betrayed;
                ev.push(Event::AttackStart {
                    from: Target::Unit(u.id),
                    to: target,
                });
            }
        }
        Some(Engage::Hold) => {}
        None => {
            // Carried passengers never walk on their own; the crab carries
            // them (their x stays synced around this tick).
            if u.fx.carrier.is_some() || crate::bulwark::anchored(state, &u) {
                return;
            }
            // Drumguard B2: the bonded ally ignores slows and fish.
            let mut speed = if u.fx.march_ignore_slow {
                st.speed
            } else {
                match u.status {
                    Status::Slowed { bp, .. } => scale_down(st.speed, bp),
                    _ => st.speed,
                }
            };
            // D-56: a Cargo Crab carrying a passenger moves at the speed of the
            // faster of the two units before carry bonuses are applied.
            if let Some(pid) = u.fx.passenger {
                if let Some(pi) = state.unit_index(pid) {
                    let pu = &state.units[pi];
                    let pst = state.stats_of(pu);
                    let pass_speed = if pu.fx.march_ignore_slow {
                        pst.speed
                    } else {
                        match pu.status {
                            Status::Slowed { bp, .. } => scale_down(pst.speed, bp),
                            _ => pst.speed,
                        }
                    };
                    speed = speed.max(pass_speed);
                }
            }
            // Old Finn fish (P45): movement only, multiplicative with any
            // status slow, so no combination reaches zero. Never touches
            // the attack interval. Oilpress A2 immunity covers it.
            let fish_bp: i64 =
                u.fx.fish
                    .iter()
                    .filter(|s| s.left > 0)
                    .map(|s| i64::from(s.value))
                    .sum();
            if fish_bp > 0
                && !u.fx.march_ignore_slow
                && !crate::afflictions::is_immune_to_control(&u)
            {
                speed = (i64::from(speed) * (BP - fish_bp.min(BP)) / BP) as i32;
            }
            let mut haste = augment::haste_move_bp(&state.player(u.seat).augments, &u, state.tick);
            if !u.is_summon() {
                let army = state.army_of(u.seat);
                haste += arcana::move_bonus_bp(
                    &state.player(u.seat).arcanas,
                    &state.rules().arcana,
                    state.rules().def(u.kind).archetype,
                    army.tanks > 0,
                );
            }
            // Drumguard march bonus joins haste under the BR-X03 cap, as does
            // the Cargo Crab's carry bonus (B1 steady, B2 surge while armed).
            let bonus = (i64::from(haste)
                + i64::from(u.fx.march_move_bp)
                + i64::from(crate::support::carry_move_bp(state, &u)))
            .min(5_000);
            if bonus > 0 {
                speed = ((i64::from(speed) * (BP + bonus) / BP).min(i64::from(i32::MAX))) as i32;
            }
            let mut x = u.x + u.seat.dir() * speed;
            x = formations.clamp(state, &u, x);
            x = escort_clamp(state, &u, x);
            state.units[idx].x = x.clamp(0, LANE_LEN);
        }
    }
}

/// Drumguard escort (P47, §C.6): a bonded drummer never walks past its ally.
fn escort_clamp(state: &MatchState, u: &Unit, new_x: i32) -> i32 {
    let Some(bid) = u.fx.bond else {
        return new_x;
    };
    let Some(bi) = state.unit_index(bid) else {
        return new_x;
    };
    if state.units[bi].hp <= 0 {
        return new_x;
    }
    let ally_x = state.units[bi].x;
    let dir = u.seat.dir();
    if dir > 0 {
        if u.x <= ally_x && new_x > ally_x {
            ally_x
        } else {
            new_x
        }
    } else if u.x >= ally_x && new_x < ally_x {
        ally_x
    } else {
        new_x
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

/// Each Stronghold shoots the nearest enemy unit in range, a Rampart in
/// range first (Guard, BR-U09).
pub(crate) fn stronghold_fire(state: &mut MatchState, ev: &mut Vec<Event>) {
    for seat in Seat::BOTH {
        if state.player(seat).stronghold_cooldown > 0 {
            continue;
        }
        // Kind and Fortification shape the real shot (BR-SK03, BR-SK06).
        let tower = state.stronghold(seat);
        let sx = stronghold_x(seat);
        let target = state
            .units
            .iter()
            .filter(|u| {
                u.seat != seat
                    && u.hp > 0
                    && crate::targeting::targetable(u)
                    && (u.x - sx).abs() <= tower.range
            })
            .map(|u| {
                let guard = state
                    .rules()
                    .def(u.kind)
                    .ability(u.tier, u.branch)
                    .contains(&Ability::Guard);
                (!guard, (u.x - sx).abs(), u.id)
            })
            .min();
        if let Some((_, _, id)) = target {
            let to = Target::Unit(id);
            ev.push(Event::AttackStart {
                from: Target::Stronghold(seat),
                to,
            });
            let p = state.player(seat);
            let bp =
                augment::stronghold_damage_bp(&p.augments, state.tick, p.fury_until, p.fury_stacks);
            let amount = if bp == 0 {
                tower.damage
            } else {
                (i64::from(tower.damage) * (BP + i64::from(bp)) / BP) as i32
            };
            let was_alive = state
                .unit_index(id)
                .map(|i| state.units[i].hp > 0 && !state.units[i].is_summon())
                .unwrap_or(false);
            let removed = damage(state, to, amount.max(1), ev);
            #[cfg(feature = "lab")]
            crate::lab::stronghold(state, seat, removed);
            let killed = was_alive && state.unit_index(id).is_some_and(|i| state.units[i].hp <= 0);
            if killed && removed > 0 {
                if let Some((max, ticks)) = augment::tower_fury_rule(&state.player(seat).augments) {
                    let until = state.tick.saturating_add(u32::from(ticks));
                    let p = state.player_mut(seat);
                    p.fury_stacks = p.fury_stacks.saturating_add(1).min(max);
                    p.fury_until = until;
                }
            }
            state.player_mut(seat).stronghold_cooldown = tower.interval;
        }
    }
}

/// A2 shrapnel blast radius in milli-tiles (§C.12: 1.0 tile).
const SHRAPNEL_RADIUS: i32 = 1_000;

pub(crate) fn remove_dead(state: &mut MatchState, ev: &mut Vec<Event>) {
    // P48-A: dismount first. A dead crab drops its passenger behind it
    // (re-board lock); a dead passenger frees its crab at once.
    let mut drops: Vec<(UnitId, UnitId)> = Vec::new();
    let mut freed: Vec<UnitId> = Vec::new();
    for u in state.units.iter().filter(|u| u.hp <= 0) {
        if u.kind == UnitKind::CargoCrab {
            if let Some(passenger) = u.fx.passenger {
                drops.push((u.id, passenger));
            }
        }
        if let Some(carrier) = u.fx.carrier {
            freed.push(carrier);
        }
    }
    for (crab, passenger) in drops {
        crate::support::dismount(state, crab, passenger, ev);
    }
    for carrier in freed {
        if let Some(ci) = state.unit_index(carrier) {
            state.units[ci].fx.passenger = None;
        }
    }
    // P46: drain the shrapnel queue before removal. Chains terminate:
    // shrapnel only spawns on plate breaks, and plates only decrease.
    while !state.shrapnel.is_empty() {
        let queue = std::mem::take(&mut state.shrapnel);
        for s in queue {
            let enemy = s.seat.other();
            let victims: Vec<UnitId> = state
                .units
                .iter()
                .filter(|u| u.seat == enemy && u.hp > 0 && (u.x - s.x).abs() <= SHRAPNEL_RADIUS)
                .map(|u| u.id)
                .collect();
            for id in victims {
                crate::hits::damage(state, Target::Unit(id), s.amount, ev);
            }
        }
    }
    let mut dead = [0u32; 2];
    for u in state.units.iter().filter(|u| u.hp <= 0) {
        ev.push(Event::Death { unit: u.id });
        if !u.is_summon() {
            dead[u.seat.index()] += 1;
        }
    }
    state.units.retain(|u| u.hp > 0);
    // Clear a dead prey before projecting counters in this tick's views.
    crate::roster_slots::tick_marks(state);
    crate::specials::cancel_gone(state);
    #[cfg(feature = "lab")]
    if dead[0] + dead[1] > 0 {
        crate::lab::progress(state);
    }
    for seat in Seat::BOTH {
        for _ in 0..dead[seat.index()] {
            augment::on_eligible_kill(state, seat);
        }
    }
}
