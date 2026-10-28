//! The diorama arena: grass island over water, dirt lane with tile
//! markers, two stone Strongholds with three damage states, edge props.
//! Static parts are merged into a few vertex-colored meshes.
//!
//! World axes: the lane runs along X from x = 0 (own Stronghold) to
//! x = 16 (enemy); y is up; the lane is 3 wide along Z.

use bevy::prelude::*;
use tbg_core::content::STRONGHOLD_HP;

use crate::art::mesh::{self, Shape::*};
use crate::art::palette::*;
use crate::config;
use crate::sync::LatestView;

mod props;

/// A tower piece that disappears below an HP fraction (own or enemy tower).
#[derive(Component)]
struct TowerPiece {
    own: bool,
    hide_below: f32,
}

pub fn plugin(app: &mut App) {
    app.add_systems(Startup, spawn_arena)
        .add_systems(Update, damage_states);
}

fn painted(parts: Vec<(crate::art::mesh::Shape, Color, Transform)>) -> Mesh {
    let parts = parts
        .into_iter()
        .map(|(s, c, t)| (mesh::paint(mesh::build(s), c), t))
        .collect();
    mesh::merge(parts).unwrap_or_else(|| Cuboid::new(0.01, 0.01, 0.01).into())
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
    let water = mats.add(StandardMaterial {
        base_color: WATER,
        perceptual_roughness: 0.25,
        ..default()
    });
    let at = |x: f32, y: f32, z: f32| Transform::from_xyz(x, y, z);
    let ground = painted(vec![
        (
            Cyl(1.0, 1.0),
            GRASS,
            at(8.0, -0.2, 0.0).with_scale(Vec3::new(13.0, 0.4, 6.4)),
        ),
        (
            Cyl(1.0, 1.0),
            CLIFF,
            at(8.0, -0.55, 0.0).with_scale(Vec3::new(13.2, 0.5, 6.6)),
        ),
    ]);
    let mut lane = vec![(Box(17.6, 0.04, 3.2), LANE, at(8.0, -0.01, 0.0))];
    for i in 0..=16 {
        lane.push((Box(0.06, 0.042, 3.0), LANE_DARK, at(i as f32, -0.009, 0.0)));
    }
    let mut spawn = |m: Mesh, material: &Handle<StandardMaterial>| {
        commands.spawn((
            Mesh3d(meshes.add(m)),
            MeshMaterial3d(material.clone()),
            Transform::default(),
        ));
    };
    spawn(ground, &mat);
    spawn(painted(lane), &mat);
    spawn(painted(props::scatter()), &mat);
    spawn(Plane3d::default().mesh().size(90.0, 70.0).into(), &water);
    for (own, x) in [(true, -0.9), (false, 16.9)] {
        for (piece, hide_below) in tower(own) {
            commands.spawn((
                Mesh3d(meshes.add(piece)),
                MeshMaterial3d(mat.clone()),
                Transform::from_xyz(x, 0.0, 0.0),
                TowerPiece { own, hide_below },
            ));
        }
    }
    lights(&mut commands);
}

const BASE: &[[f32; 2]] = &[[1.0, 0.0], [1.0, 0.18], [0.82, 0.24], [0.76, 0.95]];
const MID: &[[f32; 2]] = &[[0.76, 0.95], [0.7, 1.65], [0.84, 1.75], [0.84, 1.85]];

/// Base, middle, and crown of a tower; the middle hides at ≤ 33% HP, the
/// crown at ≤ 66%.
fn tower(own: bool) -> [(Mesh, f32); 3] {
    let banner = if own { ALLY } else { ENEMY };
    let at = |y: f32| Transform::from_xyz(0.0, y, 0.0);
    let base = painted(vec![
        (Lathe(BASE), STONE, at(0.0)),
        (Cyl(0.76, 0.02), STONE, at(0.95)),
        (
            Box(0.32, 0.5, 0.06),
            WOOD,
            Transform::from_xyz(0.78, 0.25, 0.0).with_rotation(Quat::from_rotation_y(1.57)),
        ),
    ]);
    let mid = painted(vec![
        (Lathe(MID), STONE, at(0.0)),
        (Cyl(0.84, 0.02), ROCK, at(1.85)),
        (
            Box(0.06, 0.55, 0.4),
            banner,
            Transform::from_xyz(0.78, 1.25, 0.0),
        ),
    ]);
    let mut crown = Vec::new();
    for i in 0..8 {
        let a = i as f32 / 8.0 * std::f32::consts::TAU;
        crown.push((
            Box(0.22, 0.24, 0.22),
            STONE,
            Transform::from_xyz(a.cos() * 0.72, 1.97, a.sin() * 0.72),
        ));
    }
    crown.push((Cyl(0.03, 0.9), WOOD, at(2.3)));
    crown.push((
        Box(0.04, 0.32, 0.5),
        banner,
        Transform::from_xyz(0.0, 2.55, 0.26),
    ));
    [(base, -1.0), (mid, 1.0 / 3.0), (painted(crown), 2.0 / 3.0)]
}

fn lights(commands: &mut Commands) {
    let shadows = config::url_param("shadows").as_deref() != Some("0");
    commands.spawn((
        DirectionalLight {
            color: KEY,
            shadow_maps_enabled: shadows,
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

/// Towers lose their crown, then their middle, as Stronghold HP drops.
fn damage_states(view: Res<LatestView>, mut pieces: Query<(&TowerPiece, &mut Visibility)>) {
    if config::url_param("gallery").is_some() && config::url_param("silhouette").is_some() {
        return;
    }
    let hp = view.0.as_ref().map(|v| {
        let me = v.you.index();
        [v.stronghold_hp[me], v.stronghold_hp[1 - me]].map(|h| h as f32 / STRONGHOLD_HP as f32)
    });
    for (piece, mut vis) in &mut pieces {
        let fraction = hp.map_or(1.0, |[own, enemy]| if piece.own { own } else { enemy });
        let want = if fraction > piece.hide_below {
            Visibility::Inherited
        } else {
            Visibility::Hidden
        };
        if *vis != want {
            *vis = want;
        }
    }
}
