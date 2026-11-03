//! Deaths: bounties, Essence, experience, kill score, camp timers (M-12..M-21).

use crate::content::{self, SHARE_RANGE};
use crate::geom::Pos;
use crate::state::{Brain, Credit, EndReason, Event, MobaMatch, Outcome};
use crate::types::{Seat, Side, Team, UnitKind};

struct Death {
    kind: UnitKind,
    side: Side,
    owner: Option<Seat>,
    pos: Pos,
    credit: Option<Credit>,
    camp: Option<u8>,
}

pub(super) fn resolve_deaths(m: &mut MobaMatch) {
    let deaths: Vec<Death> = m
        .units
        .iter()
        .filter(|u| !u.alive())
        .map(|u| Death {
            kind: u.kind,
            side: u.side,
            owner: u.owner,
            pos: u.pos,
            credit: u.last_hit,
            camp: match u.brain {
                Brain::Neutral { camp } => Some(camp),
                _ => None,
            },
        })
        .collect();
    if deaths.is_empty() {
        return;
    }
    m.units.retain(|u| u.alive());
    for d in &deaths {
        reward(m, d);
    }
    for camp in deaths.iter().filter_map(|d| d.camp) {
        close_camp(
            m,
            camp,
            deaths
                .iter()
                .find(|d| d.camp == Some(camp))
                .and_then(|d| d.credit),
        );
    }
}

/// The team credited with a death: the last hitter's, else the victim's enemy.
fn killer_team(d: &Death) -> Option<Team> {
    d.credit
        .and_then(|c| c.side.team())
        .or_else(|| d.side.team().map(Team::enemy))
}

fn reward(m: &mut MobaMatch, d: &Death) {
    let killer = d.credit.and_then(|c| c.seat);
    let Some(team) = killer_team(d) else { return };
    share_xp(m, team, d.pos, content::xp_reward(d.kind));
    match d.kind {
        UnitKind::Minion(_) => grant(m, killer, content::MINION_BOUNTY, 0),
        UnitKind::Troop(kind) => grant(m, killer, content::troop_cost(kind) / 3, 0),
        UnitKind::Neutral(kind) => {
            let (gold, essence, _) = content::neutral_rewards(kind);
            grant(m, killer, gold, essence);
        }
        UnitKind::Commander(_) => commander_death(m, d, team, killer),
        UnitKind::Tower(tier, lane) => {
            for seat in Seat::all().filter(|s| s.team() == team) {
                grant(
                    m,
                    Some(seat),
                    content::TOWER_TEAM_BOUNTY,
                    content::TOWER_TEAM_ESSENCE,
                );
            }
            m.events.push(Event::TowerDestroyed {
                team: team.enemy(),
                tier,
                lane,
            });
        }
        UnitKind::Stronghold => {
            let outcome = Outcome {
                winner: Some(team),
                reason: EndReason::StrongholdDestroyed,
            };
            m.outcome = Some(outcome);
            m.events.push(Event::Ended(outcome));
        }
    }
}

fn commander_death(m: &mut MobaMatch, d: &Death, team: Team, killer: Option<Seat>) {
    let Some(victim) = d.owner else { return };
    let level = m.player(victim).level;
    let tick = m.tick;
    let p = m.player_mut(victim);
    p.commander = 0;
    p.deaths += 1;
    p.respawn_at = tick + content::respawn_ticks(level);
    m.score[team.index()] += 1;
    let killer = killer.filter(|s| s.team() == team);
    if let Some(k) = killer {
        m.player_mut(k).kills += 1;
        grant(
            m,
            Some(k),
            content::COMMANDER_BOUNTY,
            content::COMMANDER_KILL_ESSENCE,
        );
    }
    for seat in commanders_near(m, team, d.pos) {
        if Some(seat) != killer {
            grant(m, Some(seat), content::ASSIST_BOUNTY, 0);
        }
    }
    m.events.push(Event::CommanderKilled {
        victim,
        killer,
        by: team,
    });
}

fn grant(m: &mut MobaMatch, seat: Option<Seat>, gold: u32, essence: u32) {
    if let Some(seat) = seat {
        let p = m.player_mut(seat);
        p.gold += gold;
        p.gold_earned += gold;
        p.essence += essence;
    }
}

fn commanders_near(m: &MobaMatch, team: Team, at: Pos) -> Vec<Seat> {
    m.units
        .iter()
        .filter(|u| u.team() == Some(team) && matches!(u.kind, UnitKind::Commander(_)))
        .filter(|u| u.pos.within(at, SHARE_RANGE))
        .filter_map(|u| u.owner)
        .collect()
}

/// Every Commander of `team` near the death gains the full experience.
fn share_xp(m: &mut MobaMatch, team: Team, at: Pos, xp: u32) {
    if xp == 0 {
        return;
    }
    for seat in commanders_near(m, team, at) {
        m.player_mut(seat).xp += xp;
        while level_up(m, seat) {}
    }
}

/// Spends experience on one level; false when none is due.
fn level_up(m: &mut MobaMatch, seat: Seat) -> bool {
    let p = m.player_mut(seat);
    if p.level >= content::MAX_LEVEL || p.xp < content::xp_to_next(p.level) {
        return false;
    }
    p.xp -= content::xp_to_next(p.level);
    p.level += 1;
    let (level, id) = (p.level, p.commander);
    if let Some(slot) = m.slot(id) {
        let u = &mut m.units[slot];
        u.max_hp += content::COMMANDER_LEVEL_HP;
        u.hp += content::COMMANDER_LEVEL_HP;
        u.stats.damage += content::COMMANDER_LEVEL_DAMAGE;
    }
    m.events.push(Event::LevelUp { seat, level });
    true
}

fn close_camp(m: &mut MobaMatch, camp: u8, credit: Option<Credit>) {
    let up = m.units.iter().any(|u| u.brain == Brain::Neutral { camp });
    let state = &mut m.camps[camp as usize];
    if up || state.respawn_at.is_some() {
        return;
    }
    state.respawn_at = Some(m.tick + content::camp_content(crate::map::camp_kind(camp as usize)).1);
    m.events.push(Event::CampCleared {
        camp,
        by: credit.and_then(|c| c.seat),
    });
}
