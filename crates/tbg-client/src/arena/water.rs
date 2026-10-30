//! Turquoise sea, shallow coastal bands, and ten offshore rocks with foam rings.

use super::{
    geometry::{outline, Surface, SEGMENTS},
    painted,
    theme::Theme,
};
use crate::art::{mesh::Shape::*, palette::*};
use bevy::prelude::*;

pub(super) fn sea(theme: Theme) -> Mesh {
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

pub(super) fn coast(theme: Theme) -> Mesh {
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

pub(super) fn rocks(theme: Theme) -> Mesh {
    if theme != Theme::Forest {
        return surroundings(theme);
    }
    let mut parts = Vec::new();
    for i in 0..10 {
        let a = (i as f32 + 0.35) * std::f32::consts::TAU / 10.0;
        let p = Vec3::new(8.0 + a.cos() * 13.0, -1.30, a.sin() * 6.6);
        let s = 0.35 + (i % 3) as f32 * 0.16;
        parts.push((
            Ball(1.0),
            ROCK,
            Transform::from_translation(p)
                .with_scale(Vec3::new(s, s * 1.3, s * 0.9))
                .with_rotation(Quat::from_rotation_y(a)),
        ));
        parts.push((
            Ring(0.028, s * 1.18, 360.0),
            Color::srgb(0.76, 0.93, 0.89),
            Transform::from_translation(p - Vec3::Y * 0.065).with_scale(Vec3::new(1.0, 0.3, 0.9)),
        ));
    }
    painted(parts)
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
