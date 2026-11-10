//! Forest, bushes and rocks scattered where nothing walks, in culling chunks.

use bevy::prelude::*;
use tbg_moba_core::map::{self, PLAZA, RIVER_HALF_WIDTH, SIZE};
use tbg_moba_core::Team;

use super::mesh::{hash, merge, noise, Part, Shape};
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
    super::terrain::jungle_path_distance(p) < 2.4
        || lane_distance(p) < 7.0
        || river_distance(p) < RIVER_HALF_WIDTH as f32 + 3.5
        || p.distance(v(PLAZA)) < 15.0
        || tower_near
        || camp_near
        || base_near
}

fn pine(parts: &mut Vec<Part>, at: Vec3, s: f32, _tint: f32, _yaw: f32) {
    let mut shared = Vec::new();
    super::kit::pine(&mut shared, at, s * 2.1);
    append(parts, shared);
}
fn bush(parts: &mut Vec<Part>, at: Vec3, s: f32) {
    let mut shared = Vec::new();
    super::kit::bush(&mut shared, at, s * 1.6);
    append(parts, shared);
}
pub fn rock(parts: &mut Vec<Part>, at: Vec3, s: f32, seed: f32) {
    let mut shared = Vec::new();
    super::kit::boulder(&mut shared, at, s * 1.7, seed * 6.);
    append(parts, shared);
}
fn append(parts: &mut Vec<Part>, shared: Vec<super::kit::Part>) {
    parts.extend(shared.into_iter().map(|(s, c, t)| (Shape::Shared(s), c, t)));
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

/// Tiny verge plants have their own batch so they never cast a shadow.
pub fn details() -> Mesh {
    let mut parts = Vec::new();
    for i in 0..1500 {
        let p = Vec2::new(hash(i, 0, 113), hash(i, 1, 113)) * SIZE as f32;
        let lane = lane_distance(p);
        let river = river_distance(p);
        if clear_of_structures(p) || !(4.8..8.).contains(&lane) && !(8.5..11.).contains(&river) {
            continue;
        }
        let at = to_world(p.x, p.y, height(p));
        if i % 7 == 0 {
            super::kit::flowers(&mut parts, at);
        } else {
            super::kit::grass(&mut parts, at, hash(i, 2, 113) * 6.);
        }
    }
    super::kit::merge(parts)
}
