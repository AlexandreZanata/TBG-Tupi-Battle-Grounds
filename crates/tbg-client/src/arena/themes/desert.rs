//! A sandstone island temple with ceremonial obelisks, guardians and coastal palms.

use super::{
    boulder,
    monuments::{block, column, guardian, palm, steps},
    scatter, Kit,
};
use crate::{
    arena::{
        theme::{rgb, Theme},
        Part,
    },
    art::mesh::Shape::*,
};
use bevy::prelude::*;

pub(super) fn build(kit: &mut Kit) {
    let p = Theme::Desert.palette();
    temple(&mut kit.props, &mut kit.glow);
    for (x, z, h) in [
        (1.5, -3.55, 2.8),
        (3.6, -4.25, 3.4),
        (12.4, -4.25, 3.4),
        (14.5, -3.55, 2.8),
    ] {
        obelisk(&mut kit.props, &mut kit.glow, Vec3::new(x, 0.0, z), h);
    }
    for x in [5.0, 11.0] {
        guardian(
            &mut kit.props,
            Vec3::new(x, 0.0, -3.24),
            1.05,
            p.stone,
            rgb(0xb9985c),
        );
    }
    for (x, z, s) in [
        (-0.1, -4.0, 1.0),
        (1.0, -4.3, 1.2),
        (3.1, -4.85, 0.85),
        (12.9, -4.85, 0.85),
        (15.0, -4.3, 1.2),
        (16.1, -4.0, 1.0),
        (0.7, 3.4, 0.60),
        (15.3, 3.4, 0.60),
    ] {
        palm(&mut kit.props, Vec3::new(x, 0.0, z), s);
    }
    for (i, at, s) in scatter(0xde5e47).into_iter().filter(|(i, _, _)| i % 3 == 0) {
        if i % 2 == 0 {
            boulder(&mut kit.props, at, s, p.dark);
        } else {
            vessel(&mut kit.props, at, s);
        }
    }
    for (x, z) in [(3.0, 4.3), (13.0, 4.3)] {
        for dx in [-0.38, 0.0, 0.38] {
            block(
                &mut kit.props,
                Vec3::new(x + dx, 0.13, z),
                Vec3::new(0.34, 0.26, 0.48),
                p.stone,
            );
        }
        vessel(&mut kit.props, Vec3::new(x, 0.26, z), 0.65);
    }
    for (x, z) in [(-3.4, -4.5), (19.4, -4.5), (1.3, -7.1), (14.7, -7.1)] {
        let at = Vec3::new(x, -1.0, z);
        boulder(&mut kit.props, at, 1.4, p.stone);
        palm(&mut kit.props, at + Vec3::Y * 0.80, 0.70);
    }
}

fn temple(parts: &mut Vec<Part>, glow: &mut Vec<Part>) {
    let p = Theme::Desert.palette();
    block(
        parts,
        Vec3::new(8.0, 1.24, -4.60),
        Vec3::new(4.0, 2.48, 0.86),
        p.dark,
    );
    block(
        parts,
        Vec3::new(8.0, 1.3, -4.13),
        Vec3::new(1.15, 1.95, 0.08),
        rgb(0x816440),
    );
    for x in [6.0, 7.0, 9.0, 10.0] {
        column(parts, Vec3::new(x, 0.0, -3.95), 2.9, p.stone, p.accent);
    }
    for (y, w) in [(2.84, 4.8), (3.03, 5.2), (3.20, 4.65)] {
        block(
            parts,
            Vec3::new(8.0, y, -4.28),
            Vec3::new(w, 0.17, 1.18),
            p.stone,
        );
    }
    steps(parts, Vec3::new(8.0, 0.0, -3.10), 2.8, p.stone);
    block(
        parts,
        Vec3::new(8.0, 3.04, -3.59),
        Vec3::new(0.72, 0.72, 0.12),
        p.accent,
    );
    glow.push((
        Bevel(0.39, 0.39, 0.14, 0.035),
        rgb(0x55bfb5),
        Transform::from_xyz(8.0, 3.04, -3.50)
            .with_rotation(Quat::from_rotation_z(std::f32::consts::FRAC_PI_4)),
    ));
    for x in [6.15, 9.85] {
        block(
            parts,
            Vec3::new(x, 1.64, -3.53),
            Vec3::new(0.32, 1.30, 0.025),
            rgb(0x548f9a),
        );
    }
}

fn obelisk(parts: &mut Vec<Part>, glow: &mut Vec<Part>, at: Vec3, h: f32) {
    let p = Theme::Desert.palette();
    block(
        parts,
        at + Vec3::Y * 0.12,
        Vec3::new(0.95, 0.24, 0.85),
        p.dark,
    );
    parts.push((
        Frustum(0.22, 0.35, h - 0.7),
        p.stone,
        Transform::from_translation(at + Vec3::Y * (h - 0.22) * 0.5),
    ));
    parts.push((
        Cone(0.32, 0.58),
        p.accent,
        Transform::from_translation(at + Vec3::Y * (h - 0.25)),
    ));
    for y in [0.65, 0.88, 1.11] {
        block(
            parts,
            at + Vec3::new(0.0, y, 0.31),
            Vec3::new(0.14, 0.055, 0.025),
            p.accent,
        );
    }
    glow.push((
        Bevel(0.23, 0.23, 0.05, 0.02),
        rgb(0x58bdba),
        Transform::from_translation(at + Vec3::new(0.0, h - 0.95, 0.27))
            .with_rotation(Quat::from_rotation_z(std::f32::consts::FRAC_PI_4)),
    ));
}

fn vessel(parts: &mut Vec<Part>, at: Vec3, s: f32) {
    for (shape, y, col) in [
        (Cyl(0.20 * s, 0.10 * s), 0.05, rgb(0xa36d43)),
        (Frustum(0.22 * s, 0.29 * s, 0.37 * s), 0.28, rgb(0xc58c51)),
        (Cyl(0.18 * s, 0.12 * s), 0.50, rgb(0xe3b36a)),
        (Cyl(0.13 * s, 0.015), 0.57, rgb(0x6c5842)),
    ] {
        parts.push((
            shape,
            col,
            Transform::from_translation(at + Vec3::Y * y * s),
        ));
    }
}
