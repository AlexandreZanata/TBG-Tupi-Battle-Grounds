//! Basalt organ pipes, a crater, charred branches and contained lava fissures.

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
    let p = Theme::Volcanic.palette();
    for (i, (at, s)) in back_row().into_iter().enumerate() {
        if (at.x - 8.0).abs() < 2.0 && at.z < -3.8 {
            continue;
        }
        if i % 4 == 0 {
            charred(&mut kit.props, at, s);
        } else {
            basalt(&mut kit.props, at, s);
        }
    }
    let at = Vec3::new(8.0, 0.0, -4.25);
    kit.props.extend([
        (
            Frustum(0.77, 1.65, 1.9),
            p.dark,
            Transform::from_translation(at + Vec3::Y * 0.9),
        ),
        (
            Frustum(0.65, 0.8, 0.3),
            p.stone,
            Transform::from_translation(at + Vec3::Y * 1.93),
        ),
    ]);
    kit.glow.push((
        Cyl(0.65, 0.015),
        rgb(0xffba57),
        Transform::from_translation(at + Vec3::Y * 2.085),
    ));
    for (i, at, s) in scatter(0xf04e) {
        if i % 3 == 0 {
            basalt(&mut kit.props, at, s * 0.65);
        } else {
            boulder(
                &mut kit.props,
                at,
                s,
                if i % 2 == 0 { p.dark } else { p.plant },
            );
        }
        if i % 5 == 0 {
            fissure(&mut kit.glow, at, s);
        }
    }
    for (x, z, s) in [
        (0.0, 3.2, 0.7),
        (4.3, 4.0, 1.0),
        (10.3, 4.4, 1.2),
        (16.5, 3.5, 0.9),
    ] {
        basalt(&mut kit.props, Vec3::new(x, 0.0, z), s * 0.48);
        fissure(&mut kit.glow, Vec3::new(x + 0.5, 0.0, z), s);
    }
    for x in [-2.2, 18.2] {
        kit.props.extend([
            (
                Box(0.55, 0.24, 0.5),
                p.dark,
                Transform::from_xyz(x, 0.12, 1.4),
            ),
            (
                Box(0.28, 0.3, 0.3),
                p.stone,
                Transform::from_xyz(x, 0.38, 1.4),
            ),
            (
                Box(0.75, 0.18, 0.35),
                rgb(0x83747b),
                Transform::from_xyz(x, 0.59, 1.4),
            ),
            (
                Cone(0.3, 0.6),
                p.dark,
                Transform::from_xyz(x + 0.25, 0.22, -1.3)
                    .with_rotation(Quat::from_rotation_z(-0.4)),
            ),
        ]);
    }
}

fn basalt(parts: &mut Vec<Part>, at: Vec3, s: f32) {
    let p = Theme::Volcanic.palette();
    contact(parts, at, s * 0.7, rgb(0x38303c));
    for (dx, dz, h, r) in [
        (0.0, 0.0, 1.8, 0.34),
        (0.4, 0.1, 1.2, 0.28),
        (-0.32, 0.1, 0.8, 0.24),
    ] {
        let at = at + Vec3::new(dx, h * 0.5, dz) * s;
        parts.push((Cyl(r * s, h * s), p.dark, Transform::from_translation(at)));
        parts.push((
            Cyl(r * s * 1.01, 0.04 * s),
            p.stone,
            Transform::from_translation(at + Vec3::Y * h * s * 0.49),
        ));
    }
}

fn fissure(parts: &mut Vec<Part>, at: Vec3, s: f32) {
    for i in 0..3 {
        parts.push((
            Box(0.55 * s, 0.012, 0.055),
            rgb(0xffa947),
            Transform::from_translation(
                at + Vec3::new((i as f32 - 1.0) * s * 0.4, -0.018, (i % 2) as f32 * 0.1),
            )
            .with_rotation(Quat::from_rotation_y(if i % 2 == 0 {
                0.35
            } else {
                -0.35
            })),
        ));
    }
}

fn charred(parts: &mut Vec<Part>, at: Vec3, s: f32) {
    let p = Theme::Volcanic.palette();
    parts.push((
        Frustum(0.06 * s, 0.2 * s, 1.6 * s),
        p.wood,
        Transform::from_translation(at + Vec3::Y * 0.75 * s),
    ));
    for side in [-1.0, 1.0] {
        parts.push((
            Cone(0.09 * s, 0.8 * s),
            p.wood,
            Transform::from_translation(at + Vec3::new(side * 0.25, 1.1, 0.0) * s)
                .with_rotation(Quat::from_rotation_z(-side * 0.7)),
        ));
    }
}
