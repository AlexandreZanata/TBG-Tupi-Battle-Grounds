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
        let u = &mut m.units[i];
        let step = u.stats.speed;
        u.pos.step_toward(goal, step);
        u.pos = u.pos.clamp_to_map();
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

/// Pushes overlapping units apart; structures never move (M-09).
#[allow(clippy::needless_range_loop)] // pushes are gathered per slot, applied after
pub(super) fn separate(m: &mut MobaMatch) {
    let n = m.units.len();
    let mut push = vec![(0i64, 0i64); n];
    for i in 0..n {
        let a = &m.units[i];
        if !a.alive() || !a.mobile() {
            continue;
        }
        m.grid.near(a.pos, 6 * MM, |j| {
            if j == i || j >= n {
                return;
            }
            let b = &m.units[j];
            if !b.alive() {
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
        // Cap each tick's push so crowds relax smoothly instead of jumping.
        let cap = (u.stats.speed as i64).max(1);
        u.pos.x += dx.clamp(-cap, cap) as i32;
        u.pos.y += dy.clamp(-cap, cap) as i32;
        u.pos = u.pos.clamp_to_map();
    }
}
