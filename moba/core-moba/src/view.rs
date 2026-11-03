//! Builds the wire views from the authoritative match.

use crate::content::MATCH_TICKS;
use crate::protocol::{PrivateView, SeatView, UnitView, WorldView};
use crate::state::{CommandMode, Controller, MobaMatch};
use crate::types::{Seat, Team};

fn centi(mm: i32) -> u16 {
    (mm / 10).clamp(0, u16::MAX as i32) as u16
}

fn permille(hp: i32, max: i32) -> u16 {
    (hp.max(0) as i64 * 1000 / max.max(1) as i64) as u16
}

pub fn world(m: &MobaMatch) -> WorldView {
    let seats = Seat::all()
        .map(|seat| {
            let p = m.player(seat);
            let unit = m.unit(p.commander);
            SeatView {
                human: p.controller == Controller::Human,
                level: p.level,
                hp_permille: unit.map_or(0, |u| permille(u.hp, u.max_hp)),
                respawn_ticks: if unit.is_some() {
                    0
                } else {
                    p.respawn_at.saturating_sub(m.tick).min(u16::MAX as u32) as u16
                },
                kills: p.kills,
                deaths: p.deaths,
                commander: p.commander,
            }
        })
        .collect();
    let units = m
        .units
        .iter()
        .map(|u| UnitView {
            id: u.id,
            kind: u.kind,
            side: u.side,
            owner: u.owner,
            x: centi(u.pos.x),
            y: centi(u.pos.y),
            hp_permille: permille(u.hp, u.max_hp),
            target: u.target,
            attacked: m.tick > 0 && u.last_attack == m.tick,
        })
        .collect();
    WorldView {
        tick: m.tick,
        score: m.score,
        towers_left: [m.towers_left(Team::Blue), m.towers_left(Team::Red)],
        team_gold: [m.team_gold(Team::Blue), m.team_gold(Team::Red)],
        seats,
        units,
        events: m.events.clone(),
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
        commander_auto: p.mode == CommandMode::Auto,
        offers: p.offers,
        slot_rules: crate::content::slot_rules(seat.role()).map(crate::content::archetype_mask),
        locked: p.locked,
    }
}

/// Seconds left on the match clock.
pub fn seconds_left(tick: u32) -> u32 {
    MATCH_TICKS.saturating_sub(tick) / crate::content::TICK_HZ
}
