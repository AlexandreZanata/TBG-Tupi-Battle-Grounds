//! Forest, bushes and rocks scattered where nothing walks, in culling chunks.

use bevy::prelude::*;
use tbg_moba_core::map::{self, PLAZA, RIVER_HALF_WIDTH, SIZE};
use tbg_moba_core::Team;

use super::mesh::{hash, merge, mix, noise, Part, Shape};
use super::terrain::{height, lane_distance, river_distance, v};
use super::to_world;

const SPACING: f32 = 3.6;
const CHUNK: f32 = 56.0;

/// True where trees would block a road, the river, a base, a camp or a tower.
fn clear(p: Vec2) -> bool {
    let tower_near = map::towers()
        .any(|(_, _, _, t)| p.distance(Vec2::new(t.x as f32, t.y as f32) / 1000.0) < 6.0);
    let camp_near = (0..map::CAMP_COUNT).any(|c| {
        let (at, _) = map::camp(c);
        p.distance(Vec2::new(at.x as f32, at.y as f32) / 1000.0) < map::CAMP_RADIUS as f32 + 4.5
    });
    let base_near = Team::ALL.iter().any(|&t| {
        let s = map::stronghold(t);
        p.distance(Vec2::new(s.x as f32, s.y as f32) / 1000.0) < 24.0
    });
    lane_distance(p) < 7.0
        || river_distance(p) < RIVER_HALF_WIDTH as f32 + 3.5
        || p.distance(v(PLAZA)) < 15.0
        || tower_near
        || camp_near
        || base_near
}

fn pine(parts: &mut Vec<Part>, at: Vec3, s: f32, tint: f32, yaw: f32) {
    let rot = Quat::from_rotation_y(yaw);
    parts.push((
        Shape::Cyl(0.22 * s, 1.2 * s),
        Color::srgb(0.36, 0.24, 0.14),
        Transform::from_translation(at + Vec3::Y * 0.6 * s),
    ));
    let greens = [
        mix(
            Color::srgb(0.13, 0.33, 0.16),
            Color::srgb(0.20, 0.42, 0.15),
            tint,
        ),
        mix(
            Color::srgb(0.17, 0.40, 0.18),
            Color::srgb(0.27, 0.50, 0.17),
            tint,
        ),
        mix(
            Color::srgb(0.24, 0.48, 0.20),
            Color::srgb(0.36, 0.58, 0.20),
            tint,
        ),
    ];
    for (k, (r, h, y)) in [(1.55, 2.1, 1.9), (1.2, 1.8, 2.9), (0.8, 1.5, 3.8)]
        .into_iter()
        .enumerate()
    {
        parts.push((
            Shape::Cone(r * s, h * s),
            greens[k],
            Transform::from_translation(at + Vec3::Y * y * s).with_rotation(rot),
        ));
    }
}

fn bush(parts: &mut Vec<Part>, at: Vec3, s: f32) {
    for (off, r) in [
        (Vec3::new(-0.35, 0.3, 0.0), 0.6),
        (Vec3::new(0.35, 0.35, 0.1), 0.7),
        (Vec3::new(0.0, 0.25, -0.4), 0.5),
    ] {
        parts.push((
            Shape::Ball(r * s),
            Color::srgb(0.24, 0.48, 0.17),
            Transform::from_translation(at + off * s).with_scale(Vec3::new(1.1, 0.8, 1.0)),
        ));
    }
}

pub fn rock(parts: &mut Vec<Part>, at: Vec3, s: f32, seed: f32) {
    let grey = mix(
        Color::srgb(0.42, 0.41, 0.39),
        Color::srgb(0.60, 0.58, 0.54),
        seed,
    );
    parts.push((
        Shape::Ball(s),
        grey,
        Transform::from_translation(at + Vec3::Y * s * 0.25)
            .with_rotation(Quat::from_rotation_y(seed * 6.0))
            .with_scale(Vec3::new(1.3, 0.75, 1.0)),
    ));
}

/// The scattered vegetation, one merged mesh per map chunk.
pub fn forest() -> Vec<Mesh> {
    let chunks = ((SIZE as f32 + 40.0) / CHUNK).ceil() as i32;
    let mut parts: Vec<Vec<Part>> = vec![Vec::new(); (chunks * chunks) as usize];
    let n = ((SIZE as f32 + 36.0) / SPACING) as i32;
    for j in 0..n {
        for i in 0..n {
            let jitter = Vec2::new(hash(i, j, 1), hash(i, j, 2)) - 0.5;
            let p = Vec2::new(-18.0 + i as f32 * SPACING, -18.0 + j as f32 * SPACING)
                + jitter * SPACING * 0.9;
            if clear(p) {
                continue;
            }
            let h = height(p);
            let roll = hash(i, j, 3);
            let density = noise(p.x, p.y, 17.0, 9);
            let chunk = (((p.y + 20.0) / CHUNK) as i32).clamp(0, chunks - 1) * chunks
                + (((p.x + 20.0) / CHUNK) as i32).clamp(0, chunks - 1);
            let bucket = &mut parts[chunk as usize];
            let at = to_world(p.x, p.y, h - 0.1);
            if h > 4.0 {
                if roll < 0.35 {
                    rock(bucket, at, 0.8 + roll * 2.5, hash(i, j, 4));
                } else if roll < 0.75 && h < 14.0 {
                    pine(bucket, at, 0.9 + hash(i, j, 5) * 0.5, density, roll * 9.0);
                }
            } else if roll < 0.35 + density * 0.6 {
                pine(bucket, at, 0.85 + hash(i, j, 5) * 0.55, density, roll * 9.0);
            } else if roll > 0.93 {
                bush(bucket, at, 0.9);
            } else if roll > 0.89 {
                rock(bucket, at, 0.6, hash(i, j, 6));
            }
        }
    }
    edge_bushes(&mut parts[0]);
    parts
        .into_iter()
        .filter(|p| !p.is_empty())
        .map(|p| merge(&p))
        .collect()
}

/// A few bushes along road verges break the hard road edge.
fn edge_bushes(parts: &mut Vec<Part>) {
    for k in 0..400 {
        let p = Vec2::new(hash(k, 0, 31), hash(k, 1, 31)) * SIZE as f32;
        let d = lane_distance(p);
        if (5.2..6.6).contains(&d) && !clear_of_structures(p) {
            bush(
                parts,
                to_world(p.x, p.y, height(p)),
                0.7 + hash(k, 2, 31) * 0.4,
            );
        }
    }
}

fn clear_of_structures(p: Vec2) -> bool {
    river_distance(p) < RIVER_HALF_WIDTH as f32 + 3.0
        || map::towers()
            .any(|(_, _, _, t)| p.distance(Vec2::new(t.x as f32, t.y as f32) / 1000.0) < 4.0)
}
