//! Click/tap walkers: capsules spawned at the own gate, walking +X.
//!
//! Left click or touch release spawns one capsule; each moves at
//! 1.0 tile/s and despawns at the enemy gate. No gameplay. All walkers
//! share one mesh and four materials so the renderer can batch them.

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

/// Shared handles plus the round-robin index for color and lane.
#[derive(Resource)]
pub struct WalkerAssets {
    mesh: Handle<Mesh>,
    materials: [Handle<StandardMaterial>; 4],
    next: usize,
}

/// How many walkers are alive. Read by the overlay.
#[derive(Resource, Default)]
pub struct WalkerCount(pub usize);

pub fn plugin(app: &mut App) {
    app.init_resource::<WalkerCount>()
        .add_systems(PreStartup, init_assets)
        .add_systems(Update, (spawn_on_input, move_walkers));
}

fn init_assets(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // Capsule3d::new(radius, segment length): total height 0.2 + 2 * 0.3 = 0.8.
    let mesh = meshes.add(Capsule3d::new(0.3, 0.2));
    let materials = COLORS.map(|c| {
        materials.add(StandardMaterial {
            base_color: Color::srgb_u8(c[0], c[1], c[2]),
            perceptual_roughness: 0.8,
            ..default()
        })
    });
    commands.insert_resource(WalkerAssets {
        mesh,
        materials,
        next: 0,
    });
}

fn spawn_on_input(
    mut commands: Commands,
    mouse: Res<ButtonInput<MouseButton>>,
    touches: Res<Touches>,
    mut assets: ResMut<WalkerAssets>,
    mut count: ResMut<WalkerCount>,
) {
    let spawns =
        usize::from(mouse.just_pressed(MouseButton::Left)) + touches.iter_just_released().count();
    for _ in 0..spawns {
        spawn_walker_at(&mut commands, &mut assets, &mut count, SPAWN_X);
    }
}

/// Spawn one walker at `x` (used by taps, `?stress=N`, and respawn).
pub fn spawn_walker_at(
    commands: &mut Commands,
    assets: &mut WalkerAssets,
    count: &mut WalkerCount,
    x: f32,
) {
    let i = assets.next;
    assets.next += 1;
    commands.spawn((
        Walker,
        Mesh3d(assets.mesh.clone()),
        MeshMaterial3d(assets.materials[i % COLORS.len()].clone()),
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
