//! Three bounded, cosmetic brass gears; scene teardown owns all meshes and entities.

use super::super::monuments::block;
use crate::{
    arena::{
        painted,
        theme::{rgb, Theme},
        ArenaScene, Part,
    },
    art::mesh::{self, Shape::*},
};
use bevy::prelude::*;

#[derive(Component)]
pub(in crate::arena) struct Gear(f32);

pub(in crate::arena) fn spawn(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    mat: &Handle<StandardMaterial>,
) -> usize {
    let mut triangles = 0;
    for (at, r, speed) in [
        (Vec3::new(8.0, 2.08, -3.88), 1.0, 0.10),
        (Vec3::new(6.72, 1.37, -3.87), 0.50, -0.20),
        (Vec3::new(9.28, 1.37, -3.87), 0.50, -0.20),
    ] {
        let m = painted(gear(r));
        triangles += mesh::triangles(&m);
        commands.spawn((
            ArenaScene,
            Name::new("arena brass gear"),
            Gear(speed),
            Mesh3d(meshes.add(m)),
            MeshMaterial3d(mat.clone()),
            Transform::from_translation(at),
        ));
    }
    triangles
}

fn gear(r: f32) -> Vec<Part> {
    let p = Theme::Clockwork.palette();
    let mut parts = vec![
        (
            Ring(0.10 * r, 0.76 * r, 360.0),
            p.accent,
            Transform::default().with_rotation(Quat::from_rotation_x(std::f32::consts::FRAC_PI_2)),
        ),
        (
            Cyl(0.23 * r, 0.28 * r),
            rgb(0xe3b568),
            Transform::default().with_rotation(Quat::from_rotation_x(std::f32::consts::FRAC_PI_2)),
        ),
    ];
    for i in 0..12 {
        let a = i as f32 * std::f32::consts::TAU / 12.0;
        let rot = Quat::from_rotation_z(a);
        parts.push((
            Bevel(0.23 * r, 0.26 * r, 0.24 * r, 0.025 * r),
            p.accent,
            Transform::from_translation(rot * Vec3::Y * 0.90 * r).with_rotation(rot),
        ));
        if i % 2 == 0 {
            parts.push((
                Box(0.12 * r, 0.58 * r, 0.16 * r),
                rgb(0xb28143),
                Transform::from_translation(rot * Vec3::Y * 0.45 * r).with_rotation(rot),
            ));
        }
    }
    block(
        &mut parts,
        Vec3::Z * 0.17 * r,
        Vec3::splat(0.16 * r),
        p.dark,
    );
    parts
}

pub(in crate::arena) fn animate(time: Res<Time>, mut gears: Query<(&Gear, &mut Transform)>) {
    for (gear, mut tr) in &mut gears {
        tr.rotation = Quat::from_rotation_z(time.elapsed_secs() * gear.0);
    }
}
