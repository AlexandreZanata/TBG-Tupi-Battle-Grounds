//! Batched cobbles, verge fittings, layered cliff rocks and river-exit waterfalls.

use super::{
    mesh::{hash, merge, Part, Shape},
    terrain::{self, v},
    to_world,
};
use crate::palette;
use bevy::prelude::*;
use tbg_moba_core::{map, Lane, Team};

const STONE: Color = Color::srgb(0.56, 0.55, 0.49);
const WOOD: Color = Color::srgb(0.36, 0.24, 0.13);
fn part(p: &mut Vec<Part>, s: Shape, c: Color, at: Vec3, rotation: Quat) {
    p.push((
        s,
        c,
        Transform::from_translation(at).with_rotation(rotation),
    ));
}

/// Raised individual stones and roadside props never cross the lane walking ribbon.
pub fn streets() -> Mesh {
    let mut p = Vec::new();
    for lane in Lane::ALL {
        for w in map::lane_units(lane).windows(2) {
            let (a, b) = (v(w[0]), v(w[1]));
            let delta = b - a;
            let dir = delta.normalize_or_zero();
            let side = Vec2::new(-dir.y, dir.x);
            let yaw = Quat::from_rotation_y(dir.y.atan2(dir.x));
            let count = (delta.length() / 2.).ceil() as i32;
            for i in 0..count {
                let centre = a.lerp(b, (i as f32 + 0.5) / count as f32);
                if terrain::river_distance(centre) < map::RIVER_HALF_WIDTH as f32 + 3. {
                    continue;
                }
                for edge in [-1., 1.] {
                    let at = centre + side * (terrain::LANE_HALF_WIDTH + 0.25) * edge;
                    part(
                        &mut p,
                        Shape::Shared(crate::art::mesh::Shape::Bevel(1.75, 0.22, 0.5, 0.08)),
                        STONE,
                        to_world(at.x, at.y, terrain::road_height(at) + 0.12),
                        yaw,
                    );
                }
                for j in [-2, -1, 0, 1, 2] {
                    let at = centre
                        + side * (j as f32 * 1.25)
                        + dir * (if j % 2 == 0 { 0.22 } else { -0.22 });
                    let worn = if j == 0 { 0.10 } else { 0.16 };
                    let tint = 0.72 + hash(i, j, lane as u32 + 82) * 0.15;
                    part(
                        &mut p,
                        Shape::Shared(crate::art::mesh::Shape::Bevel(1.45, worn, 1.1, 0.10)),
                        Color::srgb(tint, tint * 0.91, tint * 0.74),
                        to_world(at.x, at.y, terrain::road_height(at) + 0.10),
                        yaw,
                    );
                }
            }
            let at = (a + b) * 0.5 + side * 7.;
            if terrain::height(at) > 1. || terrain::river_distance(at) < 12. {
                continue;
            }
            let y = terrain::height(at);
            let team = if at.x + at.y < 200. {
                Team::Blue
            } else {
                Team::Red
            };
            // Short fence run and a team pennant on each road segment.
            for k in [-1., 1.] {
                let site = at + dir * k * 1.8;
                part(
                    &mut p,
                    Shape::Cyl(0.13, 1.7),
                    WOOD,
                    to_world(site.x, site.y, y + 0.85),
                    Quat::IDENTITY,
                );
            }
            for h in [0.7, 1.25] {
                part(
                    &mut p,
                    Shape::Box(3.6, 0.13, 0.14),
                    WOOD,
                    to_world(at.x, at.y, y + h),
                    yaw,
                );
            }
            part(
                &mut p,
                Shape::Box(0.8, 1.1, 0.08),
                palette::team(team),
                to_world(at.x, at.y, y + 1.),
                yaw,
            );
        }
    }
    // Braziers match the animated flame anchors in motion.rs.
    for at in super::motion::fire_sites() {
        part(
            &mut p,
            Shape::Shared(crate::art::mesh::Shape::Bevel(0.9, 1.8, 0.9, 0.14)),
            STONE,
            at - Vec3::Y * 1.1,
            Quat::IDENTITY,
        );
        part(
            &mut p,
            Shape::Frustum(0.58, 0.28, 0.35),
            Color::srgb(0.16, 0.17, 0.18),
            at - Vec3::Y * 0.15,
            Quat::IDENTITY,
        );
    }
    merge(&p)
}

/// Rock strata and moss shelves, with gaps at the river mouths.
pub fn rim() -> Mesh {
    let mut p = Vec::new();
    for edge in 0..4 {
        for k in 0..42 {
            let f = k as f32 * 5.;
            let at = match edge {
                0 => Vec2::new(1., f),
                1 => Vec2::new(199., f),
                2 => Vec2::new(f, 1.),
                _ => Vec2::new(f, 199.),
            };
            if terrain::river_distance(at) < 11. {
                continue;
            }
            for layer in 0..3 {
                let h = terrain::height(at) - layer as f32 * 1.8;
                let seed = hash(edge, k, 99) * 6.;
                part(
                    &mut p,
                    Shape::Shared(crate::art::mesh::Shape::Rock(
                        2.7,
                        ((edge * 100 + k) * 3 + layer) as u32,
                    )),
                    if layer == 0 {
                        Color::srgb(0.38, 0.45, 0.28)
                    } else {
                        Color::srgb(0.43, 0.43, 0.40)
                    },
                    to_world(at.x, at.y, h),
                    Quat::from_rotation_y(seed),
                );
            }
        }
    }
    merge(&p)
}

/// Vertical faceted water and foam at the two valley outlets.
pub fn waterfalls() -> Mesh {
    let mut p = Vec::new();
    for (i, endpoint) in [map::RIVER[0], map::RIVER[map::RIVER.len() - 1]]
        .into_iter()
        .enumerate()
    {
        let at = v(endpoint);
        let direction = if i == 0 {
            Vec2::new(-1., 1.)
        } else {
            Vec2::new(1., -1.)
        };
        let yaw = Quat::from_rotation_y(direction.y.atan2(direction.x));
        for k in 0..9 {
            let side = Vec2::new(-direction.y, direction.x).normalize_or_zero();
            let q = at + side * (k as f32 - 4.) * 1.2 + direction * 1.6;
            part(
                &mut p,
                Shape::Box(1.18, 9.5, 0.45),
                if k % 3 == 0 {
                    Color::srgb(0.68, 0.91, 0.91)
                } else {
                    Color::srgb(0.22, 0.65, 0.75)
                },
                to_world(q.x, q.y, -5.2),
                yaw,
            );
        }
        part(
            &mut p,
            Shape::Disc(6.5),
            Color::srgba(0.73, 0.89, 0.88, 1.),
            to_world(at.x + direction.x * 2., at.y + direction.y * 2., -9.8),
            Quat::IDENTITY,
        );
    }
    merge(&p)
}
