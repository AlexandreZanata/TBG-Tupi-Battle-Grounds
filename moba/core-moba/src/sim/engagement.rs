//! Attack positions (M-77): every engaged walker claims a persistent place on
//! a ring around its target, so a group surrounds the target instead of
//! queueing on one line and shoving each other in and out of range.

use super::bodies::{gap, solid, Bodies};
use super::decision::{in_position_at, reach};
use super::navigation::Navigation;
use crate::geom::Pos;
use crate::map;
use crate::state::Unit;

/// 32 compass directions scaled by 1024 (11.25° apart), integer for replay.
pub(super) const DIRECTIONS: [(i32, i32); 32] = [
    (1024, 0),
    (1004, 200),
    (946, 392),
    (851, 569),
    (724, 724),
    (569, 851),
    (392, 946),
    (200, 1004),
    (0, 1024),
    (-200, 1004),
    (-392, 946),
    (-569, 851),
    (-724, 724),
    (-851, 569),
    (-946, 392),
    (-1004, 200),
    (-1024, 0),
    (-1004, -200),
    (-946, -392),
    (-851, -569),
    (-724, -724),
    (-569, -851),
    (-392, -946),
    (-200, -1004),
    (0, -1024),
    (200, -1004),
    (392, -946),
    (569, -851),
    (724, -724),
    (851, -569),
    (946, -392),
    (1004, -200),
];

/// How far around the ring a walker looks for a free place (±157.5°).
const SEARCH: i32 = 14;

/// The compass index closest to the vector `(dx, dy)` (ties: lower index).
pub(super) fn direction_of(dx: i64, dy: i64) -> usize {
    (0..DIRECTIONS.len())
        .max_by_key(|&k| {
            let (x, y) = DIRECTIONS[k];
            (dx * i64::from(x) + dy * i64::from(y), std::cmp::Reverse(k))
        })
        .unwrap_or(0)
}

pub(super) fn offset(at: Pos, index: usize, distance: i32) -> Pos {
    let (x, y) = DIRECTIONS[index % DIRECTIONS.len()];
    Pos {
        x: at.x + (i64::from(x) * i64::from(distance) / 1024) as i32,
        y: at.y + (i64::from(y) * i64::from(distance) / 1024) as i32,
    }
}

/// A free, legal place to act on `b` from, keeping the previous choice while
/// it stays valid. `None` leaves the caller with the direct approach.
pub(super) fn point(bodies: &Bodies, a: &Unit, b: &Unit, nav: &mut Navigation) -> Option<Pos> {
    // Rings from the full reach in to body contact: a long reach that the
    // camp leash or the water rules out still finds a closer legal place.
    let outer = (reach(a, b) - 50).max(a.stats.radius + b.stats.radius);
    let inner =
        (a.stats.radius + b.stats.radius + tbg_core::content::occupancy::GAP / 2).min(outer);
    let rings = [
        outer,
        inner + (outer - inner) * 2 / 3,
        inner + (outer - inner) / 3,
        inner,
    ];
    // Places already claimed on this target by allies that choose first.
    let claims: Vec<(Pos, i32)> = bodies
        .units
        .iter()
        .filter(|c| c.side == a.side && c.id < a.id && c.alive())
        .filter_map(|c| match c.navigation.claim {
            Some((t, q)) if t == b.id => Some((q, gap(a, c))),
            _ => None,
        })
        .collect();
    let valid = |p: Pos| free(bodies, a, b, &claims, p) && in_position_at(a, b, p);
    if let Some((target, index, ring)) = nav.slot {
        let p = offset(b.pos, usize::from(index), rings[usize::from(ring).min(3)]);
        if target == b.id && valid(p) {
            nav.claim = Some((b.id, p));
            return Some(p);
        }
    }
    let base = direction_of(i64::from(a.pos.x - b.pos.x), i64::from(a.pos.y - b.pos.y)) as i32;
    // Nearest side first; the lower id decides an exact tie.
    let order = std::iter::once(0).chain((1..=SEARCH).flat_map(|k| {
        if a.id.is_multiple_of(2) {
            [k, -k]
        } else {
            [-k, k]
        }
    }));
    let order: Vec<i32> = order.collect();
    for (ring, &distance) in rings.iter().enumerate() {
        for &step in &order {
            let index = (base + step).rem_euclid(DIRECTIONS.len() as i32) as usize;
            let p = offset(b.pos, index, distance);
            if valid(p) {
                nav.slot = Some((b.id, index as u8, ring as u8));
                nav.claim = Some((b.id, p));
                return Some(p);
            }
        }
    }
    nav.slot = None;
    nav.claim = None;
    None
}

/// The place is dry, inside the rim, clear of blocking bodies and not
/// claimed by an earlier ally (lower id) acting on the same target.
fn free(bodies: &Bodies, a: &Unit, b: &Unit, claims: &[(Pos, i32)], p: Pos) -> bool {
    if !map::surface::walkable(p, a.stats.radius)
        || claims.iter().any(|&(q, room)| p.within(q, room))
    {
        return false;
    }
    let mut taken = false;
    bodies.near(p, a.stats.radius, |c| {
        taken |= c.id != b.id && solid(a, c) && p.within(c.pos, gap(a, c));
    });
    !taken
}
