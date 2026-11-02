//! Remove live actors, untracked corpses and transient effects at a match/session boundary.
use super::UnitEntities;
use crate::{
    art::aim::ImpactRing,
    visuals::{mount::Mounts, UnitVisual},
};
use bevy::prelude::*;

pub(crate) type BattleVisuals = Or<(
    With<UnitVisual>,
    With<crate::vfx::Vfx>,
    With<crate::vfx::shots::Shot>,
    With<crate::vfx::flame::FlameJet>,
)>;

pub(crate) fn clear(
    commands: &mut Commands,
    units: &mut UnitEntities,
    mounts: &mut Mounts,
    roots: &Query<Entity, BattleVisuals>,
    impacts: &mut ImpactRing,
) {
    // Deaths leave the live-id map before their cosmetic bodies fade.
    units.0.clear();
    for root in roots {
        commands.entity(root).try_despawn();
    }
    mounts.clear();
    *impacts = default();
}
