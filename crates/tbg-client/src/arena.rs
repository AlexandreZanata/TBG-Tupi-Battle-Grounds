//! Placeholder arena: ground, lane strip, two Strongholds, lights.
//!
//! World axes: lane runs along X from `x = 0.0` (own Stronghold) to
//! `x = 16.0` (enemy Stronghold); `y` is up; lane width 3.0 along Z.

use bevy::prelude::*;

pub fn plugin(app: &mut App) {
    app.add_systems(Startup, spawn_arena);
}

fn spawn_arena(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // Ground plane 24 x 10 centered at (8, 0, 0), grass green #7BB661.
    commands.spawn((
        Mesh3d(meshes.add(Plane3d::default().mesh().size(24.0, 10.0))),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: Color::srgb_u8(0x7B, 0xB6, 0x61),
            perceptual_roughness: 1.0,
            ..default()
        })),
        Transform::from_xyz(8.0, 0.0, 0.0),
    ));
    // Lane strip 16 x 3 at y = 0.01, sand #C9A46A.
    commands.spawn((
        Mesh3d(meshes.add(Plane3d::default().mesh().size(16.0, 3.0))),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: Color::srgb_u8(0xC9, 0xA4, 0x6A),
            perceptual_roughness: 1.0,
            ..default()
        })),
        Transform::from_xyz(8.0, 0.01, 0.0),
    ));
    // Own Stronghold at x = -0.8, ally blue #3FA9F5.
    commands.spawn((
        Mesh3d(meshes.add(Cuboid::new(1.6, 2.0, 2.4))),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: Color::srgb_u8(0x3F, 0xA9, 0xF5),
            perceptual_roughness: 0.8,
            ..default()
        })),
        Transform::from_xyz(-0.8, 1.0, 0.0),
    ));
    // Enemy Stronghold at x = 16.8, enemy red #F2545B.
    commands.spawn((
        Mesh3d(meshes.add(Cuboid::new(1.6, 2.0, 2.4))),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: Color::srgb_u8(0xF2, 0x54, 0x5B),
            perceptual_roughness: 0.8,
            ..default()
        })),
        Transform::from_xyz(16.8, 1.0, 0.0),
    ));
    // One directional light with shadows, pointing down at about 50
    // degrees elevation (12 up over 10 across).
    commands.spawn((
        DirectionalLight {
            shadow_maps_enabled: true,
            ..default()
        },
        Transform::from_xyz(8.0, 12.0, 10.0).looking_at(Vec3::new(8.0, 0.0, 0.0), Vec3::Y),
    ));
    // Low ambient fill so shadowed faces stay readable.
    commands.insert_resource(GlobalAmbientLight {
        color: Color::WHITE,
        brightness: 250.0,
        ..default()
    });
}
