//! Body collision queries for walkers: swept circles against the terrain and
//! the bodies that block them, answered from the spatial grid (M-74, M-77).

use super::decision::Phase;
use crate::geom::{Pos, MM};
use crate::grid::Grid;
use crate::map;
use crate::state::Unit;

/// Largest body radius (6.75 units) plus the furthest a body moves after the
/// grid was built this tick, so a grid query never misses a blocker.
const QUERY_MARGIN: i32 = 9 * MM;

/// A unit planted in an action: finishing a windup, recovering or supporting.
/// It holds its ground; walking allies go around it instead of shoving it.
pub fn planted(u: &Unit) -> bool {
    matches!(
        u.decision.phase,
        Phase::Windup | Phase::Recover | Phase::Support
    )
}

/// Whether `b` blocks `a`'s body: enemies, neutrals, structures and planted
/// allies are solid; walking allies stay soft lateral pressure (M-72).
pub fn solid(a: &Unit, b: &Unit) -> bool {
    a.id != b.id
        && b.alive()
        && b.fx.carrier == 0
        && (a.team().is_none() || a.side != b.side || !b.mobile() || planted(b) && !planted(a))
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

/// The bodies of one tick, indexed by the grid built from the same slice.
#[derive(Clone, Copy)]
pub struct Bodies<'a> {
    pub units: &'a [Unit],
    grid: &'a Grid,
    /// Lane mission routes only avoid what cannot move out of the way.
    permanent: bool,
}

impl<'a> Bodies<'a> {
    pub fn new(units: &'a [Unit], grid: &'a Grid) -> Self {
        Self {
            units,
            grid,
            permanent: false,
        }
    }

    /// Structures, neutral monsters and immobile allies only.
    pub fn permanent(self) -> Self {
        Self {
            permanent: true,
            ..self
        }
    }

    fn blocks(&self, a: &Unit, b: &Unit) -> bool {
        solid(a, b)
            && (!self.permanent
                || b.kind.structure()
                || b.side == crate::types::Side::Neutral
                || b.side == a.side && !b.mobile())
    }

    /// The first blocking body along the sweep (distance, then slot order).
    pub fn blocker(&self, a: &Unit, from: Pos, to: Pos) -> Option<&'a Unit> {
        let mid = Pos {
            x: (from.x + to.x) / 2,
            y: (from.y + to.y) / 2,
        };
        let reach = (from.dist(to) / 2) as i32 + a.stats.radius + QUERY_MARGIN;
        let mut best: Option<(i64, usize)> = None;
        self.grid.near(mid, reach, |k| {
            let Some(b) = self.units.get(k) else { return };
            if self.blocks(a, b) && intersects(from, to, b.pos, gap(a, b)) {
                let key = (from.dist2(b.pos), k);
                if best.is_none_or(|old| key < old) {
                    best = Some(key);
                }
            }
        });
        best.map(|(_, k)| &self.units[k])
    }

    pub fn clear(&self, a: &Unit, from: Pos, to: Pos) -> bool {
        map::surface::clear_segment(from, to, a.stats.radius) && self.blocker(a, from, to).is_none()
    }

    /// Calls `f` with every unit whose cell lies near `at`.
    pub fn near(&self, at: Pos, radius: i32, mut f: impl FnMut(&'a Unit)) {
        self.grid.near(at, radius + QUERY_MARGIN, |k| {
            if let Some(u) = self.units.get(k) {
                f(u);
            }
        });
    }
}
