//! Shared dry ground, bridge decks and plaza clearance for circular bodies.
use super::{BRIDGE_HALF_WIDTH, EDGE, PLAZA_RADIUS, SIZE, WATER_HALF_WIDTH};
use crate::geom::{Pos, MM};

// Project onto the river normal/tangent with deterministic integer millimetres.
fn axes(p: Pos) -> (i64, i64) {
    (
        i64::from(p.x + p.y - SIZE * MM) * 707_107 / 1_000_000,
        i64::from(p.x - p.y) * 707_107 / 1_000_000,
    )
}

/// A disk may overlap dry land and a crossing; it need not fit wholly on the deck.
pub fn walkable(p: Pos, radius: i32) -> bool {
    let low = EDGE * MM + radius;
    let high = (SIZE - EDGE) * MM - radius;
    if p.x < low || p.y < low || p.x > high || p.y > high {
        return false;
    }
    let (n, t) = axes(p);
    let r = i64::from(radius + 2); // Conservative rounding of the integer projection.
    let water = i64::from(WATER_HALF_WIDTH);
    let near = (n.abs() - water).max(0);
    if near >= r {
        return true;
    }
    let lateral = (r * r - near * near).isqrt() + 1;
    if super::bridges().any(|c| (t - axes(c).1).abs() + lateral <= i64::from(BRIDGE_HALF_WIDTH)) {
        return true;
    }
    // Farthest point of the body's wet slice from the plaza centre.
    let d = (n * n + t * t).isqrt().max(1);
    let outer_n = (n * (d + r) / d).clamp(-water, water);
    let delta = outer_n - n;
    if delta.abs() > r {
        return false;
    }
    let outer_t = t.abs() + (r * r - delta * delta).isqrt() + 1;
    outer_n * outer_n + outer_t * outer_t <= i64::from(PLAZA_RADIUS * MM).pow(2)
}

pub fn clamp_body(p: Pos, radius: i32) -> Pos {
    let low = EDGE * MM + radius;
    let high = (SIZE - EDGE) * MM - radius;
    Pos {
        x: p.x.clamp(low, high),
        y: p.y.clamp(low, high),
    }
}

/// Sweep with sub-body samples so a long planned segment cannot jump over water.
pub fn clear_segment(from: Pos, to: Pos, radius: i32) -> bool {
    let steps = (from.dist(to) / 250 + 1) as i32;
    (1..=steps).all(|i| {
        walkable(
            Pos {
                x: from.x + ((i64::from(to.x - from.x) * i64::from(i)) / i64::from(steps)) as i32,
                y: from.y + ((i64::from(to.y - from.y) * i64::from(i)) / i64::from(steps)) as i32,
            },
            radius,
        )
    })
}
