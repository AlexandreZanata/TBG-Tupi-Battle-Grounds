//! Bounded Hatch, cargo production and non-recursive drill fusion using 1v1 catalog profiles.
use super::{abilities, cue};
use crate::{
    state::{Brain, MobaMatch, Unit},
    types::{Champion, UnitKind},
};
use tbg_core::content::{self, Ability};
pub fn tick(g: &mut MobaMatch) {
    fuse(g);
    let n = g.units.len();
    for i in 0..n {
        let u = g.units[i].clone();
        if !u.alive() {
            continue;
        }
        if u.fx.expires > 0 && u.fx.expires <= g.tick {
            g.units[i].hp = 0;
            continue;
        }
        g.units[i].fx.summon_cd = u.fx.summon_cd.saturating_sub(1);
        if matches!(u.kind, UnitKind::Troop(Champion::Crate)) {
            if g.units[i].fx.summon_cd == 0 {
                open(g, &u);
                g.units[i].hp = 0;
            }
            continue;
        }
        for (slot, a) in abilities(&u).into_iter().enumerate() {
            match a {
                Ability::Hatch if slot == 0 && g.units[i].fx.summon_cd == 0 => {
                    let count = g
                        .units
                        .iter()
                        .filter(|v| {
                            v.alive()
                                && v.owner == u.owner
                                && v.kind == UnitKind::Troop(Champion::Beetling)
                        })
                        .count();
                    let own = g
                        .units
                        .iter()
                        .filter(|v| {
                            v.alive()
                                && v.fx.summon == u.id
                                && v.kind == UnitKind::Troop(Champion::Beetling)
                        })
                        .count();
                    if count < content::BEETLES_PER_PLAYER && own < content::BEETLES_PER_NESTER[0] {
                        spawn(g, &u, Champion::Beetling, content::BEETLE_LIFETIME);
                        g.units[i].fx.summon_cd = content::HATCH_EVERY;
                        cue(g, &u, 0, u.pos, 0, false);
                    }
                }
                Ability::Cargo { every, open, .. } if g.units[i].fx.summon_cd == 0 => {
                    let id = spawn(g, &u, Champion::Crate, 0);
                    if let Some(j) = g.slot(id) {
                        g.units[j].brain = Brain::Static;
                        g.units[j].fx.summon_cd = open;
                    }
                    g.units[i].fx.summon_cd = every;
                    cue(g, &u, 0, u.pos, 0, false);
                }
                _ => {}
            }
        }
    }
}
fn rank(k: UnitKind) -> u8 {
    match k {
        UnitKind::Troop(Champion::Microdrill) => 1,
        UnitKind::Troop(Champion::Duodrill) => 2,
        UnitKind::Troop(Champion::Megadrill) => 3,
        _ => 0,
    }
}
fn fuse(g: &mut MobaMatch) {
    let ids: Vec<_> = g
        .units
        .iter()
        .filter(|u| u.alive() && (1..3).contains(&rank(u.kind)))
        .map(|u| u.id)
        .collect();
    let mut used = Vec::new();
    for (n, id) in ids.iter().enumerate() {
        if used.contains(id) {
            continue;
        }
        let Some(a) = g.unit(*id).cloned() else {
            continue;
        };
        let b = ids[n + 1..]
            .iter()
            .filter(|id| !used.contains(id))
            .filter_map(|id| g.unit(*id))
            .find(|b| {
                b.owner == a.owner
                    && b.kind == a.kind
                    && a.pos.within(b.pos, a.stats.radius + b.stats.radius + 200)
            })
            .cloned();
        let Some(b) = b else { continue };
        let kind = if rank(a.kind) == 1 {
            Champion::Duodrill
        } else {
            Champion::Megadrill
        };
        let life = if rank(a.kind) == 1 { 280 } else { 320 };
        let out = spawn(g, &a, kind, life);
        if let Some(j) = g.slot(out) {
            let v = &mut g.units[j];
            v.pos.x = (a.pos.x + b.pos.x) / 2;
            v.pos.y = (a.pos.y + b.pos.y) / 2;
            v.hp = ((i64::from(a.hp + b.hp) * i64::from(v.max_hp))
                / i64::from(a.max_hp + b.max_hp).max(1))
            .max(1) as i32;
            v.cooldown = a.cooldown.max(b.cooldown);
            v.fx.slow = a.fx.slow.max(b.fx.slow);
            v.fx.slow_until = a.fx.slow_until.max(b.fx.slow_until);
            v.fx.root_until = a.fx.root_until.max(b.fx.root_until);
        }
        used.extend([a.id, b.id]);
        if let Some(u) = g.unit(out).cloned() {
            cue(g, &u, 0, u.pos, 0, false);
        }
    }
    // Fusion is replacement, never a combat death or a bounty.
    g.units.retain(|u| !used.contains(&u.id));
}
fn spawn(g: &mut MobaMatch, u: &Unit, kind: Champion, life: u32) -> u32 {
    let id = g.spawn(UnitKind::Troop(kind), u.side, u.owner, u.pos, u.brain);
    if let Some(i) = g.slot(id) {
        g.units[i].fx.summon = if u.fx.summon == 0 { u.id } else { u.fx.summon };
        g.units[i].fx.expires = if life > 0 { g.tick + life } else { 0 };
    }
    id
}
fn open(g: &mut MobaMatch, u: &Unit) {
    let table = g
        .unit(u.fx.summon)
        .and_then(|v| {
            abilities(v).into_iter().find_map(|a| {
                if let Ability::Cargo { table, .. } = a {
                    Some(table)
                } else {
                    None
                }
            })
        })
        .unwrap_or([20, 42, 28, 8, 2]);
    let mut draw = g.rng.below(table.iter().map(|v| u32::from(*v)).sum());
    let mut band = 0;
    for (i, weight) in table.into_iter().enumerate() {
        if draw < u32::from(weight) {
            band = i;
            break;
        }
        draw -= u32::from(weight);
    }
    if band == 0 {
        return;
    }
    let pool: Vec<_> = crate::types::POOL
        .iter()
        .copied()
        .filter(|k| {
            !matches!(
                k,
                Champion::Nester | Champion::Smuggler | Champion::Volthar | Champion::Astreon
            )
        })
        .filter(|k| {
            let cost = content::def(*k).cost / content::ENERGY_SCALE;
            match band {
                1 => cost == 3,
                2 => (4..=6).contains(&cost),
                _ => true,
            }
        })
        .collect();
    if pool.is_empty() {
        return;
    }
    let k = pool[g.rng.below(pool.len() as u32) as usize];
    spawn(g, u, k, 0);
    cue(g, u, 0, u.pos, 0, false);
}
pub fn died(g: &mut MobaMatch, u: &Unit) {
    if u.fx.passenger != 0 {
        if let Some(i) = g.slot(u.fx.passenger) {
            g.units[i].fx.carrier = 0;
            g.units[i].pos = u.pos;
        }
    }
}

pub fn drill_ready(g: &MobaMatch, u: &Unit) -> bool {
    g.units
        .iter()
        .filter(|v| v.alive() && v.owner == u.owner && rank(v.kind) > 0)
        .count()
        < 8
}
pub fn drill(g: &mut MobaMatch, u: &Unit) {
    if drill_ready(g, u) {
        spawn(g, u, Champion::Microdrill, 240);
    }
}
