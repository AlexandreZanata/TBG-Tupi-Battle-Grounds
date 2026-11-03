//! Persistent deterministic body-safe routes; mobile allies remain soft obstacles.
use crate::{
    geom::{Pos, MM},
    map,
    state::{MobaMatch, Unit},
};
use serde::{Deserialize, Serialize};
use std::{cmp::Reverse, collections::BinaryHeap};

const CELL: i32 = 2 * MM;
const WIDTH: usize = 101;
const STALL: u16 = 40;

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct Navigation {
    pub target: u32,
    pub goal: Pos,
    pub path: Vec<Pos>,
    pub stalled: u16,
    pub rejected: Vec<(u32, u32)>,
    pub retry_at: u32,
    pub progress: Option<(Pos, i64)>,
}

pub fn rejected(u: &Unit, id: u32, tick: u32) -> bool {
    u.navigation
        .rejected
        .iter()
        .any(|&(old, until)| old == id && tick < until)
}

pub fn solid(a: &Unit, b: &Unit) -> bool {
    a.id != b.id
        && b.alive()
        && b.fx.carrier == 0
        && (a.team().is_none() || a.side != b.side || !b.mobile())
}
pub fn gap(a: &Unit, b: &Unit) -> i32 {
    a.stats.radius
        + b.stats.radius
        + if a.side == b.side {
            tbg_core::content::occupancy::GAP
        } else {
            0
        }
}

/// Swept disk vs disk. An initial overlap may only move outward.
pub fn intersects(from: Pos, to: Pos, center: Pos, radius: i32) -> bool {
    let dx = i64::from(to.x - from.x);
    let dy = i64::from(to.y - from.y);
    let len = dx * dx + dy * dy;
    let dot = i64::from(center.x - from.x) * dx + i64::from(center.y - from.y) * dy;
    if from.dist2(center) < i64::from(radius).pow(2) {
        return dot > 0;
    }
    let along = dot.clamp(0, len);
    let p = Pos {
        x: from.x + (dx * along / len.max(1)) as i32,
        y: from.y + (dy * along / len.max(1)) as i32,
    };
    p.dist2(center) < i64::from(radius).pow(2)
}

pub fn clear(units: &[Unit], a: &Unit, from: Pos, to: Pos) -> bool {
    map::surface::clear_segment(from, to, a.stats.radius)
        && units
            .iter()
            .filter(|b| solid(a, b))
            .all(|b| !intersects(from, to, b.pos, gap(a, b)))
}

fn acceptable(a: &Unit, target: Option<&Unit>, at: Pos, goal: Pos) -> bool {
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

fn heuristic(a: &Unit, target: Option<&Unit>, at: Pos, goal: Pos) -> i64 {
    let reach = target.map_or(0, |b| crate::champions::reach(a, b));
    (at.dist(goal) - i64::from(reach)).max(0)
}

/// Search the fixed 2-unit lattice only after a direct approach fails. Stable
/// (cost, node id) ties and ordered neighbours make routes replayable.
fn route(units: &[Unit], a: &Unit, target: Option<&Unit>, goal: Pos) -> Option<Vec<Pos>> {
    let node = |id: usize| Pos {
        x: (id % WIDTH) as i32 * CELL,
        y: (id / WIDTH) as i32 * CELL,
    };
    let mut cost = vec![i64::MAX; WIDTH * WIDTH];
    let mut parent = vec![usize::MAX; WIDTH * WIDTH];
    let mut open = BinaryHeap::new();
    let sx = a.pos.x / CELL;
    let sy = a.pos.y / CELL;
    for y in sy - 1..=sy + 1 {
        for x in sx - 1..=sx + 1 {
            if !(0..WIDTH as i32).contains(&x) || !(0..WIDTH as i32).contains(&y) {
                continue;
            }
            let id = y as usize * WIDTH + x as usize;
            let p = node(id);
            if clear(units, a, a.pos, p) {
                cost[id] = a.pos.dist(p);
                open.push(Reverse((
                    cost[id] + heuristic(a, target, p, goal),
                    cost[id],
                    id,
                )));
            }
        }
    }
    while let Some(Reverse((_, distance, id))) = open.pop() {
        if distance != cost[id] {
            continue;
        }
        let at = node(id);
        let finish = if acceptable(a, target, at, goal) {
            Some(at)
        } else if target.is_some() && heuristic(a, target, at, goal) <= i64::from(2 * CELL) {
            let mut probe = a.clone();
            probe.pos = at;
            let end = approach(&probe, target, goal);
            (acceptable(a, target, end, goal) && clear(units, a, at, end)).then_some(end)
        } else {
            None
        };
        if let Some(end) = finish {
            let mut path = if end == at { vec![at] } else { vec![end, at] };
            let mut n = id;
            while parent[n] != usize::MAX {
                n = parent[n];
                path.push(node(n));
            }
            path.reverse();
            return Some(simplify(units, a, path));
        }
        for (dx, dy) in [
            (0, -1),
            (-1, 0),
            (1, 0),
            (0, 1),
            (-1, -1),
            (1, -1),
            (-1, 1),
            (1, 1),
        ] {
            let x = id as i32 % WIDTH as i32 + dx;
            let y = id as i32 / WIDTH as i32 + dy;
            if !(0..WIDTH as i32).contains(&x) || !(0..WIDTH as i32).contains(&y) {
                continue;
            }
            let next = y as usize * WIDTH + x as usize;
            let p = node(next);
            let distance = distance + at.dist(p);
            if distance < cost[next] && clear(units, a, at, p) {
                cost[next] = distance;
                parent[next] = id;
                open.push(Reverse((
                    distance + heuristic(a, target, p, goal),
                    distance,
                    next,
                )));
            }
        }
    }
    None
}

/// Remove lattice corners visible from the previous corner. Besides smoother
/// routes, this avoids trying to stand exactly on a seed cell under ally pressure.
fn simplify(units: &[Unit], a: &Unit, path: Vec<Pos>) -> Vec<Pos> {
    let mut result = Vec::new();
    let mut from = a.pos;
    let mut first = 0;
    while first < path.len() {
        let next = (first..path.len())
            .rev()
            .find(|&j| clear(units, a, from, path[j]))
            .unwrap_or(first);
        result.push(path[next]);
        from = path[next];
        first = next + 1;
    }
    result
}

fn approach(a: &Unit, target: Option<&Unit>, goal: Pos) -> Pos {
    let Some(b) = target else {
        return map::surface::clamp_body(goal, a.stats.radius);
    };
    let mut p = goal;
    let radius = crate::champions::reach(a, b)
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

pub fn step(m: &mut MobaMatch, snapshot: &[Unit], i: usize, goal: Pos, speed: i32) -> Pos {
    let a = &m.units[i];
    let target = snapshot.iter().find(|u| u.id == a.target && u.alive());
    let direct = approach(a, target, goal);
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
    // Retain a chosen detour until it finishes, rather than switching sides each tick.
    if nav.path.is_empty()
        && clear(snapshot, a, a.pos, direct)
        && acceptable(a, target, direct, goal)
    {
        nav.path.push(direct);
    }
    if nav.path.is_empty() && m.tick >= nav.retry_at {
        nav.path = route(snapshot, a, target, goal).unwrap_or_default();
        nav.retry_at = m.tick + u32::from(STALL);
    }
    while nav.path.first().is_some_and(|p| a.pos.within(*p, 50)) {
        nav.path.remove(0);
    }
    let mut to = a.pos;
    if let Some(&next) = nav.path.first() {
        to.step_toward(next, speed);
        if !clear(snapshot, a, a.pos, to) {
            // Project the allowed displacement onto an obstacle edge. The route
            // itself keeps the passing side fixed while the crowd settles.
            let candidates = [
                Pos {
                    x: to.x,
                    y: a.pos.y,
                },
                Pos {
                    x: a.pos.x,
                    y: to.y,
                },
            ];
            to = candidates
                .into_iter()
                .filter(|p| clear(snapshot, a, a.pos, *p))
                .min_by_key(|p| p.dist2(next))
                .unwrap_or(a.pos);
        }
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
    let failed = nav.stalled >= STALL || nav.path.is_empty() && !acceptable(a, target, a.pos, goal);
    if failed {
        nav.path.clear();
        if a.target != 0 {
            nav.rejected.push((a.target, m.tick + u32::from(STALL)));
            m.units[i].target = 0;
        }
        nav.stalled = 0;
    }
    m.units[i].navigation = nav;
    to
}
