//! Theme-specific lane fittings. The Strongholds are the kind-based D-46
//! towers in every arena (D-55), so the themed tower facades were dropped.

use super::super::{
    camp::BRAZIERS,
    theme::{rgb, Theme},
    Part,
};
use crate::art::mesh::Shape::*;
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
