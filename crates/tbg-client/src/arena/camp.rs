//! Fence runs, brazier pedestals, crates, barrels, and a fallen log.

use super::{trees, Part};
use crate::art::{mesh::Shape::*, palette::*};
use bevy::prelude::*;

pub(super) const BRAZIERS: [(f32, f32); 8] = [
    (1.9, -2.2),
    (14.1, -2.2),
    (1.9, 2.4),
    (14.1, 2.4),
    (-2.1, -1.5),
    (18.1, -1.5),
    (-1.9, 1.7),
    (17.9, 1.7),
];

pub(super) fn fences(parts: &mut Vec<Part>) {
    for x in [3.9, 12.1] {
        for z in [-2.5, 2.5] {
            for dx in [-1.05, 0.0, 1.05] {
                let at = Vec3::new(x + dx, 0.0, z);
                trees::ao(parts, at, 0.16);
                parts.push((
                    Cyl(0.095, 0.67),
                    TIMBER,
                    Transform::from_translation(at + Vec3::Y * 0.31),
                ));
                parts.push((
                    Cyl(0.105, 0.04),
                    WOOD,
                    Transform::from_translation(at + Vec3::Y * 0.66),
                ));
            }
            for y in [0.23, 0.48] {
                parts.push((Box(2.1, 0.09, 0.075), WOOD, Transform::from_xyz(x, y, z)));
            }
        }
    }
    for (x, z) in BRAZIERS {
        trees::ao(parts, Vec3::new(x, 0.0, z), 0.41);
        parts.push((Box(0.53, 0.18, 0.53), ROCK, Transform::from_xyz(x, 0.07, z)));
        parts.push((
            Box(0.41, 0.30, 0.41),
            STONE,
            Transform::from_xyz(x, 0.26, z),
        ));
        parts.push((
            Frustum(0.25, 0.12, 0.18),
            IRON,
            Transform::from_xyz(x, 0.47, z),
        ));
        for dx in [-0.16, 0.16] {
            parts.push((
                Box(0.04, 0.21, 0.04),
                IRON,
                Transform::from_xyz(x + dx, 0.54, z),
            ));
        }
    }
}

pub(super) fn supplies(parts: &mut Vec<Part>) {
    for (x, direction) in [(-2.2, 1.0), (18.2, -1.0)] {
        crate_box(parts, Vec3::new(x, 0.0, -0.85), 0.44);
        crate_box(parts, Vec3::new(x + direction * 0.30, 0.0, 1.20), 0.36);
        barrel(parts, Vec3::new(x - direction * 0.33, 0.0, 1.65), 0.42);
    }
    barrel(parts, Vec3::new(13.6, 0.0, 4.1), 0.35);
}

fn crate_box(parts: &mut Vec<Part>, at: Vec3, size: f32) {
    trees::ao(parts, at, size);
    parts.push((
        Box(size, size, size),
        WOOD,
        Transform::from_translation(at + Vec3::Y * size * 0.5),
    ));
    for side in [-1.0, 1.0] {
        for x in [-0.40, 0.40] {
            parts.push((
                Box(size * 0.12, size, size * 0.11),
                TIMBER,
                Transform::from_translation(
                    at + Vec3::new(x * size, size * 0.5, side * size * 0.5),
                ),
            ));
        }
        parts.push((
            Box(size * 0.12, size * 1.15, size * 0.10),
            TIMBER,
            Transform::from_translation(at + Vec3::new(0.0, size * 0.5, side * size * 0.5))
                .with_rotation(Quat::from_rotation_z(-0.72)),
        ));
    }
}

fn barrel(parts: &mut Vec<Part>, at: Vec3, s: f32) {
    trees::ao(parts, at, s * 0.5);
    parts.push((
        Cyl(s * 0.40, s),
        WOOD,
        Transform::from_translation(at + Vec3::Y * s * 0.5),
    ));
    for y in [0.16, 0.8] {
        parts.push((
            Cyl(s * 0.415, s * 0.07),
            IRON,
            Transform::from_translation(at + Vec3::Y * s * y),
        ));
    }
    parts.push((
        Cyl(s * 0.32, 0.012),
        TIMBER,
        Transform::from_translation(at + Vec3::Y * (s + 0.01)),
    ));
}

pub(super) fn log(parts: &mut Vec<Part>, at: Vec3) {
    trees::ao(parts, at, 0.55);
    let rot = Quat::from_euler(EulerRot::XYZ, 0.0, -0.4, std::f32::consts::FRAC_PI_2);
    parts.push((
        Cyl(0.19, 1.25),
        BARK,
        Transform::from_translation(at + Vec3::Y * 0.18).with_rotation(rot),
    ));
    for side in [-1.0, 1.0] {
        parts.push((
            Cyl(0.145, 0.012),
            TIMBER,
            Transform::from_translation(at + Vec3::Y * 0.18 + rot * Vec3::Y * 0.63 * side)
                .with_rotation(rot),
        ));
    }
    parts.push((
        Cyl(0.07, 0.30),
        BARK,
        Transform::from_translation(at + Vec3::new(0.1, 0.40, 0.0))
            .with_rotation(Quat::from_rotation_z(-0.6)),
    ));
}
