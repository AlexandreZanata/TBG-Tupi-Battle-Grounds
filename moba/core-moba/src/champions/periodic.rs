//! Independent special readiness, locked windups and two-dimensional area resolution.
use super::{abilities, cue, effects, nearby, state::Cast};
use crate::{
    geom::Pos,
    state::{MobaMatch, Unit},
};
use tbg_core::content::{completion::Ability as C, Ability, WINDUP};
fn period(a: Ability) -> Option<u16> {
    match a {
        Ability::DrillNest { period }
        | Ability::StickyWeb { period, .. }
        | Ability::FearPulse { period, .. }
        | Ability::LivingWall { period, .. }
        | Ability::AreaCrush { period, .. }
        | Ability::RaisedCarapace { period, .. }
        | Ability::CelestialRain { period, .. }
        | Ability::HarmonicPulse { period, .. }
        | Ability::SolarBarrier { period, .. }
        | Ability::BloodyRend { period, .. }
        | Ability::SwampBelly { period, .. }
        | Ability::ShoulderPistol { period, .. }
        | Ability::Completion(
            C::Interrupt { period, .. } | C::Beam { period, .. } | C::Rhythm { period, .. },
        ) => Some(period),
        _ => None,
    }
}
fn primary(g: &MobaMatch, u: &Unit, a: Ability) -> Option<u32> {
    let reach = match a {
        Ability::StickyWeb { range_add, .. } => super::reach(u, u) + range_add,
        Ability::ShoulderPistol { range, .. } | Ability::SolarBarrier { range, .. } => range,
        _ => super::reach(u, u),
    };
    let allied = matches!(a, Ability::SolarBarrier { .. });
    let ids = nearby(g, u, u.pos, reach, u8::MAX, None, allied);
    ids.into_iter().find(|id| {
        let Some(v) = g.unit(*id) else { return false };
        match a {
            Ability::DrillNest { .. } => super::summons::drill_ready(g, u),
            Ability::Completion(C::Interrupt { .. }) => {
                v.fx.casts.iter().any(Option::is_some)
                    && v.fx.interrupt_until <= g.tick
                    && v.fx.ward_until <= g.tick
            }
            Ability::SolarBarrier { .. } => v.hp < v.max_hp && v.id != u.id,
            _ => true,
        }
    })
}
pub fn tick(g: &mut MobaMatch) {
    let n = g.units.len();
    for i in 0..n {
        if !g.units[i].alive() {
            continue;
        }
        let u = g.units[i].clone();
        for (slot, a) in abilities(&u).into_iter().enumerate() {
            let Some(every) = period(a) else { continue };
            if super::frozen(&u, g.tick) {
                g.units[i].fx.casts[slot] = None;
                continue;
            }
            if let Some(mut cast) = u.fx.casts[slot] {
                if g.unit(cast.target).is_none_or(|v| !v.alive())
                    && !matches!(a, Ability::Completion(C::Beam { .. }))
                {
                    g.units[i].fx.casts[slot] = None;
                    continue;
                }
                if cast.left > 1 {
                    cast.left -= 1;
                    g.units[i].fx.casts[slot] = Some(cast);
                    continue;
                }
                g.units[i].fx.casts[slot] = None;
                let centre = match a {
                    Ability::Completion(C::Beam { .. }) => cast.centre,
                    Ability::AreaCrush { .. }
                    | Ability::SwampBelly { .. }
                    | Ability::LivingWall { .. }
                    | Ability::RaisedCarapace { .. } => u.pos,
                    _ => g.unit(cast.target).map_or(cast.centre, |v| v.pos),
                };
                let radius = resolve(g, &u, a, cast.target, centre);
                cue(g, &u, cast.target, centre, radius, false);
            } else if u.fx.ready[slot] == 0 {
                let Some(target) = primary(g, &u, a) else {
                    continue;
                };
                let centre = if matches!(a, Ability::AreaCrush { .. } | Ability::SwampBelly { .. })
                {
                    u.pos
                } else {
                    g.unit(target).map_or(u.pos, |v| v.pos)
                };
                let windup = match a {
                    Ability::Completion(C::Interrupt { windup, .. }) => windup,
                    _ => WINDUP,
                };
                g.units[i].fx.casts[slot] = Some(Cast {
                    slot: slot as u8,
                    target,
                    centre,
                    left: windup,
                    windup,
                });
                g.units[i].fx.ready[slot] = every.max(1);
                cue(g, &u, target, centre, 0, true);
            }
        }
    }
}
fn resolve(g: &mut MobaMatch, u: &Unit, a: Ability, target: u32, at: Pos) -> i32 {
    match a {
        Ability::DrillNest { .. } => {
            super::summons::drill(g, u);
            0
        }
        Ability::StickyWeb {
            damage,
            root,
            slow_bp,
            slow_ticks,
            ..
        } => {
            effects::hurt(g, u, target, i32::from(damage));
            effects::root(g, target, root);
            if slow_bp > 0 {
                effects::slow(g, target, slow_bp, root.saturating_add(slow_ticks));
            }
            0
        }
        Ability::FearPulse { bp, ticks, .. } => {
            effects::slow(g, target, bp, ticks);
            0
        }
        Ability::LivingWall { bp, ticks, .. } | Ability::RaisedCarapace { bp, ticks, .. } => {
            effects::reduction(g, u.id, bp, ticks);
            0
        }
        Ability::AreaCrush { damage, radius, .. }
        | Ability::CelestialRain { damage, radius, .. } => {
            let radius = if matches!(a, Ability::AreaCrush { .. }) {
                radius + u.stats.radius
            } else {
                radius
            };
            super::basic::area(g, u, at, radius, u8::MAX, None, i32::from(damage));
            if let Ability::CelestialRain { bp, ticks, .. } = a {
                for id in nearby(g, u, at, radius, u8::MAX, None, false) {
                    effects::slow(g, id, bp, ticks);
                }
            }
            radius
        }
        Ability::HarmonicPulse { bp, ticks, .. } => {
            weaken(g, target, bp, ticks);
            0
        }
        Ability::SolarBarrier { amount, ticks, .. } => {
            effects::shield(g, target, i32::from(amount), ticks);
            0
        }
        Ability::BloodyRend {
            damage,
            radius,
            bleed,
            pulses,
            ..
        } => {
            for id in nearby(g, u, at, radius, u8::MAX, None, false) {
                effects::hurt(g, u, id, i32::from(damage));
                effects::pulse(
                    g,
                    u,
                    id,
                    super::state::PulseKind::Bleed,
                    i32::from(bleed),
                    pulses,
                    20,
                    1,
                );
            }
            radius
        }
        Ability::SwampBelly {
            damage,
            radius,
            bp,
            ticks,
            protection,
            protection_ticks,
            ..
        } => {
            let radius = radius + u.stats.radius;
            for id in nearby(g, u, at, radius, u8::MAX, None, false) {
                effects::hurt(g, u, id, i32::from(damage));
                weaken(g, id, bp, ticks);
            }
            effects::reduction(g, u.id, protection, protection_ticks);
            radius
        }
        Ability::ShoulderPistol { damage, .. } => {
            effects::hurt(g, u, target, i32::from(damage));
            0
        }
        Ability::Completion(C::Interrupt { lock_ticks, .. }) => {
            if let Some(i) = g.slot(target) {
                if g.units[i].fx.ward_until <= g.tick {
                    g.units[i].fx.casts = [None; 2];
                    g.units[i].fx.interrupt_until = g.tick + u32::from(lock_ticks);
                }
            }
            0
        }
        Ability::Completion(C::Beam {
            damage,
            radius,
            targets,
            ..
        }) => {
            super::basic::area(g, u, at, radius, targets, None, i32::from(damage));
            radius
        }
        Ability::Completion(C::Rhythm {
            radius,
            targets,
            bp,
            ticks,
            ..
        }) => {
            for id in nearby(g, u, at, radius, targets, None, false) {
                effects::slow(g, id, bp, ticks);
            }
            radius
        }
        _ => 0,
    }
}

fn weaken(g: &mut MobaMatch, id: u32, bp: u16, ticks: u16) {
    if let Some(i) = g.slot(id) {
        let f = &mut g.units[i].fx;
        if f.ward_until <= g.tick {
            f.weaken = f.weaken.max(bp);
            f.weaken_until = f.weaken_until.max(g.tick + u32::from(ticks));
        }
    }
}
