//! Target selection, attacks, heals and their simultaneous resolution (M-06..M-09).

use crate::champions::{self, state::Cast};
use crate::content;
use crate::map;
use crate::state::{Brain, MobaMatch, Unit};
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

/// Decide each basic before applying the completed batch. The shared windup
/// drives the same weapon release and contact poses as the BattleLine.
pub(super) fn fight(m: &mut MobaMatch) -> Vec<bool> {
    let vul = Vulnerability::of(m);
    let mut hits = Vec::new();
    let mut engaged = Vec::with_capacity(m.units.len());
    for i in 0..m.units.len() {
        engaged.push(decide(m, &vul, i, &mut hits));
    }
    for (i, j) in hits {
        land(m, i, j);
    }
    engaged
}
fn decide(
    m: &mut MobaMatch,
    vul: &Vulnerability,
    i: usize,
    hits: &mut Vec<(usize, usize)>,
) -> bool {
    let u = &mut m.units[i];
    u.cooldown = u.cooldown.saturating_sub(1);
    if !u.alive() {
        return true;
    }
    if champions::cannot_basic(u, m.tick) {
        u.fx.basic = None;
        u.fx.betrayed = false;
        return true;
    }
    if let Some(mut cast) = u.fx.basic {
        cast.left = cast.left.saturating_sub(1);
        u.fx.basic = (cast.left > 0).then_some(cast);
        if cast.left == 0 {
            if let Some(j) = m.slot(cast.target).filter(|&j| m.units[j].alive()) {
                hits.push((i, j));
            }
        }
        return true;
    }
    let heal = (u.stats.heal > 0)
        .then(|| most_injured_ally(m, i))
        .flatten();
    let cast = champions::control::target(m, &m.units[i]);
    let special = cast.flatten().and_then(|id| m.slot(id));
    let target = special.or(heal).or_else(|| select_target(m, vul, i));
    let id = target.map_or(0, |j| m.units[j].id);
    let u = &mut m.units[i];
    if u.fx.target != id && champions::same_target_counter(u) {
        u.fx.attacks = 0;
        u.fx.target = id;
    }
    u.target = id;
    let Some(j) = target else { return false };
    let (a, b) = (&m.units[i], &m.units[j]);
    if heal.is_none() && special.is_none() && !a.pos.within(b.pos, champions::reach(a, b)) {
        return false;
    }
    if a.cooldown != 0
        || ((a.stats.damage == 0 || cast.is_some()) && heal.is_none() && special.is_none())
    {
        return true;
    }
    let betrayal = if heal.is_none() && special.is_none() {
        champions::control::betrayal(m, a)
    } else {
        None
    };
    let id = betrayal.unwrap_or(id);
    let centre = m.unit(id).map_or(b.pos, |u| u.pos);
    let a = &mut m.units[i];
    a.cooldown = champions::interval(a, m.tick);
    a.last_attack = m.tick;
    a.target = id;
    a.fx.betrayed = betrayal.is_some();
    if matches!(a.kind, UnitKind::Troop(_)) {
        let windup = tbg_core::content::WINDUP;
        a.fx.basic = Some(Cast {
            slot: 0,
            target: id,
            centre,
            left: windup,
            windup,
        });
    } else {
        hits.push((i, j));
    }
    true
}
fn land(m: &mut MobaMatch, i: usize, j: usize) {
    let a = m.units[i].clone();
    let b = m.units[j].clone();
    if !b.alive() {
        return;
    }
    if champions::control::resolve(m, &a, &b) {
        return;
    }
    if a.fx.betrayed {
        let base = a.stats.damage * (100 + a.power) / 100;
        let dmg = champions::before_basic(m, i, j, base);
        champions::control::betray_hit(m, &a, &b, dmg);
        return;
    }
    if a.side == b.side {
        champions::healing(m, &a, b.id);
        return;
    }
    let base = a.stats.damage * (100 + a.power) / 100;
    let dmg = champions::before_basic(m, i, j, base);
    if champions::flame(m, &a, &b, dmg) {
        return;
    }
    let removed = champions::hurt(m, &a, b.id, dmg);
    if !b.kind.structure() && b.fx.summon == 0 {
        champions::feed(m, a.id, &b, removed);
    }
    champions::after_basic(m, a.id, b.id);
    if a.stats.splash <= 0 {
        return;
    }
    let friendly =
        champions::abilities(&a).contains(&tbg_core::content::Ability::Blast { friendly: true });
    let mut ids = Vec::new();
    m.grid.near(b.pos, a.stats.splash, |k| {
        let v = &m.units[k];
        if k != j
            && v.alive()
            && !v.kind.structure()
            && v.pos.within(b.pos, a.stats.splash)
            && (champions::enemy(&a, v) || friendly && v.side == a.side)
        {
            ids.push(v.id);
        }
    });
    ids.sort_unstable();
    for id in ids {
        champions::hurt(m, &a, id, base * content::SPLASH_PERCENT / 100);
    }
}

fn most_injured_ally(m: &MobaMatch, i: usize) -> Option<usize> {
    let a = &m.units[i];
    let mut best: Option<(i64, usize)> = None;
    let reach = a.stats.range;
    m.grid.near(a.pos, reach, |k| {
        let u = &m.units[k];
        if u.side != a.side
            || (u.id == a.id && a.kind == UnitKind::Troop(crate::Champion::GustDrummer))
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

/// Keeps a valid current target or acquires the nearest one in sight.
fn select_target(m: &MobaMatch, vul: &Vulnerability, i: usize) -> Option<usize> {
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
        (a.stats.sight * 3 / 2).max(a.stats.range)
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
    // Weapon acquisition must reach the end of the weapon range. The
    // separate Team vision radius continues to filter the wire view.
    let sight = if a.mobile() {
        a.stats.sight.max(a.stats.range)
    } else {
        a.stats.range
    };
    let mut best: Option<(bool, i64, usize)> = None;
    m.grid.near(a.pos, sight + 7 * crate::geom::MM, |k| {
        let b = &m.units[k];
        if !attackable(vul, a, b) || !a.pos.within(b.pos, sight + b.stats.radius) {
            return;
        }
        if anchor.is_some_and(|p| !b.pos.within(p, leash + a.stats.range)) {
            return;
        }
        let d = a.pos.dist2(b.pos);
        let not_guard = !champions::abilities(b).contains(&tbg_core::content::Ability::Guard);
        if best.is_none_or(|old| (not_guard, d, k) < old) {
            best = Some((not_guard, d, k));
        }
    });
    best.map(|(_, _, k)| k).or(current)
}
