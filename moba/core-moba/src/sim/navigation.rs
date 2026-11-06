//! Persistent deterministic body-safe routes and local steering: follow the
//! route, steer around a blocking body on one persistent side, and give up
//! only by the explicit stall rules (M-74, M-76, M-77).
use super::bodies::{gap, Bodies};
use super::engagement::{direction_of, offset};
use super::pathfinding::route;
use crate::{
    geom::{Pos, MM},
    map,
    state::{MobaMatch, Unit},
};
use serde::{Deserialize, Serialize};

pub(super) const CELL: i32 = 2 * MM;
const STALL: u16 = 40;
/// Ticks a chosen passing side outlives the obstruction that caused it.
const STEER_MEMORY: u32 = 20;
/// Ally contact (mm) a walker may accept while queueing; shared with crowd pressure.
pub(super) const CONTACT_TOLERANCE: i64 = 60;

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct Navigation {
    pub target: u32,
    pub goal: Pos,
    pub path: Vec<Pos>,
    pub stalled: u16,
    pub rejected: Vec<(u32, u32)>,
    pub retry_at: u32,
    pub progress: Option<(Pos, i64)>,
    pub failed_since: Option<u32>,
    /// Attack position: (target, compass index, ring from the outside in).
    pub slot: Option<(u32, u8, u8)>,
    /// Engagement progress: (target, best distance still to cover, since
    /// tick, where the target stood last tick).
    pub closing: Option<(u32, i64, u32, Pos)>,
    /// The place this walker currently claims, read by later allies.
    pub claim: Option<(u32, Pos)>,
    /// Passing side (+1 left, -1 right) kept while steering around a body.
    pub side: i8,
    pub side_until: u32,
    /// Last tick's settled motion (mm per tick); steering and inertia turn from it.
    pub velocity: (i32, i32),
}

impl Navigation {
    pub fn reset_route(&mut self) {
        self.path.clear();
        self.progress = None;
        self.stalled = 0;
        self.retry_at = 0;
        self.failed_since = None;
    }
}

pub fn rejected(u: &Unit, id: u32, tick: u32) -> bool {
    u.navigation
        .rejected
        .iter()
        .any(|&(old, until)| old == id && tick < until)
}

pub(super) fn acceptable(a: &Unit, target: Option<&Unit>, at: Pos, goal: Pos) -> bool {
    match target {
        Some(b) => super::decision::in_position_at(a, b, at),
        None => at.within(
            goal,
            match a.brain {
                crate::state::Brain::Jungle { .. } => a
                    .stats
                    .sight
                    .max(crate::content::WAYPOINT_RADIUS + a.stats.radius),
                _ => crate::content::WAYPOINT_RADIUS + a.stats.radius,
            },
        ),
    }
}

pub(super) fn heuristic(a: &Unit, target: Option<&Unit>, at: Pos, goal: Pos) -> i64 {
    let reach = target.map_or(0, |b| super::decision::reach(a, b));
    (at.dist(goal) - i64::from(reach)).max(0)
}

pub(super) fn approach(a: &Unit, target: Option<&Unit>, goal: Pos) -> Pos {
    let Some(b) = target else {
        return map::surface::clamp_body(goal, a.stats.radius);
    };
    let mut p = goal;
    let radius = super::decision::reach(a, b)
        .saturating_sub(50)
        .max(a.stats.radius + b.stats.radius);
    p.step_toward(a.pos, radius);
    if let crate::state::Brain::Neutral { camp } = b.brain {
        let home = map::camp(usize::from(camp)).0;
        if !p.within(home, crate::content::CAMP_LEASH - 50) {
            p = home;
            p.step_toward(a.pos, crate::content::CAMP_LEASH - 50);
        }
    }
    p
}

pub fn step(
    m: &mut MobaMatch,
    bodies: &Bodies,
    i: usize,
    goal: Pos,
    speed: i32,
    budget: &mut u8,
) -> Pos {
    let a = &m.units[i];
    let target = bodies.units.iter().find(|u| u.id == a.target && u.alive());
    let mut nav = a.navigation.clone();
    nav.rejected.retain(|&(_, until)| until > m.tick);
    if nav.target != a.target || !nav.goal.within(goal, CELL) {
        nav.path.clear();
        nav.progress = None;
        nav.stalled = 0;
        nav.retry_at = 0;
        nav.target = a.target;
        nav.goal = goal;
    }
    // An engagement walks to its claimed ring place; a mission to its goal.
    let direct = match target {
        Some(b) => super::engagement::point(bodies, a, b, &mut nav)
            .unwrap_or_else(|| approach(a, target, goal)),
        None => {
            nav.slot = None;
            nav.claim = None;
            approach(a, target, goal)
        }
    };
    // Reaching the old contact point requests the next leg toward the
    // current target. It never rejects an otherwise valid engagement.
    consume(&mut nav.path, a.pos, speed);
    // A clear straight line to a moving ring place follows it directly every
    // tick; otherwise retain the chosen detour until it finishes.
    if nav.path.len() <= 1 && bodies.clear(a, a.pos, direct) && acceptable(a, target, direct, goal)
    {
        nav.path = vec![direct];
        nav.failed_since = None;
    }
    if nav.path.is_empty() && m.tick >= nav.retry_at && *budget > 0 {
        *budget -= 1;
        match route(bodies, a, target, goal) {
            Some(path) => {
                nav.path = path;
                nav.failed_since = None;
            }
            None => {
                nav.failed_since.get_or_insert(m.tick);
            }
        }
        nav.retry_at = m.tick + 10;
    }
    consume(&mut nav.path, a.pos, speed);
    let mut to = a.pos;
    if let Some(&next) = nav.path.first() {
        to = steer(bodies, a, next, speed, m.tick, &mut nav);
    }
    if let Some(&next) = nav.path.first() {
        let distance = a.pos.dist(next);
        let progressing = nav
            .progress
            .is_none_or(|(old, before)| old != next || distance + 5 < before);
        nav.stalled = if progressing {
            0
        } else {
            nav.stalled.saturating_add(1)
        };
        nav.progress = Some((next, distance));
    }
    let failed = nav.stalled >= STALL
        || nav
            .failed_since
            .is_some_and(|since| m.tick.saturating_sub(since) >= u32::from(STALL))
        || target.is_some_and(|b| !closing(a, b, m.tick, &mut nav));
    if failed {
        nav.path.clear();
        if a.target != 0 {
            nav.rejected.push((a.target, m.tick + u32::from(STALL)));
        }
        nav.stalled = 0;
        nav.failed_since = None;
        nav.slot = None;
        nav.claim = None;
        nav.closing = None;
    }
    let stationary = to == a.pos;
    let blocked = stationary && !acceptable(a, target, a.pos, goal);
    m.units[i].navigation = nav;
    if blocked {
        super::decision::phase(&mut m.units[i], super::decision::Phase::Repath);
        m.units[i].decision.reason = super::decision::Reason::Obstructed;
    } else if stationary {
        super::decision::phase(&mut m.units[i], super::decision::Phase::Wait);
        m.units[i].decision.reason = super::decision::Reason::Arrived;
    }
    to
}

/// Candidate turns from the preferred heading, in compass steps (11.25°):
/// straight, then alternating sides out to a quarter turn.
const TURNS: [i32; 13] = [0, 1, -1, 2, -2, 3, -3, 4, -4, 6, -6, 8, -8];

/// Sampled local avoidance (a deterministic, integer cousin of RVO): try the
/// straight step and small turns at full and half speed, drop any that hits
/// terrain or a solid body, and keep the cheapest. Costs favour the route
/// direction, the side already being passed on, a smooth turn from the last
/// step and room from walking allies, so a walker neither zig-zags nor
/// shuffles back and forth. Nothing valid: wait in place this tick.
fn steer(bodies: &Bodies, a: &Unit, next: Pos, speed: i32, tick: u32, nav: &mut Navigation) -> Pos {
    let from = a.pos;
    let remaining = i32::try_from(from.dist(next)).unwrap_or(i32::MAX);
    let heading = direction_of(i64::from(next.x - from.x), i64::from(next.y - from.y)) as i32;
    let previous = (nav.velocity != (0, 0))
        .then(|| direction_of(i64::from(nav.velocity.0), i64::from(nav.velocity.1)) as i32);
    let side = if tick < nav.side_until {
        i32::from(nav.side)
    } else {
        0
    };
    let mut best: Option<(i64, Pos, i32)> = None;
    for (rank, &turn) in TURNS.iter().enumerate() {
        // Costs only grow along TURNS: stop once no later turn can win.
        if best.is_some_and(|(c, _, _)| c <= i64::from(turn.abs()) * 10) {
            break;
        }
        for (half, length) in [(false, speed), (true, speed / 2)] {
            // Arrive exactly instead of overshooting the route point.
            let length = if turn == 0 {
                length.min(remaining)
            } else {
                length
            };
            if length <= 0 {
                continue;
            }
            let index = (heading + turn).rem_euclid(32) as usize;
            let p = if turn == 0 && length == remaining {
                next
            } else {
                map::surface::clamp_body(offset(from, index, length), a.stats.radius)
            };
            if p == from || !bodies.clear(a, from, p) {
                continue;
            }
            let mut cost = i64::from(turn.abs()) * 10 + if half { 15 } else { 0 };
            if side != 0 && turn != 0 && turn.signum() != side {
                cost += 25;
            }
            if let Some(prev) = previous {
                let swing = ((index as i32 - prev).rem_euclid(32))
                    .min((prev - index as i32).rem_euclid(32));
                cost += i64::from(swing) * 4 + if swing > 8 { 200 } else { 0 };
            }
            cost += crowding(bodies, a, p);
            let key = (cost, p, rank as i32);
            if best.is_none_or(|(c, _, r)| (cost, rank as i32) < (c, r)) {
                best = Some(key);
            }
        }
    }
    let Some((_, p, rank)) = best else {
        return from;
    };
    let turn = TURNS[rank as usize];
    if turn != 0 {
        nav.side = turn.signum() as i8;
        nav.side_until = tick + STEER_MEMORY;
    } else if tick >= nav.side_until {
        nav.side = 0;
    }
    p
}

/// Soft cost of standing in a walking ally's personal space at `p`.
fn crowding(bodies: &Bodies, a: &Unit, p: Pos) -> i64 {
    let mut cost = 0;
    bodies.near(p, a.stats.radius, |c| {
        if c.id != a.id && c.alive() && c.side == a.side && c.mobile() && c.fx.carrier == 0 {
            let room = i64::from(gap(a, c));
            let d = p.dist(c.pos);
            if d < room {
                cost += (room - d) * 40 / room.max(1);
            }
        }
    });
    cost
}

/// Drops route points already reached or passed: an intermediate point
/// within one step, or one the walker has gone beyond (it is nearer the
/// following point than the point itself is). Walking back to a point
/// passed on the side of an obstacle is a visible reversal (M-77).
fn consume(path: &mut Vec<Pos>, at: Pos, speed: i32) {
    while let Some(&first) = path.first() {
        let reached = match path.get(1) {
            Some(&second) => {
                at.within(first, speed.max(50)) || at.dist2(second) <= first.dist2(second)
            }
            None => at.within(first, 50),
        };
        if !reached {
            break;
        }
        path.remove(0);
    }
}

/// Whether an engagement still gets nearer a legal attack position. The
/// remaining distance must improve by at least 50 mm within `STALL` ticks;
/// circling or shuffling without that proves a standing target unreachable
/// from here (M-77). Being in position, or a target that moved since last
/// tick (a pursuit), is never such proof and restarts the clock.
fn closing(a: &Unit, b: &Unit, tick: u32, nav: &mut Navigation) -> bool {
    let remaining = (a.pos.dist(b.pos) - i64::from(super::decision::reach(a, b))).max(0);
    let fresh = Some((b.id, remaining, tick, b.pos));
    match nav.closing {
        Some((target, best, since, last)) if target == b.id => {
            if remaining == 0 || remaining + 50 <= best || !b.pos.within(last, 10) {
                nav.closing = fresh;
                true
            } else {
                nav.closing = Some((target, best, since, b.pos));
                tick.saturating_sub(since) < u32::from(STALL)
            }
        }
        _ => {
            nav.closing = fresh;
            true
        }
    }
}
