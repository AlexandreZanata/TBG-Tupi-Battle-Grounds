//! Automatic spawns: purchased Troops, lane waves and neutral camps.

use crate::content;
use crate::geom::{Pos, MM};
use crate::map;
use crate::state::{Brain, MobaMatch};
use crate::types::{Champion, Destination, Lane, Seat, Side, Team, UnitKind};

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

pub(crate) fn troop(m: &mut MobaMatch, seat: Seat, kind: Champion, to: Destination) {
    let team = seat.team();
    let pos = offset(map::spawn_point(team), fan(m.next_id));
    let brain = match to {
        Destination::Lane(lane) => Brain::Lane { lane, waypoint: 1 },
        Destination::Jungle => Brain::Jungle { camp: None },
    };
    let power = m.player(seat).tier as i32 * content::TIER_PERCENT;
    m.spawn(
        UnitKind::Troop(kind),
        Side::Team(team),
        Some(seat),
        pos,
        brain,
    );
    let unit = m.units.last_mut().expect("just spawned");
    unit.power = power;
    unit.max_hp = unit.max_hp * (100 + power) / 100;
    unit.hp = unit.max_hp;
}

pub(super) fn waves(m: &mut MobaMatch) {
    if m.tick < content::FIRST_WAVE_TICK
        || !(m.tick - content::FIRST_WAVE_TICK).is_multiple_of(content::WAVE_EVERY_TICKS)
    {
        return;
    }
    for team in Team::ALL {
        for lane in Lane::ALL {
            let base = map::waypoint(lane, team, 0);
            let next = map::waypoint(lane, team, 1);
            // Leave the Stronghold a third of the way to the first waypoint.
            let start = Pos {
                x: base.x + (next.x - base.x) * 2 / 5,
                y: base.y + (next.y - base.y) * 2 / 5,
            };
            for (k, kind) in content::WAVE.into_iter().enumerate() {
                let brain = Brain::Lane { lane, waypoint: 1 };
                m.spawn(
                    UnitKind::Minion(kind),
                    Side::Team(team),
                    None,
                    offset(start, fan(k as u32)),
                    brain,
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
