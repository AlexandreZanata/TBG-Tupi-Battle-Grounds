//! Diorama assembly: shared vertex-colored layers and authoritative Stronghold views.

use crate::{
    art::{
        mesh::{self, Shape},
        palette::*,
    },
    config,
};
use bevy::prelude::*;

mod cliffs;
mod geometry;
mod ground;
mod props;
mod towers;
mod water;

pub(super) type Part = (Shape, Color, Transform);

pub fn plugin(app: &mut App) {
    app.add_systems(Startup, spawn_arena)
        .add_systems(Update, towers::damage_states);
}

pub(super) fn painted(parts: Vec<Part>) -> Mesh {
    mesh::merge(
        parts
            .into_iter()
            .map(|(s, c, t)| (mesh::paint(mesh::build(s), c), t))
            .collect(),
    )
    .unwrap_or_else(|| Cuboid::new(0.01, 0.01, 0.01).into())
}

fn spawn_arena(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut mats: ResMut<Assets<StandardMaterial>>,
) {
    let mat = mats.add(StandardMaterial {
        base_color: Color::WHITE,
        perceptual_roughness: 1.0,
        ..default()
    });
    let mut total = 0;
    for (name, m) in [
        ("meadow", ground::meadow()),
        ("BattleLine", ground::lane()),
        ("cliffs", cliffs::mesh()),
        ("sea", water::sea()),
        ("foam", water::coast()),
        ("offshore rocks", water::rocks()),
        ("props", painted(props::scatter())),
    ] {
        let triangles = mesh::triangles(&m);
        total += triangles;
        info!("arena {name}: {triangles} triangles");
        commands.spawn((
            Name::new(format!("arena {name}")),
            Mesh3d(meshes.add(m)),
            MeshMaterial3d(mat.clone()),
            Transform::default(),
        ));
    }
    for (own, x) in [(true, -0.9), (false, 16.9)] {
        for (piece, hide_below) in towers::tower(own) {
            total += mesh::triangles(&piece);
            commands.spawn((
                Mesh3d(meshes.add(piece)),
                MeshMaterial3d(mat.clone()),
                Transform::from_xyz(x, 0.0, 0.0),
                towers::TowerPiece { own, hide_below },
            ));
        }
    }
    info!("arena total: {total} triangles");
    lights(&mut commands);
}

fn lights(commands: &mut Commands) {
    commands.spawn((
        DirectionalLight {
            color: KEY,
            shadow_maps_enabled: config::url_param("shadows").as_deref() != Some("0"),
            ..default()
        },
        Transform::from_xyz(4.0, 12.0, 9.0).looking_at(Vec3::new(8.0, 0.0, 0.0), Vec3::Y),
    ));
    commands.insert_resource(GlobalAmbientLight {
        color: AMBIENT_SKY,
        brightness: 420.0,
        ..default()
    });
}
