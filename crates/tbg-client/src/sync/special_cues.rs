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
        Event::AbilityStarted { unit, .. } => {
            if let Some((_, _, mut anim)) = map.0.get(&unit).and_then(|e| visuals.get_mut(*e).ok())
            {
                anim.start_special();
            }
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
                    Ability::AreaCrush { radius, .. } => Some(radius),
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
    let (impact, target, area) = match kind {
        UnitKind::Crowcaller => (Impact::Plague, primary, Area::default()),
        UnitKind::Whisperer => (Impact::Mind, primary, Area::default()),
        UnitKind::Bulwark => (Impact::Sparks, Some(unit), Area::default()),
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
