//! Automatic spawns: purchased Troops, lane waves and neutral camps.

use crate::content;
use crate::geom::{Pos, MM};
use crate::map;
use crate::state::{Brain, MobaMatch};
use crate::types::{Champion, Destination, Lane, Seat, Side, Team, UnitKind};
use tbg_core::content::occupancy::{CELL, GAP};

/// Small fixed fan of offsets so units do not spawn stacked.
fn fan(k: u32) -> Pos {
    const RING: [(i32, i32); 6] = [(0, 0), (2, -1), (-1, 2), (2, 2), (-2, 0), (0, -2)];
    let (x, y) = RING[k as usize % RING.len()];
    Pos {
        x: x * MM,
        y: y * MM,
    }
}

fn offset(p: Pos, d: Pos) -> Pos {
    Pos {
        x: p.x + d.x,
        y: p.y + d.y,
    }
    .clamp_to_map()
}

#[derive(Clone, Copy)]
struct Deployment {
    kind: UnitKind,
    team: Team,
    owner: Option<Seat>,
    destination: Destination,
    power: i32,
}

pub(crate) fn troop(m: &mut MobaMatch, seat: Seat, kind: Champion, to: Destination) {
    deploy_at_exit(
        m,
        Deployment {
            kind: UnitKind::Troop(kind),
            team: seat.team(),
            owner: Some(seat),
            destination: to,
            power: i32::from(m.player(seat).tier) * content::TIER_PERCENT,
        },
    );
}

fn deploy_at_exit(m: &mut MobaMatch, order: Deployment) {
    let (a, b, brain) = match order.destination {
        Destination::Lane(lane) => (
            map::waypoint(lane, order.team, 0),
            map::waypoint(lane, order.team, 1),
            Brain::Lane { lane, waypoint: 1 },
        ),
        Destination::Jungle => {
            let origin = map::stronghold(order.team);
            let camp = (0..map::CAMP_COUNT)
                .map(map::camp)
                .filter(|(_, team)| *team == order.team)
                .min_by_key(|(at, _)| origin.dist2(*at))
                .map_or(origin, |(at, _)| at);
            (origin, camp, Brain::Jungle { camp: None })
        }
    };
    let radius = content::base_stats(order.kind).radius;
    let length = a.dist(b).max(1) as i32;
    let dx = b.x - a.x;
    let dy = b.y - a.y;
    // PK-01: start at the deployment line, find an open lateral cell, then
    // try ranks behind it. Each 5v5 route has its own deployment strip.
    let gate = (crate::map::BASE_RADIUS * MM + radius * 2).min(length - content::WAYPOINT_RADIUS);
    let rear = content::base_stats(UnitKind::Stronghold).radius + radius + GAP;
    let side = (crate::map::LANE_HALF_WIDTH - radius).max(0) / CELL;
    let blockers: Vec<_> = m
        .units
        .iter()
        .filter(|u| {
            u.alive()
                && u.fx.carrier == 0
                && a.within(
                    u.pos,
                    gate + radius + u.stats.radius + crate::map::LANE_HALF_WIDTH,
                )
        })
        .collect();
    let point = |progress: i32, lateral: i32| Pos {
        x: a.x
            + (i64::from(dx) * i64::from(progress) - i64::from(dy) * i64::from(lateral))
                .div_euclid(i64::from(length)) as i32,
        y: a.y
            + (i64::from(dy) * i64::from(progress) + i64::from(dx) * i64::from(lateral))
                .div_euclid(i64::from(length)) as i32,
    };
    let mut best = None;
    'ranks: for progress in (rear..=gate).rev().step_by((radius / 2).max(CELL) as usize) {
        for cell in 0..=side * 2 {
            let lateral = ((cell + 1) / 2) * if cell % 2 == 0 { -CELL } else { CELL };
            let pos = point(progress, lateral);
            let mut hard = 0;
            let mut crowd = 0;
            for unit in &blockers {
                let overlap =
                    (i64::from(radius + unit.stats.radius + GAP) - pos.dist(unit.pos)).max(0);
                if unit.mobile() && unit.side == Side::Team(order.team) {
                    crowd += overlap;
                } else {
                    hard += overlap;
                }
            }
            // A free cell is preferred, but never required to deploy. In a
            // full gate overlap teammates before overlapping a structure.
            let score = (hard, crowd);
            if best.is_none_or(|(old, _)| score < old) {
                best = Some((score, pos));
            }
            if score == (0, 0) {
                break 'ranks;
            }
        }
    }
    let pos = best.map_or_else(|| point(gate, 0), |(_, pos)| pos);
    m.spawn(order.kind, Side::Team(order.team), order.owner, pos, brain);
    if let Some(unit) = m.units.last_mut() {
        unit.power = order.power;
        unit.max_hp = unit.max_hp * (100 + order.power) / 100;
        unit.hp = unit.max_hp;
    }
}

pub(super) fn waves(m: &mut MobaMatch) {
    if m.tick < content::FIRST_WAVE_TICK
        || !(m.tick - content::FIRST_WAVE_TICK).is_multiple_of(content::WAVE_EVERY_TICKS)
    {
        return;
    }
    for team in Team::ALL {
        for lane in Lane::ALL {
            for kind in content::WAVE {
                deploy_at_exit(
                    m,
                    Deployment {
                        kind: UnitKind::Minion(kind),
                        team,
                        owner: None,
                        destination: Destination::Lane(lane),
                        power: 0,
                    },
                );
            }
        }
    }
}

pub(super) fn camps(m: &mut MobaMatch) {
    for c in 0..map::CAMP_COUNT {
        match m.camps[c].respawn_at {
            Some(at) if m.tick >= at => {}
            _ => continue,
        }
        m.camps[c].respawn_at = None;
        let (center, _) = map::camp(c);
        let brain = Brain::Neutral { camp: c as u8 };
        for (i, &kind) in content::camp_content(map::camp_kind(c))
            .0
            .iter()
            .enumerate()
        {
            m.spawn(
                UnitKind::Neutral(kind),
                Side::Neutral,
                None,
                offset(center, fan(i as u32)),
                brain,
            );
        }
    }
}

/// True while a camp's monsters are up.
pub(crate) fn camp_alive(m: &MobaMatch, camp: usize) -> bool {
    m.camps[camp].respawn_at.is_none()
}

/// Nearest live camp to `p`, optionally only inside `team`'s jungle.
pub(crate) fn nearest_camp(m: &MobaMatch, p: Pos, team: Option<Team>) -> Option<u8> {
    (0..map::CAMP_COUNT)
        .filter(|&c| camp_alive(m, c) && team.is_none_or(|t| map::camp(c).1 == t))
        .min_by_key(|&c| map::camp(c).0.dist2(p))
        .map(|c| c as u8)
}
