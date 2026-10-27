//! Click/tap walkers: capsules spawned at the own gate, walking +X.
//!
//! Left click or touch release spawns one capsule; each moves at
//! 1.0 tile/s and despawns at the enemy gate. No gameplay.

use bevy::prelude::*;

const SPAWN_X: f32 = 0.5;
const SPAWN_Y: f32 = 0.7;
const DESPAWN_X: f32 = 15.5;
const SPEED: f32 = 1.0;
const LANES: [f32; 3] = [-0.8, 0.0, 0.8];
const COLORS: [[u8; 3]; 4] = [
    [0xFF, 0xD9, 0x3B],
    [0x7C, 0xF2, 0x9A],
    [0xC7, 0x7D, 0xFF],
    [0xFF, 0x8A, 0x3D],
];

#[derive(Component)]
pub struct Walker;

#[derive(Resource, Default)]
pub struct SpawnState {
    next: usize,
}

/// How many walkers are alive. Read by the overlay.
#[derive(Resource, Default)]
pub struct WalkerCount(pub usize);

pub fn plugin(app: &mut App) {
    app.init_resource::<SpawnState>()
        .init_resource::<WalkerCount>()
        .add_systems(Update, (spawn_on_input, move_walkers));
}

fn spawn_on_input(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mouse: Res<ButtonInput<MouseButton>>,
    touches: Res<Touches>,
    mut state: ResMut<SpawnState>,
    mut count: ResMut<WalkerCount>,
) {
    let mut spawns = 0;
    if mouse.just_pressed(MouseButton::Left) {
        spawns += 1;
    }
    spawns += touches.iter_just_released().count();
    for _ in 0..spawns {
        let i = state.next;
        state.next += 1;
        let c = COLORS[i % COLORS.len()];
        commands.spawn((
            Walker,
            Mesh3d(meshes.add(Capsule3d::new(0.3, 0.2))),
            MeshMaterial3d(materials.add(StandardMaterial {
                base_color: Color::srgb_u8(c[0], c[1], c[2]),
                perceptual_roughness: 0.8,
                ..default()
            })),
            Transform::from_xyz(SPAWN_X, SPAWN_Y, LANES[i % LANES.len()]),
        ));
        count.0 += 1;
    }
}

/// Spawn one walker at `x` (used by `?stress=N` and respawn).
pub fn spawn_walker_at(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
    state: &mut SpawnState,
    count: &mut WalkerCount,
    x: f32,
) {
    let i = state.next;
    state.next += 1;
    let c = COLORS[i % COLORS.len()];
    commands.spawn((
        Walker,
        Mesh3d(meshes.add(Capsule3d::new(0.3, 0.2))),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: Color::srgb_u8(c[0], c[1], c[2]),
            perceptual_roughness: 0.8,
            ..default()
        })),
        Transform::from_xyz(x, SPAWN_Y, LANES[i % LANES.len()]),
    ));
    count.0 += 1;
}

fn move_walkers(
    mut commands: Commands,
    time: Res<Time>,
    mut walkers: Query<(Entity, &mut Transform), With<Walker>>,
    mut count: ResMut<WalkerCount>,
) {
    let dt = time.delta_secs();
    for (entity, mut transform) in &mut walkers {
        transform.translation.x += SPEED * dt;
        if transform.translation.x >= DESPAWN_X {
            commands.entity(entity).despawn();
            count.0 = count.0.saturating_sub(1);
        }
    }
}
