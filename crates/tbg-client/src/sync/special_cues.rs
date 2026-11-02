//! Server-announced timed specials drive an independent pose channel and exact-center effects.

use super::{
    events::{Fx, Visuals},
    world_x, UnitEntities,
};
use crate::vfx::impacts::{self, Area, Impact};
use bevy::prelude::*;
use tbg_core::{
    content::{self, Ability},
    Event, UnitId, UnitKind,
};

pub(super) fn cue(
    commands: &mut Commands,
    fx: &Fx,
    map: &UnitEntities,
    visuals: &mut Visuals,
    event: Event,
) {
    match event {
        Event::AbilityStarted {
            unit,
            kind,
            primary,
            ..
        } => {
            let target = primary.and_then(|id| position(map, visuals, id));
            if let Some((_, _, mut anim)) = map.0.get(&unit).and_then(|e| visuals.get_mut(*e).ok())
            {
                if kind == UnitKind::RattleTick {
                    anim.special_at = target;
                }
                anim.start_special();
            }
        }
        Event::BarrelBurst {
            center,
            center_z,
            radius,
            tier,
            branch,
            ..
        } => {
            impacts::land(
                commands,
                fx.vfx,
                Impact::Champion(UnitKind::GustDrummer, tier, branch),
                Vec3::new(world_x(center, fx.you), 0.25, center_z as f32 / 1000.0),
                Area {
                    splash: radius as f32 / 1000.0,
                    ..default()
                },
            );
        }
        Event::AbilityResolved { .. } => resolved(commands, fx, map, visuals, event),
        _ => {}
    }
}
fn position(map: &UnitEntities, visuals: &Visuals, id: UnitId) -> Option<Vec3> {
    map.0
        .get(&id)
        .and_then(|e| visuals.get(*e).ok())
        .map(|(_, t, _)| t.translation + Vec3::Y * 0.6)
}
fn area(map: &UnitEntities, visuals: &Visuals, id: UnitId) -> Area {
    let radius = map
        .0
        .get(&id)
        .and_then(|e| visuals.get(*e).ok())
        .and_then(|(v, _, anim)| {
            content::def(v.kind())
                .ability(anim.form.0, anim.form.1)
                .into_iter()
                .find_map(|a| match a {
                    Ability::CelestialRain { radius, .. }
                    | Ability::BloodyRend { radius, .. }
                    | Ability::SwampBelly { radius, .. }
                    | Ability::AreaCrush { radius, .. } => Some(radius),
                    _ => None,
                })
        })
        .unwrap_or(0);
    Area {
        splash: radius as f32 / 1_000.0,
        ..default()
    }
}
fn resolved(
    commands: &mut Commands,
    fx: &Fx,
    map: &UnitEntities,
    visuals: &mut Visuals,
    event: Event,
) {
    let Event::AbilityResolved {
        unit,
        kind,
        primary,
        center,
        ..
    } = event
    else {
        return;
    };
    if let Some((_, _, mut anim)) = map.0.get(&unit).and_then(|e| visuals.get_mut(*e).ok()) {
        anim.resolve_special();
    }
    if crate::art::kits::completion::kit(kind).is_some()
        || crate::art::kits::ogres::contains(kind)
        || crate::art::kits::abyssals::contains(kind)
        || matches!(kind, UnitKind::Volthar | UnitKind::Astreon)
    {
        let at = Vec3::new(world_x(center, fx.you), 0.6, 0.0);
        let form = map
            .0
            .get(&unit)
            .and_then(|e| visuals.get(*e).ok())
            .map_or((0, 0), |(_, _, a)| a.form);
        let splash = if kind == UnitKind::GustDrummer {
            content::def(kind)
                .ability(form.0, form.1)
                .into_iter()
                .find_map(|a| match a {
                    Ability::Completion(tbg_core::content::completion::Ability::Barrel {
                        radius,
                        ..
                    }) => Some(radius as f32 / 1000.0),
                    _ => None,
                })
                .unwrap_or(1.25)
        } else if kind == UnitKind::Blastbeetle {
            0.9
        } else {
            0.0
        };
        impacts::land(
            commands,
            fx.vfx,
            Impact::Champion(kind, form.0, form.1),
            at,
            Area {
                splash,
                ..Default::default()
            },
        );
        return;
    }
    let (impact, target, area) = match kind {
        UnitKind::Crowcaller => (Impact::Plague, primary, Area::default()),
        UnitKind::Whisperer => (Impact::Mind, primary, Area::default()),
        UnitKind::Bulwark => (Impact::Sparks, Some(unit), Area::default()),
        UnitKind::RattleTick => {
            if let (Some(to), Some((visual, root, anim))) = (
                primary.and_then(|id| position(map, visuals, id)),
                map.0.get(&unit).and_then(|e| visuals.get(*e).ok()),
            ) {
                let (clip, t) = anim.clip_time();
                let pose = anim.pose(clip, t);
                let frame = crate::art::presentation::pose_world(kind, &anim.plan, &pose);
                let socket = crate::art::rig::Node::Prop(crate::art::rig::Socket::Nock).index();
                let figure = Transform::from_rotation(Quat::from_rotation_y(visual.yaw))
                    .with_scale(Vec3::splat(anim.plan.scale));
                let from = (root.to_matrix() * figure.to_matrix() * frame[socket])
                    .transform_point3(Vec3::new(0.0, 0.04, 0.24));
                crate::vfx::shots::fire(
                    commands,
                    fx.vfx,
                    crate::vfx::shots::ShotKind::Bullet,
                    from,
                    to,
                    Area::default(),
                );
            }
            return;
        }
        UnitKind::Capybara => (Impact::Belly, Some(unit), area(map, visuals, unit)),
        UnitKind::Trident => (Impact::Rend, primary, area(map, visuals, unit)),
        UnitKind::SolarSentinel => (Impact::Solar, primary, Area::default()),
        UnitKind::LuminousOracle => (Impact::Harmonic, primary, Area::default()),
        UnitKind::CelestialArcher => (Impact::Purify, primary, Area::default()),
        UnitKind::Justiciar => (Impact::Judgment, primary, area(map, visuals, unit)),
        UnitKind::Jawguard => (Impact::Sparks, Some(unit), Area::default()),
        UnitKind::Mantis => (Impact::Precise, primary, Area::default()),
        UnitKind::Weaver => (Impact::Web, primary, Area::default()),
        UnitKind::Drillnest => (Impact::Sparks, Some(unit), Area::default()),
        UnitKind::Hornet => (Impact::Venom, primary, Area::default()),
        UnitKind::Demolisher => (Impact::Piston, Some(unit), area(map, visuals, unit)),
        _ => return,
    };
    if let Some(at) = target.and_then(|id| position(map, visuals, id)) {
        impacts::land(
            commands,
            fx.vfx,
            impact,
            at.with_x(world_x(center, fx.you)),
            area,
        );
    }
}
