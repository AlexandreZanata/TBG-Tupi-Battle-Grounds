//! Ally bonds, transport, cleansing and non-stacking care effects.
use super::{abilities, ally, effects, nearby, scaled, state::PulseKind};
use crate::{
    state::{MobaMatch, Unit},
    types::UnitKind,
};
use tbg_core::{
    content::{completion::Ability as C, Ability},
    Archetype,
};
pub fn auras(g: &mut MobaMatch) {
    for u in &mut g.units {
        u.fx.move_bp = 0;
        u.fx.aura_dr = 0;
        u.fx.ignore_slow = false;
    }
    for i in 0..g.units.len() {
        let u = g.units[i].clone();
        if !u.alive() {
            continue;
        }
        for a in abilities(&u) {
            match a {
                Ability::March {
                    reach,
                    move_bp,
                    dr_bp,
                    hysteresis,
                    ignore_slow,
                } => {
                    let nearest = nearby(g, &u, u.pos, reach, u8::MAX, Some(u.id), true)
                        .into_iter()
                        .find(|id| {
                            g.unit(*id)
                                .is_some_and(|v| v.kind != u.kind && v.fx.summon == 0)
                        });
                    let held = g
                        .unit(u.fx.bond)
                        .filter(|v| ally(&u, v) && u.pos.within(v.pos, reach));
                    let bond = if let Some(held) = held {
                        let hd = u.pos.dist2(held.pos).isqrt();
                        nearest
                            .filter(|id| {
                                g.unit(*id).is_some_and(|v| {
                                    u.pos.dist2(v.pos).isqrt() + i64::from(hysteresis) <= hd
                                })
                            })
                            .or(Some(held.id))
                    } else {
                        nearest
                    };
                    g.units[i].fx.bond = bond.unwrap_or(0);
                    if let Some(j) = bond.and_then(|id| g.slot(id)) {
                        let f = &mut g.units[j].fx;
                        f.move_bp = f.move_bp.max(move_bp);
                        f.aura_dr = f.aura_dr.max(dr_bp);
                        f.ignore_slow |= ignore_slow;
                    }
                }
                Ability::Carry {
                    board,
                    lock,
                    dr_bp,
                    move_bp,
                    ..
                } => {
                    if u.fx.passenger != 0 {
                        if let Some(j) = g.slot(u.fx.passenger).filter(|&j| g.units[j].alive()) {
                            g.units[j].pos = u.pos;
                            g.units[j].fx.aura_dr = g.units[j].fx.aura_dr.max(dr_bp);
                            g.units[i].fx.move_bp = g.units[i].fx.move_bp.max(move_bp);
                            continue;
                        }
                        g.units[i].fx.passenger = 0;
                    }
                    let candidate=nearby(g,&u,u.pos,board,u8::MAX,Some(u.id),true).into_iter().find(|id|g.unit(*id).is_some_and(|v|matches!(v.kind,UnitKind::Troop(k) if tbg_core::content::def(k).archetype!=Archetype::Tank)&&v.fx.carrier==0&&v.fx.summon==0&&v.fx.board_until<=g.tick));
                    if let Some(id) = candidate {
                        if let Some(j) = g.slot(id) {
                            g.units[j].fx.carrier = u.id;
                            g.units[j].fx.board_until = g.tick + u32::from(lock);
                            g.units[j].pos = u.pos;
                            g.units[i].fx.passenger = id;
                        }
                    }
                }
                _ => {}
            }
        }
    }
}
pub fn healing(g: &mut MobaMatch, u: &Unit, target: u32) {
    let Some(patient) = g.unit(target).cloned() else {
        return;
    };
    if super::barrels::launch(g, u, &patient) {
        return;
    }
    let mut amount = u.stats.heal;
    for a in abilities(u) {
        if let Ability::Completion(C::Courage {
            heal_per_enemy,
            enemies,
            radius,
        }) = a
        {
            amount += i32::from(heal_per_enemy)
                * nearby(g, u, patient.pos, radius, enemies, None, false).len() as i32;
        }
    }
    let channel = abilities(u).into_iter().find_map(|a| {
        if let Ability::Completion(C::Channel {
            pulses,
            gap,
            first_bp,
        }) = a
        {
            Some((pulses, gap, first_bp))
        } else {
            None
        }
    });
    if let Some((pulses, gap, first_bp)) = channel {
        effects::heal(g, u, target, scaled(amount, i32::from(first_bp)));
        if let Some(i) = g.slot(u.id) {
            if pulses > 1 {
                g.units[i].fx.channel = Some(super::state::Channel {
                    target,
                    amount,
                    gap,
                    left: pulses - 1,
                    next: g.tick + u32::from(gap),
                });
            }
        }
    } else {
        effects::heal(g, u, target, amount);
    }
    after_heal(g, u.id, target, amount);
}
pub fn after_heal(g: &mut MobaMatch, source: u32, target: u32, _amount: i32) {
    let Some(u) = g.unit(source).cloned() else {
        return;
    };
    let Some(i) = g.slot(target) else { return };
    for a in abilities(&u) {
        match a {
            Ability::Brew {
                strip_poison, ward, ..
            } => {
                let mut left = strip_poison;
                g.units[i].fx.pulses.retain(|p| {
                    if p.kind == PulseKind::Poison && left > 0 {
                        left -= 1;
                        false
                    } else {
                        true
                    }
                });
                let f = &mut g.units[i].fx;
                f.fish.clear();
                f.slow = 0;
                f.slow_until = 0;
                f.root_until = 0;
                f.confused_until = 0;
                f.ward_until = g.tick + u32::from(ward);
                f.weaken_until = 0;
                f.cut_until = 0;
                f.spark_until = 0;
                for caster in &mut g.units {
                    if caster.fx.mark == target {
                        caster.fx.mark = 0;
                    }
                }
            }
            Ability::Completion(C::Echo { heal, ticks }) => {
                let f = &mut g.units[i].fx;
                f.echo = i32::from(heal);
                f.echo_until = g.tick + u32::from(ticks);
                f.echo_credit = Some(super::credit(&u));
            }
            Ability::Completion(C::Regrowth { heal, pulses, gap }) => effects::pulse(
                g,
                &u,
                target,
                PulseKind::Regrowth,
                i32::from(heal),
                pulses,
                gap,
                1,
            ),
            _ => {}
        }
    }
}

pub fn channels(g: &mut MobaMatch) {
    for i in 0..g.units.len() {
        let Some(mut cast) = g.units[i].fx.channel else {
            continue;
        };
        let u = &g.units[i];
        if !u.alive()
            || u.fx.root_until > g.tick
            || g.unit(cast.target)
                .is_none_or(|v| !ally(u, v) || !v.pos.within(u.pos, u.stats.range))
        {
            g.units[i].fx.channel = None;
            continue;
        }
        if cast.next > g.tick {
            continue;
        }
        let u = u.clone();
        effects::heal(g, &u, cast.target, cast.amount);
        cast.left -= 1;
        cast.next += u32::from(cast.gap);
        g.units[i].fx.channel = (cast.left > 0).then_some(cast);
    }
}
