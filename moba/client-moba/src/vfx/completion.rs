//! Stand-in for the 1v1 completion-roster effects (champions outside the
//! shared online pool): a generic impact and evolution burst.

use bevy::prelude::*;
use tbg_core::UnitKind;

use super::impacts::{self, Area, Impact};
use super::VfxAssets;

pub fn impact(
    c: &mut Commands,
    a: &VfxAssets,
    _kind: UnitKind,
    _tier: u8,
    _branch: u8,
    at: Vec3,
    area: Area,
) {
    impacts::land(c, a, Impact::Blast, at, area);
}

pub fn evolution(c: &mut Commands, a: &VfxAssets, _kind: UnitKind, at: Vec3) {
    impacts::land(c, a, Impact::Blast, at, Area::default());
}
