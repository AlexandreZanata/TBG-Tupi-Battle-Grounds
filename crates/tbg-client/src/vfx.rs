//! Short-lived effects that make combat readable: splash rings, heal
//! sparkles, evolve bursts, synergy pulses, Stronghold sparks, death puffs.
//! Shared meshes/materials; each effect is a few entities that scale,
//! rise, and despawn. (Spawning is cheap at this unit count, so no pool.)

use bevy::prelude::*;

use crate::art::palette::*;

#[derive(Resource)]
pub struct VfxAssets {
    pub(super) ring: Handle<Mesh>,
    pub(super) ball: Handle<Mesh>,
    pub(super) gold: Handle<StandardMaterial>,
    pub(super) green: Handle<StandardMaterial>,
    pub(super) white: Handle<StandardMaterial>,
    pub(super) orange: Handle<StandardMaterial>,
    pub(super) grey: Handle<StandardMaterial>,
    pub(super) ally: Handle<StandardMaterial>,
    pub(super) enemy: Handle<StandardMaterial>,
}

#[derive(Clone, Copy)]
pub(super) enum Motion {
    /// Flat ring growing from 0 to `radius`.
    Grow { radius: f32 },
    /// Rises and shrinks.
    Rise,
    /// Flies outward along `dir` and shrinks.
    Burst { dir: Vec3 },
}

#[derive(Component)]
struct Vfx {
    motion: Motion,
    origin: Vec3,
    delay: f32,
    t: f32,
    dur: f32,
}

mod effects;
pub use effects::{evolve, heal, puff, sparks, splash, synergy};

pub fn plugin(app: &mut App) {
    app.add_systems(PreStartup, init)
        .add_systems(Update, animate);
}

fn init(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut mats: ResMut<Assets<StandardMaterial>>,
) {
    let mut glow = |c: Color| {
        mats.add(StandardMaterial {
            base_color: c,
            unlit: true,
            ..default()
        })
    };
    commands.insert_resource(VfxAssets {
        ring: meshes.add(
            Torus::new(0.9, 1.0)
                .mesh()
                .minor_resolution(4)
                .major_resolution(24),
        ),
        ball: meshes.add(
            Sphere::new(1.0)
                .mesh()
                .ico(0)
                .unwrap_or_else(|_| Sphere::new(1.0).into()),
        ),
        gold: glow(GOLD),
        green: glow(GLOW),
        white: glow(Color::WHITE),
        orange: glow(ORANGE),
        grey: glow(ROCK),
        ally: glow(ALLY),
        enemy: glow(ENEMY),
    });
}

pub(super) fn spawn(
    commands: &mut Commands,
    mesh: &Handle<Mesh>,
    mat: &Handle<StandardMaterial>,
    at: Vec3,
    motion: Motion,
    delay: f32,
    dur: f32,
) {
    commands.spawn((
        Mesh3d(mesh.clone()),
        MeshMaterial3d(mat.clone()),
        Transform::from_translation(at).with_scale(Vec3::ZERO),
        Vfx {
            motion,
            origin: at,
            delay,
            t: 0.0,
            dur,
        },
    ));
}

fn animate(
    mut commands: Commands,
    time: Res<Time>,
    mut q: Query<(Entity, &mut Vfx, &mut Transform)>,
) {
    let dt = time.delta_secs();
    for (e, mut v, mut t) in &mut q {
        if v.delay > 0.0 {
            v.delay -= dt;
            continue;
        }
        v.t += dt;
        let k = v.t / v.dur;
        if k >= 1.0 {
            commands.entity(e).despawn();
            continue;
        }
        let fade = 1.0 - k;
        match v.motion {
            Motion::Grow { radius } => {
                let r = radius * (0.2 + 0.8 * k);
                t.scale = Vec3::new(r, 0.3 * fade + 0.05, r);
            }
            Motion::Rise => {
                t.translation = v.origin + Vec3::Y * 0.6 * k;
                t.scale = Vec3::splat(0.05 * fade);
            }
            Motion::Burst { dir } => {
                t.translation = v.origin + dir * 0.8 * k;
                t.scale = Vec3::splat(0.07 * fade);
            }
        }
    }
}
