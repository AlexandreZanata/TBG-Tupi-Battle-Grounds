//! Layered pines, low shrubs, white flowers, and grass with baked base occlusion.

use super::Part;
use crate::art::{mesh::Shape::*, palette::*};
use bevy::prelude::*;

pub(super) fn ao(parts: &mut Vec<Part>, at: Vec3, radius: f32) {
    parts.push((
        Cyl(radius, 0.008),
        Color::srgb(0.34, 0.47, 0.15),
        Transform::from_translation(at + Vec3::Y * -0.025).with_scale(Vec3::new(1.0, 1.0, 0.72)),
    ));
}

pub(super) fn pine(parts: &mut Vec<Part>, at: Vec3, s: f32) {
    ao(parts, at, 0.49 * s);
    parts.push((
        Cyl(0.105 * s, 0.65 * s),
        BARK,
        Transform::from_translation(at + Vec3::Y * 0.30 * s),
    ));
    for (radius, height, y, color) in [
        (0.68, 1.06, 0.96, PINE),
        (0.54, 0.96, 1.48, Color::srgb(0.37, 0.55, 0.18)),
        (0.36, 0.86, 1.98, Color::srgb(0.52, 0.66, 0.23)),
    ] {
        parts.push((
            Canopy(
                radius * s,
                height * s,
                at.x.to_bits() ^ at.z.to_bits() ^ (at.z + y).to_bits(),
            ),
            color,
            Transform::from_translation(at + Vec3::Y * y * s)
                .with_rotation(Quat::from_rotation_y(at.x * 0.8)),
        ));
    }
}

pub(super) fn bush(parts: &mut Vec<Part>, at: Vec3, s: f32) {
    ao(parts, at, 0.5 * s);
    for (off, radius, color) in [
        (Vec3::new(-0.14, 0.22, 0.0), 0.4, LEAF_DARK),
        (Vec3::new(0.13, 0.30, -0.06), 0.42, LEAF),
        (Vec3::new(0.28, 0.14, 0.17), 0.25, GRASS),
    ] {
        parts.push((
            Ball(radius * s),
            color,
            Transform::from_translation(at + off * s).with_scale(Vec3::new(1.1, 0.85, 1.0)),
        ));
    }
}

pub(super) fn grass(parts: &mut Vec<Part>, at: Vec3, angle: f32) {
    for i in 0..3 {
        parts.push((
            Cone(0.045, 0.24 + i as f32 * 0.04),
            if i % 2 == 0 { LEAF } else { GRASS },
            Transform::from_translation(at + Vec3::new((i as f32 - 1.0) * 0.07, 0.09, 0.0))
                .with_rotation(Quat::from_euler(
                    EulerRot::XYZ,
                    0.0,
                    angle,
                    (i as f32 - 1.0) * 0.25,
                )),
        ));
    }
}

pub(super) fn flowers(parts: &mut Vec<Part>, at: Vec3) {
    grass(parts, at, 0.0);
    for flower in 0..3 {
        let p = at
            + Vec3::new(
                flower as f32 * 0.14 - 0.14,
                0.16,
                (flower % 2) as f32 * 0.13,
            );
        parts.push((Cyl(0.043, 0.035), GOLD, Transform::from_translation(p)));
        for petal in 0..5 {
            let a = petal as f32 * std::f32::consts::TAU / 5.0;
            parts.push((
                Cyl(0.045, 0.025),
                LINEN,
                Transform::from_translation(p + Vec3::new(a.cos() * 0.07, 0.0, a.sin() * 0.07))
                    .with_scale(Vec3::new(1.0, 0.35, 1.0)),
            ));
        }
    }
}
