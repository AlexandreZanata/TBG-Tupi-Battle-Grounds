//! Faceted themed water, molten channels, broken foam and offshore silhouettes.

use super::{shore_distance, LEVEL};
use crate::{
    arena::{
        geometry::{outline, Surface, SEGMENTS},
        painted,
        theme::{rgb, Theme},
        Part,
    },
    art::mesh::{self, Shape::*},
};
use bevy::prelude::*;
use tbg_core::Pcg32;

pub(super) fn sea(theme: Theme) -> Mesh {
    let p = theme.palette();
    let base = p.sea.to_srgba();
    let near = p.shore[2].to_srgba();
    let mut surface = Surface::default();
    let mut rng = Pcg32::new(0x05ea_f4ce, 9);
    let point = |x: usize, z: usize| {
        let (x, z) = (x as f32, z as f32);
        Vec3::new(
            -16.0 + x * 0.75 + (x * 1.7 + z * 2.3).sin() * 0.13,
            LEVEL + (x * 0.6 + z * 0.8).sin() * 0.006,
            -20.0 + z * (32.0 / 48.0) + (x * 2.1 - z * 1.3).cos() * 0.11,
        )
    };
    for z in 0..48 {
        for x in 0..64 {
            let [a, b, c, d] = [
                point(x, z),
                point(x, z + 1),
                point(x + 1, z + 1),
                point(x + 1, z),
            ];
            if [a, b, c, d].iter().all(|v| shore_distance(v.x, v.z) < 0.93) {
                continue;
            }
            let coast = ((1.65 - shore_distance(a.x, a.z)) / 0.65).clamp(0.0, 1.0);
            let noise = rng.below(1000) as f32 / 1000.0 - 0.5;
            let variation = if theme == Theme::Volcanic {
                0.12
            } else {
                0.045
            };
            let color = |k: f32| rgb_mix(base, near, coast * 0.65, noise * variation + k);
            surface.triangle(a, b, c, color(0.0));
            surface.triangle(a, c, d, color(0.014));
        }
    }
    if theme == Theme::Volcanic {
        molten_channels(&mut surface);
    }
    mesh::merge(vec![
        (
            painted(vec![(
                Box(180.0, 0.02, 180.0),
                p.sea,
                Transform::from_xyz(8.0, LEVEL - 0.04, 0.0),
            )]),
            Transform::default(),
        ),
        (surface.finish(), Transform::default()),
    ])
    .unwrap_or_else(|| Cuboid::new(0.01, 0.01, 0.01).into())
}

fn rgb_mix(a: Srgba, b: Srgba, t: f32, delta: f32) -> Color {
    Color::srgb(
        (a.red + (b.red - a.red) * t + delta * 0.35).clamp(0.0, 1.0),
        (a.green + (b.green - a.green) * t + delta).clamp(0.0, 1.0),
        (a.blue + (b.blue - a.blue) * t + delta * 0.8).clamp(0.0, 1.0),
    )
}

pub(super) fn coast(theme: Theme) -> Mesh {
    let rim = outline();
    let center = Vec3::new(8.0, 0.0, 0.0);
    let p = theme.palette();
    let mut surface = Surface::default();
    for (band, (inside, outside, color)) in [(0.99, 1.05, p.shore[1]), (1.05, 1.12, p.shore[2])]
        .into_iter()
        .enumerate()
    {
        for i in 0..SEGMENTS {
            let j = (i + 1) % SEGMENTS;
            let point = |k: usize, s: f32| {
                let mut at = center + (rim[k] - center) * s;
                at.y = LEVEL + 0.027 + band as f32 * 0.002;
                at
            };
            surface.quad(
                [
                    point(i, inside),
                    point(j, inside),
                    point(j, outside),
                    point(i, outside),
                ],
                color,
            );
            if band == 0 && i % 6 != 2 {
                surface.quad(
                    [
                        point(i, 1.012) + Vec3::Y * 0.012,
                        point(j, 1.015) + Vec3::Y * 0.012,
                        point(j, 1.024) + Vec3::Y * 0.012,
                        point(i, 1.022) + Vec3::Y * 0.012,
                    ],
                    p.shore[0],
                );
            }
        }
    }
    for (i, (at, r)) in sites().into_iter().enumerate() {
        ripple(
            &mut surface,
            at,
            Vec2::new(r * 1.35, r * 0.95),
            p.shore[0],
            i,
        );
    }
    // Short irregular crests stay on the open surface, never across the lane.
    for i in 0..85 {
        let x = -12.0 + (i * 47 % 101) as f32 * 0.4;
        let z = -12.0 + (i * 31 % 89) as f32 * 0.26;
        if shore_distance(x, z) < 1.2 {
            continue;
        }
        let a = Vec3::new(x, LEVEL + 0.046, z);
        let w = 0.17 + (i % 4) as f32 * 0.10;
        surface.quad(
            [
                a,
                a + Vec3::new(w * 0.4, 0.0, 0.025),
                a + Vec3::new(w, 0.0, 0.0),
                a + Vec3::new(w * 0.4, 0.0, -0.018),
            ],
            p.shore[1],
        );
    }
    surface.finish()
}

fn ripple(s: &mut Surface, at: Vec3, r: Vec2, color: Color, phase: usize) {
    for i in 0..18 {
        if (i + phase) % 6 == 2 {
            continue;
        }
        let point = |j: usize, k: f32| {
            let a = j as f32 * std::f32::consts::TAU / 18.0;
            at + Vec3::new(a.cos() * r.x * k, 0.035, a.sin() * r.y * k)
        };
        s.quad(
            [
                point(i, 1.0),
                point(i + 1, 1.0),
                point(i + 1, 1.025),
                point(i, 1.025),
            ],
            color,
        );
    }
}

pub(super) fn rocks(theme: Theme) -> Mesh {
    if theme == Theme::Tundra {
        return super::ice::mesh(sites());
    }
    let p = theme.palette();
    let mut parts = Vec::new();
    for (i, (at, r)) in sites().into_iter().enumerate() {
        let yaw = Quat::from_rotation_y(i as f32 * 1.31);
        match theme {
            Theme::Volcanic => {
                let h = 0.5 + (i % 4) as f32 * 0.55;
                parts.push((
                    Frustum(r * 0.70, r, h),
                    p.dark,
                    Transform::from_translation(at + Vec3::Y * (h * 0.5 - 0.12)).with_rotation(yaw),
                ));
                parts.push((
                    Cyl(r * 0.70, 0.06),
                    p.stone,
                    Transform::from_translation(at + Vec3::Y * (h - 0.09)).with_rotation(yaw),
                ));
                for side in [-1.0, 1.0] {
                    parts.push((
                        Box(0.055, h * 0.72, 0.035),
                        rgb(0xf78226),
                        Transform::from_translation(
                            at + Vec3::new(side * r * 0.42, h * 0.36, r * 0.65),
                        ),
                    ));
                }
            }
            Theme::Ruins => {
                rock(
                    &mut parts,
                    at + Vec3::Y * 0.17,
                    Vec3::new(r, r * 0.55, r * 0.75),
                    p.cliff[1],
                );
                let h = 0.65 + (i % 4) as f32 * 0.5;
                parts.push((
                    Box(r * 0.65, h, r * 0.62),
                    p.stone,
                    Transform::from_translation(at + Vec3::Y * (h * 0.5 + 0.15)).with_rotation(
                        yaw * Quat::from_rotation_z(if i % 3 == 0 { 0.22 } else { 0.0 }),
                    ),
                ));
                parts.push((
                    Box(r * 1.05, 0.19, r * 0.85),
                    rgb(0xeee3c7),
                    Transform::from_translation(at + Vec3::Y * (h + 0.19)).with_rotation(yaw),
                ));
                rock(
                    &mut parts,
                    at + Vec3::new(0.15, h + 0.28, 0.0),
                    Vec3::new(r * 0.45, 0.10, r * 0.28),
                    rgb(0x729940),
                );
            }
            _ => {
                rock(
                    &mut parts,
                    at + Vec3::Y * r * 0.36,
                    Vec3::new(r * 0.9, r * 0.88, r * 0.72),
                    p.cliff[1],
                );
                rock(
                    &mut parts,
                    at + Vec3::Y * r * 0.82,
                    Vec3::new(r * 0.70, r * 0.12, r * 0.58),
                    rgb(0xa9bdcc),
                );
            }
        }
    }
    painted(parts)
}

fn rock(parts: &mut Vec<Part>, at: Vec3, scale: Vec3, color: Color) {
    parts.push((
        Ball(1.0),
        color,
        Transform::from_translation(at)
            .with_scale(scale)
            .with_rotation(Quat::from_rotation_y(at.x * 1.7)),
    ));
}

fn sites() -> Vec<(Vec3, f32)> {
    (0..18)
        .map(|i| {
            let a = (i as f32 + 0.35) * std::f32::consts::TAU / 18.0;
            (
                Vec3::new(
                    8.0 + a.cos() * (13.5 + (i % 3) as f32 * 0.55),
                    LEVEL,
                    a.sin() * (7.1 + (i % 4) as f32 * 0.5),
                ),
                0.42 + (i % 4) as f32 * 0.25,
            )
        })
        .collect()
}

fn molten_channels(surface: &mut Surface) {
    let mut rng = Pcg32::new(0x1a7a_7e17, 13);
    for _ in 0..85 {
        let x = -15.0 + rng.below(4500) as f32 * 0.01;
        let z = -18.0 + rng.below(2900) as f32 * 0.01;
        if shore_distance(x, z) < 1.05 {
            continue;
        }
        let angle = rng.below(628) as f32 * 0.01;
        let mut a = Vec3::new(x, LEVEL + 0.026, z);
        for j in 0..5 {
            let angle = angle + (j as f32 * 1.8).sin() * 0.42;
            let length = 0.6 + rng.below(70) as f32 * 0.01;
            let b = a + Vec3::new(angle.cos() * length, 0.0, angle.sin() * length);
            let d = (b - a).normalize_or_zero();
            let n = Vec3::new(-d.z, 0.0, d.x);
            for (w, lift, color) in [(0.13, 0.0, rgb(0xf59625)), (0.045, 0.004, rgb(0xffd464))] {
                let up = Vec3::Y * lift;
                surface.quad(
                    [
                        a + n * w + up,
                        b + n * w + up,
                        b - n * w + up,
                        a - n * w + up,
                    ],
                    color,
                );
            }
            a = b;
        }
    }
}
