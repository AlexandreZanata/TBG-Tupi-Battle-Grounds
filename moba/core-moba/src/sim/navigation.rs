//! Persistent deterministic body-safe routes; mobile allies remain soft obstacles.
use super::pathfinding::route;
use crate::{
    geom::{Pos, MM},
    map,
    state::{MobaMatch, Unit},
};
use serde::{Deserialize, Serialize};

pub(super) const CELL: i32 = 2 * MM;
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
    pub failed_since: Option<u32>,
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
    snapshot: &[Unit],
    i: usize,
    goal: Pos,
    speed: i32,
    budget: &mut u8,
) -> Pos {
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
    // Reaching the old contact point requests the next leg toward the
    // current target. It never rejects an otherwise valid engagement.
    while nav.path.first().is_some_and(|p| a.pos.within(*p, 50)) {
        nav.path.remove(0);
    }
    // Retain a chosen detour until it finishes, rather than switching sides each tick.
    if nav.path.is_empty()
        && clear(snapshot, a, a.pos, direct)
        && acceptable(a, target, direct, goal)
    {
        nav.path.push(direct);
        nav.failed_since = None;
    }
    if nav.path.is_empty() && m.tick >= nav.retry_at && *budget > 0 {
        *budget -= 1;
        match route(snapshot, a, target, goal) {
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
    let failed = nav.stalled >= STALL
        || nav
            .failed_since
            .is_some_and(|since| m.tick.saturating_sub(since) >= u32::from(STALL));
    if failed {
        nav.path.clear();
        if a.target != 0 {
            nav.rejected.push((a.target, m.tick + u32::from(STALL)));
        }
        nav.stalled = 0;
        nav.failed_since = None;
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
