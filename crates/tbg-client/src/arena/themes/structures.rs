//! Theme-specific lane fittings and Stronghold facades with the existing damage thresholds.

use super::super::{
    camp::BRAZIERS,
    painted,
    theme::{rgb, Theme},
    Part,
};
use crate::art::{
    mesh::Shape::*,
    palette::{ALLY, ENEMY, LINEN},
};
use bevy::prelude::*;

pub(super) fn fittings(parts: &mut Vec<Part>, theme: Theme) {
    let p = theme.palette();
    for x in [3.9, 12.1] {
        for z in [-2.5, 2.5] {
            for dx in [-1.05, 0.0, 1.05] {
                let shape = match theme {
                    Theme::Tundra => Frustum(0.07, 0.15, 0.7),
                    Theme::Volcanic => Box(0.18, 0.65, 0.18),
                    Theme::Ruins => Cyl(0.13, 0.65),
                    _ => Frustum(0.06, 0.14, 0.7),
                };
                parts.push((shape, p.stone, Transform::from_xyz(x + dx, 0.31, z)));
                parts.push((Ball(0.12), p.accent, Transform::from_xyz(x + dx, 0.66, z)));
            }
            match theme {
                Theme::Tundra => {
                    parts.push((
                        Box(2.1, 0.18, 0.13),
                        p.accent,
                        Transform::from_xyz(x, 0.4, z),
                    ));
                    parts.push((
                        Box(2.1, 0.05, 0.19),
                        rgb(0xe9f5f6),
                        Transform::from_xyz(x, 0.52, z),
                    ));
                }
                Theme::Ruins => {
                    parts.push((
                        Box(2.1, 0.16, 0.2),
                        p.stone,
                        Transform::from_xyz(x, 0.53, z),
                    ));
                    for dx in [-0.7, -0.35, 0.35, 0.7] {
                        parts.push((
                            Box(0.11, 0.35, 0.14),
                            p.dark,
                            Transform::from_xyz(x + dx, 0.3, z),
                        ));
                    }
                }
                _ => {
                    for y in [0.22, 0.47] {
                        parts.push((Box(2.1, 0.055, 0.06), p.wood, Transform::from_xyz(x, y, z)));
                    }
                }
            }
        }
    }
    for (x, z) in BRAZIERS {
        let (base, shaft, bowl) = match theme {
            Theme::Tundra => (
                Cyl(0.3, 0.15),
                Frustum(0.13, 0.21, 0.32),
                Frustum(0.25, 0.13, 0.12),
            ),
            Theme::Volcanic => (
                Box(0.58, 0.16, 0.58),
                Box(0.32, 0.3, 0.32),
                Ring(0.08, 0.23, 360.0),
            ),
            Theme::Ruins => (
                Box(0.59, 0.15, 0.59),
                Cyl(0.16, 0.3),
                Frustum(0.32, 0.16, 0.15),
            ),
            _ => (
                Cyl(0.31, 0.15),
                Frustum(0.1, 0.2, 0.3),
                Ring(0.045, 0.22, 360.0),
            ),
        };
        parts.extend([
            (base, p.dark, Transform::from_xyz(x, 0.07, z)),
            (shaft, p.stone, Transform::from_xyz(x, 0.28, z)),
            (bowl, p.accent, Transform::from_xyz(x, 0.49, z)),
        ]);
    }
}

pub(in crate::arena) fn tower(theme: Theme, own: bool) -> [(Mesh, f32); 3] {
    let p = theme.palette();
    let banner = if own { ALLY } else { ENEMY };
    let mut base = vec![
        (
            Box(1.8, 0.18, 1.8),
            p.dark,
            Transform::from_xyz(0.0, 0.07, 0.0),
        ),
        (
            Box(1.55, 0.15, 1.55),
            p.stone,
            Transform::from_xyz(0.0, 0.23, 0.0),
        ),
    ];
    let (body, middle) = match theme {
        Theme::Tundra => (Cyl(0.73, 0.68), Cyl(0.69, 0.86)),
        Theme::Volcanic => (Box(1.4, 0.68, 1.4), Frustum(0.65, 0.77, 0.86)),
        Theme::Ruins => (Box(1.4, 0.68, 1.4), Box(1.25, 0.86, 1.25)),
        _ => (Frustum(0.62, 0.78, 0.68), Cyl(0.62, 0.86)),
    };
    base.push((body, p.stone, Transform::from_xyz(0.0, 0.61, 0.0)));
    base.push((
        Box(0.34, 0.49, 0.055),
        p.dark,
        Transform::from_xyz(0.0, 0.55, 0.73),
    ));
    let mut mid = vec![
        (middle, p.stone, Transform::from_xyz(0.0, 1.38, 0.0)),
        (
            Box(0.45, 0.65, 0.04),
            banner,
            Transform::from_xyz(0.0, 1.29, 0.72),
        ),
        (
            Box(0.13, 0.13, 0.045),
            LINEN,
            Transform::from_xyz(0.0, 1.33, 0.75).with_rotation(Quat::from_rotation_z(0.785)),
        ),
    ];
    let mut crown = Vec::new();
    match theme {
        Theme::Tundra => {
            base.push((
                Box(1.6, 0.055, 1.6),
                rgb(0xe7f3f5),
                Transform::from_xyz(0.0, 0.33, 0.0),
            ));
            crown.extend([
                (Cyl(0.82, 0.17), p.dark, Transform::from_xyz(0.0, 1.88, 0.0)),
                (
                    Cone(0.96, 0.6),
                    p.accent,
                    Transform::from_xyz(0.0, 2.23, 0.0),
                ),
                (
                    Cone(0.86, 0.42),
                    rgb(0xf0f7f7),
                    Transform::from_xyz(0.0, 2.33, 0.0),
                ),
            ]);
        }
        Theme::Volcanic => {
            for side in [-1.0, 1.0] {
                mid.push((
                    Box(0.16, 0.85, 0.2),
                    p.dark,
                    Transform::from_xyz(side * 0.62, 1.38, 0.62),
                ));
                crown.push((
                    Box(0.28, 0.62, 0.35),
                    p.dark,
                    Transform::from_xyz(side * 0.53, 2.07, -0.45),
                ));
                crown.push((
                    Box(0.18, 0.06, 0.24),
                    p.accent,
                    Transform::from_xyz(side * 0.53, 2.39, -0.45),
                ));
            }
            crown.push((
                Box(1.65, 0.17, 1.65),
                p.dark,
                Transform::from_xyz(0.0, 1.9, 0.0),
            ));
            crown.push((
                Frustum(0.62, 0.75, 0.22),
                p.accent,
                Transform::from_xyz(0.0, 2.08, 0.0),
            ));
        }
        Theme::Ruins => {
            for x in [-0.55, 0.55] {
                for z in [-0.55, 0.55] {
                    mid.push((Cyl(0.13, 0.85), p.dark, Transform::from_xyz(x, 1.38, z)));
                }
            }
            crown.extend([
                (
                    Box(1.65, 0.2, 1.65),
                    p.stone,
                    Transform::from_xyz(0.0, 1.9, 0.0),
                ),
                (
                    Box(1.38, 0.18, 1.38),
                    p.dark,
                    Transform::from_xyz(0.0, 2.08, 0.0),
                ),
                (
                    Frustum(0.15, 0.72, 0.37),
                    p.stone,
                    Transform::from_xyz(0.0, 2.34, 0.0),
                ),
            ]);
        }
        _ => {
            for y in [1.03, 1.74] {
                mid.push((
                    Ring(0.07, 0.64, 360.0),
                    p.dark,
                    Transform::from_xyz(0.0, y, 0.0),
                ));
            }
            crown.extend([
                (Cyl(0.82, 0.18), p.dark, Transform::from_xyz(0.0, 1.9, 0.0)),
                (
                    Cone(0.75, 0.37),
                    p.stone,
                    Transform::from_xyz(0.0, 2.16, 0.0),
                ),
                (
                    Ring(0.055, 0.23, 360.0),
                    p.accent,
                    Transform::from_xyz(0.0, 2.43, 0.0).with_rotation(Quat::from_rotation_x(1.57)),
                ),
            ]);
        }
    }
    crown.push((Cyl(0.03, 0.9), p.wood, Transform::from_xyz(0.0, 2.3, 0.0)));
    [
        (painted(base), -1.0),
        (painted(mid), 1.0 / 3.0),
        (painted(crown), 2.0 / 3.0),
    ]
}
