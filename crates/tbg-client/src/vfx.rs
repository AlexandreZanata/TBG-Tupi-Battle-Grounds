//! Short-lived effects that make combat readable: splash rings, heal
//! sparkles, evolve bursts, synergy pulses, Stronghold sparks, death puffs,
//! and the kit projectiles, muzzles and impacts (`shots`, `impacts`).
//! Shared meshes/materials; each effect is a few entities that scale,
//! move, and despawn. (Spawning is cheap at this unit count, so no pool.)

use bevy::{light::NotShadowCaster, prelude::*};

use crate::art::{
    mesh::{self, Shape},
    palette::*,
};

#[derive(Resource)]
pub struct VfxAssets {
    pub(super) ring: Handle<Mesh>,
    pub(super) ball: Handle<Mesh>,
    /// A smoother ball for smoke and dust that grows large.
    pub(super) puff: Handle<Mesh>,
    /// Unit cone along +Y with its base at the origin (root spikes, thorns).
    pub(super) spike: Handle<Mesh>,
    /// Unit cube (debris, chips).
    pub(super) chip: Handle<Mesh>,
    /// A flat half ring in the XZ plane (scythe and cleaver sweeps).
    pub(super) crescent: Handle<Mesh>,
    pub(super) gold: Handle<StandardMaterial>,
    pub(super) green: Handle<StandardMaterial>,
    pub(super) white: Handle<StandardMaterial>,
    pub(super) orange: Handle<StandardMaterial>,
    pub(super) grey: Handle<StandardMaterial>,
    pub(super) ally: Handle<StandardMaterial>,
    pub(super) enemy: Handle<StandardMaterial>,
    pub(super) smoke: Handle<StandardMaterial>,
    pub(super) soot: Handle<StandardMaterial>,
    pub(super) dust: Handle<StandardMaterial>,
    pub(super) fire: Handle<StandardMaterial>,
    pub(super) flash: Handle<StandardMaterial>,
    pub(super) bone: Handle<StandardMaterial>,
    pub(super) root: Handle<StandardMaterial>,
    pub(super) spore: Handle<StandardMaterial>,
    pub(super) rock: Handle<StandardMaterial>,
    pub(super) leaf: Handle<StandardMaterial>,
    pub(super) amber: Handle<StandardMaterial>,
    pub(super) violet: Handle<StandardMaterial>,
    pub(super) blood: Handle<StandardMaterial>,
    pub(super) water: Handle<StandardMaterial>,
    pub(super) turquoise: Handle<StandardMaterial>,
    pub(super) petal: Handle<StandardMaterial>,
    pub(super) oil: Handle<StandardMaterial>,
    /// Lit vertex colors, for painted projectile meshes.
    pub(super) painted: Handle<StandardMaterial>,
    pub(super) shots: shots::ShotMeshes,
}

#[derive(Clone, Copy)]
pub(super) enum Motion {
    /// Flat ring growing from 0 to `radius`.
    Grow { radius: f32 },
    /// Rises and shrinks.
    Rise,
    /// Flies outward along `dir` and shrinks.
    Burst { dir: Vec3 },
    /// Drifts with `vel` (slowing down), swells to `size`, then shrinks.
    Puff { vel: Vec3, size: f32 },
    /// Thrown with `vel` under gravity, tumbling, resting on the ground.
    Debris { vel: Vec3, size: f32 },
    /// Pops to `size` and vanishes (muzzle flashes, blast cores).
    Flash { size: f32 },
    /// Grows out of the ground to `size` (its own +Y), holds, sinks back.
    Sprout { size: Vec3, hold: f32 },
    /// Sweeps `turn` radians about +Y while fading (crescents).
    Sweep { size: f32, turn: f32 },
}

#[derive(Component)]
struct Vfx {
    motion: Motion,
    origin: Vec3,
    rotation: Quat,
    delay: f32,
    t: f32,
    dur: f32,
}

mod effects;
pub mod impacts;
pub mod shots;
pub use effects::{evolve, heal, puff, sparks, splash, synergy};

pub fn plugin(app: &mut App) {
    app.add_systems(PreStartup, init)
        .add_systems(Update, (animate, shots::fly));
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
    let (gold, green, white, orange) = (glow(GOLD), glow(GLOW), glow(Color::WHITE), glow(ORANGE));
    let (grey, ally, enemy) = (glow(ROCK), glow(ALLY), glow(ENEMY));
    let fire = glow(Color::srgb_u8(0xFF, 0x9A, 0x2E));
    let flash = glow(Color::srgb_u8(0xFF, 0xF1, 0xB8));
    let spore = glow(Color::srgb_u8(0xD8, 0xF0, 0x6A));
    let amber = glow(SAP);
    let violet = glow(Color::srgb_u8(0xA8, 0x7B, 0xFF));
    let blood = glow(Color::srgb_u8(0xC8, 0x1E, 0x32));
    let water = glow(Color::srgb_u8(0x8C, 0xD8, 0xFF));
    let petal = glow(BLOSSOM);
    let turquoise = glow(TURQUOISE);
    let mut lit = |c: Color| {
        mats.add(StandardMaterial {
            base_color: c,
            perceptual_roughness: 1.0,
            ..default()
        })
    };
    let smoke = lit(Color::srgb_u8(0xE4, 0xE0, 0xD8));
    let soot = lit(Color::srgb_u8(0x5E, 0x58, 0x52));
    let dust = lit(Color::srgb_u8(0xD2, 0xB8, 0x8C));
    let bone = lit(BONE);
    let root = lit(BARK);
    let rock = lit(ROCK);
    let leaf = lit(LEAF);
    let painted = lit(Color::WHITE);
    let oil = lit(Color::srgb_u8(0x24, 0x20, 0x28));
    let shots = shots::ShotMeshes::new(&mut meshes);
    let ball = |level: u32| {
        Sphere::new(1.0)
            .mesh()
            .ico(level)
            .unwrap_or_else(|_| Sphere::new(1.0).into())
    };
    commands.insert_resource(VfxAssets {
        ring: meshes.add(
            Torus::new(0.9, 1.0)
                .mesh()
                .minor_resolution(4)
                .major_resolution(24),
        ),
        ball: meshes.add(ball(0)),
        puff: meshes.add(ball(1)),
        spike: meshes.add(mesh::build(Shape::Cone(0.5, 1.0)).translated_by(Vec3::Y * 0.5)),
        chip: meshes.add(Cuboid::new(1.0, 1.0, 1.0)),
        crescent: meshes.add(
            Torus::new(0.82, 1.0)
                .mesh()
                .minor_resolution(3)
                .major_resolution(16)
                .angle_range(0.0..=std::f32::consts::PI),
        ),
        gold,
        green,
        white,
        orange,
        grey,
        ally,
        enemy,
        smoke,
        soot,
        dust,
        fire,
        flash,
        bone,
        root,
        spore,
        rock,
        leaf,
        amber,
        violet,
        blood,
        water,
        petal,
        turquoise,
        oil,
        painted,
        shots,
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
    spawn_turned(
        commands,
        mesh,
        mat,
        (at, Quat::IDENTITY),
        motion,
        (delay, dur),
    );
}

pub(super) fn spawn_turned(
    commands: &mut Commands,
    mesh: &Handle<Mesh>,
    mat: &Handle<StandardMaterial>,
    (at, rotation): (Vec3, Quat),
    motion: Motion,
    (delay, dur): (f32, f32),
) {
    commands.spawn((
        Mesh3d(mesh.clone()),
        MeshMaterial3d(mat.clone()),
        Transform::from_translation(at)
            .with_rotation(rotation)
            .with_scale(Vec3::ZERO),
        Vfx {
            motion,
            origin: at,
            rotation,
            delay,
            t: 0.0,
            dur,
        },
        NotShadowCaster,
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
        *t = pose(&v, k);
    }
}

/// Where an effect is `k` (0..1) through its life.
fn pose(v: &Vfx, k: f32) -> Transform {
    let fade = 1.0 - k;
    let at = Transform::from_translation(v.origin).with_rotation(v.rotation);
    match v.motion {
        Motion::Grow { radius } => {
            let r = radius * (0.2 + 0.8 * k);
            at.with_scale(Vec3::new(r, 0.3 * fade + 0.05, r))
        }
        Motion::Rise => at
            .with_translation(v.origin + Vec3::Y * 0.6 * k)
            .with_scale(Vec3::splat(0.05 * fade)),
        Motion::Burst { dir } => at
            .with_translation(v.origin + dir * 0.8 * k)
            .with_scale(Vec3::splat(0.07 * fade)),
        Motion::Puff { vel, size } => {
            // Ease-out drift; swell over the first fifth, shrink after.
            let travel = 1.0 - (1.0 - k).powi(2);
            let swell = (k / 0.2).min(1.0) * (1.0 - ((k - 0.4) / 0.6).max(0.0).powi(2));
            at.with_translation(v.origin + vel * travel * v.dur * 0.5)
                .with_scale(Vec3::splat(size * (0.35 + 0.65 * swell)))
        }
        Motion::Debris { vel, size } => {
            let s = k * v.dur;
            let mut p = v.origin + vel * s - Vec3::Y * 4.9 * s * s;
            p.y = p.y.max(0.03);
            let spin = Quat::from_scaled_axis(vel.cross(Vec3::Y).normalize_or_zero() * s * 9.0);
            at.with_translation(p)
                .with_rotation(spin * v.rotation)
                .with_scale(Vec3::splat(size * (fade * 2.0).min(1.0)))
        }
        Motion::Flash { size } => {
            let pop = (k / 0.25).min(1.0);
            at.with_scale(Vec3::splat(
                size * pop * (1.0 - ((k - 0.25) / 0.75).max(0.0)),
            ))
        }
        Motion::Sprout { size, hold } => {
            let life = k * v.dur;
            let rise = (life / 0.09).min(1.0);
            let sink = ((life - 0.09 - hold) / 0.2).clamp(0.0, 1.0);
            let h = (1.0 - (1.0 - rise).powi(3)) * (1.0 - sink * sink);
            at.with_scale(Vec3::new(
                size.x * (0.6 + 0.4 * h),
                size.y * h,
                size.z * (0.6 + 0.4 * h),
            ))
        }
        Motion::Sweep { size, turn } => {
            let swing = 1.0 - (1.0 - k).powi(3);
            at.with_rotation(Quat::from_rotation_y(turn * swing) * v.rotation)
                .with_scale(Vec3::new(size, size * (0.3 + fade), size))
        }
    }
}
