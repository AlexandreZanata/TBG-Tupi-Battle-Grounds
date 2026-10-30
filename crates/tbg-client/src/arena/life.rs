//! Allocation-free flame flicker, rigid cloth segments, and bounded coastal drift.

use super::{camp::BRAZIERS, painted, theme::Theme, towers::TowerPiece};
use crate::art::{
    mesh::{self, Shape::*},
    palette::*,
};
use bevy::{light::NotShadowCaster, prelude::*};

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
pub(super) struct Glow {
    material: Handle<StandardMaterial>,
    color: LinearRgba,
    wind: f32,
}

pub(super) fn spawn(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    mats: &mut Assets<StandardMaterial>,
    material: &Handle<StandardMaterial>,
    theme: Theme,
) -> usize {
    let colors = theme.palette().fire;
    let glow = if theme == Theme::Forest {
        LinearRgba::new(1.6, 0.45, 0.04, 1.0)
    } else {
        colors[0].to_linear() * 1.6
    };
    let fire_mat = mats.add(StandardMaterial {
        base_color: Color::WHITE,
        emissive: glow,
        unlit: true,
        ..default()
    });
    commands.insert_resource(Glow {
        material: fire_mat.clone(),
        color: glow,
        wind: if theme == Theme::Storm { 2.0 } else { 1.0 },
    });
    let fire = painted(vec![
        (
            Cone(0.17, 0.38),
            colors[0],
            Transform::from_xyz(0.0, 0.16, 0.0),
        ),
        (
            Cone(0.105, 0.30),
            colors[1],
            Transform::from_xyz(0.0, 0.15, 0.07),
        ),
        (
            Cone(0.05, 0.20),
            colors[2],
            Transform::from_xyz(0.0, 0.12, 0.11),
        ),
    ]);
    let mut triangles = mesh::triangles(&fire) * BRAZIERS.len();
    let fire = meshes.add(fire);
    let cloth_meshes = [ALLY, ENEMY].map(|color| {
        [false, true].map(|crest| {
            let mut parts = vec![(Box(0.36, 0.16, 0.028), color, Transform::default())];
            if crest {
                parts.push((
                    Box(0.10, 0.10, 0.035),
                    LINEN,
                    Transform::from_xyz(0.0, 0.0, 0.018)
                        .with_rotation(Quat::from_rotation_z(0.785)),
                ));
            }
            meshes.add(painted(parts))
        })
    });
    for (x, z) in BRAZIERS {
        commands.spawn((
            Mesh3d(fire.clone()),
            MeshMaterial3d(fire_mat.clone()),
            Transform::from_xyz(x, 0.53, z),
            Flame(x + z),
            NotShadowCaster,
        ));
    }
    for x in [3.9, 12.1] {
        for z in [-2.5, 2.5] {
            triangles += flag(
                commands,
                &cloth_meshes,
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
            &cloth_meshes,
            material,
            Vec3::new(x, 2.65, 0.0),
            own,
            true,
        );
    }
    // Eight flames, eighteen cloth segments, and one foam layer.
    info!(
        "arena life: {triangles} triangles, {} animated entities",
        if theme == Theme::Tundra { 26 } else { 27 }
    );
    triangles
}

fn flag(
    commands: &mut Commands,
    meshes: &[[Handle<Mesh>; 2]; 2],
    mat: &Handle<StandardMaterial>,
    at: Vec3,
    own: bool,
    tower: bool,
) -> usize {
    for segment in 0..3 {
        let m = &meshes[usize::from(!own)][usize::from(segment == 1)];
        let rest = at + Vec3::new(0.0, -0.08 - segment as f32 * 0.15, 0.0);
        let mut entity = commands.spawn((
            Mesh3d(m.clone()),
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
    48
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
    if let Some(mut mat) = mats.get_mut(&glow.material) {
        mat.emissive = LinearRgba::new(
            glow.color.red + (t * 9.0).sin() * 0.2,
            glow.color.green,
            glow.color.blue,
            1.0,
        );
    }
    for (f, mut tr) in &mut flames {
        let wave = (t * 10.0 + f.0).sin();
        tr.scale = Vec3::new(1.0 + wave * 0.07, 1.0 + wave * 0.15, 1.0 - wave * 0.06);
    }
    for (c, mut tr) in &mut cloth {
        let wave = (t * 1.8 * glow.wind + c.phase).sin();
        tr.translation = c.rest + Vec3::Z * wave * 0.018 * glow.wind;
        tr.rotation = Quat::from_rotation_y(wave * 0.13 * glow.wind);
    }
    for mut tr in &mut foam {
        tr.translation.x = (t * 0.24).sin() * 0.045;
        tr.translation.z = (t * 0.19).cos() * 0.045;
    }
}
