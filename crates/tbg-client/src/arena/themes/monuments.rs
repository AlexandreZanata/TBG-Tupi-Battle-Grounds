//! Shared masonry, ceremonial guardians and coastal gardens for the three new kits.

use crate::{
    arena::{theme::rgb, Part},
    art::mesh::Shape::{self, *},
};
use bevy::prelude::*;

pub(super) fn block(parts: &mut Vec<Part>, at: Vec3, size: Vec3, color: Color) {
    parts.push((
        Bevel(size.x, size.y, size.z, 0.035),
        color,
        Transform::from_translation(at),
    ));
}

pub(super) fn column(parts: &mut Vec<Part>, at: Vec3, h: f32, stone: Color, trim: Color) {
    for (size, y, color) in [
        (Vec3::new(0.85, 0.18, 0.85), 0.09, stone),
        (Vec3::new(0.67, 0.12, 0.67), 0.24, trim),
        (Vec3::new(0.63, 0.14, 0.63), h - 0.21, trim),
        (Vec3::new(0.85, 0.22, 0.85), h - 0.04, stone),
    ] {
        block(parts, at + Vec3::Y * y, size, color);
    }
    parts.push((
        Frustum(0.22, 0.29, h - 0.56),
        stone,
        Transform::from_translation(at + Vec3::Y * (h * 0.5)),
    ));
    for side in [-1.0, 1.0] {
        block(
            parts,
            at + Vec3::new(side * 0.12, h * 0.51, 0.23),
            Vec3::new(0.035, h - 0.85, 0.025),
            trim,
        );
    }
}

pub(super) fn steps(parts: &mut Vec<Part>, at: Vec3, width: f32, stone: Color) {
    for i in 0..4 {
        let level = i as f32;
        block(
            parts,
            at + Vec3::new(0.0, 0.07 + level * 0.12, -level * 0.30),
            Vec3::new(width - level * 0.18, 0.14 + level * 0.24, 0.42),
            stone,
        );
    }
}

/// A readable helmet, shoulders, shield and planted feet, rather than an abstract pillar.
pub(super) fn guardian(parts: &mut Vec<Part>, at: Vec3, s: f32, stone: Color, trim: Color) {
    let mut statue = Vec::new();
    for (size, pos, col) in [
        (Vec3::new(1.0, 0.24, 0.85), Vec3::Y * 0.12, stone),
        (
            Vec3::new(0.22, 0.72, 0.27),
            Vec3::new(-0.19, 0.65, 0.0),
            stone,
        ),
        (
            Vec3::new(0.22, 0.72, 0.27),
            Vec3::new(0.19, 0.65, 0.0),
            stone,
        ),
        (Vec3::new(0.70, 0.66, 0.40), Vec3::Y * 1.28, stone),
        (Vec3::new(0.91, 0.17, 0.48), Vec3::Y * 1.58, trim),
        (Vec3::new(0.22, 0.20, 0.24), Vec3::Y * 1.73, stone),
        (
            Vec3::new(0.19, 0.61, 0.25),
            Vec3::new(0.36, 1.25, 0.03),
            stone,
        ),
        (Vec3::new(0.34, 0.36, 0.34), Vec3::Y * 1.92, stone),
        (
            Vec3::new(0.12, 0.07, 0.06),
            Vec3::new(0.0, 1.96, 0.19),
            trim,
        ),
        (
            Vec3::new(0.41, 0.69, 0.10),
            Vec3::new(-0.35, 1.17, 0.30),
            trim,
        ),
        (
            Vec3::new(0.08, 1.74, 0.09),
            Vec3::new(0.48, 1.18, 0.17),
            trim,
        ),
    ] {
        block(&mut statue, pos, size, col);
    }
    statue.push((Cone(0.24, 0.42), trim, Transform::from_xyz(0.0, 2.25, 0.0)));
    statue.push((
        Cone(0.13, 0.31),
        stone,
        Transform::from_xyz(0.48, 2.16, 0.17),
    ));
    for (shape, col, mut tr) in statue {
        tr.translation = at + tr.translation * s;
        tr.scale *= s;
        parts.push((shape, col, tr));
    }
}

pub(super) fn palm(parts: &mut Vec<Part>, at: Vec3, s: f32) {
    for i in 0..4 {
        parts.push((
            Frustum(0.065 * s, 0.095 * s, 0.40 * s),
            rgb(0x94734b),
            Transform::from_translation(
                at + Vec3::new(i as f32 * 0.04, 0.20 + i as f32 * 0.37, 0.0) * s,
            )
            .with_rotation(Quat::from_rotation_z(-0.10)),
        ));
    }
    let crown = at + Vec3::new(0.15, 1.67, 0.0) * s;
    for i in 0..7 {
        let angle = i as f32 * std::f32::consts::TAU / 7.0;
        let rot = Quat::from_rotation_y(angle);
        parts.push((
            Ball(1.0),
            if i % 2 == 0 {
                rgb(0x568b43)
            } else {
                rgb(0x85a64c)
            },
            Transform::from_translation(crown + rot * Vec3::new(0.0, -0.06, 0.48) * s)
                .with_rotation(rot * Quat::from_rotation_x(0.20))
                .with_scale(Vec3::new(0.20, 0.055, 0.73) * s),
        ));
    }
    parts.push((
        Ball(0.12 * s),
        rgb(0x80603c),
        Transform::from_translation(crown - Vec3::Y * 0.12 * s),
    ));
}

pub(super) fn planter(parts: &mut Vec<Part>, at: Vec3, stone: Color, green: Color) {
    block(
        parts,
        at + Vec3::Y * 0.11,
        Vec3::new(0.9, 0.22, 0.65),
        stone,
    );
    for dx in [-0.25, 0.0, 0.25] {
        parts.push((
            Shape::Rock(0.30, (at.x + dx).to_bits()),
            green,
            Transform::from_translation(at + Vec3::new(dx, 0.27, 0.0))
                .with_scale(Vec3::new(1.0, 0.66, 0.8)),
        ));
    }
}
