//! Fixed-point world geometry: positions in millimetres on a 200 x 200 unit map.

use serde::{Deserialize, Serialize};

/// Millimetres per world unit.
pub const MM: i32 = 1000;

/// A point on the ground plane. `y` grows from the Blue base towards the Red base.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Pos {
    pub x: i32,
    pub y: i32,
}

impl Pos {
    /// A point given in whole world units.
    pub const fn units(x: i32, y: i32) -> Self {
        Self {
            x: x * MM,
            y: y * MM,
        }
    }

    pub fn dist2(self, other: Pos) -> i64 {
        let dx = (self.x - other.x) as i64;
        let dy = (self.y - other.y) as i64;
        dx * dx + dy * dy
    }

    pub fn dist(self, other: Pos) -> i64 {
        self.dist2(other).isqrt()
    }

    /// True when `other` lies within `radius` millimetres.
    pub fn within(self, other: Pos, radius: i32) -> bool {
        self.dist2(other) <= (radius as i64) * (radius as i64)
    }

    /// Moves at most `step` millimetres towards `goal`; returns true on arrival.
    pub fn step_toward(&mut self, goal: Pos, step: i32) -> bool {
        let d = self.dist(goal);
        if d <= step as i64 {
            *self = goal;
            return true;
        }
        self.x += ((goal.x - self.x) as i64 * step as i64 / d) as i32;
        self.y += ((goal.y - self.y) as i64 * step as i64 / d) as i32;
        false
    }

    /// Keeps the point inside the playable square.
    pub fn clamp_to_map(self) -> Pos {
        let lo = crate::map::EDGE * MM;
        let hi = (crate::map::SIZE - crate::map::EDGE) * MM;
        Pos {
            x: self.x.clamp(lo, hi),
            y: self.y.clamp(lo, hi),
        }
    }
}
