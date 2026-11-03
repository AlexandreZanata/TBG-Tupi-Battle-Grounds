//! Automatic lane/jungle walking, neutral leashes, chasing and spacing.

use super::spawn::{camp_alive, nearest_camp};
use crate::content::WAYPOINT_RADIUS;
use crate::geom::{Pos, MM};
use crate::map;
use crate::state::{Brain, MobaMatch};
use crate::types::{Lane, Team};

// Slot loop: brains read the whole match while one unit moves.
#[allow(clippy::needless_range_loop)]
pub(super) fn advance(m: &mut MobaMatch, engaged: &[bool]) {
    for i in 0..engaged.len() {
        let u = &m.units[i];
        if !u.alive() || !u.mobile() || engaged[i] {
            continue;
        }
        let chase = m.slot(u.target).map(|j| m.units[j].pos);
        let goal = match chase {
            Some(p) => p,
            None => match brain_goal(m, i) {
                Some(p) => p,
                None => continue,
            },
        };
        let step = crate::champions::speed(&m.units[i], m.tick);
        if step == 0 {
            continue;
        }
        let from = m.units[i].pos;
        let mut to = from;
        to.step_toward(goal, step);
        let to = to.clamp_to_map();
        if formation_holds(m, i, to) {
            continue;
        }
        m.units[i].pos = blocked_step(m, i, from, to, goal, step);
        if m.units[i].pos != from {
            crate::champions::walked(&mut m.units[i]);
        }
    }
}

/// Where the unit's behaviour wants it when it has no target; may update its
/// brain (next waypoint, next camp, arrival).
fn brain_goal(m: &mut MobaMatch, i: usize) -> Option<Pos> {
    let u = &m.units[i];
    match u.brain {
        Brain::Static => None,
        Brain::Lane { lane, waypoint } => Some(lane_goal(m, i, lane, waypoint)),
        Brain::Jungle { camp } => Some(jungle_goal(m, i, camp, None)),
        Brain::Neutral { camp } => {
            let home = map::camp(camp as usize).0;
            let u = &mut m.units[i];
            if u.pos.within(home, 4 * MM) {
                u.hp = u.max_hp;
            }
            Some(home)
        }
    }
}

fn lane_goal(m: &mut MobaMatch, i: usize, lane: Lane, waypoint: u8) -> Pos {
    let u = &mut m.units[i];
    let team = u.team().unwrap_or(Team::Blue);
    let goal = map::waypoint(lane, team, waypoint as usize);
    if u.pos.within(goal, WAYPOINT_RADIUS) && (waypoint as usize) + 1 < map::waypoint_count(lane) {
        u.brain = Brain::Lane {
            lane,
            waypoint: waypoint + 1,
        };
    }
    goal
}

/// Walks to a live camp (any jungle for Troops);
/// Troops with no camp left join the mid lane.
fn jungle_goal(m: &mut MobaMatch, i: usize, camp: Option<u8>, only: Option<Team>) -> Pos {
    let pos = m.units[i].pos;
    let chosen = camp
        .filter(|&c| camp_alive(m, c as usize))
        .or_else(|| nearest_camp(m, pos, only));
    let u = &mut m.units[i];
    match (&mut u.brain, chosen) {
        (Brain::Jungle { camp }, Some(c)) => *camp = Some(c),
        (Brain::Jungle { .. }, None) => {
            let team = u.team().unwrap_or(Team::Blue);
            let waypoint = map::nearest_waypoint(Lane::Mid, team, pos) as u8;
            u.brain = Brain::Lane {
                lane: Lane::Mid,
                waypoint,
            };
        }
        _ => {}
    }
    chosen.map_or(pos, |c| map::camp(c as usize).0)
}

/// An engaged Oilpress anchors nearby followers behind its battle front.
fn formation_holds(m: &MobaMatch, i: usize, to: Pos) -> bool {
    use tbg_core::content::Ability;
    let u = &m.units[i];
    let mut hold = false;
    m.grid.near(u.pos, u.stats.sight, |j| {
        let a = &m.units[j];
        if hold || i == j || a.side != u.side || !a.alive() {
            return;
        }
        let Some((engage, gap)) =
            crate::champions::abilities(a)
                .into_iter()
                .find_map(|p| match p {
                    Ability::Formation { engage, gap } => Some((engage, gap)),
                    _ => None,
                })
        else {
            return;
        };
        let Some(v) = m.unit(a.target).filter(|v| {
            v.alive()
                && a.side.hostile(v.side)
                && v.pos
                    .within(a.pos, engage + a.stats.radius + v.stats.radius)
        }) else {
            return;
        };
        let dx = i64::from(v.pos.x - a.pos.x);
        let dy = i64::from(v.pos.y - a.pos.y);
        let len = (dx * dx + dy * dy).isqrt();
        if len == 0 {
            return;
        }
        let side = |p: Pos| (i64::from(p.x - a.pos.x) * dx + i64::from(p.y - a.pos.y) * dy) / len;
        let stop = -i64::from(gap + a.stats.radius + u.stats.radius);
        hold = u.pos.within(a.pos, u.stats.sight) && side(u.pos) <= stop && side(to) > stop;
    });
    hold
}

/// Swept body contact in the map plane, with the shared PK-01 gap.
fn intersects(from: Pos, to: Pos, center: Pos, gap: i32) -> bool {
    let dx = i64::from(to.x - from.x);
    let dy = i64::from(to.y - from.y);
    let len = dx * dx + dy * dy;
    let projection = i64::from(center.x - from.x) * dx + i64::from(center.y - from.y) * dy;
    let along = projection.clamp(0, len);
    let nearest = Pos {
        x: from.x + (dx * along / len.max(1)) as i32,
        y: from.y + (dy * along / len.max(1)) as i32,
    };
    let initial = from.dist2(center);
    let limit = i64::from(gap).pow(2);
    if initial < limit {
        projection > 0
    } else {
        nearest.dist2(center) < limit
    }
}

fn gap(a: &crate::state::Unit, b: &crate::state::Unit) -> i32 {
    a.stats.radius
        + b.stats.radius
        + if a.side == b.side {
            tbg_core::content::occupancy::GAP
        } else {
            0
        }
}

fn solid(a: &crate::state::Unit, b: &crate::state::Unit) -> bool {
    b.alive() && b.fx.carrier == 0 && (a.team().is_none() || a.side != b.side || !b.mobile())
}

fn blocker(m: &MobaMatch, i: usize, from: Pos, to: Pos) -> Option<usize> {
    let a = &m.units[i];
    let mut best = None;
    m.grid.near(from, a.stats.radius + 8 * MM, |j| {
        let b = &m.units[j];
        if i != j && solid(a, b) && intersects(from, to, b.pos, gap(a, b)) {
            let key = (from.dist2(b.pos), j);
            if best.is_none_or(|old| key < old) {
                best = Some(key);
            }
        }
    });
    best.map(|(_, j)| j)
}

fn blocked_step(m: &mut MobaMatch, i: usize, from: Pos, to: Pos, goal: Pos, step: i32) -> Pos {
    // Like the BattleLine's persistent lateral slot, keep one passing side
    // until the obstacle is clear. A fresh direct step every tick oscillates.
    let active = m.units[i].bypass.and_then(|(id, side)| {
        let j = m.slot(id)?;
        let b = &m.units[j];
        (solid(&m.units[i], b) && intersects(from, goal, b.pos, gap(&m.units[i], b)))
            .then_some((j, side))
    });
    if let Some((j, side)) = active {
        let next = tangent(from, m.units[j].pos, step, side);
        if next != from && blocker(m, i, from, next).is_none() {
            return next;
        }
    }
    let Some(j) = blocker(m, i, from, to) else {
        if active.is_none() {
            m.units[i].bypass = None;
        }
        return to;
    };
    let obstacle = &m.units[j];
    let preferred = active.filter(|(old, _)| *old == j).map(|(_, side)| side);
    let passing = [1, -1]
        .into_iter()
        .filter_map(|side| {
            if m.units[i].target == obstacle.id || preferred.is_some_and(|old| old != side) {
                return None;
            }
            let next = tangent(from, obstacle.pos, step, side);
            (next != from && blocker(m, i, from, next).is_none()).then_some((next, side))
        })
        .min_by_key(|(next, _)| next.dist2(goal));
    if let Some((next, side)) = passing {
        m.units[i].bypass = Some((obstacle.id, side));
        return next;
    }
    // No free side: close the gap and wait behind the next body.
    let mut lo = 0;
    let mut hi = 16;
    let at = |k| Pos {
        x: from.x + (to.x - from.x) * k / 16,
        y: from.y + (to.y - from.y) * k / 16,
    };
    while hi - lo > 1 {
        let mid = (hi + lo) / 2;
        if blocker(m, i, from, at(mid)).is_none() {
            lo = mid;
        } else {
            hi = mid;
        }
    }
    at(lo)
}

fn tangent(from: Pos, center: Pos, step: i32, side: i32) -> Pos {
    let dx = from.x - center.x;
    let dy = from.y - center.y;
    let goal = Pos {
        x: from.x - dy * side + dx / 10,
        y: from.y + dx * side + dy / 10,
    };
    let mut next = from;
    next.step_toward(goal, step);
    next.clamp_to_map()
}

/// Shared PK-01 bodies and gaps give allies soft pressure instead of a stop.
pub(super) fn separate(m: &mut MobaMatch) {
    let mut pushes = vec![(0i64, 0i64); m.units.len()];
    for (i, a) in m.units.iter().enumerate() {
        if !a.alive() || !a.mobile() || a.fx.carrier != 0 {
            continue;
        }
        m.grid.near(a.pos, a.stats.radius + 8 * MM, |j| {
            let b = &m.units[j];
            if i == j || !b.alive() || b.fx.carrier != 0 {
                return;
            }
            let limit = i64::from(gap(a, b));
            let d2 = a.pos.dist2(b.pos);
            if d2 >= limit * limit {
                return;
            }
            let distance = d2.isqrt();
            let share = (limit - distance) / if b.mobile() { 2 } else { 1 };
            let (dx, dy) = separation_direction(a, b);
            let len = (dx * dx + dy * dy).isqrt().max(1);
            pushes[i].0 += dx * share / len;
            pushes[i].1 += dy * share / len;
        });
    }
    for (i, (dx, dy)) in pushes.into_iter().enumerate() {
        let u = &m.units[i];
        if dx == 0 && dy == 0 {
            continue;
        }
        // A crowd can spread without cancelling a walking tick's advance.
        if u.team().is_none() {
            let cap = i64::from(u.stats.speed.max(1));
            m.units[i].pos = Pos {
                x: u.pos.x + dx.clamp(-cap, cap) as i32,
                y: u.pos.y + dy.clamp(-cap, cap) as i32,
            }
            .clamp_to_map();
            continue;
        }
        let cap = i64::from((u.stats.speed / 2).max(1));
        let len = (dx * dx + dy * dy).isqrt().max(cap);
        let from = u.pos;
        let to = Pos {
            x: from.x + (dx * cap / len) as i32,
            y: from.y + (dy * cap / len) as i32,
        }
        .clamp_to_map();
        if blocker(m, i, from, to).is_none() {
            m.units[i].pos = to;
        }
    }
}

fn separation_direction(a: &crate::state::Unit, b: &crate::state::Unit) -> (i64, i64) {
    let dx = i64::from(a.pos.x - b.pos.x);
    let dy = i64::from(a.pos.y - b.pos.y);
    let tie = if a.id < b.id { -1 } else { 1 };
    if a.team().is_some() && a.side == b.side && b.mobile() {
        if let (Some(team), Brain::Lane { lane, waypoint }) = (a.team(), a.brain) {
            let from = map::waypoint(lane, team, usize::from(waypoint.saturating_sub(1)));
            let to = map::waypoint(lane, team, usize::from(waypoint));
            let x = i64::from(from.y - to.y);
            let y = i64::from(to.x - from.x);
            let side = (dx * x + dy * y).signum();
            let side = if side == 0 { tie } else { side };
            if x != 0 || y != 0 {
                return (x * side, y * side);
            }
        }
    }
    if dx == 0 && dy == 0 {
        (tie, 0)
    } else {
        (dx, dy)
    }
}
