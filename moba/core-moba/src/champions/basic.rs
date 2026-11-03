//! Completed basic-hit hooks: counters, conditional damage, afflictions and bounded secondary targets.
use super::{abilities, cue, effects, nearby, scaled, state::PulseKind};
use crate::{
    state::{MobaMatch, Unit},
    types::UnitKind,
};
use tbg_core::content::{completion::Ability as C, Ability};

pub fn counter(u: &Unit) -> u8 {
    u.fx.attacks.min(u16::from(u8::MAX)) as u8
}
fn nth(u: &Unit, every: u8) -> bool {
    u.fx.attacks.saturating_add(1) >= u16::from(every.max(1))
}
pub fn bonus(g: &mut MobaMatch, i: usize, j: usize, damage: i32) -> i32 {
    let u = g.units[i].clone();
    let v = g.units[j].clone();
    let mut bp = 10_000;
    let mut flat = 0;
    if u.fx.weaken_until > g.tick {
        bp -= i32::from(u.fx.weaken);
    }
    for a in abilities(&u) {
        match a {
            Ability::Plates { count, dmg_bp, .. } => {
                bp += i32::from(count.saturating_sub(u.fx.plates)) * i32::from(dmg_bp)
            }
            Ability::Essence { dmg, .. } => flat += i32::from(dmg) * i32::from(u.fx.essence),
            Ability::Furnace { burn_bp, max_bp } => {
                let burn = scaled(u.max_hp, i32::from(burn_bp)).max(1);
                g.units[i].hp = (u.hp - burn).max(0);
                if !v.kind.structure() {
                    bp += scaled(
                        i32::from(max_bp),
                        (i64::from(u.max_hp - g.units[i].hp) * 10_000 / i64::from(u.max_hp.max(1)))
                            as i32,
                    );
                }
            }
            _ => {}
        }
    }
    if !v.kind.structure() {
        for a in abilities(&u) {
            match a {
                Ability::PreciseStrike { every, damage }
                    if u.fx.target == v.id && nth(&u, every) =>
                {
                    flat += i32::from(damage)
                }
                Ability::PredatoryFrenzy {
                    step_bp,
                    increments,
                    ..
                } if u.fx.target == v.id && u.fx.mark_until > g.tick => {
                    bp += i32::from(step_bp) * i32::from(counter(&u).min(increments))
                }
                Ability::CrowMark { bonus_bp, .. }
                    if u.fx.mark == v.id && u.fx.mark_until > g.tick =>
                {
                    bp += i32::from(bonus_bp)
                }
                Ability::PurifyingArrow { every, damage } if nth(&u, every) => {
                    g.units[j].fx.shield = 0;
                    g.units[j].fx.shield_until = 0;
                    flat += i32::from(damage)
                }
                Ability::StormJudgment {
                    every, primary_bp, ..
                } if nth(&u, every) => bp += i32::from(primary_bp),
                Ability::VerdictLance {
                    every,
                    max_hp_bp,
                    cap,
                    low_hp_bp,
                    execute_bp,
                } if u.fx.target == v.id && nth(&u, every) => {
                    flat += scaled(v.max_hp, i32::from(max_hp_bp)).min(i32::from(cap));
                    if i64::from(v.hp) * 10_000 < i64::from(v.max_hp) * i64::from(low_hp_bp) {
                        bp += i32::from(execute_bp);
                    }
                }
                Ability::Completion(C::Finisher { below_bp, bonus_bp })
                    if i64::from(v.hp) * 10_000 < i64::from(v.max_hp) * i64::from(below_bp) =>
                {
                    bp += i32::from(bonus_bp)
                }
                Ability::Completion(C::Pounce { bonus_bp, .. }) if u.fx.charged => {
                    bp += i32::from(bonus_bp);
                    g.units[i].fx.charged = false;
                    g.units[i].fx.walked = 0;
                }
                Ability::Completion(C::Eclipse {
                    high_bp,
                    solar_bp,
                    lunar_bp,
                }) => {
                    if u.fx.attacks.is_multiple_of(2)
                        && i64::from(v.hp) * 10_000 >= i64::from(v.max_hp) * i64::from(high_bp)
                    {
                        bp += i32::from(solar_bp);
                    } else if !u.fx.attacks.is_multiple_of(2)
                        && i64::from(v.hp) * 10_000 < i64::from(v.max_hp) * i64::from(high_bp)
                    {
                        flat += scaled(v.max_hp, i32::from(lunar_bp));
                    }
                }
                Ability::Completion(C::Crusher { max_hp_bp, cap }) => {
                    flat += scaled(crate::content::base_stats(v.kind).hp, i32::from(max_hp_bp))
                        .min(i32::from(cap))
                }
                Ability::Completion(C::Opportunity { every, bonus_bp })
                    if nth(&u, every) && v.fx.basic.is_some() =>
                {
                    bp += i32::from(bonus_bp)
                }
                _ => {}
            }
        }
        if v.fx.spark_until > g.tick
            && v.fx.spark_credit != Some(super::credit(&u))
            && u.stats.range <= tbg_core::content::completion::SPARK_MELEE_RANGE
        {
            flat += v.fx.spark;
            g.units[j].fx.spark_until = 0;
        }
    }
    scaled(damage, bp).saturating_add(flat)
}
pub fn completed(g: &mut MobaMatch, source: u32, target: u32) {
    let (Some(i), Some(j)) = (g.slot(source), g.slot(target)) else {
        return;
    };
    let u = g.units[i].clone();
    let v = g.units[j].clone();
    if v.kind.structure() {
        return;
    }
    let mut every = None;
    let mut same = false;
    for a in abilities(&u) {
        match a {
            Ability::Poison { dps, pulses, cap } => effects::pulse(
                g,
                &u,
                target,
                PulseKind::Poison,
                i32::from(dps),
                pulses,
                tbg_core::content::REGEN_PERIOD as u16,
                usize::from(cap),
            ),
            Ability::Fish {
                bp,
                ticks,
                cap,
                per_hit,
            } => {
                for _ in 0..per_hit {
                    if g.units[j].fx.fish.len() < usize::from(cap) {
                        g.units[j].fx.fish.push((bp, g.tick + u32::from(ticks)));
                    }
                }
            }
            Ability::PreciseStrike { every: e, .. } | Ability::VerdictLance { every: e, .. } => {
                every = Some(e);
                same = true;
            }
            Ability::PredatoryFrenzy {
                increments, gap, ..
            } => {
                same = true;
                every = Some(increments.saturating_add(1));
                g.units[i].fx.mark_until = g.tick + u32::from(gap);
            }
            Ability::VenomousSting {
                every: e,
                damage,
                pulses,
            } => {
                every = Some(e);
                if nth(&u, e) {
                    effects::pulse(
                        g,
                        &u,
                        target,
                        PulseKind::Venom,
                        i32::from(damage),
                        pulses,
                        tbg_core::content::REGEN_PERIOD as u16,
                        1,
                    );
                    cue(g, &u, target, v.pos, 0, false);
                }
            }
            Ability::PurifyingArrow { every: e, .. } => {
                every = Some(e);
                if nth(&u, e) {
                    cue(g, &u, target, v.pos, 0, false);
                }
            }
            Ability::CrowMark {
                period,
                ticks,
                slow_bp,
                cut_bp,
                ..
            } if u.fx.ready[0] == 0 => {
                g.units[i].fx.ready[0] = period;
                if !v.alive() || v.fx.ward_until > g.tick {
                    continue;
                }
                g.units[i].fx.mark = target;
                g.units[i].fx.mark_until = g.tick + u32::from(ticks);
                cut(g, target, cut_bp, ticks);
                effects::slow(g, target, slow_bp, ticks);
                cue(g, &u, target, v.pos, 0, false);
            }
            Ability::Crossfire {
                every: e,
                ratio,
                radius,
            } => {
                every = Some(e);
                if nth(&u, e) {
                    for id in nearby(g, &u, v.pos, radius, 1, Some(target), false) {
                        effects::hurt(g, &u, id, scaled(u.stats.damage, i32::from(ratio)));
                        cue(g, &u, id, v.pos, radius, false);
                    }
                }
            }
            Ability::CorrosiveSpit {
                every: e,
                damage,
                radius,
                targets,
            } => {
                every = Some(e);
                if nth(&u, e) {
                    for id in nearby(g, &u, v.pos, radius, targets, Some(target), false) {
                        if let Some(at) = g.unit(id).map(|w| w.pos) {
                            trace(g, &u, v.pos, at);
                        }
                        effects::hurt(g, &u, id, i32::from(damage));
                    }
                    cue(g, &u, target, v.pos, radius, false);
                }
            }
            Ability::StormJudgment {
                every: e,
                jumps,
                radius,
                ratios,
                root,
                ..
            } => {
                every = Some(e);
                if nth(&u, e) {
                    let mut at = v.pos;
                    let mut seen = vec![target];
                    for ratio in ratios.into_iter().take(usize::from(jumps)) {
                        let id = nearby(g, &u, at, radius, u8::MAX, None, false)
                            .into_iter()
                            .find(|id| !seen.contains(id));
                        let Some(id) = id else { break };
                        seen.push(id);
                        let from = at;
                        at = g.unit(id).map_or(at, |v| v.pos);
                        trace(g, &u, from, at);
                        effects::hurt(g, &u, id, scaled(u.stats.damage, i32::from(ratio)));
                        if root > 0 {
                            effects::root(g, id, root);
                        }
                        cue(g, &u, id, at, radius, false);
                    }
                }
            }
            Ability::Echo { every: e, extra } => {
                every = Some(e);
                if nth(&u, e) {
                    let ids = nearby(g, &u, u.pos, u.stats.range, extra, Some(target), false);
                    for id in ids {
                        for a in abilities(&u) {
                            if let Ability::Poison { dps, pulses, cap } = a {
                                effects::pulse(
                                    g,
                                    &u,
                                    id,
                                    PulseKind::Poison,
                                    i32::from(dps),
                                    pulses,
                                    tbg_core::content::REGEN_PERIOD as u16,
                                    usize::from(cap),
                                );
                            }
                        }
                    }
                }
            }
            Ability::Completion(c) => {
                if let Some(e) = completion(g, &u, &v, c) {
                    every = Some(e);
                }
            }
            _ => {}
        }
    }
    let n = if same && u.fx.target != target {
        1
    } else {
        u.fx.attacks.saturating_add(1)
    };
    if let Some(e) = every {
        g.units[i].fx.attacks = if matches!(abilities(&u)[0], Ability::PredatoryFrenzy { .. }) {
            n.min(u16::from(e.saturating_sub(1)))
        } else if n >= u16::from(e.max(1)) {
            0
        } else {
            n
        };
    }
    if same && !g.units[j].alive() {
        g.units[i].fx.attacks = 0;
    }
    if same
        && nth(&u, every.unwrap_or(u8::MAX))
        && abilities(&u).into_iter().any(|a| {
            matches!(
                a,
                Ability::PreciseStrike { .. } | Ability::VerdictLance { .. }
            )
        })
    {
        cue(g, &u, target, v.pos, 0, false);
    }
    g.units[i].fx.target = target;
    if let UnitKind::Troop(k) = u.kind {
        if let Some(on_hit) = tbg_core::content::def(k).on_hit {
            match on_hit {
                tbg_core::OnHit::Slow { bp, ticks } => effects::slow(g, target, bp, ticks),
                tbg_core::OnHit::Root { ticks } => effects::root(g, target, ticks),
            }
        }
    }
}
pub fn feed(g: &mut MobaMatch, source: u32, victim: &Unit, removed: i32) {
    let Some(i) = g.slot(source) else { return };
    let u = g.units[i].clone();
    for a in abilities(&u) {
        match a {
            Ability::Essence { base, step, .. }
                if victim.fx.summon == 0 && matches!(victim.kind, UnitKind::Troop(_)) =>
            {
                let f = &mut g.units[i].fx;
                f.essence_progress = f.essence_progress.saturating_add(removed.max(0) as u32);
                loop {
                    let cost = u32::from(base)
                        .saturating_add(u32::from(step) * u32::from(f.essence))
                        .max(1);
                    if f.essence == u16::MAX || f.essence_progress < cost {
                        break;
                    }
                    f.essence_progress -= cost;
                    f.essence += 1;
                }
            }
            Ability::Siphon {
                heal_bp,
                low_bp,
                split,
            } => {
                let mut allies: Vec<_> = g
                    .units
                    .iter()
                    .filter(|v| {
                        super::ally(&u, v)
                            && v.fx.summon == 0
                            && matches!(v.kind, UnitKind::Troop(_))
                            && v.hp < v.max_hp
                    })
                    .collect();
                allies.sort_by(|a, b| {
                    (i64::from(a.hp) * i64::from(b.max_hp))
                        .cmp(&(i64::from(b.hp) * i64::from(a.max_hp)))
                        .then_with(|| (a.pos.dist2(u.pos), a.id).cmp(&(b.pos.dist2(u.pos), b.id)))
                });
                let n = if split { allies.len().clamp(1, 2) } else { 1 };
                let amount = scaled(removed, i32::from(heal_bp)) / n as i32;
                let heals: Vec<_> = allies
                    .into_iter()
                    .take(n)
                    .map(|v| {
                        (
                            v.id,
                            if i64::from(v.hp) * 10_000 < i64::from(v.max_hp) * i64::from(low_bp) {
                                2 * amount
                            } else {
                                amount
                            },
                        )
                    })
                    .collect();
                for (id, hp) in heals {
                    if effects::heal(g, &u, id, hp) > 0 {
                        if let Some(to) = g.unit(id).map(|v| v.pos) {
                            trace(g, &u, victim.pos, to);
                        }
                    }
                }
            }
            _ => {}
        }
    }
}
fn completion(g: &mut MobaMatch, u: &Unit, v: &Unit, a: C) -> Option<u8> {
    match a {
        C::Sweep {
            every,
            damage,
            radius,
            targets,
        } => {
            if nth(u, every) {
                area(g, u, v.pos, radius, targets, Some(v.id), i32::from(damage));
                cue(g, u, v.id, v.pos, radius, false);
            }
            Some(every)
        }
        C::Pierce {
            every,
            ratio_bp,
            behind,
        } => {
            if nth(u, every) {
                let dx = i64::from(v.pos.x - u.pos.x);
                let dy = i64::from(v.pos.y - u.pos.y);
                let id = nearby(g, u, v.pos, behind, u8::MAX, Some(v.id), false)
                    .into_iter()
                    .find(|id| {
                        g.unit(*id).is_some_and(|w| {
                            i64::from(w.pos.x - v.pos.x) * dx + i64::from(w.pos.y - v.pos.y) * dy
                                >= 0
                        })
                    });
                if let Some(id) = id {
                    effects::hurt(g, u, id, scaled(u.stats.damage, i32::from(ratio_bp)));
                    cue(g, u, id, v.pos, behind, false);
                }
            }
            Some(every)
        }
        C::Spark {
            every,
            damage,
            ticks,
        } => {
            if nth(u, every) && v.fx.ward_until <= g.tick {
                if let Some(i) = g.slot(v.id) {
                    g.units[i].fx.spark = i32::from(damage);
                    g.units[i].fx.spark_until = g.tick + u32::from(ticks);
                    g.units[i].fx.spark_credit = Some(super::credit(u));
                }
            }
            Some(every)
        }
        C::Bleeder { damage, pulses } => {
            effects::pulse(
                g,
                u,
                v.id,
                PulseKind::Bleed,
                i32::from(damage),
                pulses,
                tbg_core::content::REGEN_PERIOD as u16,
                1,
            );
            None
        }
        C::Opportunity { every, .. } => Some(every),
        C::Eclipse { .. } => Some(2),
        _ => None,
    }
}
pub fn area(
    g: &mut MobaMatch,
    u: &Unit,
    at: crate::geom::Pos,
    radius: i32,
    cap: u8,
    exclude: Option<u32>,
    damage: i32,
) {
    for id in nearby(g, u, at, radius, cap, exclude, false) {
        effects::hurt(g, u, id, damage);
    }
}
pub fn cut(g: &mut MobaMatch, id: u32, bp: u16, ticks: u16) {
    if let Some(i) = g.slot(id) {
        if g.units[i].fx.ward_until <= g.tick {
            g.units[i].fx.cut = g.units[i].fx.cut.max(bp);
            g.units[i].fx.cut_until = g.units[i].fx.cut_until.max(g.tick + u32::from(ticks));
        }
    }
}
pub fn death(g: &mut MobaMatch, u: &Unit) {
    for a in abilities(u) {
        if let Ability::Completion(C::DeathPop {
            damage,
            radius,
            targets,
        }) = a
        {
            if u.last_hit.is_some() {
                area(g, u, u.pos, radius, targets, None, i32::from(damage));
                cue(g, u, 0, u.pos, radius, false);
            }
        }
    }
}

pub fn trace(g: &mut MobaMatch, u: &Unit, from: crate::geom::Pos, to: crate::geom::Pos) {
    if let UnitKind::Troop(kind) = u.kind {
        g.events.push(crate::state::Event::CombatTrace {
            kind,
            unit: u.id,
            from,
            to,
        });
    }
}
/// The 1v1 line becomes a swept 2D segment against authored body envelopes.
pub fn flame(g: &mut MobaMatch, u: &Unit, v: &Unit, damage: i32) -> bool {
    let Some((bp, ticks)) = abilities(u).into_iter().find_map(|a| match a {
        Ability::ProfaneFire { cut_bp, cut_ticks } => Some((cut_bp, cut_ticks)),
        _ => None,
    }) else {
        return false;
    };
    let len = u.pos.dist(v.pos).max(1);
    let reach = super::reach(u, v);
    let end = crate::geom::Pos {
        x: u.pos.x + (i64::from(v.pos.x - u.pos.x) * i64::from(reach) / len) as i32,
        y: u.pos.y + (i64::from(v.pos.y - u.pos.y) * i64::from(reach) / len) as i32,
    }
    .clamp_to_map();
    let dx = i64::from(end.x - u.pos.x);
    let dy = i64::from(end.y - u.pos.y);
    let span = dx * dx + dy * dy;
    let victims: Vec<_> = g
        .units
        .iter()
        .filter(|w| {
            w.id != u.id
                && w.alive()
                && !w.kind.structure()
                && (super::enemy(u, w) || super::ally(u, w))
        })
        .filter(|w| {
            let along = i64::from(w.pos.x - u.pos.x) * dx + i64::from(w.pos.y - u.pos.y) * dy;
            if !(0..=span).contains(&along) {
                return false;
            }
            let at = crate::geom::Pos {
                x: u.pos.x + (dx * along / span.max(1)) as i32,
                y: u.pos.y + (dy * along / span.max(1)) as i32,
            };
            at.within(w.pos, w.stats.radius)
        })
        .map(|w| w.id)
        .collect();
    trace(g, u, u.pos, end);
    for id in victims {
        effects::hurt(g, u, id, damage);
        cut(g, id, bp, ticks);
    }
    if v.kind.structure() {
        effects::hurt(g, u, v.id, damage);
    }
    true
}
