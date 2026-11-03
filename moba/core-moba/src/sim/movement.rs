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
        m.units[i].pos = blocked_step(m, i, from, to);
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

/// Check the full segment so even a shallow crossing cannot tunnel through a body.
fn clear_step(m: &MobaMatch, i: usize, from: Pos, to: Pos) -> bool {
    let a = &m.units[i];
    let dx = i64::from(to.x - from.x);
    let dy = i64::from(to.y - from.y);
    let len = dx * dx + dy * dy;
    let mut clear = true;
    m.grid.near(from, a.stats.radius + 8 * MM, |j| {
        let b = &m.units[j];
        if !clear || i == j || !b.alive() || b.fx.carrier != 0 {
            return;
        }
        let gap = i64::from(a.stats.radius + b.stats.radius);
        let initial = from.dist2(b.pos);
        if initial < gap * gap {
            clear = to.dist2(b.pos) >= initial;
            return;
        }
        let along =
            (i64::from(b.pos.x - from.x) * dx + i64::from(b.pos.y - from.y) * dy).clamp(0, len);
        let p = Pos {
            x: from.x + (dx * along / len.max(1)) as i32,
            y: from.y + (dy * along / len.max(1)) as i32,
        };
        clear = p.dist2(b.pos) >= gap * gap;
    });
    clear
}
fn blocked_step(m: &MobaMatch, i: usize, from: Pos, to: Pos) -> Pos {
    if clear_step(m, i, from, to) {
        return to;
    }
    let dx = to.x - from.x;
    let dy = to.y - from.y;
    // Try approaching contact before following a tangent around the blocker.
    let mut lo = 0;
    let mut hi = 16;
    while hi - lo > 1 {
        let mid = (hi + lo) / 2;
        let p = Pos {
            x: from.x + dx * mid / 16,
            y: from.y + dy * mid / 16,
        };
        if clear_step(m, i, from, p) {
            lo = mid;
        } else {
            hi = mid;
        }
    }
    if lo > 0 {
        return Pos {
            x: from.x + dx * lo / 16,
            y: from.y + dy * lo / 16,
        };
    }
    for sign in [1, -1] {
        let p = Pos {
            x: from.x + (dx - sign * dy) * 7 / 10,
            y: from.y + (dy + sign * dx) * 7 / 10,
        }
        .clamp_to_map();
        if clear_step(m, i, from, p) {
            return p;
        }
    }
    for sign in [1, -1] {
        let p = Pos {
            x: from.x - sign * dy,
            y: from.y + sign * dx,
        }
        .clamp_to_map();
        if clear_step(m, i, from, p) {
            return p;
        }
    }
    from
}

/// Pushes overlapping units apart; structures never move (M-09).
#[allow(clippy::needless_range_loop)] // pushes are gathered per slot, applied after
pub(super) fn separate(m: &mut MobaMatch) {
    let n = m.units.len();
    let mut push = vec![(0i64, 0i64); n];
    for i in 0..n {
        let a = &m.units[i];
        if !a.alive() || !a.mobile() || a.fx.carrier != 0 {
            continue;
        }
        m.grid.near(a.pos, a.stats.radius + 8 * MM, |j| {
            if j == i || j >= n {
                return;
            }
            let b = &m.units[j];
            if !b.alive() || b.fx.carrier != 0 {
                return;
            }
            let min = (a.stats.radius + b.stats.radius) as i64;
            let d2 = a.pos.dist2(b.pos);
            if d2 >= min * min {
                return;
            }
            let d = d2.isqrt();
            let overlap = min - d;
            let share = if b.mobile() { overlap / 2 } else { overlap };
            let (dx, dy) = if d == 0 {
                (if i < j { -1 } else { 1 }, 0)
            } else {
                ((a.pos.x - b.pos.x) as i64, (a.pos.y - b.pos.y) as i64)
            };
            let len = (dx * dx + dy * dy).isqrt().max(1);
            push[i].0 += dx * share / len;
            push[i].1 += dy * share / len;
        });
    }
    for (u, (dx, dy)) in m.units.iter_mut().zip(push) {
        if !u.alive() || !u.mobile() || u.fx.carrier != 0 {
            continue;
        }
        // Cap each tick's push so crowds relax smoothly instead of jumping.
        let cap = (u.stats.speed as i64).max(1);
        u.pos.x += dx.clamp(-cap, cap) as i32;
        u.pos.y += dy.clamp(-cap, cap) as i32;
        u.pos = u.pos.clamp_to_map();
    }
}
