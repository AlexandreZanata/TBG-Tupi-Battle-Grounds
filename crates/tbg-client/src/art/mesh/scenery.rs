//! Broad beveled planes and seeded organic silhouettes for environment parts.

use bevy::{asset::RenderAssetUsages, prelude::*, render::render_resource::PrimitiveTopology};

fn noise(seed: u32, i: u32) -> f32 {
    let mut n = seed.wrapping_add(i.wrapping_mul(0x9e37_79b9));
    n = (n ^ (n >> 16)).wrapping_mul(0x7feb_352d);
    n = (n ^ (n >> 15)).wrapping_mul(0x846c_a68b);
    ((n ^ (n >> 16)) & 65535) as f32 / 65535.0
}

fn face(positions: &mut Vec<[f32; 3]>, vertices: &[Vec3]) {
    let middle = vertices.iter().copied().sum::<Vec3>() / vertices.len() as f32;
    let outward = (vertices[1] - vertices[0])
        .cross(vertices[2] - vertices[0])
        .dot(middle)
        >= 0.0;
    for i in 1..vertices.len() - 1 {
        let (b, c) = if outward {
            (vertices[i], vertices[i + 1])
        } else {
            (vertices[i + 1], vertices[i])
        };
        positions.extend([vertices[0].to_array(), b.to_array(), c.to_array()]);
    }
}

fn finish(positions: Vec<[f32; 3]>) -> Mesh {
    Mesh::new(
        PrimitiveTopology::TriangleList,
        RenderAssetUsages::default(),
    )
    .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, positions)
}

pub(super) fn bevel(size: Vec3, edge: f32) -> Mesh {
    let h = size * 0.5;
    let b = edge.clamp(0.001, h.min_element() * 0.8);
    let inner = h - Vec3::splat(b);
    let mut positions = Vec::with_capacity(132);
    for axis in 0..3 {
        let u = (axis + 1) % 3;
        let v = (axis + 2) % 3;
        for sign in [-1.0, 1.0] {
            let corners = [(-1.0, -1.0), (1.0, -1.0), (1.0, 1.0), (-1.0, 1.0)].map(|(a, c)| {
                let mut p = Vec3::ZERO;
                p[axis] = h[axis] * sign;
                p[u] = inner[u] * a;
                p[v] = inner[v] * c;
                p
            });
            face(&mut positions, &corners);
        }
        for su in [-1.0, 1.0] {
            for sv in [-1.0, 1.0] {
                let corners =
                    [(-1.0, false), (1.0, false), (1.0, true), (-1.0, true)].map(|(s, other)| {
                        let mut p = Vec3::ZERO;
                        p[axis] = inner[axis] * s;
                        p[u] = if other { inner[u] } else { h[u] } * su;
                        p[v] = if other { h[v] } else { inner[v] } * sv;
                        p
                    });
                face(&mut positions, &corners);
            }
        }
    }
    for x in [-1.0, 1.0] {
        for y in [-1.0, 1.0] {
            for z in [-1.0, 1.0] {
                let signs = Vec3::new(x, y, z);
                face(
                    &mut positions,
                    &[
                        Vec3::new(h.x, inner.y, inner.z) * signs,
                        Vec3::new(inner.x, h.y, inner.z) * signs,
                        Vec3::new(inner.x, inner.y, h.z) * signs,
                    ],
                );
            }
        }
    }
    finish(positions)
}

pub(super) fn rock(radius: f32, seed: u32, snow: bool) -> Mesh {
    const N: usize = 9;
    let point = |i: usize, band: usize| {
        let a = i as f32 * std::f32::consts::TAU / N as f32 + noise(seed, 40) * 0.7;
        let (r, y) = [(0.68, -0.5), (1.0, -0.15), (0.85, 0.29), (0.50, 0.57)][band];
        let r = r * (0.86 + noise(seed, i as u32) * 0.25);
        Vec3::new(
            a.cos() * r,
            y + (noise(seed, i as u32 + 12) - 0.5) * 0.10,
            a.sin() * r,
        ) * radius
    };
    let mut positions = Vec::new();
    let start = if snow { 2 } else { 0 };
    for band in start..3 {
        for i in 0..N {
            let j = (i + 1) % N;
            let up = if snow {
                Vec3::Y * radius * 0.012
            } else {
                Vec3::ZERO
            };
            face(
                &mut positions,
                &[
                    point(i, band) + up,
                    point(j, band) + up,
                    point(j, band + 1) + up,
                    point(i, band + 1) + up,
                ],
            );
        }
    }
    if snow {
        for i in 0..N {
            let j = (i + 1) % N;
            let skirt = |k| {
                point(k, 1).lerp(point(k, 2), 0.70 + noise(seed, k as u32 + 60) * 0.20)
                    + Vec3::Y * radius * 0.013
            };
            face(
                &mut positions,
                &[
                    skirt(i),
                    skirt(j),
                    point(j, 2) + Vec3::Y * radius * 0.012,
                    point(i, 2) + Vec3::Y * radius * 0.012,
                ],
            );
        }
    }
    for i in 0..N {
        let j = (i + 1) % N;
        let lift = if snow { 0.012 } else { 0.0 };
        face(
            &mut positions,
            &[
                Vec3::Y * radius * (0.56 + lift),
                point(i, 3) + Vec3::Y * radius * lift,
                point(j, 3) + Vec3::Y * radius * lift,
            ],
        );
        if !snow {
            face(
                &mut positions,
                &[Vec3::NEG_Y * radius * 0.5, point(j, 0), point(i, 0)],
            );
        }
    }
    finish(positions)
}

pub(super) fn canopy(radius: f32, height: f32, seed: u32) -> Mesh {
    const N: usize = 9;
    let point = |i: usize, upper: bool| {
        let a = i as f32 * std::f32::consts::TAU / N as f32;
        let (r, y) = if upper { (0.63, -0.08) } else { (1.0, -0.5) };
        let r = radius * r * (0.88 + noise(seed, i as u32) * 0.22);
        Vec3::new(
            a.cos() * r,
            height * (y + (noise(seed, i as u32 + 17) - 0.5) * 0.16),
            a.sin() * r,
        )
    };
    let mut positions = Vec::new();
    for i in 0..N {
        let j = (i + 1) % N;
        face(
            &mut positions,
            &[
                point(i, false),
                point(j, false),
                point(j, true),
                point(i, true),
            ],
        );
        face(
            &mut positions,
            &[point(i, true), point(j, true), Vec3::Y * height * 0.5],
        );
        face(
            &mut positions,
            &[Vec3::NEG_Y * height * 0.5, point(j, false), point(i, false)],
        );
    }
    finish(positions)
}
