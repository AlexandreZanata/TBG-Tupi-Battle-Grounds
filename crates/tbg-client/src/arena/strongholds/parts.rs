//! Modular tower parts: named part definitions, their placement as pieces,
//! and the mesh cache that builds each part once per team color.

use std::collections::HashMap;
use std::f32::consts::FRAC_PI_2;

use crate::arena::{painted, Part};
use crate::art::{mesh::Shape::*, palette::*};
use bevy::prelude::*;

/// Damage level at which a piece disappears: 1 hides at <= 66 % HP, 2 at
/// <= 33 %; `NEVER` keeps the kind readable on a nearly dead tower.
pub(super) const NEVER: u8 = 3;

/// One reusable part: its primitives are built for a team color on demand.
/// The name is the cache key, so it must be unique.
pub(super) struct PartDef {
    pub name: &'static str,
    pub build: fn(Color) -> Vec<Part>,
}

/// How a piece moves after it is placed.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(super) enum Motion {
    /// The gun barrel: aimed (yaw and elevation about its own origin) and
    /// sliding back along its axis by this distance on each shot.
    Kick(f32),
    /// Turns with the gun's yaw about the tower's vertical axis (carriage).
    Swivel,
    /// Aimed whole about its own origin (the ballista).
    Aim,
    /// Turns around its local Y by this angle on each shot.
    Notch(f32),
    /// Turns around its local Y at this speed (rad/s), all the time.
    Turn(f32),
}

/// How a piece follows the side its tower stands on.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum Facing {
    /// The same on both towers.
    Fixed,
    /// Points at the enemy (+X on the left-hand tower, -X on the right).
    Aimed,
    /// Swaps front-left for front-right but stays on the camera side.
    Mirrored,
}

/// A part placed on a tower.
pub(super) struct Piece {
    pub def: &'static PartDef,
    pub at: Transform,
    /// Damage level at which it disappears (see `NEVER`).
    pub lost: u8,
    pub facing: Facing,
    pub motion: Option<Motion>,
}

pub(super) fn piece(def: &'static PartDef, at: Transform) -> Piece {
    Piece {
        def,
        at,
        lost: NEVER,
        facing: Facing::Fixed,
        motion: None,
    }
}

/// A piece built around the tower origin.
pub(super) fn whole(def: &'static PartDef) -> Piece {
    piece(def, Transform::IDENTITY)
}

impl Piece {
    pub fn lost(mut self, level: u8) -> Self {
        self.lost = level;
        self
    }
    pub fn aimed(mut self) -> Self {
        self.facing = Facing::Aimed;
        self
    }
    pub fn mirrored(mut self) -> Self {
        self.facing = Facing::Mirrored;
        self
    }
    pub fn moving(mut self, motion: Motion) -> Self {
        self.motion = Some(motion);
        self
    }
    /// Where it sits on the tower that faces +X (`own`) or -X.
    pub fn placed(&self, own: bool) -> Transform {
        let at = self.at;
        match self.facing {
            _ if own => at,
            Facing::Fixed => at,
            // Turned around its own vertical axis, its origin moved to the
            // other side of x = 0: it keeps its depth, so it stays on the
            // camera side.
            Facing::Aimed => {
                let p = at.translation;
                let half_turn = Quat::from_rotation_y(std::f32::consts::PI);
                at.with_translation(Vec3::new(-p.x, p.y, p.z))
                    .with_rotation(half_turn * at.rotation)
            }
            // Reflect across x = 0; only for parts symmetric across their own
            // X, since the mesh itself is not mirrored.
            Facing::Mirrored => {
                let (q, p) = (at.rotation, at.translation);
                at.with_translation(Vec3::new(-p.x, p.y, p.z))
                    .with_rotation(Quat::from_xyzw(q.x, -q.y, -q.z, q.w))
            }
        }
    }
}

/// Fortification progress on one branch, the five forms every kind has.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum Form {
    Base,
    A1,
    A2,
    B1,
    B2,
}

impl Form {
    pub fn of(tier: u8, branch: u8) -> Form {
        match (tier.min(2), branch) {
            (0, _) => Form::Base,
            (1, 0) => Form::A1,
            (_, 0) => Form::A2,
            (1, _) => Form::B1,
            _ => Form::B2,
        }
    }
    pub fn tier(self) -> u8 {
        match self {
            Form::Base => 0,
            Form::A1 | Form::B1 => 1,
            Form::A2 | Form::B2 => 2,
        }
    }
    pub fn a(self) -> bool {
        matches!(self, Form::A1 | Form::A2)
    }
    pub fn b(self) -> bool {
        matches!(self, Form::B1 | Form::B2)
    }
}

pub(super) fn t(x: f32, y: f32, z: f32) -> Transform {
    Transform::from_xyz(x, y, z)
}

/// A Y-axis primitive (cylinder, cone) lying along +X.
pub(super) fn along_x(x: f32, y: f32, z: f32) -> Transform {
    t(x, y, z).with_rotation(Quat::from_rotation_z(-FRAC_PI_2))
}

/// A Y-axis primitive standing up toward +Z: a disc facing the camera.
pub(super) fn along_z(x: f32, y: f32, z: f32) -> Transform {
    t(x, y, z).with_rotation(Quat::from_rotation_x(FRAC_PI_2))
}

/// `n` points on a circle of radius `r` at height `y`.
pub(super) fn around(n: usize, r: f32, y: f32) -> impl Iterator<Item = Transform> {
    (0..n).map(move |i| {
        let a = i as f32 / n as f32 * std::f32::consts::TAU;
        t(a.cos() * r, y, a.sin() * r).with_rotation(Quat::from_rotation_y(-a))
    })
}

/// A toothed wheel around its local Y axis, centered at the origin.
pub(super) fn gear(r: f32, color: Color) -> Vec<Part> {
    let mut parts = vec![
        (Cyl(r, 0.05), color, Transform::IDENTITY),
        (Cyl(r * 0.3, 0.08), DARK, Transform::IDENTITY),
    ];
    for at in around(8, r, 0.0) {
        parts.push((Box(0.07, 0.05, 0.07), color, at));
    }
    parts
}

/// Meshes and materials every tower shares: each part is built once per
/// team color and reused by every tower and every form.
#[derive(Resource)]
pub(crate) struct StrongholdArt {
    pub material: Handle<StandardMaterial>,
    pub ring_material: Handle<StandardMaterial>,
    meshes: HashMap<(&'static str, bool), Handle<Mesh>>,
    rings: HashMap<(i32, bool), Handle<Mesh>>,
}

impl FromWorld for StrongholdArt {
    fn from_world(world: &mut World) -> Self {
        let mut mats = world.resource_mut::<Assets<StandardMaterial>>();
        let material = mats.add(StandardMaterial {
            base_color: Color::WHITE,
            perceptual_roughness: 1.0,
            ..default()
        });
        // Faint and unlit: the aura reads as a ground marking, not an object.
        let ring_material = mats.add(StandardMaterial {
            base_color: Color::srgba(1.0, 1.0, 1.0, 0.55),
            alpha_mode: AlphaMode::Blend,
            unlit: true,
            cull_mode: None,
            ..default()
        });
        Self {
            material,
            ring_material,
            meshes: HashMap::new(),
            rings: HashMap::new(),
        }
    }
}

impl StrongholdArt {
    pub fn mesh(&mut self, def: &PartDef, own: bool, meshes: &mut Assets<Mesh>) -> Handle<Mesh> {
        self.meshes
            .entry((def.name, own))
            .or_insert_with(|| meshes.add(build(def, own)))
            .clone()
    }

    pub fn ring(
        &mut self,
        radius_milli: i32,
        own: bool,
        meshes: &mut Assets<Mesh>,
    ) -> Handle<Mesh> {
        self.rings
            .entry((radius_milli, own))
            .or_insert_with(|| meshes.add(super::aura::arc(radius_milli, team(own))))
            .clone()
    }
}

pub(super) fn team(own: bool) -> Color {
    if own {
        ALLY
    } else {
        ENEMY
    }
}

pub(super) fn build(def: &PartDef, own: bool) -> Mesh {
    painted((def.build)(team(own)))
}
