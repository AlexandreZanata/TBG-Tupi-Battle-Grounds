//! Wind-bent conifers, conductor stones, silver grass and low offshore squalls.

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
    let p = Theme::Storm.palette();
    for (i, (at, s)) in back_row().into_iter().enumerate() {
        if i % 4 == 0 {
            conductor(&mut kit.props, &mut kit.glow, at, s);
        } else {
            bent_tree(&mut kit.props, at, s);
        }
    }
    for (i, at, s) in scatter(0x5704) {
        match i % 3 {
            0 => boulder(&mut kit.props, at, s, p.stone),
            1 => grass(&mut kit.props, at, s),
            _ => boulder(&mut kit.props, at, s * 0.65, p.dark),
        }
    }
    for (x, z, s) in [
        (0.2, 3.0, 0.5),
        (5.0, 4.2, 0.65),
        (11.0, 4.3, 0.6),
        (16.5, 3.0, 0.5),
    ] {
        conductor(&mut kit.props, &mut kit.glow, Vec3::new(x, 0.0, z), s);
    }
    // Low, opaque cloud banks stay beyond the island, below the BattleLine.
    for (x, z) in [
        (0.5, -7.8),
        (8.0, -8.8),
        (15.8, -7.8),
        (4.0, 8.5),
        (13.0, 8.6),
    ] {
        for i in 0..4 {
            kit.props.push((
                Ball(1.0),
                if i % 2 == 0 {
                    rgb(0x8599b5)
                } else {
                    rgb(0xa4b6cb)
                },
                Transform::from_xyz(x + (i as f32 - 1.5) * 0.75, -1.05, z).with_scale(Vec3::new(
                    1.1,
                    0.38 + (i % 2) as f32 * 0.16,
                    0.62,
                )),
            ));
        }
    }
    for x in [-2.1, 18.1] {
        kit.props.extend([
            (
                Box(0.65, 0.12, 0.65),
                p.dark,
                Transform::from_xyz(x, 0.06, 1.6),
            ),
            (Cyl(0.19, 0.42), p.stone, Transform::from_xyz(x, 0.3, 1.6)),
            (
                Ring(0.04, 0.3, 360.0),
                p.accent,
                Transform::from_xyz(x, 0.48, 1.6),
            ),
        ]);
    }
}

fn bent_tree(parts: &mut Vec<Part>, at: Vec3, s: f32) {
    let p = Theme::Storm.palette();
    contact(parts, at, s * 0.55, rgb(0x4c6074));
    parts.push((
        Frustum(0.06, 0.16, 1.7),
        p.wood,
        Transform::from_translation(at + Vec3::new(0.2, 0.8, 0.0) * s)
            .with_rotation(Quat::from_rotation_z(-0.24))
            .with_scale(Vec3::splat(s)),
    ));
    for (r, h, y) in [(0.64, 0.85, 0.9), (0.48, 0.78, 1.4), (0.3, 0.7, 1.85)] {
        parts.push((
            Cone(r, h),
            p.plant,
            Transform::from_translation(at + Vec3::new(y * 0.23, y, 0.0) * s)
                .with_rotation(Quat::from_rotation_z(-0.24))
                .with_scale(Vec3::new(1.2, 1.0, 0.8) * s),
        ));
        parts.push((
            Cone(r * 0.83, h * 0.45),
            rgb(0x8095a4),
            Transform::from_translation(at + Vec3::new(y * 0.23 + 0.1, y + h * 0.21, 0.0) * s)
                .with_rotation(Quat::from_rotation_z(-0.24))
                .with_scale(Vec3::new(1.2, 1.0, 0.8) * s),
        ));
    }
}

fn conductor(parts: &mut Vec<Part>, glow: &mut Vec<Part>, at: Vec3, s: f32) {
    let p = Theme::Storm.palette();
    contact(parts, at, s * 0.6, rgb(0x465a70));
    parts.extend([
        (
            Box(0.8, 0.18, 0.8),
            p.dark,
            Transform::from_translation(at + Vec3::Y * 0.09 * s).with_scale(Vec3::splat(s)),
        ),
        (
            Frustum(0.16, 0.36, 1.65),
            p.stone,
            Transform::from_translation(at + Vec3::Y * 0.96 * s).with_scale(Vec3::splat(s)),
        ),
        (
            Ring(0.055, 0.37, 360.0),
            p.accent,
            Transform::from_translation(at + Vec3::Y * 1.8 * s)
                .with_rotation(Quat::from_rotation_x(1.57))
                .with_scale(Vec3::splat(s)),
        ),
        (
            Cone(0.1, 0.62),
            p.dark,
            Transform::from_translation(at + Vec3::Y * 2.15 * s).with_scale(Vec3::splat(s)),
        ),
    ]);
    for (dx, y, angle) in [(-0.06, 0.8, -0.25), (0.04, 1.05, 0.5), (-0.04, 1.3, -0.25)] {
        glow.push((
            Box(0.055, 0.3, 0.025),
            rgb(0xc4e8fa),
            Transform::from_translation(at + Vec3::new(dx, y, 0.28) * s)
                .with_rotation(Quat::from_rotation_z(angle))
                .with_scale(Vec3::splat(s)),
        ));
    }
}

fn grass(parts: &mut Vec<Part>, at: Vec3, s: f32) {
    for i in 0..4 {
        parts.push((
            Cone(0.045, 0.6),
            if i % 2 == 0 {
                rgb(0x98acb3)
            } else {
                rgb(0x6d8995)
            },
            Transform::from_translation(at + Vec3::new(i as f32 * 0.07, 0.18, 0.0))
                .with_rotation(Quat::from_rotation_z(-0.6))
                .with_scale(Vec3::splat(s + 0.4)),
        ));
    }
}
