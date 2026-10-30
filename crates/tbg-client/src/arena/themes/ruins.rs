//! Weathered colonnades, fallen masonry, guardian statues, moss and reed beds.

use super::{back_row, boulder, contact, scatter, Kit};
use crate::{
    arena::{
        theme::{rgb, Theme},
        Part,
    },
    art::mesh::Shape::*,
};
use bevy::prelude::*;

pub(super) fn build(kit: &mut Kit) {
    let p = Theme::Ruins.palette();
    for (i, (at, s)) in back_row().into_iter().enumerate() {
        if i % 3 == 0 {
            column(&mut kit.props, at, s);
        } else {
            shrub(&mut kit.props, at, s * 0.5);
        }
    }
    for x in [5.7, 10.3] {
        column(&mut kit.props, Vec3::new(x, 0.0, -4.1), 1.2);
    }
    kit.props.extend([
        (
            Box(5.5, 0.35, 0.7),
            p.stone,
            Transform::from_xyz(8.0, 2.46, -4.1),
        ),
        (
            Box(4.6, 0.14, 0.82),
            p.dark,
            Transform::from_xyz(8.0, 2.68, -4.1),
        ),
        (
            Box(0.6, 0.6, 0.16),
            p.accent,
            Transform::from_xyz(8.0, 2.45, -3.7).with_rotation(Quat::from_rotation_z(0.785)),
        ),
    ]);
    for (i, at, s) in scatter(0xa6e5) {
        match i % 4 {
            0 => kit.props.push((
                Box(s * 1.5, s * 0.5, s),
                p.stone,
                Transform::from_translation(at + Vec3::Y * s * 0.23)
                    .with_rotation(Quat::from_rotation_y(at.x)),
            )),
            1 => shrub(&mut kit.props, at, s),
            2 => reeds(&mut kit.props, at),
            _ => boulder(&mut kit.props, at, s, p.dark),
        }
    }
    for (x, z, s) in [(3.5, 4.0, 0.75), (12.5, 4.0, 0.75)] {
        fallen(&mut kit.props, Vec3::new(x, 0.0, z), s);
    }
    for x in [-2.25, 18.25] {
        statue(&mut kit.props, Vec3::new(x, 0.0, -1.4));
        kit.props.push((
            Frustum(0.25, 0.18, 0.5),
            p.stone,
            Transform::from_xyz(x, 0.24, 1.6),
        ));
        shrub(&mut kit.props, Vec3::new(x, 0.45, 1.6), 0.4);
    }
}

fn column(parts: &mut Vec<Part>, at: Vec3, s: f32) {
    let p = Theme::Ruins.palette();
    contact(parts, at, 0.55 * s, rgb(0x7e8567));
    for (shape, y, color) in [
        (Box(0.78, 0.18, 0.78), 0.09, p.dark),
        (Cyl(0.31, 1.6), 0.98, p.stone),
        (Box(0.72, 0.22, 0.72), 1.87, p.stone),
        (Box(0.78, 0.08, 0.78), 2.02, p.dark),
    ] {
        parts.push((
            shape,
            color,
            Transform::from_translation(at + Vec3::Y * y * s).with_scale(Vec3::splat(s)),
        ));
    }
    for side in [-1.0, 1.0] {
        parts.push((
            Box(0.06, 1.3, 0.025),
            p.dark,
            Transform::from_translation(at + Vec3::new(side * 0.16, 0.95, 0.3) * s)
                .with_scale(Vec3::splat(s)),
        ));
    }
    shrub(parts, at + Vec3::new(0.2, 0.04, 0.15) * s, s * 0.33);
}

fn fallen(parts: &mut Vec<Part>, at: Vec3, s: f32) {
    let p = Theme::Ruins.palette();
    parts.push((
        Cyl(0.32, 1.8),
        p.stone,
        Transform::from_translation(at + Vec3::Y * 0.26)
            .with_rotation(Quat::from_euler(EulerRot::XYZ, 0.0, -0.3, 1.57))
            .with_scale(Vec3::splat(s)),
    ));
    shrub(parts, at + Vec3::new(0.4, 0.0, 0.3), s * 0.45);
}

fn statue(parts: &mut Vec<Part>, at: Vec3) {
    let p = Theme::Ruins.palette();
    for (shape, off, color) in [
        (Box(0.65, 0.2, 0.65), Vec3::Y * 0.1, p.dark),
        (Frustum(0.18, 0.3, 0.65), Vec3::Y * 0.52, p.stone),
        (Ball(0.22), Vec3::Y * 0.97, p.stone),
        (Box(0.4, 0.5, 0.12), Vec3::new(0.15, 0.6, 0.25), p.accent),
    ] {
        parts.push((shape, color, Transform::from_translation(at + off)));
    }
}

fn shrub(parts: &mut Vec<Part>, at: Vec3, s: f32) {
    for i in 0..3 {
        boulder(
            parts,
            at + Vec3::new(i as f32 * 0.16 - 0.16, 0.0, (i % 2) as f32 * 0.15),
            s * 0.6,
            if i % 2 == 0 {
                rgb(0x70814d)
            } else {
                rgb(0x8c9a5c)
            },
        );
    }
}

fn reeds(parts: &mut Vec<Part>, at: Vec3) {
    for i in 0..3 {
        parts.push((
            Cone(0.035, 0.4),
            rgb(0x94965c),
            Transform::from_translation(at + Vec3::new(i as f32 * 0.08, 0.16, 0.0))
                .with_rotation(Quat::from_rotation_z(0.3)),
        ));
    }
}
