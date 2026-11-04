//! Stable body-aware path search over the map lattice and continuous bridge connections.
use super::bodies::Bodies;
use super::navigation::{acceptable, approach, heuristic, CELL};
use crate::{geom::Pos, map, state::Unit};
use std::{cmp::Reverse, collections::BinaryHeap};
const WIDTH: usize = 101;

/// Search the fixed 2-unit lattice only after a direct approach fails. Stable
/// (cost, node id) ties and ordered neighbours make routes replayable.
pub(super) fn route(
    bodies: &Bodies,
    a: &Unit,
    target: Option<&Unit>,
    goal: Pos,
) -> Option<Vec<Pos>> {
    // A mission route must reach a future combat front, not demand that
    // every distant mobile opponent has already left the entire route.
    // Engagement routes still account for the nearby contested bodies;
    // every actual step is always checked against all current bodies.
    let units = if target.is_none() && matches!(a.brain, crate::state::Brain::Lane { .. }) {
        bodies.permanent()
    } else {
        *bodies
    };
    let portals = bridge_portals(a.stats.radius);
    let mut cost = vec![i64::MAX; WIDTH * WIDTH + portals.len()];
    let mut parent = vec![usize::MAX; WIDTH * WIDTH + portals.len()];
    let mut open = BinaryHeap::new();
    let mut seeds = grid_nodes(a.pos, 1);
    seeds.extend(WIDTH * WIDTH..WIDTH * WIDTH + portals.len());
    for id in seeds {
        let p = node(id, &portals);
        if units.clear(a, a.pos, p) {
            cost[id] = a.pos.dist(p);
            open.push(Reverse((
                cost[id] + heuristic(a, target, p, goal),
                cost[id],
                id,
            )));
        }
    }
    while let Some(Reverse((_, distance, id))) = open.pop() {
        if distance != cost[id] {
            continue;
        }
        let at = node(id, &portals);
        if let Some(end) = finish(&units, a, target, at, goal) {
            let mut path = if end == at { vec![at] } else { vec![end, at] };
            let mut n = id;
            while parent[n] != usize::MAX {
                n = parent[n];
                path.push(node(n, &portals));
            }
            path.reverse();
            return Some(simplify(&units, a, path));
        }
        for next in neighbours(id, at, &portals) {
            let p = node(next, &portals);
            let distance = distance + at.dist(p);
            if distance < cost[next] && units.clear(a, at, p) {
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

/// Each valid bridge is also one continuous swept graph edge. Narrow
/// passages must not disappear merely because their centre misses a cell.
fn bridge_portals(radius: i32) -> Vec<Pos> {
    if radius + 2 >= map::BRIDGE_HALF_WIDTH {
        return Vec::new();
    }
    let offset = ((i64::from(map::WATER_HALF_WIDTH + radius + CELL) * 707_107) / 1_000_000) as i32;
    map::bridges()
        .flat_map(|p| {
            [-1, 1].map(|sign| Pos {
                x: p.x + sign * offset,
                y: p.y + sign * offset,
            })
        })
        .collect()
}
fn neighbours(id: usize, at: Pos, portals: &[Pos]) -> Vec<usize> {
    let span = if id < WIDTH * WIDTH { 1 } else { 2 };
    let mut out = grid_nodes(at, span);
    out.retain(|&next| next != id);
    for (index, &p) in portals.iter().enumerate() {
        let next = WIDTH * WIDTH + index;
        if next != id && (id >= WIDTH * WIDTH || at.within(p, 3 * CELL)) {
            out.push(next);
        }
    }
    out.sort_unstable();
    out
}

/// Remove lattice corners visible from the previous corner. Besides smoother
/// routes, this avoids trying to stand exactly on a seed cell under ally pressure.
fn simplify(units: &Bodies, a: &Unit, path: Vec<Pos>) -> Vec<Pos> {
    let mut result = Vec::new();
    let mut from = a.pos;
    let mut first = 0;
    while first < path.len() {
        let next = (first..path.len())
            .rev()
            .find(|&j| units.clear(a, from, path[j]))
            .unwrap_or(first);
        result.push(path[next]);
        from = path[next];
        first = next + 1;
    }
    result
}

fn node(id: usize, portals: &[Pos]) -> Pos {
    if id >= WIDTH * WIDTH {
        portals[id - WIDTH * WIDTH]
    } else {
        Pos {
            x: (id % WIDTH) as i32 * CELL,
            y: (id / WIDTH) as i32 * CELL,
        }
    }
}
fn grid_nodes(at: Pos, span: i32) -> Vec<usize> {
    let mut out = Vec::new();
    let (sx, sy) = (at.x / CELL, at.y / CELL);
    for y in sy - span..=sy + span {
        for x in sx - span..=sx + span {
            if (0..WIDTH as i32).contains(&x) && (0..WIDTH as i32).contains(&y) {
                out.push(y as usize * WIDTH + x as usize);
            }
        }
    }
    out
}
fn finish(units: &Bodies, a: &Unit, target: Option<&Unit>, at: Pos, goal: Pos) -> Option<Pos> {
    if acceptable(a, target, at, goal) {
        return Some(at);
    }
    if target.is_none() || heuristic(a, target, at, goal) > i64::from(2 * CELL) {
        return None;
    }
    let mut probe = a.clone();
    probe.pos = at;
    let end = approach(&probe, target, goal);
    (acceptable(a, target, end, goal) && units.clear(a, at, end)).then_some(end)
}
