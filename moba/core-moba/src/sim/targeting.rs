//! Candidate eligibility and stable acquisition; never changes a unit's mission or commitment.
use crate::{
    champions, content, map,
    state::{Brain, MobaMatch, Unit},
    types::{Lane, Side, Team, TowerTier, UnitKind},
};

#[derive(Clone, Copy)]
pub(super) struct Choice {
    pub slot: usize,
    pub support: bool,
}

pub(super) fn choose(m: &MobaMatch, vul: &Vulnerability, i: usize) -> Option<Choice> {
    let u = &m.units[i];
    // A ready support may respond to a patient; cooldown never flicks its aim
    // between a healthy ally and an enemy on successive ticks.
    if let Some(slot) = retained_patient(m, i) {
        return Some(Choice {
            slot,
            support: true,
        });
    }
    if u.cooldown > 0 {
        if let Some(slot) = retained_enemy(m, vul, i) {
            return Some(Choice {
                slot,
                support: false,
            });
        }
    }
    if let Some(slot) = champions::control::target(m, u)
        .flatten()
        .and_then(|id| m.slot(id))
    {
        return Some(Choice {
            slot,
            support: true,
        });
    }
    if u.stats.heal > 0 {
        if let Some(slot) = most_injured_ally(m, i) {
            return Some(Choice {
                slot,
                support: true,
            });
        }
    }
    (u.stats.damage > 0 && champions::control::target(m, u).is_none())
        .then(|| select_target(m, vul, i))
        .flatten()
        .map(|slot| Choice {
            slot,
            support: false,
        })
}

pub(super) fn release_reason(
    m: &MobaMatch,
    vul: &Vulnerability,
    i: usize,
) -> super::decision::Reason {
    use super::decision::Reason;
    let a = &m.units[i];
    let Some(b) = m.unit(a.target).filter(|b| b.alive()) else {
        return Reason::TargetGone;
    };
    if super::navigation::rejected(a, b.id, m.tick) {
        return Reason::Unreachable;
    }
    if a.side == b.side || !attackable(vul, a, b) {
        return Reason::Invalid;
    }
    Reason::PursuitLimit
}

/// Which structures may be damaged this tick (M-08).
pub(super) struct Vulnerability {
    /// Alive towers per team, lane and tier (Outer, Inner, Base).
    lane: [[[bool; 3]; 3]; 2],
    nexus_alive: [u8; 2],
}

impl Vulnerability {
    pub(super) fn of(m: &MobaMatch) -> Self {
        let mut v = Self {
            lane: [[[false; 3]; 3]; 2],
            nexus_alive: [0; 2],
        };
        for u in &m.units {
            let (UnitKind::Tower(tier, lane), Some(team)) = (u.kind, u.team()) else {
                continue;
            };
            match (tier, lane) {
                (TowerTier::Nexus, _) => v.nexus_alive[team.index()] += 1,
                (_, Some(lane)) => v.lane[team.index()][lane as usize][tier as usize] = true,
                _ => {}
            }
        }
        v
    }

    fn open(&self, kind: UnitKind, team: Team) -> bool {
        let t = team.index();
        match kind {
            UnitKind::Tower(TowerTier::Outer, _) => true,
            UnitKind::Tower(TowerTier::Inner, Some(lane)) => !self.lane[t][lane as usize][0],
            UnitKind::Tower(TowerTier::Base, Some(lane)) => !self.lane[t][lane as usize][1],
            UnitKind::Tower(TowerTier::Nexus, _) => {
                Lane::ALL.iter().any(|&l| !self.lane[t][l as usize][2])
            }
            UnitKind::Stronghold => self.nexus_alive[t] == 0,
            _ => true,
        }
    }
}

fn most_injured_ally(m: &MobaMatch, i: usize) -> Option<usize> {
    let a = &m.units[i];
    let reach = a.stats.range;
    let eligible = |u: &Unit| {
        u.side == a.side
            && (u.id != a.id || a.kind != UnitKind::Troop(crate::Champion::GustDrummer))
            && u.alive()
            && u.mobile()
            && u.hp < u.max_hp
            && u.pos.within(a.pos, reach)
    };
    if let Some(current) = m.slot(a.target).filter(|&j| eligible(&m.units[j])) {
        return Some(current);
    }
    let mut best: Option<(i64, usize)> = None;
    m.grid.near(a.pos, reach, |k| {
        let u = &m.units[k];
        if !eligible(u) {
            return;
        }
        let ratio = u.hp as i64 * 1000 / u.max_hp.max(1) as i64;
        if best.is_none_or(|(r, s)| (ratio, k) < (r, s)) {
            best = Some((ratio, k));
        }
    });
    best.map(|(_, k)| k)
}

/// Whether `a` may attack `b` at all right now.
fn attackable(vul: &Vulnerability, a: &Unit, b: &Unit) -> bool {
    if !b.alive() || b.fx.carrier != 0 || !a.side.hostile(b.side) {
        return false;
    }
    // Lane walkers and structures ignore the jungle; jungle Troops hunt it.
    if b.side == Side::Neutral && matches!(a.brain, Brain::Lane { .. } | Brain::Static) {
        return false;
    }
    match b.team() {
        Some(team) if b.kind.structure() => {
            champions::can_attack_structure(a) && vul.open(b.kind, team)
        }
        _ => true,
    }
}

fn retained_enemy(m: &MobaMatch, vul: &Vulnerability, i: usize) -> Option<usize> {
    let a = &m.units[i];
    let (anchor, leash) = match a.brain {
        Brain::Neutral { camp } => {
            let home = map::camp(camp as usize).0;
            if !a.pos.within(home, content::CAMP_LEASH) {
                return None;
            }
            (Some(home), content::CAMP_LEASH)
        }
        _ => (None, 0),
    };
    let keep = if a.mobile() {
        a.stats.sight.max(a.stats.range) + a.stats.sight / 2
    } else {
        a.stats.range
    };
    m.slot(a.target).filter(|&j| {
        let b = &m.units[j];
        let limit = if a.mobile() {
            keep + b.stats.radius
        } else {
            champions::reach(a, b)
        };
        !super::navigation::rejected(a, b.id, m.tick)
            && attackable(vul, a, b)
            && a.pos.within(b.pos, limit)
            && anchor.is_none_or(|p| b.pos.within(p, leash))
    })
}

/// Keeps a valid current target or acquires the nearest one in sight.
fn select_target(m: &MobaMatch, vul: &Vulnerability, i: usize) -> Option<usize> {
    let a = &m.units[i];
    let current = retained_enemy(m, vul, i);
    // Preserve commitment through movement and cooldown. Neutral monsters
    // may only acquire through the retaliation hook.
    if current.is_some() || matches!(a.brain, Brain::Neutral { .. }) {
        return current;
    }
    // Weapon acquisition must reach the end of the weapon range. The
    // separate Team vision radius continues to filter the wire view.
    let sight = if a.mobile() {
        a.stats.sight.max(a.stats.range)
    } else {
        a.stats.range
    };
    let mut best: Option<(bool, bool, i64, usize)> = None;
    m.grid.near(a.pos, sight + 7 * crate::geom::MM, |k| {
        let b = &m.units[k];
        if super::navigation::rejected(a, b.id, m.tick)
            || !attackable(vul, a, b)
            || !a.pos.within(b.pos, sight + b.stats.radius)
        {
            return;
        }
        let d = a.pos.dist2(b.pos);
        let not_guard = !champions::abilities(b).contains(&tbg_core::content::Ability::Guard);
        let out_of_range = !super::decision::in_position(a, b);
        if out_of_range && !a.mobile() {
            return;
        }
        let priority = (not_guard, out_of_range, d, k);
        if best.is_none_or(|old| priority < old) {
            best = Some(priority);
        }
    });
    best.map(|(_, _, _, k)| k).or_else(|| assist(m, vul, i))
}

/// Assist (M-77), like lane minions: a walker that sees no enemy of its own
/// joins the fight of an ally planted in an action nearby, on that ally's
/// target, if it could keep that target under the M-07 retention distance.
/// Nearest such target, then lowest id.
fn assist(m: &MobaMatch, vul: &Vulnerability, i: usize) -> Option<usize> {
    let a = &m.units[i];
    if !a.mobile() || matches!(a.brain, Brain::Neutral { .. }) {
        return None;
    }
    let keep = a.stats.sight.max(a.stats.range) + a.stats.sight / 2;
    let mut best: Option<(i64, u32, usize)> = None;
    m.grid.near(a.pos, a.stats.sight, |k| {
        let c = &m.units[k];
        if c.id == a.id
            || c.side != a.side
            || !c.alive()
            || !super::bodies::planted(c)
            || !c.pos.within(a.pos, a.stats.sight + c.stats.radius)
        {
            return;
        }
        let Some(j) = m.slot(c.target) else { return };
        let b = &m.units[j];
        if super::navigation::rejected(a, b.id, m.tick)
            || !attackable(vul, a, b)
            || !a.pos.within(b.pos, keep + b.stats.radius)
        {
            return;
        }
        let key = (a.pos.dist2(b.pos), b.id, j);
        if best.is_none_or(|old| key < old) {
            best = Some(key);
        }
    });
    best.map(|(_, _, j)| j)
}

/// Like the BattleLine, a support without a useful action holds at the
/// enemy front instead of walking through it to search for a patient.
pub(super) fn support_holds(m: &MobaMatch, i: usize) -> bool {
    let a = &m.units[i];
    a.stats.damage == 0
        && m.units.iter().any(|b| {
            b.alive()
                && a.side.hostile(b.side)
                && b.fx.carrier == 0
                && !(b.side == Side::Neutral
                    && matches!(a.brain, Brain::Lane { .. } | Brain::Static))
                && super::decision::in_position(a, b)
        })
}

fn retained_patient(m: &MobaMatch, i: usize) -> Option<usize> {
    let a = &m.units[i];
    let j = m.slot(a.target)?;
    let b = &m.units[j];
    let is_brew = champions::abilities(a)
        .iter()
        .any(|x| matches!(x, tbg_core::content::Ability::Brew { .. }));
    let useful = if is_brew {
        champions::control::afflicted(b, m.tick)
    } else {
        a.stats.heal > 0 && b.hp < b.max_hp
    };
    (b.alive()
        && b.mobile()
        && b.side == a.side
        && (useful || a.cooldown > 0)
        && (a.stats.heal > 0 || is_brew)
        && !super::navigation::rejected(a, b.id, m.tick)
        && b.pos
            .within(a.pos, a.stats.range + a.stats.sight / 2 + b.stats.radius))
    .then_some(j)
}
