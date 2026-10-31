//! Styled projectiles: each kit and Stronghold fires its own shot along a
//! parabola from its muzzle to its target in a fixed time, nose along the
//! flight tangent, leaving a trail; it lands as its own impact. Cosmetic
//! only — the simulation already decided the hit.

use bevy::prelude::*;

use super::impacts::{self, Area, Impact};
use super::{spawn, spawn_turned, Motion, VfxAssets};
use crate::art::kits::aim::{flight, tangent};
use crate::art::{
    mesh::{self, Shape::*},
    palette::*,
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ShotKind {
    /// Siege Walker: base, Longshot, Artillery, Shrapnel, Concussion.
    Shell,
    LongShell,
    GreatShell,
    Shrapnel,
    Hammer,
    /// Rootcaller: a root ridge running along the ground.
    Roots,
    Strangle,
    Spores,
    /// Strongholds.
    Cannon,
    GreatCannon,
    RapidCannon,
    Ballista,
    Crossbow,
    /// Treasury: a spinning purse of gold lobbed high.
    Coins,
    /// Kit champions (P55).
    Arrow,
    /// Archer Arrow Rain (B2): smaller, faster arrows trailing sparks.
    ArrowStorm,
    Glob,
    Fish,
    Bomb,
    Hex,
    Petal,
    Seed,
    Flask,
    Blood,
    Puppet,
    Acid,
    Thought,
    PlagueBolt,
    Bullet,
    CrossBullet,
    SunSpark,
    CelestialArrow,
    PurifyingArrow,
}

#[derive(Clone, Copy)]
enum Body {
    Shell,
    LongShell,
    GreatShell,
    Shrapnel,
    Hammer,
    Ball,
    Bolt,
    Purse,
    Arrow,
    Glob,
    Fish,
    Bomb,
    Orb,
    Petal,
    Seed,
    Flask,
    Acid,
    RitualBolt,
    SunMote,
    RadiantArrow,
}

#[derive(Clone, Copy)]
enum Trail {
    /// Small glowing motes of one effect material.
    Motes(Mote),
    Smoke {
        every: f32,
        size: f32,
    },
    Sparks,
    Ridge {
        every: f32,
        size: f32,
    },
}

/// How a kind looks and flies.
struct Look {
    body: Option<(Body, f32)>,
    /// Apex height as a fraction of the flight length.
    arc: f32,
    secs: f32,
    trail: Trail,
    /// Spin about the nose, radians per second.
    roll: f32,
    impact: Impact,
}

/// The material of a mote trail.
#[derive(Clone, Copy)]
enum Mote {
    Spore,
    Water,
    Violet,
    Petal,
    Blood,
    Turquoise,
}

/// Seconds a unit shot flies: Release at 0.18 s plus this lands at the
/// simulation's Hit (0.30 s, ART-DIRECTION §4).
pub const UNIT_FLIGHT: f32 = 0.12;
/// A cast's root ridge: Release at 0.22 s plus this lands at 0.30 s.
pub const CAST_FLIGHT: f32 = 0.08;

fn look(kind: ShotKind) -> Look {
    use ShotKind::*;
    let smoke = |every, size| Trail::Smoke { every, size };
    let (body, arc, secs, trail, roll, impact) = match kind {
        Shell => (
            Some((Body::Shell, 1.0)),
            0.06,
            UNIT_FLIGHT,
            smoke(0.014, 0.1),
            0.0,
            Impact::Blast,
        ),
        LongShell => (
            Some((Body::LongShell, 1.0)),
            0.04,
            UNIT_FLIGHT,
            smoke(0.012, 0.09),
            0.0,
            Impact::Blast,
        ),
        GreatShell => (
            Some((Body::GreatShell, 1.0)),
            0.08,
            UNIT_FLIGHT,
            smoke(0.011, 0.14),
            0.0,
            Impact::GreatBlast,
        ),
        Shrapnel => (
            Some((Body::Shrapnel, 1.0)),
            0.07,
            UNIT_FLIGHT,
            smoke(0.014, 0.11),
            18.0,
            Impact::Shrapnel,
        ),
        Hammer => (
            Some((Body::Hammer, 1.0)),
            0.08,
            UNIT_FLIGHT,
            smoke(0.012, 0.13),
            0.0,
            Impact::Shockwave,
        ),
        Roots => (
            None,
            0.0,
            CAST_FLIGHT,
            Trail::Ridge {
                every: 0.012,
                size: 0.8,
            },
            0.0,
            Impact::Roots,
        ),
        Strangle => (
            None,
            0.0,
            CAST_FLIGHT,
            Trail::Ridge {
                every: 0.01,
                size: 1.1,
            },
            0.0,
            Impact::Strangle,
        ),
        Spores => (
            None,
            0.0,
            CAST_FLIGHT,
            Trail::Ridge {
                every: 0.012,
                size: 0.8,
            },
            0.0,
            Impact::Spores,
        ),
        Cannon => (
            Some((Body::Ball, 1.0)),
            0.14,
            0.3,
            smoke(0.03, 0.13),
            0.0,
            Impact::Cannon,
        ),
        GreatCannon => (
            Some((Body::Ball, 1.5)),
            0.16,
            0.34,
            smoke(0.026, 0.18),
            0.0,
            Impact::GreatCannon,
        ),
        RapidCannon => (
            Some((Body::Ball, 0.8)),
            0.1,
            0.24,
            smoke(0.04, 0.1),
            0.0,
            Impact::Cannon,
        ),
        Ballista => (
            Some((Body::Bolt, 1.6)),
            0.04,
            0.22,
            Trail::Sparks,
            0.0,
            Impact::Bolt,
        ),
        Crossbow => (
            Some((Body::Bolt, 1.0)),
            0.06,
            0.22,
            Trail::Sparks,
            0.0,
            Impact::Bolt,
        ),
        Coins => (
            Some((Body::Purse, 1.0)),
            0.28,
            0.42,
            Trail::Sparks,
            14.0,
            Impact::Coins,
        ),
        Arrow => (
            Some((Body::Arrow, 1.0)),
            0.03,
            UNIT_FLIGHT,
            Trail::Sparks,
            0.0,
            Impact::Bolt,
        ),
        ArrowStorm => (
            Some((Body::Arrow, 0.72)),
            0.03,
            UNIT_FLIGHT * 0.8,
            Trail::Sparks,
            0.0,
            Impact::Bolt,
        ),
        CelestialArrow | PurifyingArrow => (
            Some((
                Body::RadiantArrow,
                if kind == PurifyingArrow { 1.3 } else { 1.0 },
            )),
            0.03,
            UNIT_FLIGHT,
            Trail::Sparks,
            0.0,
            Impact::Sparks,
        ),
        SunSpark => (
            Some((Body::SunMote, 1.0)),
            0.04,
            CAST_FLIGHT,
            Trail::Sparks,
            1.0,
            Impact::Harmonic,
        ),
        Bullet | CrossBullet => (
            Some((Body::Bolt, 0.24)),
            0.0,
            if kind == CrossBullet {
                0.04
            } else {
                UNIT_FLIGHT
            },
            Trail::Sparks,
            0.0,
            Impact::Sparks,
        ),
        PlagueBolt => (
            Some((Body::RitualBolt, 1.0)),
            0.03,
            UNIT_FLIGHT,
            Trail::Motes(Mote::Spore),
            0.0,
            Impact::Plague,
        ),
        Thought => (
            Some((Body::Orb, 0.8)),
            0.04,
            CAST_FLIGHT,
            Trail::Motes(Mote::Turquoise),
            0.0,
            Impact::Mind,
        ),
        Acid => (
            Some((Body::Acid, 1.0)),
            0.06,
            UNIT_FLIGHT,
            Trail::Motes(Mote::Turquoise),
            2.0,
            Impact::Acid,
        ),
        Glob => (
            Some((Body::Glob, 1.0)),
            0.12,
            UNIT_FLIGHT,
            Trail::Motes(Mote::Spore),
            8.0,
            Impact::Poison,
        ),
        Fish => (
            Some((Body::Fish, 1.0)),
            0.35,
            CAST_FLIGHT,
            Trail::Motes(Mote::Water),
            22.0,
            Impact::Splash,
        ),
        Bomb => (
            Some((Body::Bomb, 1.0)),
            0.25,
            UNIT_FLIGHT,
            smoke(0.016, 0.09),
            6.0,
            Impact::Blast,
        ),
        Hex => (
            Some((Body::Orb, 0.9)),
            0.05,
            CAST_FLIGHT,
            Trail::Motes(Mote::Violet),
            0.0,
            Impact::Hex,
        ),
        Petal => (
            Some((Body::Petal, 1.0)),
            0.12,
            CAST_FLIGHT,
            Trail::Motes(Mote::Petal),
            10.0,
            Impact::Bloom,
        ),
        Seed => (
            Some((Body::Seed, 1.0)),
            0.2,
            CAST_FLIGHT,
            Trail::Motes(Mote::Spore),
            12.0,
            Impact::Seed,
        ),
        Flask => (
            Some((Body::Flask, 1.0)),
            0.3,
            0.32,
            Trail::Motes(Mote::Spore),
            14.0,
            Impact::Cleanse,
        ),
        Blood => (
            Some((Body::Orb, 1.0)),
            0.15,
            0.3,
            Trail::Motes(Mote::Blood),
            0.0,
            Impact::Blood,
        ),
        Puppet => (
            Some((Body::Orb, 0.8)),
            0.1,
            0.26,
            Trail::Motes(Mote::Violet),
            0.0,
            Impact::Puppet,
        ),
    };
    Look {
        body,
        arc,
        secs,
        trail,
        roll,
        impact,
    }
}

/// The arc a kind flies, so its launcher can aim along the same curve.
pub fn arc(kind: ShotKind) -> f32 {
    look(kind).arc
}

/// Shared projectile meshes, built along +Z (nose forward) once.
pub struct ShotMeshes {
    shell: Handle<Mesh>,
    long_shell: Handle<Mesh>,
    great_shell: Handle<Mesh>,
    shrapnel: Handle<Mesh>,
    hammer: Handle<Mesh>,
    ball: Handle<Mesh>,
    bolt: Handle<Mesh>,
    purse: Handle<Mesh>,
    arrow: Handle<Mesh>,
    glob: Handle<Mesh>,
    acid: Handle<Mesh>,
    ritual_bolt: Handle<Mesh>,
    sun_mote: Handle<Mesh>,
    radiant_arrow: Handle<Mesh>,
    fish: Handle<Mesh>,
    bomb: Handle<Mesh>,
    orb: Handle<Mesh>,
    petal: Handle<Mesh>,
    seed: Handle<Mesh>,
    flask: Handle<Mesh>,
}

impl ShotMeshes {
    pub fn new(meshes: &mut Assets<Mesh>) -> Self {
        let z = Quat::from_rotation_x(std::f32::consts::FRAC_PI_2);
        let along = |z0: f32| Transform::from_xyz(0.0, 0.0, z0).with_rotation(z);
        let mut build = |parts: Vec<(crate::art::mesh::Shape, Color, Transform)>| {
            let parts = parts
                .into_iter()
                .map(|(s, c, t)| (mesh::paint(mesh::build(s), c), t))
                .collect();
            meshes.add(mesh::merge(parts).unwrap_or_else(|| Cuboid::new(0.05, 0.05, 0.05).into()))
        };
        let shell = |r: f32, len: f32, band: Color| {
            vec![
                (Cyl(r, len), GUNMETAL, along(0.0)),
                (Cone(r, len * 0.55), IRON, along(len * 0.77)),
                (Cyl(r * 1.08, len * 0.18), band, along(len * 0.2)),
                (Cyl(r * 0.6, 0.04), ORANGE, along(-len * 0.52)),
                (
                    Box(r * 2.6, 0.012, len * 0.3),
                    IRON,
                    Transform::from_xyz(0.0, 0.0, -len * 0.4),
                ),
                (
                    Box(0.012, r * 2.6, len * 0.3),
                    IRON,
                    Transform::from_xyz(0.0, 0.0, -len * 0.4),
                ),
            ]
        };
        let mut shrapnel = shell(0.08, 0.24, RUST);
        for i in 0..6 {
            let a = i as f32 / 6.0 * std::f32::consts::TAU;
            shrapnel.push((
                Cone(0.025, 0.07),
                STEEL,
                Transform::from_xyz(a.cos() * 0.085, a.sin() * 0.085, 0.02)
                    .with_rotation(Quat::from_rotation_z(a - std::f32::consts::FRAC_PI_2)),
            ));
        }
        Self {
            shell: build(shell(0.07, 0.24, BRASS)),
            long_shell: build(shell(0.055, 0.34, BRASS)),
            great_shell: build(shell(0.1, 0.32, GOLD)),
            shrapnel: build(shrapnel),
            hammer: build(vec![
                (
                    Box(0.2, 0.14, 0.16),
                    IRON,
                    Transform::from_xyz(0.0, 0.0, 0.1),
                ),
                (
                    Box(0.22, 0.04, 0.17),
                    ORANGE,
                    Transform::from_xyz(0.0, 0.0, 0.1),
                ),
                (Cyl(0.04, 0.24), WOOD, along(-0.08)),
                (Cyl(0.05, 0.04), ORANGE, along(-0.2)),
            ]),
            ball: build(vec![
                (Ball(0.1), DARK, Transform::IDENTITY),
                (Ball(0.03), IRON, Transform::from_xyz(0.05, 0.05, 0.05)),
            ]),
            bolt: build(vec![
                (Cyl(0.014, 0.42), TIMBER, along(0.0)),
                (Cone(0.035, 0.09), STEEL, along(0.25)),
                (
                    Box(0.004, 0.06, 0.08),
                    LINEN,
                    Transform::from_xyz(0.0, 0.0, -0.18),
                ),
                (
                    Box(0.06, 0.004, 0.08),
                    LINEN,
                    Transform::from_xyz(0.0, 0.0, -0.18),
                ),
            ]),
            purse: build(vec![
                (Ball(0.08), LEATHER, Transform::IDENTITY),
                (Cyl(0.035, 0.05), GOLD, along(0.07)),
                (Cyl(0.06, 0.012), GOLD, Transform::from_xyz(0.0, 0.07, 0.0)),
                (Ball(0.025), GOLD, Transform::from_xyz(0.05, 0.05, 0.0)),
            ]),
            arrow: build(vec![
                (Cyl(0.012, 0.6), TIMBER, along(0.0)),
                (Cone(0.03, 0.09), STEEL, along(0.34)),
                (
                    Box(0.004, 0.06, 0.1),
                    LINEN,
                    Transform::from_xyz(0.0, 0.0, -0.26),
                ),
                (
                    Box(0.06, 0.004, 0.1),
                    LINEN,
                    Transform::from_xyz(0.0, 0.0, -0.26),
                ),
            ]),
            radiant_arrow: build(vec![
                (Cyl(0.007, 0.40), IVORY, along(0.0)),
                (Cone(0.028, 0.08), GOLD, along(0.23)),
                (
                    Box(0.07, 0.012, 0.07),
                    NAVY,
                    Transform::from_xyz(0.0, 0.0, -0.17),
                ),
            ]),
            sun_mote: build(vec![
                (Ball(0.04), IVORY, Transform::IDENTITY),
                (Ring(0.009, 0.06, 360.0), GOLD, Transform::IDENTITY),
            ]),
            ritual_bolt: build(vec![
                (Cyl(0.012, 0.28), DARK_LEATHER, along(0.0)),
                (Cone(0.032, 0.09), BONE, along(0.18)),
                (Ball(0.027), GLOW, Transform::from_xyz(0.0, 0.0, 0.10)),
            ]),
            acid: build(vec![
                (
                    Ball(0.07),
                    TURQUOISE,
                    Transform::from_scale(Vec3::new(0.8, 0.8, 1.5)),
                ),
                (Ball(0.035), IVORY, Transform::from_xyz(0.0, 0.025, 0.06)),
            ]),
            glob: build(vec![
                (
                    Ball(0.075),
                    Color::srgb(0.55, 0.85, 0.2),
                    Transform::IDENTITY,
                ),
                (
                    Ball(0.04),
                    Color::srgb(0.75, 0.95, 0.35),
                    Transform::from_xyz(0.03, 0.04, -0.06),
                ),
                (Ball(0.03), MUSHROOM, Transform::from_xyz(-0.04, 0.03, 0.02)),
            ]),
            fish: build(vec![
                (
                    Ball(0.07),
                    Color::srgb(0.55, 0.65, 0.72),
                    Transform::from_scale(Vec3::new(0.7, 0.9, 1.8)),
                ),
                (
                    Cone(0.06, 0.1),
                    Color::srgb(0.45, 0.55, 0.65),
                    along(-0.16).with_scale(Vec3::new(1.0, 1.0, 0.25)),
                ),
                (Ball(0.015), DARK, Transform::from_xyz(0.035, 0.03, 0.08)),
                (
                    Box(0.01, 0.05, 0.08),
                    Color::srgb(0.85, 0.55, 0.45),
                    Transform::from_xyz(0.0, 0.07, 0.0),
                ),
            ]),
            bomb: build(vec![
                (Ball(0.1), DARK, Transform::IDENTITY),
                (Cyl(0.03, 0.05), IRON, Transform::from_xyz(0.0, 0.1, 0.0)),
                (Cyl(0.008, 0.08), LINEN, Transform::from_xyz(0.0, 0.16, 0.0)),
                (Ball(0.025), ORANGE, Transform::from_xyz(0.0, 0.2, 0.0)),
            ]),
            orb: build(vec![
                (Ball(0.07), Color::srgb(0.95, 0.9, 1.0), Transform::IDENTITY),
                (
                    Ring(0.012, 0.09, 360.0),
                    Color::srgb(0.6, 0.45, 0.9),
                    Transform::IDENTITY,
                ),
            ]),
            petal: build(vec![
                (Ball(0.03), GOLD, Transform::IDENTITY),
                (Ball(0.045), BLOSSOM, Transform::from_xyz(0.05, 0.0, 0.0)),
                (Ball(0.045), BLOSSOM, Transform::from_xyz(-0.05, 0.0, 0.0)),
                (Ball(0.045), BLOSSOM, Transform::from_xyz(0.0, 0.05, 0.0)),
                (Ball(0.045), BLOSSOM, Transform::from_xyz(0.0, -0.05, 0.0)),
            ]),
            seed: build(vec![
                (
                    Ball(0.05),
                    BARK,
                    Transform::from_scale(Vec3::new(0.8, 0.8, 1.3)),
                ),
                (
                    Cone(0.04, 0.08),
                    LEAF,
                    Transform::from_xyz(0.0, 0.06, 0.0).with_scale(Vec3::new(1.0, 1.0, 0.3)),
                ),
            ]),
            flask: build(vec![
                (Ball(0.06), Color::srgb(0.6, 0.95, 0.5), Transform::IDENTITY),
                (Cyl(0.022, 0.06), LINEN, Transform::from_xyz(0.0, 0.07, 0.0)),
                (Cyl(0.026, 0.025), WOOD, Transform::from_xyz(0.0, 0.1, 0.0)),
            ]),
        }
    }
    fn get(&self, body: Body) -> &Handle<Mesh> {
        match body {
            Body::Shell => &self.shell,
            Body::LongShell => &self.long_shell,
            Body::GreatShell => &self.great_shell,
            Body::Shrapnel => &self.shrapnel,
            Body::Hammer => &self.hammer,
            Body::Ball => &self.ball,
            Body::Bolt => &self.bolt,
            Body::Purse => &self.purse,
            Body::Arrow => &self.arrow,
            Body::Glob => &self.glob,
            Body::Acid => &self.acid,
            Body::RitualBolt => &self.ritual_bolt,
            Body::SunMote => &self.sun_mote,
            Body::RadiantArrow => &self.radiant_arrow,
            Body::Fish => &self.fish,
            Body::Bomb => &self.bomb,
            Body::Orb => &self.orb,
            Body::Petal => &self.petal,
            Body::Seed => &self.seed,
            Body::Flask => &self.flask,
        }
    }
}

#[derive(Component)]
pub struct Shot {
    kind: ShotKind,
    from: Vec3,
    to: Vec3,
    t: f32,
    age: f32,
    trail: f32,
    area: Area,
}

/// Fire `kind` from `from` to `to`; it lands as its own impact.
pub fn fire(
    commands: &mut Commands,
    a: &VfxAssets,
    kind: ShotKind,
    from: Vec3,
    to: Vec3,
    area: Area,
) {
    let l = look(kind);
    let shot = Shot {
        kind,
        from,
        to,
        t: 0.0,
        age: 0.0,
        trail: 0.0,
        area,
    };
    let at = Transform::from_translation(from);
    match l.body {
        Some((body, size)) => commands.spawn((
            Mesh3d(a.shots.get(body).clone()),
            MeshMaterial3d(a.painted.clone()),
            at.with_scale(Vec3::splat(size)),
            shot,
            bevy::light::NotShadowCaster,
        )),
        None => commands.spawn((at, shot)),
    };
}

pub(super) fn fly(
    mut commands: Commands,
    time: Res<Time>,
    a: Res<VfxAssets>,
    mut shots: Query<(Entity, &mut Shot, &mut Transform)>,
) {
    let dt = time.delta_secs();
    for (e, mut s, mut t) in &mut shots {
        let l = look(s.kind);
        s.age += dt;
        s.t = (s.t + dt / l.secs).min(1.0);
        let ground = l.body.is_none();
        let at = if ground {
            s.from.lerp(s.to, s.t).with_y(0.02)
        } else {
            flight(s.from, s.to, l.arc, s.t)
        };
        let dir = tangent(s.from, s.to, l.arc, s.t);
        t.translation = at;
        t.rotation = Quat::from_rotation_arc(Vec3::Z, dir) * Quat::from_rotation_z(l.roll * s.age);
        s.trail += dt;
        let every = match l.trail {
            Trail::Smoke { every, .. } | Trail::Ridge { every, .. } => every,
            Trail::Sparks => 0.03,
            Trail::Motes(_) => 0.02,
        };
        while s.trail >= every {
            s.trail -= every;
            trail(&mut commands, &a, l.trail, at, dir, s.age);
        }
        if s.t >= 1.0 {
            impacts::land(&mut commands, &a, l.impact, s.to, s.area);
            commands.entity(e).despawn();
        }
    }
}

fn trail(commands: &mut Commands, a: &VfxAssets, trail: Trail, at: Vec3, dir: Vec3, age: f32) {
    // A cheap deterministic jitter from the shot's age.
    let j = Vec3::new((age * 91.0).sin(), (age * 57.0).cos(), (age * 73.0).sin());
    match trail {
        Trail::Motes(mote) => {
            let mat = match mote {
                Mote::Turquoise => &a.turquoise,
                Mote::Spore => &a.spore,
                Mote::Water => &a.water,
                Mote::Violet => &a.violet,
                Mote::Petal => &a.petal,
                Mote::Blood => &a.blood,
            };
            let drift = Vec3::new(j.x * 0.2, 0.25 + 0.1 * j.y, j.z * 0.2);
            spawn(
                commands,
                &a.ball,
                mat,
                at,
                Motion::Puff {
                    vel: drift,
                    size: 0.035,
                },
                0.0,
                0.4,
            );
        }
        Trail::Smoke { size, .. } => {
            spawn(commands, &a.puff, &a.smoke, at, smoke(j, size), 0.0, 0.55);
            if (age * 100.0) as i32 % 3 == 0 {
                spawn(
                    commands,
                    &a.puff,
                    &a.fire,
                    at,
                    Motion::Flash { size: size * 0.7 },
                    0.0,
                    0.08,
                );
            }
        }
        Trail::Sparks => spawn(
            commands,
            &a.ball,
            &a.gold,
            at,
            Motion::Burst {
                dir: -dir * 0.15 + j * 0.05,
            },
            0.0,
            0.15,
        ),
        Trail::Ridge { size, .. } => {
            let lean = Quat::from_rotation_arc(
                Vec3::Y,
                (Vec3::Y + dir.with_y(0.0) * 0.6 + j * 0.25).normalize(),
            );
            spawn_turned(
                commands,
                &a.spike,
                &a.root,
                (at.with_y(0.0), lean),
                Motion::Sprout {
                    size: Vec3::new(0.12, 0.34, 0.12) * size * (0.8 + 0.3 * j.x.abs()),
                    hold: 0.05,
                },
                (0.0, 0.34),
            );
            spawn(
                commands,
                &a.puff,
                &a.dust,
                at.with_y(0.04),
                Motion::Puff {
                    vel: j.with_y(0.6) * 0.3,
                    size: 0.06 * size,
                },
                0.0,
                0.35,
            );
        }
    }
}

fn smoke(j: Vec3, size: f32) -> Motion {
    Motion::Puff {
        vel: Vec3::new(j.x * 0.15, 0.35 + j.y.abs() * 0.15, j.z * 0.15),
        size: size * (1.0 + 0.25 * j.x),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn unit_shots_land_on_the_simulation_hit() {
        // ART-DIRECTION §4: Release 0.18 s / cast 0.22 s, Hit at 0.30 s.
        for kind in [
            ShotKind::Shell,
            ShotKind::LongShell,
            ShotKind::GreatShell,
            ShotKind::Shrapnel,
            ShotKind::Hammer,
        ] {
            assert!((0.18 + look(kind).secs - 0.30).abs() < 1e-6, "{kind:?}");
        }
        for kind in [ShotKind::Roots, ShotKind::Strangle, ShotKind::Spores] {
            assert!((0.22 + look(kind).secs - 0.30).abs() < 1e-6, "{kind:?}");
        }
    }
}
