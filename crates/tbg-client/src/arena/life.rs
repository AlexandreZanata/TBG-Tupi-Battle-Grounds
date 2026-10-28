//! Allocation-free flame flicker, rigid cloth segments, and bounded coastal drift.

use super::{camp::BRAZIERS, painted, towers::TowerPiece};
use crate::art::{
    mesh::{self, Shape::*},
    palette::*,
};
use bevy::prelude::*;

#[derive(Component)]
pub(super) struct Flame(f32);

#[derive(Component)]
pub(super) struct Cloth {
    phase: f32,
    rest: Vec3,
}

#[derive(Component)]
pub(super) struct Foam;

#[derive(Resource)]
pub(super) struct Glow(Handle<StandardMaterial>);

pub(super) fn spawn(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    mats: &mut Assets<StandardMaterial>,
    material: &Handle<StandardMaterial>,
) -> usize {
    let fire_mat = mats.add(StandardMaterial {
        base_color: Color::WHITE,
        emissive: LinearRgba::new(1.6, 0.45, 0.04, 1.0),
        unlit: true,
        ..default()
    });
    commands.insert_resource(Glow(fire_mat.clone()));
    let fire = painted(vec![
        (
            Cone(0.17, 0.38),
            Color::srgb(1.0, 0.32, 0.025),
            Transform::from_xyz(0.0, 0.16, 0.0),
        ),
        (
            Cone(0.105, 0.30),
            Color::srgb(1.0, 0.81, 0.14),
            Transform::from_xyz(0.0, 0.15, 0.07),
        ),
        (
            Cone(0.05, 0.20),
            LINEN,
            Transform::from_xyz(0.0, 0.12, 0.11),
        ),
    ]);
    let mut triangles = mesh::triangles(&fire) * BRAZIERS.len();
    let fire = meshes.add(fire);
    for (x, z) in BRAZIERS {
        commands.spawn((
            Mesh3d(fire.clone()),
            MeshMaterial3d(fire_mat.clone()),
            Transform::from_xyz(x, 0.53, z),
            Flame(x + z),
        ));
    }
    for x in [3.9, 12.1] {
        for z in [-2.5, 2.5] {
            triangles += flag(
                commands,
                meshes,
                material,
                Vec3::new(x, 0.65, z + 0.08),
                x < 8.0,
                false,
            );
        }
    }
    for (own, x) in [(true, -0.9), (false, 16.9)] {
        triangles += flag(
            commands,
            meshes,
            material,
            Vec3::new(x, 2.65, 0.0),
            own,
            true,
        );
    }
    // Eight flames, eighteen cloth segments, and one foam layer.
    info!("arena life: {triangles} triangles, 27 animated entities");
    triangles
}

fn flag(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    mat: &Handle<StandardMaterial>,
    at: Vec3,
    own: bool,
    tower: bool,
) -> usize {
    let color = if own { ALLY } else { ENEMY };
    let mut total = 0;
    for segment in 0..3 {
        let mut parts = vec![(Box(0.36, 0.16, 0.028), color, Transform::default())];
        if segment == 1 {
            parts.push((
                Box(0.10, 0.10, 0.035),
                LINEN,
                Transform::from_xyz(0.0, 0.0, 0.018).with_rotation(Quat::from_rotation_z(0.785)),
            ));
        }
        let m = painted(parts);
        total += mesh::triangles(&m);
        let rest = at + Vec3::new(0.0, -0.08 - segment as f32 * 0.15, 0.0);
        let mut entity = commands.spawn((
            Mesh3d(meshes.add(m)),
            MeshMaterial3d(mat.clone()),
            Transform::from_translation(rest),
            Cloth {
                phase: at.x + segment as f32 * 0.7,
                rest,
            },
        ));
        if tower {
            entity.insert(TowerPiece {
                own,
                hide_below: 2.0 / 3.0,
            });
        }
    }
    total
}

#[allow(clippy::type_complexity)] // Disjoint animation queries keep transform access safe.
pub(super) fn animate(
    time: Res<Time>,
    glow: Res<Glow>,
    mut mats: ResMut<Assets<StandardMaterial>>,
    mut flames: Query<(&Flame, &mut Transform), (Without<Cloth>, Without<Foam>)>,
    mut cloth: Query<(&Cloth, &mut Transform), (Without<Flame>, Without<Foam>)>,
    mut foam: Query<&mut Transform, (With<Foam>, Without<Cloth>, Without<Flame>)>,
) {
    let t = time.elapsed_secs();
    if let Some(mut mat) = mats.get_mut(&glow.0) {
        mat.emissive = LinearRgba::new(1.6 + (t * 9.0).sin() * 0.2, 0.45, 0.04, 1.0);
    }
    for (f, mut tr) in &mut flames {
        let wave = (t * 10.0 + f.0).sin();
        tr.scale = Vec3::new(1.0 + wave * 0.07, 1.0 + wave * 0.15, 1.0 - wave * 0.06);
    }
    for (c, mut tr) in &mut cloth {
        let wave = (t * 1.8 + c.phase).sin();
        tr.translation = c.rest + Vec3::Z * wave * 0.018;
        tr.rotation = Quat::from_rotation_y(wave * 0.13);
    }
    for mut tr in &mut foam {
        tr.translation.x = (t * 0.24).sin() * 0.045;
        tr.translation.z = (t * 0.19).cos() * 0.045;
    }
}
