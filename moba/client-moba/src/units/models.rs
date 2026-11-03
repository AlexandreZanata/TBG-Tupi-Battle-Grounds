//! Low-poly figures for every unit kind, baked per team (feet at origin, facing -Z).

use bevy::prelude::*;
use std::collections::HashMap;
use tbg_moba_core::types::{MinionKind, NeutralKind, Side, TowerTier};
use tbg_moba_core::{Role, Team, TroopKind, UnitKind};

use crate::palette;
use crate::scene::mesh::{merge, mix, Part, Shape};

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

fn staff(p: &mut Vec<Part>, orb: Color) {
    p.push((Shape::Cyl(0.05, 1.8), WOOD, at(0.42, 0.95, -0.12)));
    p.push((Shape::Ball(0.17), orb, at(0.42, 1.92, -0.12)));
}

fn commander(p: &mut Vec<Part>, role: Role, team: Team) {
    let c = palette::team(team);
    humanoid(p, c, DARK, SKIN);
    p.push((
        Shape::Box(0.62, 0.16, 0.42),
        palette::GOLD,
        at(0.0, 1.08, 0.0),
    ));
    p.push((
        Shape::Box(0.6, 0.85, 0.06),
        palette::team_dark(team),
        at(0.0, 0.85, 0.24),
    ));
    p.push((Shape::Frustum(0.18, 0.26, 0.24), STEEL, at(0.0, 1.46, 0.0)));
    p.push((Shape::Cone(0.08, 0.35), palette::GOLD, at(0.0, 1.72, 0.0)));
    for x in [-0.36, 0.36] {
        p.push((Shape::Ball(0.17), STEEL, at(x, 1.08, 0.0)));
    }
    match role {
        Role::Top => sword(p, 0.42, 1.1),
        Role::Jungle => {
            p.push((Shape::Cyl(0.05, 1.1), WOOD, at(0.42, 0.9, -0.18)));
            p.push((Shape::Box(0.08, 0.36, 0.42), STEEL, at(0.42, 1.35, -0.3)));
        }
        Role::Mid => staff(p, palette::glow(team)),
        Role::Bot => {
            p.push((Shape::Box(0.1, 0.12, 0.8), WOOD, at(0.38, 0.95, -0.3)));
            p.push((Shape::Box(0.7, 0.06, 0.08), WOOD, at(0.38, 0.95, -0.6)));
        }
        Role::Support => {
            staff(p, palette::GOLD);
            p.push((
                Shape::Box(0.06, 0.5, 0.4),
                palette::glow(team),
                at(0.42, 1.55, 0.05),
            ));
        }
    }
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

fn troop(p: &mut Vec<Part>, kind: TroopKind, team: Team) {
    let c = palette::team(team);
    match kind {
        TroopKind::Guardian => {
            humanoid(p, STEEL, DARK, SKIN);
            p.push((Shape::Frustum(0.2, 0.26, 0.3), STEEL, at(0.0, 1.42, 0.0)));
            p.push((Shape::Box(0.06, 0.12, 0.4), c, at(0.0, 1.62, 0.05)));
            p.push((Shape::Box(0.1, 0.95, 0.72), c, at(-0.46, 0.85, -0.2)));
            p.push((
                Shape::Box(0.12, 0.3, 0.3),
                palette::GOLD,
                at(-0.5, 0.9, -0.2),
            ));
            sword(p, 0.42, 0.75);
        }
        TroopKind::Healer => {
            p.push((
                Shape::Cone(0.46, 1.25),
                Color::srgb(0.95, 0.94, 0.9),
                at(0.0, 0.62, 0.0),
            ));
            p.push((Shape::Box(0.5, 0.12, 0.38), c, at(0.0, 0.95, 0.0)));
            p.push((Shape::Ball(0.21), SKIN, at(0.0, 1.32, 0.0)));
            p.push((
                Shape::Cone(0.27, 0.42),
                Color::srgb(0.95, 0.94, 0.9),
                at(0.0, 1.5, 0.02),
            ));
            staff(p, Color::srgb(0.6, 1.0, 0.6));
        }
        TroopKind::Bard => {
            humanoid(p, Color::srgb(0.55, 0.36, 0.2), DARK, SKIN);
            p.push((
                Shape::Cyl(0.32, 0.06),
                Color::srgb(0.4, 0.26, 0.14),
                at(0.0, 1.48, 0.0),
            ));
            p.push((
                Shape::Cone(0.18, 0.3),
                Color::srgb(0.4, 0.26, 0.14),
                at(0.0, 1.62, 0.0),
            ));
            p.push((Shape::Box(0.05, 0.35, 0.05), c, at(0.12, 1.7, 0.1)));
            p.push((Shape::Ball(0.26), WOOD, at(0.1, 0.8, -0.3)));
            p.push((Shape::Box(0.06, 0.06, 0.6), WOOD, at(0.3, 0.95, -0.55)));
        }
        TroopKind::Colossus => {
            let rock = mix(STONE, palette::team_dark(team), 0.15);
            for x in [-0.4, 0.4] {
                p.push((Shape::Box(0.5, 0.8, 0.55), rock, at(x, 0.4, 0.0)));
                p.push((Shape::Box(0.48, 1.2, 0.5), rock, at(x * 2.3, 1.2, 0.0)));
            }
            p.push((Shape::Box(1.5, 1.1, 0.95), rock, at(0.0, 1.35, 0.0)));
            p.push((Shape::Box(0.6, 0.45, 0.55), rock, at(0.0, 2.1, -0.1)));
            for x in [-0.14, 0.14] {
                p.push((
                    Shape::Box(0.12, 0.08, 0.05),
                    palette::glow(team),
                    at(x, 2.15, -0.38),
                ));
            }
            p.push((
                Shape::Crystal(0.25, 0.7),
                palette::glow(team),
                at(0.0, 1.5, 0.48),
            ));
        }
        TroopKind::Summoner => {
            let robe = Color::srgb(0.42, 0.24, 0.62);
            p.push((Shape::Cone(0.45, 1.25), robe, at(0.0, 0.62, 0.0)));
            p.push((Shape::Box(0.48, 0.1, 0.36), c, at(0.0, 0.9, 0.0)));
            p.push((Shape::Ball(0.2), SKIN, at(0.0, 1.3, 0.0)));
            p.push((Shape::Cone(0.3, 0.75), robe, at(0.0, 1.65, 0.0)));
            staff(p, palette::glow(team));
        }
    }
}

fn neutral(p: &mut Vec<Part>, kind: NeutralKind) {
    match kind {
        NeutralKind::Small => {
            let fur = Color::srgb(0.38, 0.27, 0.2);
            p.push((Shape::Box(0.6, 0.55, 1.1), fur, at(0.0, 0.6, 0.0)));
            p.push((Shape::Box(0.45, 0.42, 0.45), fur, at(0.0, 0.75, -0.65)));
            for (x, z) in [(-0.22, -0.4), (0.22, -0.4), (-0.22, 0.4), (0.22, 0.4)] {
                p.push((Shape::Box(0.14, 0.4, 0.14), DARK, at(x, 0.2, z)));
            }
            for x in [-0.15, 0.15] {
                p.push((Shape::Cone(0.05, 0.25), Color::WHITE, at(x, 0.62, -0.9)));
            }
        }
        NeutralKind::Big => {
            let rock = Color::srgb(0.36, 0.30, 0.27);
            p.push((Shape::Ball(0.95), rock, at(0.0, 1.3, 0.0)));
            for x in [-1.0, 1.0] {
                p.push((Shape::Ball(0.45), rock, at(x, 1.1, -0.1)));
                p.push((Shape::Box(0.4, 0.9, 0.4), rock, at(x * 0.45, 0.45, 0.0)));
            }
            for (x, z, s) in [(0.0, 0.55, 1.0), (-0.45, 0.4, 0.7), (0.45, 0.4, 0.75)] {
                p.push((
                    Shape::Crystal(0.25 * s, 1.0 * s),
                    palette::NEUTRAL,
                    at(x, 2.0, z),
                ));
            }
            p.push((
                Shape::Box(0.5, 0.1, 0.05),
                Color::srgb(1.0, 0.6, 0.2),
                at(0.0, 1.5, -0.92),
            ));
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
        UnitKind::Commander(role) => commander(&mut p, role, team),
        UnitKind::Minion(m) => minion(&mut p, m, team),
        UnitKind::Troop(t) => troop(&mut p, t, team),
        UnitKind::Neutral(n) => neutral(&mut p, n),
        UnitKind::Tower(tier, _) => tower(&mut p, tier, team),
        UnitKind::Stronghold => stronghold(&mut p, team),
    }
    merge(&p)
}

/// Visual scale per kind (models are authored at minion size).
pub fn scale(kind: UnitKind) -> f32 {
    match kind {
        UnitKind::Commander(_) => 1.45,
        UnitKind::Minion(_) => 0.95,
        UnitKind::Troop(TroopKind::Colossus) => 1.25,
        UnitKind::Troop(_) => 1.15,
        UnitKind::Neutral(NeutralKind::Big) => 1.25,
        UnitKind::Neutral(NeutralKind::Small) => 1.1,
        UnitKind::Tower(..) | UnitKind::Stronghold => 1.0,
    }
}

/// Height of the health bar above the feet.
pub fn bar_height(kind: UnitKind) -> f32 {
    match kind {
        UnitKind::Commander(_) => 3.3,
        UnitKind::Troop(TroopKind::Colossus) => 3.4,
        UnitKind::Neutral(NeutralKind::Big) => 3.6,
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
        UnitKind::Neutral(NeutralKind::Big) | UnitKind::Troop(TroopKind::Colossus) => 1.8,
        _ => 1.2,
    }
}
