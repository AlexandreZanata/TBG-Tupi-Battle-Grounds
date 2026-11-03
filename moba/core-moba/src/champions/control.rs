//! Brew and Puppetmaster casts replace ordinary basics and resolve only after the windup.
use super::{abilities, cue, effects, nearby};
use crate::state::{MobaMatch, Unit};
use tbg_core::content::Ability;

pub fn target(g: &MobaMatch, u: &Unit) -> Option<Option<u32>> {
    for a in abilities(u) {
        match a {
            Ability::Brew { .. } => {
                let targets = nearby(g, u, u.pos, u.stats.range, u8::MAX, None, true)
                    .into_iter()
                    .filter(|id| g.unit(*id).is_some_and(|v| afflicted(v, g.tick)));
                return Some(preferred(targets, u.target));
            }
            Ability::Puppet { .. } => return Some(preferred(puppets(g, u).into_iter(), u.target)),
            _ => {}
        }
    }
    None
}
fn preferred(mut targets: impl Iterator<Item = u32>, current: u32) -> Option<u32> {
    let first = targets.next()?;
    if first == current || targets.any(|id| id == current) {
        Some(current)
    } else {
        Some(first)
    }
}

fn afflicted(u: &Unit, now: u32) -> bool {
    let f = &u.fx;
    f.root_until > now
        || f.slow_until > now
        || f.weaken_until > now
        || f.cut_until > now
        || f.confused_until > now
        || f.spark_until > now
        || !f.fish.is_empty()
        || f.pulses.iter().any(|p| {
            matches!(
                p.kind,
                super::state::PulseKind::Poison
                    | super::state::PulseKind::Bleed
                    | super::state::PulseKind::Venom
            )
        })
}
fn puppets(g: &MobaMatch, u: &Unit) -> Vec<u32> {
    let mut ids = nearby(g, u, u.pos, u.stats.range, u8::MAX, None, false);
    ids.retain(|id| {
        g.unit(*id).is_some_and(|v| {
            v.fx.summon == 0
                && v.owner.is_some()
                && v.stats.damage > 0
                && v.stats.heal == 0
                && v.fx.confused_until <= g.tick
                && v.fx.immune_until <= g.tick
                && v.fx.ward_until <= g.tick
        })
    });
    ids.sort_by_key(|id| {
        g.unit(*id)
            .map_or((0, 0, 0), |v| (-v.stats.damage, v.pos.dist2(u.pos), v.id))
    });
    ids
}
pub fn resolve(g: &mut MobaMatch, u: &Unit, v: &Unit) -> bool {
    for a in abilities(u) {
        match a {
            Ability::Brew { .. } => {
                super::support::after_heal(g, u.id, v.id, 0);
                cue(g, u, v.id, v.pos, 0, false);
                return true;
            }
            Ability::Puppet {
                window,
                immune,
                bonus_bp,
                targets,
                ..
            } => {
                let mut ids = puppets(g, u);
                if let Some(index) = ids.iter().position(|id| *id == v.id) {
                    ids.swap(0, index);
                }
                for id in ids.into_iter().take(usize::from(targets.max(1))) {
                    if let Some(j) = g.slot(id) {
                        let f = &mut g.units[j].fx;
                        f.confused_until = g.tick + u32::from(window);
                        f.betray_bp = bonus_bp;
                        f.betray_immune = immune;
                        f.betray_credit = Some(super::credit(u));
                        let at = g.units[j].pos;
                        cue(g, u, id, at, 0, false);
                    }
                }
                return true;
            }
            _ => {}
        }
    }
    false
}
pub fn betrayal(g: &MobaMatch, u: &Unit) -> Option<u32> {
    (u.fx.confused_until > g.tick)
        .then(|| {
            nearby(g, u, u.pos, u.stats.range, u8::MAX, Some(u.id), true)
                .into_iter()
                .find(|id| g.unit(*id).is_some_and(|v| v.fx.carrier == 0))
        })
        .flatten()
}
pub fn betray_hit(g: &mut MobaMatch, u: &Unit, v: &Unit, amount: i32) {
    let mut source = u.clone();
    if let Some(c) = u.fx.betray_credit {
        source.owner = c.seat;
        source.side = c.side;
    }
    effects::hurt(
        g,
        &source,
        v.id,
        super::scaled(amount, 10_000 + i32::from(u.fx.betray_bp)),
    );
    if let Some(i) = g.slot(u.id) {
        let f = &mut g.units[i].fx;
        f.confused_until = 0;
        f.immune_until = g.tick + u32::from(u.fx.betray_immune);
        f.betrayed = false;
    }
}
