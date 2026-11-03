//! Low-poly figures for every unit kind, baked per team (feet at origin, facing -Z).

use bevy::prelude::*;
use std::collections::HashMap;
use tbg_moba_core::types::{MinionKind, NeutralKind, Side, TowerTier};
use tbg_moba_core::{Team, UnitKind};

use crate::palette;
use crate::scene::mesh::{merge, Part, Shape};

const SKIN: Color = Color::srgb(0.93, 0.76, 0.60);
const STEEL: Color = Color::srgb(0.72, 0.74, 0.78);
const DARK: Color = Color::srgb(0.20, 0.18, 0.17);
const WOOD: Color = Color::srgb(0.46, 0.31, 0.17);
pub const STONE: Color = Color::srgb(0.66, 0.64, 0.60);

fn at(x: f32, y: f32, z: f32) -> Transform {
    Transform::from_xyz(x, y, z)
}

/// Shared legs, torso and head; returns nothing, pushes parts.
fn humanoid(p: &mut Vec<Part>, body: Color, legs: Color, head: Color) {
    for x in [-0.13, 0.13] {
        p.push((Shape::Box(0.2, 0.5, 0.24), legs, at(x, 0.25, 0.0)));
    }
    p.push((Shape::Box(0.56, 0.62, 0.36), body, at(0.0, 0.8, 0.0)));
    p.push((Shape::Ball(0.22), head, at(0.0, 1.3, 0.0)));
    for x in [-0.36, 0.36] {
        p.push((Shape::Box(0.16, 0.5, 0.18), body, at(x, 0.8, 0.0)));
    }
}

fn sword(p: &mut Vec<Part>, x: f32, len: f32) {
    p.push((
        Shape::Box(0.08, len, 0.08),
        STEEL,
        at(x, 0.6 + len / 2.0, -0.18),
    ));
    p.push((
        Shape::Box(0.3, 0.06, 0.1),
        palette::GOLD_DARK,
        at(x, 0.62, -0.18),
    ));
}

fn minion(p: &mut Vec<Part>, kind: MinionKind, team: Team) {
    let c = palette::team(team);
    match kind {
        MinionKind::Melee => {
            humanoid(p, c, DARK, SKIN);
            p.push((Shape::Cone(0.25, 0.3), STEEL, at(0.0, 1.5, 0.0)));
            sword(p, 0.38, 0.6);
            p.push((
                Shape::Box(0.08, 0.5, 0.42),
                palette::team_dark(team),
                at(-0.42, 0.85, -0.12),
            ));
        }
        MinionKind::Ranged => {
            p.push((Shape::Cone(0.38, 1.1), c, at(0.0, 0.55, 0.0)));
            p.push((Shape::Ball(0.2), SKIN, at(0.0, 1.22, 0.0)));
            p.push((
                Shape::Cone(0.24, 0.4),
                palette::team_dark(team),
                at(0.0, 1.42, 0.0),
            ));
            p.push((Shape::Box(0.06, 1.0, 0.06), WOOD, at(0.38, 0.95, -0.2)));
        }
    }
}

fn tower(p: &mut Vec<Part>, tier: TowerTier, team: Team) {
    let s = if tier == TowerTier::Nexus { 1.15 } else { 1.0 };
    let trim = palette::team(team);
    p.push((
        Shape::Frustum(1.7 * s, 2.2 * s, 1.0),
        STONE,
        at(0.0, 0.5, 0.0),
    ));
    p.push((Shape::Cyl(1.25 * s, 4.6 * s), STONE, at(0.0, 3.2 * s, 0.0)));
    p.push((Shape::Cyl(1.32 * s, 0.35), trim, at(0.0, 2.4 * s, 0.0)));
    p.push((
        Shape::Frustum(1.75 * s, 1.3 * s, 0.9),
        Color::srgb(0.5, 0.49, 0.47),
        at(0.0, 5.9 * s, 0.0),
    ));
    for k in 0..6 {
        let a = k as f32 / 6.0 * std::f32::consts::TAU;
        p.push((
            Shape::Box(0.55, 0.6, 0.55),
            STONE,
            at(a.cos() * 1.5 * s, 6.6 * s, a.sin() * 1.5 * s),
        ));
    }
    p.push((
        Shape::Cyl(0.7 * s, 0.3),
        palette::team_dark(team),
        at(0.0, 6.5 * s, 0.0),
    ));
}

fn stronghold(p: &mut Vec<Part>, team: Team) {
    p.push((Shape::Cyl(5.5, 1.2), STONE, at(0.0, 0.6, 0.0)));
    p.push((
        Shape::Cyl(4.3, 1.0),
        Color::srgb(0.5, 0.49, 0.47),
        at(0.0, 1.6, 0.0),
    ));
    p.push((
        Shape::Cyl(3.0, 0.5),
        palette::team_dark(team),
        at(0.0, 2.3, 0.0),
    ));
    for k in 0..6 {
        let a = k as f32 / 6.0 * std::f32::consts::TAU + 0.5;
        let (x, z) = (a.cos() * 4.7, a.sin() * 4.7);
        p.push((Shape::Cyl(0.45, 5.0), STONE, at(x, 3.6, z)));
        p.push((Shape::Cone(0.55, 0.9), palette::GOLD, at(x, 6.5, z)));
        p.push((
            Shape::Box(0.08, 1.4, 0.9),
            palette::team(team),
            at(x * 1.06, 4.4, z * 1.06),
        ));
    }
}

/// The glowing crystal floating over a tower or Stronghold.
pub fn crystal_offset(kind: UnitKind) -> Option<(f32, f32)> {
    match kind {
        UnitKind::Tower(tier, _) => Some((if tier == TowerTier::Nexus { 8.6 } else { 7.6 }, 1.0)),
        UnitKind::Stronghold => Some((7.5, 2.4)),
        _ => None,
    }
}

pub fn crystal() -> Mesh {
    merge(&[(Shape::Crystal(0.75, 2.4), Color::WHITE, Transform::IDENTITY)])
}

/// Every unit figure, keyed by kind and side.
#[derive(Resource, Default)]
pub struct Models {
    meshes: HashMap<(UnitKind, Side), Handle<Mesh>>,
}

impl Models {
    pub fn get(&mut self, kind: UnitKind, side: Side, meshes: &mut Assets<Mesh>) -> Handle<Mesh> {
        self.meshes
            .entry((kind, side))
            .or_insert_with(|| meshes.add(figure(kind, side)))
            .clone()
    }
}

pub fn figure(kind: UnitKind, side: Side) -> Mesh {
    let team = side.team().unwrap_or(Team::Blue);
    let mut p = Vec::new();
    match kind {
        UnitKind::Minion(m) => minion(&mut p, m, team),
        UnitKind::Neutral(n) => return super::monsters::figure(n),
        UnitKind::Tower(tier, _) => tower(&mut p, tier, team),
        UnitKind::Stronghold => stronghold(&mut p, team),
        // Commanders and Troops are 1v1 champion rigs (champions.rs).
        UnitKind::Commander(_) | UnitKind::Troop(_) => {}
    }
    merge(&p)
}

/// Visual scale per kind (models are authored at minion size; champion
/// rigs scale themselves in champions.rs).
pub fn scale(kind: UnitKind) -> f32 {
    match kind {
        UnitKind::Minion(_) => 0.95,
        UnitKind::Neutral(
            NeutralKind::Big
            | NeutralKind::Golem
            | NeutralKind::Jaguar
            | NeutralKind::WolfAlpha
            | NeutralKind::Frog,
        ) => 1.25,
        UnitKind::Neutral(_) => 1.1,
        _ => 1.0,
    }
}

/// Height of the health bar above the feet.
pub fn bar_height(kind: UnitKind) -> f32 {
    match kind {
        UnitKind::Neutral(
            NeutralKind::Big
            | NeutralKind::Golem
            | NeutralKind::Jaguar
            | NeutralKind::WolfAlpha
            | NeutralKind::Frog,
        ) => 3.6,
        UnitKind::Tower(..) => 10.6,
        UnitKind::Stronghold => 11.5,
        _ => 2.2,
    }
}

pub fn bar_width(kind: UnitKind) -> f32 {
    match kind {
        UnitKind::Commander(_) => 2.4,
        UnitKind::Tower(..) => 3.4,
        UnitKind::Stronghold => 6.0,
        UnitKind::Neutral(
            NeutralKind::Big
            | NeutralKind::Golem
            | NeutralKind::Jaguar
            | NeutralKind::WolfAlpha
            | NeutralKind::Frog,
        ) => 1.8,
        _ => 1.2,
    }
}
