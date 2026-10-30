//! Low-poly maritime landmarks and one rigid, cosmetic sailboat.

use super::{geometry::Surface, painted, Part};
use crate::art::{mesh, mesh::Shape::*, palette::*};
use bevy::{
    light::{NotShadowCaster, NotShadowReceiver},
    prelude::*,
};

pub(super) fn lighthouse(parts: &mut Vec<Part>, at: Vec3) {
    for (shape, color, y) in [
        (Cyl(0.54, 0.18), ROCK, 0.09),
        (Frustum(0.29, 0.43, 2.10), LINEN, 1.14),
        (Frustum(0.415, 0.435, 0.28), RED_CLOTH, 0.66),
        (Frustum(0.365, 0.385, 0.28), RED_CLOTH, 1.39),
        (Cyl(0.45, 0.10), STONE, 2.20),
        (Cyl(0.29, 0.36), SKY_TOP, 2.43),
        (Cone(0.47, 0.43), RED_CLOTH, 2.80),
    ] {
        parts.push((shape, color, Transform::from_translation(at + Vec3::Y * y)));
    }
    for angle in [
        0.0,
        std::f32::consts::FRAC_PI_2,
        std::f32::consts::PI,
        3.0 * std::f32::consts::FRAC_PI_2,
    ] {
        let offset = Vec3::new(f32::cos(angle) * 0.27, 2.43, f32::sin(angle) * 0.27);
        parts.push((
            Cyl(0.035, 0.38),
            BRASS,
            Transform::from_translation(at + offset),
        ));
    }
    parts.push((
        Box(0.19, 0.36, 0.025),
        DARK_LEATHER,
        Transform::from_translation(at + Vec3::new(0.0, 0.28, 0.415)),
    ));
    parts.push((
        Box(0.10, 0.18, 0.025),
        DARK,
        Transform::from_translation(at + Vec3::new(0.0, 1.80, 0.32)),
    ));
}

pub(super) fn pier(parts: &mut Vec<Part>, at: Vec3) {
    for plank in 0..7 {
        parts.push((
            Box(0.76, 0.07, 0.21),
            if plank % 2 == 0 { TIMBER } else { WOOD },
            Transform::from_translation(at + Vec3::Z * plank as f32 * 0.23),
        ));
    }
    for x in [-0.34, 0.34] {
        for z in [0.0, 1.38] {
            parts.push((
                Cyl(0.065, 1.05),
                BARK,
                Transform::from_translation(at + Vec3::new(x, -0.28, z)),
            ));
        }
    }
    parts.push((
        Box(0.72, 0.07, 1.0),
        WOOD,
        Transform::from_translation(at + Vec3::new(0.0, 0.27, -0.48))
            .with_rotation(Quat::from_rotation_x(0.55)),
    ));
}

#[derive(Component)]
pub(super) struct Sailboat(Vec3);

pub(super) fn spawn_boat(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    material: &Handle<StandardMaterial>,
    at: Vec3,
    scale: f32,
) -> usize {
    let mut hull = Surface::default();
    let rim = [
        (-0.94, 0.0),
        (-0.58, -0.32),
        (0.57, -0.32),
        (0.94, 0.0),
        (0.57, 0.32),
        (-0.58, 0.32),
    ];
    for i in 0..rim.len() {
        let j = (i + 1) % rim.len();
        let point = |k: usize, top: bool| {
            let (x, z) = rim[k];
            Vec3::new(
                x * if top { 1.0 } else { 0.74 },
                if top { 0.18 } else { -0.14 },
                z * if top { 1.0 } else { 0.45 },
            )
        };
        hull.quad(
            [
                point(i, true),
                point(j, true),
                point(j, false),
                point(i, false),
            ],
            WOOD,
        );
        hull.triangle(
            Vec3::new(0.0, 0.18, 0.0),
            point(j, true),
            point(i, true),
            TIMBER,
        );
    }
    let mut sail = Surface::default();
    let a = Vec3::new(0.02, 0.55, 0.07);
    let b = Vec3::new(0.80, 0.55, 0.07);
    let c = Vec3::new(0.02, 2.35, 0.07);
    let belly = Vec3::new(0.22, 1.0, 0.17);
    sail.triangle(a, b, belly, LINEN);
    sail.triangle(b, c, belly, Color::srgb(0.91, 0.87, 0.71));
    sail.triangle(c, a, belly, PARCHMENT);
    sail.triangle(c, b, a, LINEN);
    let rig = painted(vec![
        (
            Cyl(0.033, 2.5),
            DARK_LEATHER,
            Transform::from_xyz(0.0, 1.35, 0.0),
        ),
        (
            Box(0.90, 0.035, 0.04),
            WOOD,
            Transform::from_xyz(0.39, 0.53, 0.04),
        ),
        (
            Box(1.12, 0.045, 0.035),
            LINEN,
            Transform::from_xyz(0.0, 0.08, 0.32),
        ),
        (
            Box(1.12, 0.045, 0.035),
            LINEN,
            Transform::from_xyz(0.0, 0.08, -0.32),
        ),
    ]);
    let boat = mesh::merge(vec![
        (hull.finish(), Transform::default()),
        (sail.finish(), Transform::default()),
        (rig, Transform::default()),
    ])
    .unwrap_or_else(|| Cuboid::new(0.01, 0.01, 0.01).into());
    let triangles = mesh::triangles(&boat);
    commands.spawn((
        Name::new("offshore sailboat"),
        Mesh3d(meshes.add(boat)),
        MeshMaterial3d(material.clone()),
        Transform::from_translation(at).with_scale(Vec3::splat(scale)),
        Sailboat(at),
        NotShadowCaster,
        NotShadowReceiver,
    ));
    triangles
}

pub(super) fn animate(time: Res<Time>, mut boats: Query<(&Sailboat, &mut Transform)>) {
    let t = time.elapsed_secs();
    for (boat, mut tr) in &mut boats {
        tr.translation = boat.0 + Vec3::Y * (t * 0.8).sin() * 0.025;
        tr.rotation = Quat::from_euler(EulerRot::XYZ, 0.0, -0.28, (t * 0.65).sin() * 0.025);
    }
}
