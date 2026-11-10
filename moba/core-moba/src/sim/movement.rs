//! Automatic lane/jungle walking, neutral leashes, chasing and spacing.

use super::spawn::{camp_alive, nearest_camp};
use crate::content::WAYPOINT_RADIUS;
use crate::geom::{Pos, MM};
use crate::map;
use crate::state::{Brain, MobaMatch};
use crate::types::{Lane, Team};

use super::bodies::{gap, planted, Bodies};

use super::navigation::CONTACT_TOLERANCE;

// Slot loop: brains read the whole match while one unit moves.
#[allow(clippy::needless_range_loop)]
pub(super) fn advance(m: &mut MobaMatch, engaged: &[bool]) {
    let snapshot = m.units.clone();
    let mut grid = std::mem::take(&mut m.motion_grid);
    grid.rebuild(snapshot.iter().map(|u| u.pos));
    let bodies = Bodies::new(&snapshot, &grid);
    let mut route_budget = 4;
    for i in 0..engaged.len() {
        let u = &m.units[i];
        if !u.alive() || !u.mobile() || engaged[i] {
            continue;
        }
        let chase = snapshot
            .binary_search_by_key(&u.target, |v| v.id)
            .ok()
            .map(|j| &snapshot[j])
            .filter(|v| v.alive())
            .map(|v| v.pos);
        let goal = match chase {
            Some(p) => p,
            None => match brain_goal(m, i) {
                Some(p) => p,
                None => continue,
            },
        };
        let step = crate::champions::speed(&m.units[i], m.tick);
        if step == 0 {
            super::decision::phase(&mut m.units[i], super::decision::Phase::Wait);
            continue;
        }
        let from = m.units[i].pos;
        let mut to = from;
        to.step_toward(goal, step);
        if formation_holds(m, i, to) {
            super::decision::phase(&mut m.units[i], super::decision::Phase::Formation);
            m.units[i].decision.reason = super::decision::Reason::Formation;
            continue;
        }
        let next = super::navigation::step(m, &bodies, i, goal, step, &mut route_budget);
        // Decisions use the snapshot; settled bodies reserve their swept step
        // in id order so opposing movers cannot walk into each other.
        if Bodies::new(&m.units, &grid).clear(&m.units[i], from, next) {
            m.units[i].pos = next;
        } else {
            super::decision::phase(&mut m.units[i], super::decision::Phase::Repath);
            m.units[i].decision.reason = super::decision::Reason::Obstructed;
        }
        if m.units[i].pos != from {
            crate::champions::walked(&mut m.units[i]);
        }
    }
    m.motion_grid = grid;
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
    if u.pos.within(goal, WAYPOINT_RADIUS + u.stats.radius)
        && (waypoint as usize) + 1 < map::waypoint_count(lane)
    {
        u.brain = Brain::Lane {
            lane,
            waypoint: waypoint + 1,
        };
    }
    goal
}

/// Walks to a live camp (any jungle for Troops);
/// A Jungle mission waits at the next camp even before the first spawn.
fn jungle_goal(m: &mut MobaMatch, i: usize, camp: Option<u8>, only: Option<Team>) -> Pos {
    let pos = m.units[i].pos;
    let chosen = camp
        .filter(|&c| camp_alive(m, c as usize))
        .or_else(|| nearest_camp(m, pos, only))
        .or_else(|| {
            (0..map::CAMP_COUNT)
                .filter(|&c| only.is_none_or(|t| map::camp(c).1 == t))
                .min_by_key(|&c| {
                    (
                        m.camps[c].respawn_at.unwrap_or(0),
                        pos.dist2(map::camp(c).0),
                    )
                })
                .map(|c| c as u8)
        });
    let u = &mut m.units[i];
    if let (Brain::Jungle { camp }, Some(c)) = (&mut u.brain, chosen) {
        *camp = Some(c);
    }
    chosen.map_or(pos, |c| {
        m.units
            .iter()
            .filter(|u| u.alive() && u.brain == Brain::Neutral { camp: c })
            .min_by_key(|u| (pos.dist2(u.pos), u.id))
            .map_or(map::camp(c as usize).0, |u| u.pos)
    })
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

/// Crowd pressure (M-72, M-77). Walking allies nudge each other sideways;
/// a planted unit holds its ground (only a deep overlap moves it) and pushes
/// walkers like an obstacle; a walker is never pushed back against its own
/// route, so pressure cannot make it step back and forth.
pub(super) fn separate(m: &mut MobaMatch) {
    let mut grid = std::mem::take(&mut m.motion_grid);
    grid.rebuild(m.units.iter().map(|u| u.pos));
    let mut pushes = vec![(0i64, 0i64); m.units.len()];
    for (i, a) in m.units.iter().enumerate() {
        if !a.alive() || !a.mobile() || a.fx.carrier != 0 {
            continue;
        }
        let anchored = planted(a);
        grid.near(a.pos, a.stats.radius + 8 * MM, |j| {
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
            // A few millimetres of contact are tolerated: correcting them
            // every tick is what makes a crowd visibly shiver.
            if limit - distance < CONTACT_TOLERANCE || anchored && distance * 2 >= limit {
                return;
            }
            let yields = b.mobile() && !planted(b);
            let share = (limit - distance) / if yields { 2 } else { 1 };
            let (dx, dy) = separation_direction(a, b);
            let len = (dx * dx + dy * dy).isqrt().max(1);
            pushes[i].0 += dx * share / len;
            pushes[i].1 += dy * share / len;
        });
    }
    for (i, push) in pushes.into_iter().enumerate() {
        let u = &m.units[i];
        let (dx, dy) = without_retreat(u, push);
        if dx == 0 && dy == 0 {
            continue;
        }
        // A walker that is waiting in a queue yields more gently.
        let cap = i64::from(
            (u.stats.speed
                / if u.navigation.velocity == (0, 0) {
                    4
                } else {
                    2
                })
            .max(1),
        );
        let len = (dx * dx + dy * dy).isqrt().max(cap);
        let from = u.pos;
        let to = Pos {
            x: from.x + (dx * cap / len) as i32,
            y: from.y + (dy * cap / len) as i32,
        }
        .clamp_to_map();
        if Bodies::new(&m.units, &grid).clear(u, from, to) {
            m.units[i].pos = to;
        }
    }
    m.motion_grid = grid;
}

/// Drops the part of a push that points back against the walker's own step;
/// the sideways part keeps the crowd spreading.
fn without_retreat(u: &crate::state::Unit, (dx, dy): (i64, i64)) -> (i64, i64) {
    let (hx, hy) = u.navigation.velocity;
    let (hx, hy) = (i64::from(hx), i64::from(hy));
    let dot = dx * hx + dy * hy;
    let h2 = hx * hx + hy * hy;
    if planted(u) || dot >= 0 || h2 == 0 {
        return (dx, dy);
    }
    (dx - hx * dot / h2, dy - hy * dot / h2)
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
            // Nearly in single file, the lane side follows id order so two
            // walkers never trade sides back and forth across the line.
            let across = dx * x + dy * y;
            let length = (x * x + y * y).isqrt().max(1);
            let aligned = (across / length).abs() < i64::from(a.stats.radius) / 4;
            let side = if aligned { tie } else { across.signum() };
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

/// Direction inertia (M-77): whatever moved a body this tick (its own step,
/// crowd pressure or both), the part that points back against last tick's
/// motion is cancelled, so no body ever goes A → B → A on screen. A real
/// change of direction therefore takes one still beat, like turning round.
/// The kept motion is re-validated; if it would touch a body or the
/// terrain the full motion stays, since that one was already validated.
pub(super) fn settle(m: &mut MobaMatch, start: &[Pos]) {
    let mut grid = std::mem::take(&mut m.motion_grid);
    grid.rebuild(m.units.iter().map(|u| u.pos));
    for (i, &from) in start.iter().enumerate().take(m.units.len()) {
        let u = &m.units[i];
        if !u.alive() || !u.mobile() || u.fx.carrier != 0 {
            continue;
        }
        let (dx, dy) = (i64::from(u.pos.x - from.x), i64::from(u.pos.y - from.y));
        let (px, py) = u.navigation.velocity;
        let (px, py) = (i64::from(px), i64::from(py));
        let dot = dx * px + dy * py;
        let p2 = px * px + py * py;
        let mut motion = (dx, dy);
        if dot < 0 && p2 > 0 {
            let kept = Pos {
                x: from.x + (dx - px * dot / p2) as i32,
                y: from.y + (dy - py * dot / p2) as i32,
            };
            if Bodies::new(&m.units, &grid).clear(u, from, kept) {
                m.units[i].pos = kept;
                motion = (i64::from(kept.x - from.x), i64::from(kept.y - from.y));
            }
        }
        m.units[i].navigation.velocity = (motion.0 as i32, motion.1 as i32);
    }
    m.motion_grid = grid;
}
