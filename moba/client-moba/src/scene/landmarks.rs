//! Built places: base plateaus, the river plaza, jungle camps and bridges.

use bevy::prelude::*;
use tbg_moba_core::map::{self, PLAZA, RIVER_HALF_WIDTH};
use tbg_moba_core::{Lane, Team};

use super::mesh::{hash, merge, Part, Shape};
use super::nature::rock;
use super::terrain::{height, v, LANE_HALF_WIDTH};
use super::to_world;
use crate::palette;

const STONE: Color = Color::srgb(0.62, 0.60, 0.56);
const STONE_DARK: Color = Color::srgb(0.45, 0.44, 0.42);
const WOOD: Color = Color::srgb(0.46, 0.31, 0.17);

fn units(p: tbg_moba_core::geom::Pos) -> Vec2 {
    Vec2::new(p.x as f32, p.y as f32) / 1000.0
}

fn flat(parts: &mut Vec<Part>, at: Vec2, r: f32, y: f32, color: Color) {
    parts.push((
        Shape::Disc(r),
        color,
        Transform::from_translation(to_world(at.x, at.y, y)),
    ));
}

/// Directions (map space) in which roads leave a base.
fn road_angles(team: Team) -> Vec<f32> {
    Lane::ALL
        .iter()
        .map(|&l| {
            let d = units(map::waypoint(l, team, 1)) - units(map::waypoint(l, team, 0));
            d.y.atan2(d.x)
        })
        .collect()
}

pub fn bases() -> Mesh {
    let mut parts = Vec::new();
    for team in Team::ALL {
        let c = units(map::stronghold(team));
        flat(&mut parts, c, 19.0, 0.86, STONE_DARK);
        flat(&mut parts, c, 17.5, 0.9, STONE);
        flat(&mut parts, c, 9.5, 0.94, palette::team_dark(team));
        flat(&mut parts, c, 8.6, 0.96, STONE);
        let roads = road_angles(team);
        for k in 0..36 {
            let a = k as f32 / 36.0 * std::f32::consts::TAU;
            let gap = roads.iter().any(|r| {
                let d = (a - r).rem_euclid(std::f32::consts::TAU);
                d.min(std::f32::consts::TAU - d) < 0.32
            });
            if gap {
                continue;
            }
            let p = c + Vec2::new(a.cos(), a.sin()) * 19.5;
            // Wall blocks run along the circle (tangent), so turn a quarter past the radius.
            let rot = Quat::from_rotation_y(a + std::f32::consts::FRAC_PI_2);
            let at = to_world(p.x, p.y, height(p));
            parts.push((
                Shape::Box(3.6, 2.2, 1.4),
                STONE,
                Transform::from_translation(at + Vec3::Y * 1.1).with_rotation(rot),
            ));
            if k % 2 == 0 {
                parts.push((
                    Shape::Box(1.0, 0.7, 1.5),
                    STONE_DARK,
                    Transform::from_translation(at + Vec3::Y * 2.5).with_rotation(rot),
                ));
            }
            if k % 6 == 3 {
                banner(&mut parts, at, team, a);
            }
        }
    }
    merge(&parts)
}

fn banner(parts: &mut Vec<Part>, at: Vec3, team: Team, angle: f32) {
    let out = Vec3::new(angle.cos(), 0.0, -angle.sin()) * 1.2;
    parts.push((
        Shape::Cyl(0.12, 6.0),
        WOOD,
        Transform::from_translation(at + out + Vec3::Y * 3.0),
    ));
    parts.push((
        Shape::Box(1.8, 2.6, 0.12),
        palette::team(team),
        Transform::from_translation(at + out + Vec3::Y * 4.4)
            .with_rotation(Quat::from_rotation_y(angle + std::f32::consts::FRAC_PI_2)),
    ));
    parts.push((
        Shape::Box(0.7, 0.7, 0.16),
        palette::GOLD,
        Transform::from_translation(at + out + Vec3::Y * 4.6).with_rotation(
            Quat::from_rotation_y(angle + std::f32::consts::FRAC_PI_2)
                * Quat::from_rotation_z(0.785),
        ),
    ));
}

pub fn plaza() -> Mesh {
    let mut parts = Vec::new();
    let c = v(PLAZA);
    parts.push((
        Shape::Cyl(10.2, 1.7),
        STONE_DARK,
        Transform::from_translation(to_world(c.x, c.y, -0.6)),
    ));
    flat(
        &mut parts,
        c,
        map::PLAZA_RADIUS as f32 + 0.6,
        0.27,
        STONE_DARK,
    );
    flat(&mut parts, c, map::PLAZA_RADIUS as f32 - 0.6, 0.3, STONE);
    flat(&mut parts, c, 4.2, 0.34, Color::srgb(0.30, 0.55, 0.70));
    flat(&mut parts, c, 3.4, 0.37, STONE);
    parts.push((
        Shape::Cyl(2.2, 0.8),
        STONE_DARK,
        Transform::from_translation(to_world(c.x, c.y, 0.7)),
    ));
    for k in 0..8 {
        let a = k as f32 / 8.0 * std::f32::consts::TAU + 0.39;
        let p = c + Vec2::new(a.cos(), a.sin()) * (map::PLAZA_RADIUS as f32 - 1.0);
        parts.push((
            Shape::Cyl(0.45, 2.4),
            STONE,
            Transform::from_translation(to_world(p.x, p.y, 1.4)),
        ));
        parts.push((
            Shape::Ball(0.5),
            Color::srgb(0.5, 0.85, 1.0),
            Transform::from_translation(to_world(p.x, p.y, 2.8)),
        ));
    }
    for side in [-1., 1.] {
        for step in 0..5 {
            let q = c + Vec2::ONE.normalize() * (10.4 + step as f32 * 0.8) * side;
            parts.push((
                Shape::Box(6.5, 0.2, 0.9),
                STONE,
                Transform::from_translation(to_world(q.x, q.y, 0.2 - step as f32 * 0.06))
                    .with_rotation(Quat::from_rotation_y(std::f32::consts::FRAC_PI_4)),
            ));
        }
    }
    merge(&parts)
}

/// The glowing crystal on the plaza pedestal (its own emissive material).
pub fn plaza_crystal() -> Mesh {
    let c = v(PLAZA);
    merge(&[(
        Shape::Crystal(1.1, 3.6),
        Color::WHITE,
        Transform::from_translation(to_world(c.x, c.y, 2.9)),
    )])
}

pub fn camps() -> Mesh {
    let mut parts = Vec::new();
    for camp in 0..map::CAMP_COUNT {
        let (pos, _team) = map::camp(camp);
        let c = units(pos);
        let r = map::CAMP_RADIUS as f32;
        flat(&mut parts, c, r + 0.8, 0.05, Color::srgb(0.30, 0.25, 0.19));
        flat(&mut parts, c, r - 0.4, 0.08, Color::srgb(0.24, 0.21, 0.17));
        for k in 0..14 {
            let a = k as f32 / 14.0 * std::f32::consts::TAU + hash(camp as i32, k, 8);
            let p = c + Vec2::new(a.cos(), a.sin()) * (r + 0.6);
            rock(
                &mut parts,
                to_world(p.x, p.y, height(p)),
                0.7 + hash(camp as i32, k, 9) * 0.6,
                hash(camp as i32, k, 10),
            );
        }
        use tbg_moba_core::types::CampKind;
        for k in 0..6 {
            let angle = k as f32 * 1.047 + camp as f32;
            let p = c + Vec2::new(angle.cos(), angle.sin()) * (r - 1.0);
            let (shape, color, h) = match map::camp_kind(camp) {
                CampKind::Wolves => (
                    Shape::Box(0.22, 0.18, 1.1),
                    Color::srgb(0.79, 0.73, 0.58),
                    0.22,
                ),
                CampKind::Golems => (Shape::Crystal(0.4, 1.6), Color::srgb(0.28, 0.78, 0.72), 0.8),
                CampKind::Jaguar => (Shape::Ball(0.55), Color::srgb(0.23, 0.39, 0.15), 0.25),
                CampKind::Frogs => (Shape::Disc(0.75), Color::srgb(0.29, 0.53, 0.18), 0.12),
            };
            parts.push((
                shape,
                color,
                Transform::from_translation(to_world(p.x, p.y, h)),
            ));
        }
    }
    merge(&parts)
}

const BRIDGE_LENGTH: f32 = 2.0 * RIVER_HALF_WIDTH as f32 + 6.0;
const BRIDGE_TOP: f32 = 0.25 + 0.35 / 2.0;

fn bridge_axes() -> impl Iterator<Item = (Vec2, Vec2)> {
    [Lane::Top, Lane::Bot].into_iter().flat_map(|lane| {
        map::lane_units(lane).windows(2).filter_map(|w| {
            let (a, b) = (v(w[0]), v(w[1]));
            let cross = river_crossing(a, b)?;
            Some((cross, (b - a).normalize()))
        })
    })
}

pub fn bridge_surface(point: Vec2) -> Option<f32> {
    bridge_axes().find_map(|(cross, dir)| {
        let offset = point - cross;
        (offset.dot(dir).abs() <= BRIDGE_LENGTH / 2.0
            && offset.perp_dot(dir).abs() <= LANE_HALF_WIDTH)
            .then_some(BRIDGE_TOP)
    })
}

/// Stone decks where the top and bot roads cross the river.
pub fn bridges() -> Mesh {
    let mut parts = Vec::new();
    for (cross, dir) in bridge_axes() {
        let yaw = Quat::from_rotation_y(dir.y.atan2(dir.x));
        let length = BRIDGE_LENGTH;
        let at = to_world(cross.x, cross.y, 0.25);
        parts.push((
            Shape::Box(length, 0.35, LANE_HALF_WIDTH * 2.0),
            STONE,
            Transform::from_translation(at).with_rotation(yaw),
        ));
        for side in [-1.0, 1.0] {
            let offset = yaw * Vec3::new(0.0, 0.8, side * LANE_HALF_WIDTH);
            parts.push((
                Shape::Box(length, 0.38, 0.6),
                STONE,
                Transform::from_translation(at + offset).with_rotation(yaw),
            ));
            for k in 0..5 {
                let along =
                    yaw * Vec3::new((k as f32 - 2.0) * length / 4.5, 0.4, side * LANE_HALF_WIDTH);
                parts.push((
                    Shape::Box(0.8, 1.4, 0.8),
                    STONE,
                    Transform::from_translation(at + along),
                ));
            }
        }
    }
    merge(&parts)
}

/// Where a road segment crosses the river centre line, if it does.
fn river_crossing(a: Vec2, b: Vec2) -> Option<Vec2> {
    let from = a.x + a.y - map::SIZE as f32;
    let to = b.x + b.y - map::SIZE as f32;
    if from == to || from * to > 0.0 {
        return None;
    }
    Some(a.lerp(b, from / (from - to)))
}
