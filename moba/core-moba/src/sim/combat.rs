//! Target selection, attacks, heals and their simultaneous resolution (M-06..M-09).

use crate::content;
use crate::map;
use crate::state::{Brain, CommandMode, Credit, MobaMatch, Unit};
use crate::types::{Lane, Side, Team, TowerTier, UnitKind};

/// Which structures may be damaged this tick (M-08).
struct Vulnerability {
    /// Alive towers per team, lane and tier (Outer, Inner, Base).
    lane: [[[bool; 3]; 3]; 2],
    nexus_alive: [u8; 2],
}

impl Vulnerability {
    fn of(m: &MobaMatch) -> Self {
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

/// Runs every unit's combat decision on the start-of-tick state, then applies
/// all damage at once. Returns which slots stood still to fight.
// Slot loops: each step reads other units while writing this one.
#[allow(clippy::needless_range_loop)]
pub(super) fn fight(m: &mut MobaMatch) -> Vec<bool> {
    let vul = Vulnerability::of(m);
    let n = m.units.len();
    let mut engaged = vec![false; n];
    let mut hits: Vec<(usize, usize, i32)> = Vec::new();
    let mut heals: Vec<(usize, i32)> = Vec::new();
    for i in 0..n {
        let u = &mut m.units[i];
        u.cooldown = u.cooldown.saturating_sub(1);
        if !u.alive() || u.stats.damage == 0 {
            continue;
        }
        // Healing champions (1v1 `heal` > 0) mend the most injured ally first.
        if u.stats.heal > 0 {
            if let Some(j) = most_injured_ally(m, i) {
                engaged[i] = true;
                if m.units[i].cooldown == 0 {
                    heals.push((j, m.units[i].stats.heal));
                    let target = m.units[j].id;
                    let u = &mut m.units[i];
                    u.cooldown = u.stats.attack_ticks;
                    u.last_attack = m.tick;
                    u.target = target;
                }
                continue;
            }
        }
        let target = select_target(m, &vul, i);
        m.units[i].target = target.map_or(0, |j| m.units[j].id);
        let Some(j) = target else { continue };
        let (a, b) = (&m.units[i], &m.units[j]);
        if a.pos.within(b.pos, a.stats.range + b.stats.radius) {
            engaged[i] = true;
            if a.cooldown == 0 {
                let a = &m.units[i];
                hits.push((i, j, a.stats.damage * (100 + a.power) / 100));
                let a = &mut m.units[i];
                a.cooldown = a.stats.attack_ticks;
                a.last_attack = m.tick;
            }
        }
    }
    for (j, amount) in heals {
        let u = &mut m.units[j];
        u.hp = (u.hp + amount).min(u.max_hp);
    }
    for (i, j, dmg) in hits {
        land(m, i, j, dmg);
    }
    engaged
}

fn land(m: &mut MobaMatch, i: usize, j: usize, dmg: i32) {
    let (splash, center) = (m.units[i].stats.splash, m.units[j].pos);
    strike(m, i, j, dmg);
    if splash <= 0 {
        return;
    }
    // Splash champions also hurt other mobile enemies around the target.
    let mut around = Vec::new();
    m.grid.near(center, splash, |k| {
        let (a, b) = (&m.units[i], &m.units[k]);
        if k != j
            && b.alive()
            && b.mobile()
            && a.side.hostile(b.side)
            && b.pos.within(center, splash)
        {
            around.push(k);
        }
    });
    around.sort_unstable();
    for k in around {
        strike(m, i, k, dmg * content::SPLASH_PERCENT / 100);
    }
}

fn strike(m: &mut MobaMatch, i: usize, j: usize, dmg: i32) {
    let credit = Credit {
        side: m.units[i].side,
        seat: m.units[i].owner,
    };
    let attacker_id = m.units[i].id;
    let t = &mut m.units[j];
    if !t.alive() {
        return;
    }
    t.hp -= (dmg * (100 - t.stats.armor) / 100).max(1);
    t.last_hit = Some(credit);
    // Neutral monsters are passive until hit, then answer the attacker.
    if matches!(t.brain, Brain::Neutral { .. }) && t.target == 0 {
        t.target = attacker_id;
    }
}

fn most_injured_ally(m: &MobaMatch, i: usize) -> Option<usize> {
    let a = &m.units[i];
    let mut best: Option<(i64, usize)> = None;
    let reach = a.stats.range;
    m.grid.near(a.pos, reach, |k| {
        let u = &m.units[k];
        if u.side != a.side
            || !u.alive()
            || !u.mobile()
            || u.hp >= u.max_hp
            || !u.pos.within(a.pos, reach)
        {
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
    if !b.alive() || !a.side.hostile(b.side) {
        return false;
    }
    // Lane walkers and structures ignore the jungle; junglers and Commanders hunt it.
    if b.side == Side::Neutral && matches!(a.brain, Brain::Lane { .. } | Brain::Static) {
        return false;
    }
    match b.team() {
        Some(team) if b.kind.structure() => vul.open(b.kind, team),
        _ => true,
    }
}

/// Keeps a valid current target or acquires the nearest one in sight.
fn select_target(m: &MobaMatch, vul: &Vulnerability, i: usize) -> Option<usize> {
    let a = &m.units[i];
    let (anchor, leash) = match a.brain {
        Brain::Commander {
            mode: CommandMode::Move(_),
            ..
        }
        | Brain::Commander {
            retreating: true, ..
        } => return None,
        Brain::Commander {
            mode: CommandMode::Hold(p),
            ..
        } => (Some(p), content::HOLD_LEASH),
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
        a.stats.sight * 3 / 2
    } else {
        a.stats.range
    };
    let current = m.slot(a.target).filter(|&j| {
        let b = &m.units[j];
        attackable(vul, a, b)
            && a.pos.within(b.pos, keep + b.stats.radius)
            && anchor.is_none_or(|p| b.pos.within(p, leash))
    });
    // Neutral monsters never pick fights; others re-scan every 10 ticks.
    let rescan = !matches!(a.brain, Brain::Neutral { .. }) && (m.tick + a.id).is_multiple_of(10);
    if current.is_some() && !rescan || matches!(a.brain, Brain::Neutral { .. }) {
        return current;
    }
    let sight = if a.mobile() {
        a.stats.sight
    } else {
        a.stats.range
    };
    let mut best: Option<(i64, usize)> = None;
    m.grid.near(a.pos, sight + 4 * crate::geom::MM, |k| {
        let b = &m.units[k];
        if !attackable(vul, a, b) || !a.pos.within(b.pos, sight + b.stats.radius) {
            return;
        }
        if anchor.is_some_and(|p| !b.pos.within(p, leash + a.stats.range)) {
            return;
        }
        let d = a.pos.dist2(b.pos);
        if best.is_none_or(|(bd, bk)| (d, k) < (bd, bk)) {
            best = Some((d, k));
        }
    });
    best.map(|(_, k)| k).or(current)
}
