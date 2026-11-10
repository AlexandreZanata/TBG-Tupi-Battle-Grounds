//! Built places: base plateaus, the river plaza, jungle camps and bridges.

use bevy::prelude::*;
use tbg_moba_core::map;
use tbg_moba_core::{Lane, Team};

use super::blender;
use super::mesh::{merge, Part, Shape};
use super::terrain::{height, LANE_HALF_WIDTH};
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

pub fn camps() -> impl Iterator<Item = Mesh> {
    (0..map::CAMP_COUNT).map(|camp| {
        let (pos, team) = map::camp(camp);
        let c = units(pos);
        // Camp kits are authored on Blue's river bank. Their shore edge faces
        // the opposite way on Red's side.
        let yaw = if team == Team::Red {
            std::f32::consts::PI
        } else {
            0.0
        };
        let transform = Transform::from_translation(to_world(c.x, c.y, 0.0))
            .with_rotation(Quat::from_rotation_y(yaw));
        blender::camp(map::camp_kind(camp)).transformed_by(transform)
    })
}

const BRIDGE_LENGTH: f32 = map::BRIDGE_LENGTH as f32 / 1000.0;
const BRIDGE_TOP: f32 = 0.25 + 0.35 / 2.0;

fn bridge_axes() -> impl Iterator<Item = (Vec2, Vec2)> {
    map::bridges().map(|p| {
        (
            Vec2::new(p.x as f32, p.y as f32) / 1000.0,
            Vec2::ONE.normalize(),
        )
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
pub fn bridges() -> impl Iterator<Item = Mesh> {
    let deck = blender::bridge();
    bridge_axes().map(move |(cross, dir)| {
        let transform = Transform::from_translation(to_world(cross.x, cross.y, 0.0))
            .with_rotation(Quat::from_rotation_y(dir.y.atan2(dir.x)));
        deck.clone().transformed_by(transform)
    })
}
