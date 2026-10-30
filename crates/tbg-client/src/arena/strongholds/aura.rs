//! The Command War Banner aura: a faint team-colored arc on the ground at
//! the aura's authoritative radius, centered on the Stronghold's lane x.

use super::parts::{Form, StrongholdArt};
use crate::art::mesh;
use bevy::render::render_resource::PrimitiveTopology;
use bevy::{asset::RenderAssetUsages, light::NotShadowCaster, prelude::*};
use tbg_core::{Ruleset, StrongholdKind};

/// Half the BattleLine's width: the arc stops at the lane edges, where the
/// aura matters, so it never paints the meadow or the cliffs.
const LANE_HALF: f32 = 1.54;
const WIDTH: f32 = 0.07;
const SEGMENTS: usize = 16;

/// War Banner radius in milli-tiles for a Command form (BR-SK04).
pub(super) fn radius_milli(form: Form) -> i32 {
    let branch = u8::from(form.b());
    tbg_core::stronghold::stats(
        Ruleset::canon(),
        StrongholdKind::Command,
        form.tier(),
        branch,
    )
    .aura_range
}

/// The arc around the origin toward +X; one lane milli-tile is 1/1000 of
/// a world unit, as everywhere in the arena.
pub(super) fn arc(radius_milli: i32, color: Color) -> Mesh {
    let r = (radius_milli as f32 / 1000.0).max(WIDTH);
    let half = (LANE_HALF / r).min(1.0).asin();
    let point = |radius: f32, a: f32| [radius * a.cos(), 0.0, radius * a.sin()];
    let mut pos = Vec::with_capacity(SEGMENTS * 6);
    for i in 0..SEGMENTS {
        let a0 = -half + 2.0 * half * i as f32 / SEGMENTS as f32;
        let a1 = -half + 2.0 * half * (i + 1) as f32 / SEGMENTS as f32;
        let (i0, o0) = (point(r - WIDTH / 2.0, a0), point(r + WIDTH / 2.0, a0));
        let (i1, o1) = (point(r - WIDTH / 2.0, a1), point(r + WIDTH / 2.0, a1));
        pos.extend([i0, i1, o1, i0, o1, o0]);
    }
    let mut m = Mesh::new(
        PrimitiveTopology::TriangleList,
        RenderAssetUsages::default(),
    );
    m.insert_attribute(Mesh::ATTRIBUTE_POSITION, pos);
    mesh::paint(m, color)
}

/// Lay the arc for `form` under `root`, `offset` world units from the tower
/// toward its lane x (mirrored on the right-hand tower).
pub(super) fn spawn(
    commands: &mut Commands,
    art: &mut StrongholdArt,
    meshes: &mut Assets<Mesh>,
    root: Entity,
    own: bool,
    form: Form,
    offset: f32,
) {
    let mesh = art.ring(radius_milli(form), own, meshes);
    let facing = if own { 0.0 } else { std::f32::consts::PI };
    let at = Transform::from_rotation(Quat::from_rotation_y(facing))
        .mul_transform(Transform::from_xyz(offset, 0.035, 0.0));
    commands.spawn((
        Name::new("War Banner aura"),
        Mesh3d(mesh),
        MeshMaterial3d(art.ring_material.clone()),
        at,
        NotShadowCaster,
        ChildOf(root),
    ));
}
