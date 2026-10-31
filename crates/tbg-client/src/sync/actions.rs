//! Support actions the core announces as their own events rather than as an
//! `AttackStart`: the Smuggler places each crate it drops, a Bloodweaver's
//! transfer streams from enemy to ally, a Shatterback plate breaks.

use bevy::prelude::*;
use tbg_core::{Event, Seat, UnitId, UnitKind};

use super::events::{Fx, Visuals};
use super::UnitEntities;
use crate::vfx::{
    impacts::{self, Area, Impact},
    shots::{self, ShotKind},
};
use crate::visuals::{self, VisualCue};

pub(super) fn cue(
    commands: &mut Commands,
    fx: &Fx,
    map: &UnitEntities,
    visuals: &mut Visuals,
    ev: Event,
) {
    match ev {
        Event::Spawn {
            unit,
            seat,
            kind: UnitKind::Crate,
        } => toss(fx, map, visuals, seat, unit),
        Event::Transfer { from, to, .. } => {
            if let (Some(a), Some(b)) = (chest(map, visuals, from), chest(map, visuals, to)) {
                shots::fire(commands, fx.vfx, ShotKind::Blood, a, b, Area::default());
            }
        }
        Event::PlateBroken { unit } => {
            if let Some(at) = chest(map, visuals, unit) {
                impacts::land(commands, fx.vfx, Impact::Shards, at, Area::default());
            }
        }
        _ => {}
    }
}

fn chest(map: &UnitEntities, visuals: &Visuals, id: UnitId) -> Option<Vec3> {
    map.0
        .get(&id)
        .and_then(|e| visuals.get(*e).ok())
        .map(|(_, t, _)| t.translation + Vec3::Y * 0.6)
}

/// The seat's Smuggler nearest the new crate plays its toss.
fn toss(fx: &Fx, map: &UnitEntities, visuals: &mut Visuals, seat: Seat, crate_id: UnitId) {
    let own = seat == fx.you;
    let near = chest(map, visuals, crate_id).map_or(0.0, |p| p.x);
    let gap = |t: &Transform| (t.translation.x - near).abs();
    let smuggler = visuals
        .iter_mut()
        .filter(|(v, _, _)| v.own == own && v.kind() == UnitKind::Smuggler && !v.dying)
        .min_by(|a, b| gap(&a.1).total_cmp(&gap(&b.1)));
    if let Some((mut v, _, mut anim)) = smuggler {
        visuals::play_cue(&mut v, &mut anim, fx.assets, VisualCue::CargoDrop);
    }
}
