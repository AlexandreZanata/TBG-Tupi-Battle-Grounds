//! Automatic Troop deaths, last-hit bounties, kill counts and camp timers.

use crate::content;
use crate::state::{Brain, Credit, EndReason, Event, MobaMatch, Outcome};
use crate::types::{Seat, Side, Team, UnitKind};

struct Death {
    kind: UnitKind,
    at: crate::geom::Pos,
    side: Side,
    owner: Option<Seat>,
    credit: Option<Credit>,
    camp: Option<u8>,
    summon: bool,
}

pub(super) fn resolve_deaths(m: &mut MobaMatch) {
    // Freeze the initial death set: death bursts never recurse in one tick.
    let dead: Vec<_> = m.units.iter().filter(|u| !u.alive()).cloned().collect();
    for unit in &dead {
        crate::champions::died(m, unit);
    }
    for unit in m.units.iter().filter(|u| !u.alive()) {
        m.events.push(Event::UnitDied {
            unit: unit.id,
            kind: unit.kind,
            side: unit.side,
            at: unit.pos,
        });
    }

    let deaths: Vec<Death> = m
        .units
        .iter()
        .filter(|u| !u.alive())
        .map(|u| Death {
            kind: u.kind,
            at: u.pos,
            summon: u.fx.summon != 0,
            side: u.side,
            owner: if u.fx.summon == 0 { u.owner } else { None },
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
    if d.summon {
        return;
    }
    let killer = d
        .credit
        .filter(|c| c.side.hostile(d.side))
        .and_then(|c| c.seat)
        .filter(|seat| {
            seat.valid() && d.credit.is_some_and(|c| c.side.team() == Some(seat.team()))
        });
    let Some(team) = killer_team(d) else { return };
    match d.kind {
        UnitKind::Minion(_) => grant(m, d, killer, content::MINION_BOUNTY, 0, false),
        UnitKind::Troop(kind) => troop_death(m, d, killer, kind),
        UnitKind::Neutral(kind) => {
            let (gold, essence) = content::neutral_rewards(kind);
            grant(m, d, killer, gold, essence, false);
        }
        UnitKind::Commander(_) => {} // Reserved wire kind, never spawned (M-65).
        UnitKind::Tower(tier, lane) => {
            for seat in Seat::all().filter(|s| s.team() == team) {
                grant(
                    m,
                    d,
                    Some(seat),
                    content::TOWER_TEAM_BOUNTY,
                    content::TOWER_TEAM_ESSENCE,
                    true,
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

fn troop_death(m: &mut MobaMatch, d: &Death, killer: Option<Seat>, kind: crate::Champion) {
    grant(m, d, killer, content::troop_cost(kind) / 3, 0, false);
    let Some(victim) = d.owner else { return };
    m.player_mut(victim).deaths += 1;
    let Some(by) = d.credit.and_then(|c| c.side.team()) else {
        return;
    };
    if by == victim.team() {
        return;
    }
    m.score[by.index()] += 1;
    if let Some(killer) = killer {
        m.player_mut(killer).kills += 1;
    }
    m.events.push(Event::TroopKilled { victim, killer, by });
}

fn grant(
    m: &mut MobaMatch,
    death: &Death,
    seat: Option<Seat>,
    gold: u32,
    essence: u32,
    team_reward: bool,
) {
    if let Some(seat) = seat {
        let p = m.player_mut(seat);
        p.gold += gold;
        p.gold_earned += gold;
        p.essence += essence;
        m.events.push(Event::GoldGranted {
            seat,
            gold,
            essence,
            team_reward,
            origin: Some((death.kind, death.at)),
        });
    }
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
