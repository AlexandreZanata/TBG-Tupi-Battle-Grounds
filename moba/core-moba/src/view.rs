//! Builds the wire views from the authoritative match.

use crate::content::MATCH_TICKS;
use crate::protocol::{PrivateView, SeatView, UnitView, WorldView};
use crate::state::{Controller, Event, MobaMatch};
use crate::types::{Seat, Team};

fn centi(mm: i32) -> u16 {
    (mm / 10).clamp(0, u16::MAX as i32) as u16
}

fn permille(hp: i32, max: i32) -> u16 {
    (hp.max(0) as i64 * 1000 / max.max(1) as i64) as u16
}

pub fn world(m: &MobaMatch, team: Team) -> WorldView {
    let vision = crate::vision::Vision::new(m, team);
    let seats = Seat::all()
        .map(|seat| {
            let p = m.player(seat);
            SeatView {
                human: p.controller == Controller::Human,
                kills: p.kills,
                deaths: p.deaths,
            }
        })
        .collect();
    let units = m
        .units
        .iter()
        .enumerate()
        .filter(|(i, u)| vision.visible[*i] || u.kind.structure())
        .map(|(i, u)| UnitView {
            id: u.id,
            kind: u.kind,
            side: u.side,
            owner: u.owner,
            x: centi(u.pos.x),
            y: centi(u.pos.y),
            hp_permille: permille(
                if u.kind.structure() && !vision.visible[i] {
                    m.structure_hp[team.index()]
                        .iter()
                        .find(|(id, _)| *id == u.id)
                        .map_or(u.max_hp, |(_, hp)| *hp)
                } else {
                    u.hp
                },
                u.max_hp,
            ),
            target: if vision.visible[i] && vision.unit(u.target) {
                u.target
            } else {
                0
            },
            attacked: vision.visible[i]
                && vision.unit(u.target)
                && m.tick > 0
                && u.last_attack == m.tick,
        })
        .collect();
    WorldView {
        tick: m.tick,
        score: m.score,
        towers_left: [m.towers_left(Team::Blue), m.towers_left(Team::Red)],
        team_gold: [m.team_gold(Team::Blue), m.team_gold(Team::Red)],
        seats,
        units,
        camps: m
            .camps
            .iter()
            .enumerate()
            .filter_map(|(i, c)| {
                let at = c.respawn_at?;
                (at > m.tick && vision.point(crate::map::camp(i).0)).then_some(
                    crate::protocol::CampView {
                        camp: i as u8,
                        respawn_ticks: at.saturating_sub(m.tick).min(u16::MAX as u32) as u16,
                    },
                )
            })
            .collect(),
        events: m
            .events
            .iter()
            .copied()
            .filter(|event| event_visible(&vision, team, event))
            .collect(),
        outcome: m.outcome,
    }
}

pub fn private(m: &MobaMatch, seat: Seat) -> PrivateView {
    let p = m.player(seat);
    PrivateView {
        seat,
        gold: p.gold,
        essence: p.essence,
        tier: p.tier,
        troops: m.troops_alive(seat),
        destination: p.destination,
        offers: p.offers,
        slot_rules: crate::content::slot_rules(seat.role()).map(crate::content::archetype_mask),
        locked: p.locked,
    }
}

/// Seconds left on the match clock.
pub fn seconds_left(tick: u32) -> u32 {
    MATCH_TICKS.saturating_sub(tick) / crate::content::TICK_HZ
}

fn event_visible(vision: &crate::vision::Vision<'_>, team: Team, event: &Event) -> bool {
    match *event {
        // Kills, destroyed structures and outcome are public scoreboard information.
        Event::TroopKilled { .. } | Event::TowerDestroyed { .. } | Event::Ended(_) => true,
        Event::CampCleared { camp, .. } => vision.point(crate::map::camp(camp as usize).0),
        Event::TroopSent { seat, .. } | Event::Upgraded { seat, .. } => seat.team() == team,
        Event::LevelUp { .. } | Event::CommanderKilled { .. } => false,
    }
}
