//! Simulation events → visual cues and projectiles.

use bevy::prelude::*;
use tbg_core::{Archetype, Event, Seat, Target, UnitId};

use super::{world_x, UnitEntities};
use crate::visuals::{self, UnitVisual, VisualAssets, VisualCue};

pub(super) fn apply_event(
    commands: &mut Commands,
    assets: &VisualAssets,
    map: &UnitEntities,
    visuals: &mut Query<(&mut UnitVisual, &mut Transform)>,
    ev: Event,
    you: Seat,
) {
    let pos_of = |visuals: &Query<(&mut UnitVisual, &mut Transform)>, t: Target| -> Option<Vec3> {
        match t {
            Target::Unit(id) => map
                .0
                .get(&id)
                .and_then(|e| visuals.get(*e).ok())
                .map(|(_, t)| t.translation + Vec3::Y * 0.5),
            Target::Stronghold(seat) => Some(Vec3::new(
                world_x(tbg_core::state::stronghold_x(seat), you),
                1.2,
                0.0,
            )),
        }
    };
    let cue = |visuals: &mut Query<(&mut UnitVisual, &mut Transform)>, id: UnitId, c: VisualCue| {
        if let Some((mut v, _)) = map.0.get(&id).and_then(|e| visuals.get_mut(*e).ok()) {
            visuals::play_cue(&mut v, assets, c);
        }
    };
    match ev {
        Event::AttackStart { from, to } => {
            let shoots = match from {
                Target::Unit(id) => {
                    cue(visuals, id, VisualCue::AttackStart);
                    map.0
                        .get(&id)
                        .and_then(|e| visuals.get(*e).ok())
                        .is_some_and(|(v, _)| {
                            v.archetype != Archetype::Tank && v.archetype != Archetype::Dps
                        })
                }
                Target::Stronghold(_) => true,
            };
            if let (true, Some(a), Some(b)) = (shoots, pos_of(visuals, from), pos_of(visuals, to)) {
                visuals::spawn_projectile(commands, assets, a, b);
            }
        }
        Event::Hit {
            to: Target::Unit(id),
            ..
        } => cue(visuals, id, VisualCue::Hit),
        Event::Heal { to, .. } => cue(visuals, to, VisualCue::Heal),
        Event::Evolved { unit, .. } => cue(visuals, unit, VisualCue::Evolved),
        Event::Death { unit } => cue(visuals, unit, VisualCue::Death),
        _ => {}
    }
}
