//! Ground relief, lane roads and the river surface, all derived from the core map.

use bevy::prelude::*;
use tbg_moba_core::map::{self, PLAZA, PLAZA_RADIUS, RIVER, RIVER_HALF_WIDTH, SIZE};
use tbg_moba_core::Lane;

use super::mesh::{hash, mix, noise, triangles};
use super::to_world;

const CELL: f32 = 2.0;
const MARGIN: f32 = 18.0;
pub const WATER_LEVEL: f32 = -0.45;
pub const LANE_HALF_WIDTH: f32 = 3.6;

/// Distance from `p` to the segment `a`–`b`.
pub fn segment_distance(p: Vec2, a: Vec2, b: Vec2) -> f32 {
    let ab = b - a;
    let t = ((p - a).dot(ab) / ab.length_squared().max(1e-6)).clamp(0.0, 1.0);
    p.distance(a + ab * t)
}

pub fn polyline_distance(p: Vec2, points: &[(i32, i32)]) -> f32 {
    points
        .windows(2)
        .map(|w| segment_distance(p, v(w[0]), v(w[1])))
        .fold(f32::MAX, f32::min)
}

pub fn v((x, y): (i32, i32)) -> Vec2 {
    Vec2::new(x as f32, y as f32)
}

pub fn lane_distance(p: Vec2) -> f32 {
    Lane::ALL
        .iter()
        .map(|&l| polyline_distance(p, map::lane_units(l)))
        .fold(f32::MAX, f32::min)
}

pub fn river_distance(p: Vec2) -> f32 {
    polyline_distance(p, &RIVER)
}

/// Inside distance to the map border (negative outside).
fn border_distance(p: Vec2) -> f32 {
    let s = SIZE as f32;
    p.x.min(p.y).min(s - p.x).min(s - p.y)
}

/// Ground height at a map point; units are placed on it.
pub fn height(p: Vec2) -> f32 {
    let plaza = p.distance(v(PLAZA)) - PLAZA_RADIUS as f32;
    if plaza < 0.0 {
        return 0.25;
    }
    let river = river_distance(p) - RIVER_HALF_WIDTH as f32;
    if river < 3.0 {
        let k = ((3.0 - river) / 3.0).clamp(0.0, 1.0);
        return -1.7 * k * k * (3.0 - 2.0 * k);
    }
    let edge = border_distance(p);
    if edge < 9.0 {
        // Rocky rim rising around the valley.
        let rise = (9.0 - edge).min(22.0);
        return rise * 0.9 + noise(p.x, p.y, 7.0, 3) * rise * 0.35;
    }
    (noise(p.x, p.y, 23.0, 1) - 0.5) * 0.35
}

fn ground_color(p: Vec2, h: f32) -> Color {
    let grass_dark = Color::srgb(0.22, 0.40, 0.14);
    let grass_light = Color::srgb(0.43, 0.62, 0.22);
    let n = noise(p.x, p.y, 9.0, 7) * 0.7 + hash(p.x as i32, p.y as i32, 5) * 0.3;
    if h < -0.3 {
        return mix(
            Color::srgb(0.38, 0.33, 0.24),
            Color::srgb(0.28, 0.27, 0.22),
            -h / 1.7,
        );
    }
    if h > 2.0 {
        let rock = mix(
            Color::srgb(0.40, 0.38, 0.35),
            Color::srgb(0.55, 0.52, 0.47),
            n,
        );
        // Moss on the higher shelves of the rim.
        return if h > 9.0 && n > 0.45 {
            mix(rock, grass_dark, 0.6)
        } else {
            rock
        };
    }
    let mut c = mix(grass_dark, grass_light, n);
    if noise(p.x, p.y, 31.0, 11) > 0.68 {
        c = mix(c, Color::srgb(0.58, 0.60, 0.26), 0.35);
    }
    c
}

pub fn ground() -> Mesh {
    let cells = ((SIZE as f32 + 2.0 * MARGIN) / CELL) as i32;
    let mut vertices = Vec::with_capacity((cells * cells * 6) as usize);
    let corner = |i: i32, j: i32| {
        let p = Vec2::new(-MARGIN + i as f32 * CELL, -MARGIN + j as f32 * CELL);
        to_world(p.x, p.y, height(p))
    };
    for j in 0..cells {
        for i in 0..cells {
            let (a, b, c, d) = (
                corner(i, j),
                corner(i + 1, j),
                corner(i + 1, j + 1),
                corner(i, j + 1),
            );
            // Counter-clockwise seen from above, so faces point up.
            for tri in [[a, b, c], [a, c, d]] {
                let center = (tri[0] + tri[1] + tri[2]) / 3.0;
                let color = ground_color(super::to_map(center), center.y);
                vertices.extend(tri.map(|p| (p, color)));
            }
        }
    }
    triangles(vertices)
}

/// Cobbled roads: a dark kerb under a lighter, slightly varied surface.
pub fn lanes() -> Mesh {
    let mut vertices = Vec::new();
    for lane in Lane::ALL {
        let path = map::lane_units(lane);
        for w in path.windows(2) {
            ribbon(
                &mut vertices,
                v(w[0]),
                v(w[1]),
                LANE_HALF_WIDTH + 0.7,
                0.04,
                |_| Color::srgb(0.42, 0.36, 0.27),
            );
            ribbon(
                &mut vertices,
                v(w[0]),
                v(w[1]),
                LANE_HALF_WIDTH,
                0.08,
                road_color,
            );
        }
        for &p in &path[1..path.len() - 1] {
            disc(
                &mut vertices,
                v(p),
                LANE_HALF_WIDTH + 0.7,
                0.045,
                Color::srgb(0.42, 0.36, 0.27),
            );
            disc(
                &mut vertices,
                v(p),
                LANE_HALF_WIDTH,
                0.085,
                road_color(v(p)),
            );
        }
    }
    triangles(vertices)
}

fn road_color(p: Vec2) -> Color {
    let n = hash((p.x * 0.7) as i32, (p.y * 0.7) as i32, 21);
    mix(
        Color::srgb(0.70, 0.62, 0.46),
        Color::srgb(0.80, 0.72, 0.55),
        n,
    )
}

/// Road surface height: dry ground, or the bridge deck over the river.
pub fn road_height(p: Vec2) -> f32 {
    height(p).max(0.0)
}

/// A strip from `a` to `b`, cut in 2-unit tiles that follow the ground.
fn ribbon(
    out: &mut Vec<(Vec3, Color)>,
    a: Vec2,
    b: Vec2,
    half: f32,
    lift: f32,
    color: impl Fn(Vec2) -> Color,
) {
    let dir = (b - a).normalize_or_zero();
    let side = Vec2::new(-dir.y, dir.x) * half;
    let steps = ((b - a).length() / 2.0).ceil().max(1.0) as i32;
    for s in 0..steps {
        let (p0, p1) = (
            a.lerp(b, s as f32 / steps as f32),
            a.lerp(b, (s + 1) as f32 / steps as f32),
        );
        let mid = (p0 + p1) / 2.0;
        if river_distance(mid) < RIVER_HALF_WIDTH as f32 + 2.5 {
            continue; // The bridge deck replaces the road over water.
        }
        let at = |p: Vec2| to_world(p.x, p.y, road_height(p) + lift);
        let (q0, q1, q2, q3) = (at(p0 - side), at(p0 + side), at(p1 + side), at(p1 - side));
        let c = color(mid);
        out.extend([(q0, c), (q2, c), (q1, c), (q0, c), (q3, c), (q2, c)]);
    }
}

fn disc(out: &mut Vec<(Vec3, Color)>, center: Vec2, r: f32, lift: f32, color: Color) {
    let at = |p: Vec2| to_world(p.x, p.y, road_height(p) + lift);
    for i in 0..12 {
        let a0 = i as f32 / 12.0 * std::f32::consts::TAU;
        let a1 = (i + 1) as f32 / 12.0 * std::f32::consts::TAU;
        let p0 = center + Vec2::new(a0.cos(), a0.sin()) * r;
        let p1 = center + Vec2::new(a1.cos(), a1.sin()) * r;
        out.extend([(at(center), color), (at(p0), color), (at(p1), color)]);
    }
}

/// The river surface: a flat translucent ribbon, lighter along its centre.
pub fn water() -> Mesh {
    let mut vertices = Vec::new();
    let half = RIVER_HALF_WIDTH as f32 + 2.2;
    for w in RIVER.windows(2) {
        let (a, b) = (v(w[0]), v(w[1]));
        let dir = (b - a).normalize_or_zero();
        let side = Vec2::new(-dir.y, dir.x);
        let steps = ((b - a).length() / 3.0).ceil() as i32;
        for s in 0..steps {
            let (p0, p1) = (
                a.lerp(b, s as f32 / steps as f32),
                a.lerp(b, (s + 1) as f32 / steps as f32),
            );
            for (k0, k1) in [(-1.0, -0.35), (-0.35, 0.35), (0.35, 1.0)] {
                let shade = if k0 < -0.5 || k1 > 0.5 {
                    Color::srgb(0.75, 0.9, 0.95)
                } else {
                    Color::WHITE
                };
                let at = |p: Vec2, k: f32| {
                    to_world(
                        (p + side * half * k).x,
                        (p + side * half * k).y,
                        WATER_LEVEL,
                    )
                };
                let (q0, q1, q2, q3) = (at(p0, k0), at(p0, k1), at(p1, k1), at(p1, k0));
                out_quad(&mut vertices, [q0, q1, q2, q3], shade);
            }
        }
    }
    triangles(vertices)
}

fn out_quad(out: &mut Vec<(Vec3, Color)>, [a, b, c, d]: [Vec3; 4], color: Color) {
    out.extend([
        (a, color),
        (c, color),
        (b, color),
        (a, color),
        (d, color),
        (c, color),
    ]);
}
