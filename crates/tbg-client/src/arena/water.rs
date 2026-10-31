//! The sea around the island: the Forest's batched faceted seascape (owner-approved),
//! and each themed arena's own surface, shallows and offshore silhouettes.

use super::{
    geometry::{outline, Surface, SEGMENTS},
    painted,
    theme::Theme,
};
use crate::art::{mesh, mesh::Shape::*, palette::*};
use bevy::prelude::*;
use tbg_core::Pcg32;

pub(super) const LEVEL: f32 = -1.40;
const DEEP: Color = Color::srgb(0.045, 0.51, 0.75);
const FOAM: Color = Color::srgb(0.82, 0.96, 0.94);

pub(super) fn sea(theme: Theme) -> Mesh {
    if theme == Theme::Forest {
        forest_sea()
    } else {
        themed_sea(theme)
    }
}

pub(super) fn coast(theme: Theme) -> Mesh {
    if theme == Theme::Forest {
        forest_coast()
    } else {
        themed_coast(theme)
    }
}

pub(super) fn rocks(theme: Theme) -> Mesh {
    if theme == Theme::Forest {
        forest_rocks()
    } else {
        surroundings(theme)
    }
}

fn forest_sea() -> Mesh {
    let mut water = Surface::default();
    let mut rng = Pcg32::new(0x0cea_51de, 9);
    let point = |x: usize, z: usize| {
        let fx = x as f32;
        let fz = z as f32;
        Vec3::new(
            -16.0 + fx * 48.0 / 64.0 + (fx * 1.7 + fz * 2.3).sin() * 0.10,
            LEVEL + (fx * 0.6 + fz * 0.8).sin() * 0.006,
            -20.0 + fz * 32.0 / 48.0 + (fx * 2.1 - fz * 1.3).cos() * 0.08,
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
            // All four corners lie safely inside the original rocky footprint.
            if [a, b, c, d].iter().all(|p| shore_distance(p.x, p.z) < 0.93) {
                continue;
            }
            let near = ((1.5 - shore_distance(a.x, a.z)) / 0.48).clamp(0.0, 1.0);
            let noise = rng.below(1000) as f32 / 1000.0 - 0.5;
            let swell = (a.x * 1.8 + a.z * 2.6).sin() * 0.012 + noise * 0.035;
            let color = |shade: f32| {
                Color::srgb(
                    0.045 + near * 0.025 + swell * 0.3,
                    0.51 + near * 0.15 + swell + shade,
                    0.75 + near * 0.065 + swell * 0.7 + shade,
                )
            };
            water.triangle(a, b, c, color(0.0));
            water.triangle(a, c, d, color(0.012));
        }
    }
    // The coarse exterior only fills views beyond the detailed patch; no texture or shader pass.
    mesh::merge(vec![
        (
            painted(vec![(
                Box(180.0, 0.02, 180.0),
                DEEP,
                Transform::from_xyz(8.0, LEVEL - 0.04, 0.0),
            )]),
            Transform::default(),
        ),
        (water.finish(), Transform::default()),
    ])
    .unwrap_or_else(|| Cuboid::new(0.01, 0.01, 0.01).into())
}

fn shore_distance(x: f32, z: f32) -> f32 {
    (((x - 8.0) / 12.0).powi(2) + (z / 5.7).powi(2)).sqrt()
}

fn forest_coast() -> Mesh {
    let rim = outline();
    let center = Vec3::new(8.0, 0.0, 0.0);
    let mut surface = Surface::default();
    for (band, (inside, outside, color)) in [
        (0.99, 1.035, Color::srgb(0.22, 0.73, 0.79)),
        (1.035, 1.08, Color::srgb(0.09, 0.63, 0.76)),
        (1.08, 1.15, Color::srgb(0.055, 0.54, 0.73)),
    ]
    .into_iter()
    .enumerate()
    {
        for i in 0..SEGMENTS {
            let j = (i + 1) % SEGMENTS;
            let point = |k: usize, s: f32| {
                let mut p = center + (rim[k] - center) * s;
                p.y = LEVEL + 0.025;
                p
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
            if band == 0 && i % 5 != 1 {
                let a = point(i, 1.014 + (i as f32 * 1.4).sin() * 0.004);
                let b = point(j, 1.014);
                let c = center + (a - center) * 1.007;
                let d = center + (b - center) * 1.007;
                surface.quad(
                    [
                        a,
                        b,
                        Vec3::new(d.x, a.y + 0.008, d.z),
                        Vec3::new(c.x, a.y + 0.008, c.z),
                    ],
                    FOAM,
                );
            }
        }
    }
    for (p, s, _) in rock_sites() {
        foam_ring(&mut surface, p, Vec2::new(s, s * 0.9));
    }
    glints(&mut surface);
    surface.finish()
}

pub(super) fn foam_ring(surface: &mut Surface, center: Vec3, radius: Vec2) {
    for i in 0..20 {
        let a = i as f32 * std::f32::consts::TAU / 20.0;
        let b = (i + 1) as f32 * std::f32::consts::TAU / 20.0;
        let point = |angle: f32, scale: f32| {
            Vec3::new(
                center.x + angle.cos() * radius.x * scale,
                LEVEL + 0.035,
                center.z + angle.sin() * radius.y * scale,
            )
        };
        surface.quad(
            [
                point(a, 0.95),
                point(b, 0.95),
                point(b, 1.28),
                point(a, 1.28),
            ],
            Color::srgb(0.11, 0.67, 0.77),
        );
        if i % 5 != 2 {
            surface.quad(
                [
                    point(a, 1.12) + Vec3::Y * 0.008,
                    point(b, 1.12) + Vec3::Y * 0.008,
                    point(b, 1.22) + Vec3::Y * 0.008,
                    point(a, 1.22) + Vec3::Y * 0.008,
                ],
                FOAM,
            );
        }
    }
}

fn glints(surface: &mut Surface) {
    let mut rng = Pcg32::new(0x0c0a_57a1, 11);
    for _ in 0..280 {
        let x = -14.0 + rng.below(4400) as f32 * 0.01;
        let z = -18.0 + rng.below(2800) as f32 * 0.01;
        if shore_distance(x, z) < 1.12 {
            continue;
        }
        let width = 0.12 + rng.below(280) as f32 * 0.001;
        let p = Vec3::new(x, LEVEL + 0.05, z);
        let color = Color::srgb(0.20, 0.73, 0.86);
        surface.quad(
            [
                p,
                p + Vec3::new(width * 0.5, 0.0, 0.045),
                p + Vec3::new(width, 0.0, 0.0),
                p + Vec3::new(width * 0.45, 0.0, -0.025),
            ],
            color,
        );
    }
}

fn rock_sites() -> [(Vec3, f32, f32); 10] {
    std::array::from_fn(|i| {
        let a = (i as f32 + 0.35) * std::f32::consts::TAU / 10.0;
        (
            Vec3::new(8.0 + a.cos() * 13.0, -1.30, a.sin() * 6.6),
            0.35 + (i % 3) as f32 * 0.16,
            a,
        )
    })
}

fn forest_rocks() -> Mesh {
    painted(
        rock_sites()
            .into_iter()
            .map(|(p, s, a)| {
                (
                    Ball(1.0),
                    ROCK,
                    Transform::from_translation(p)
                        .with_scale(Vec3::new(s, s * 1.3, s * 0.9))
                        .with_rotation(Quat::from_rotation_y(a)),
                )
            })
            .collect(),
    )
}

fn themed_sea(theme: Theme) -> Mesh {
    let sea = painted(vec![(
        Box(180.0, 0.02, 180.0),
        theme.palette().sea,
        Transform::from_xyz(8.0, -1.40, 0.0),
    )]);
    if theme != Theme::Volcanic {
        return sea;
    }
    // Wide, broken currents read as molten rock without a water shader or particles.
    let mut currents = Surface::default();
    for band in 0..4 {
        let radius = 1.3 + band as f32 * 0.34;
        for i in 0..SEGMENTS {
            if (i + band * 7) % 11 < 3 {
                continue;
            }
            let point = |k: usize, width: f32| {
                let a = k as f32 * std::f32::consts::TAU / SEGMENTS as f32;
                let r = radius + (a * 5.0 + band as f32).sin() * 0.035 + width;
                Vec3::new(8.0 + a.cos() * 12.0 * r, -1.38, a.sin() * 5.7 * r)
            };
            currents.quad(
                [
                    point(i, 0.0),
                    point(i + 1, 0.0),
                    point(i + 1, 0.025),
                    point(i, 0.025),
                ],
                theme.palette().shore[1 + band % 2],
            );
        }
    }
    crate::art::mesh::merge(vec![
        (sea, Transform::default()),
        (currents.finish(), Transform::default()),
    ])
    .unwrap_or_else(|| Cuboid::new(0.01, 0.01, 0.01).into())
}

fn themed_coast(theme: Theme) -> Mesh {
    let rim = outline();
    let center = Vec3::new(8.0, 0.0, 0.0);
    let mut surface = Surface::default();
    let [foam, shallow, deep] = theme.palette().shore;
    for (inside, outside, color) in [
        (0.97, 1.045, foam),
        (1.045, 1.085, shallow),
        (1.085, 1.13, deep),
    ] {
        for i in 0..SEGMENTS {
            let j = (i + 1) % SEGMENTS;
            let point = |k: usize, s: f32| {
                let mut p = center + (rim[k] - center) * s;
                p.y = -1.375;
                p
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
        }
    }
    surface.finish()
}

/// Offshore silhouettes replace the base rocks, while retaining the island envelope.
fn surroundings(theme: Theme) -> Mesh {
    let p = theme.palette();
    let mut parts = Vec::new();
    for i in 0..14 {
        let a = (i as f32 + 0.35) * std::f32::consts::TAU / 14.0;
        let at = Vec3::new(8.0 + a.cos() * 13.4, -1.30, a.sin() * 7.0);
        let s = 0.4 + (i % 3) as f32 * 0.22;
        match theme {
            Theme::Tundra => {
                parts.push((
                    Cyl(s, 0.08),
                    p.shore[1],
                    Transform::from_translation(at).with_scale(Vec3::new(1.5, 1.0, 0.85)),
                ));
                parts.push((
                    Cone(s * 0.65, s * 1.15),
                    p.shore[0],
                    Transform::from_translation(at + Vec3::Y * s * 0.5)
                        .with_scale(Vec3::new(1.2, 1.0, 0.8)),
                ));
            }
            Theme::Volcanic => {
                parts.push((
                    Cyl(s * 0.55, s * 1.8),
                    p.dark,
                    Transform::from_translation(at + Vec3::Y * s * 0.4),
                ));
                parts.push((
                    Cyl(s * 0.57, 0.05),
                    p.stone,
                    Transform::from_translation(at + Vec3::Y * s * 1.3),
                ));
            }
            Theme::Ruins => {
                parts.push((Box(s, s * 0.25, s), p.dark, Transform::from_translation(at)));
                parts.push((
                    Cyl(s * 0.27, s * 1.3),
                    p.stone,
                    Transform::from_translation(at + Vec3::Y * s * 0.5)
                        .with_rotation(Quat::from_rotation_z(0.22)),
                ));
                parts.push((
                    Box(s * 0.7, s * 0.15, s * 0.7),
                    p.stone,
                    Transform::from_translation(at + Vec3::Y * s * 1.15),
                ));
            }
            _ => {
                parts.push((
                    Cone(s * 0.7, s * 1.5),
                    p.dark,
                    Transform::from_translation(at + Vec3::Y * s * 0.4)
                        .with_rotation(Quat::from_rotation_z(-0.15)),
                ));
                for dz in [-0.4, 0.4] {
                    parts.push((
                        Box(s * 1.7, 0.018, 0.06),
                        p.shore[0],
                        Transform::from_translation(at + Vec3::new(0.0, -0.05, dz * s))
                            .with_rotation(Quat::from_rotation_y(-0.2)),
                    ));
                }
            }
        }
    }
    // Broad, flat polygons give the surrounding surface its own material language.
    for i in 0..32 {
        let a = i as f32 * std::f32::consts::TAU / 32.0;
        let at = Vec3::new(
            8.0 + a.cos() * (15.0 + (i % 3) as f32 * 2.0),
            -1.375,
            a.sin() * (8.0 + (i % 4) as f32),
        );
        let (shape, color, scale) = match theme {
            Theme::Tundra => (Cyl(1.0, 0.02), p.shore[1], Vec3::new(1.2, 1.0, 0.55)),
            Theme::Volcanic => (Cyl(1.0, 0.025), p.dark, Vec3::new(1.1, 1.0, 0.65)),
            Theme::Ruins => (Ball(0.28), p.plant, Vec3::new(1.0, 0.12, 0.7)),
            _ => (Box(1.7, 0.02, 0.06), p.shore[1], Vec3::ONE),
        };
        parts.push((
            shape,
            color,
            Transform::from_translation(at)
                .with_scale(scale)
                .with_rotation(Quat::from_rotation_y(a)),
        ));
    }
    painted(parts)
}
