//! An iron foundry fortress with brass clockwork, coastal cranes and glowing furnaces.

use super::{boulder, monuments::block, Kit};
use crate::{
    arena::{
        theme::{rgb, Theme},
        Part,
    },
    art::mesh::Shape::*,
};
use bevy::prelude::*;

pub(super) mod motion;

pub(super) fn build(kit: &mut Kit) {
    let p = Theme::Clockwork.palette();
    fortress(kit);
    let parts = &mut kit.props;
    for (x, z, side) in [(0.4, -3.55, 1.0), (15.6, -3.55, -1.0)] {
        crane(parts, Vec3::new(x, 0.0, z), side);
    }
    for (x, z) in [(1.4, 3.6), (14.6, 3.6), (3.0, -3.3), (13.0, -3.3)] {
        boiler(parts, Vec3::new(x, 0.0, z));
    }
    for x in [4.2, 11.8] {
        for i in 0..4 {
            let at = Vec3::new(x + i as f32 * 0.44, 0.12, 3.70);
            block(parts, at, Vec3::new(0.38, 0.25, 0.56), p.dark);
            block(
                parts,
                at + Vec3::Y * 0.16,
                Vec3::new(0.30, 0.06, 0.48),
                p.accent,
            );
        }
    }
    for (x, z) in [(-3.3, -4.3), (19.3, -4.3), (1.5, -7.0), (14.5, -7.0)] {
        boulder(parts, Vec3::new(x, -1.1, z), 1.4, p.dark);
        block(
            parts,
            Vec3::new(x, -0.06, z),
            Vec3::new(0.65, 0.12, 0.68),
            p.accent,
        );
    }
}

fn fortress(kit: &mut Kit) {
    let p = Theme::Clockwork.palette();
    let parts = &mut kit.props;
    block(
        parts,
        Vec3::new(8.0, 1.18, -4.54),
        Vec3::new(6.5, 2.36, 0.92),
        p.dark,
    );
    block(
        parts,
        Vec3::new(8.0, 2.48, -4.54),
        Vec3::new(6.8, 0.25, 1.12),
        p.accent,
    );
    for x in [4.8, 11.2] {
        tower(parts, Vec3::new(x, 0.0, -4.25));
    }
    for x in [6.1, 9.9] {
        block(
            parts,
            Vec3::new(x, 0.60, -3.98),
            Vec3::new(0.87, 1.20, 0.32),
            p.accent,
        );
        block(
            &mut kit.glow,
            Vec3::new(x, 0.62, -3.79),
            Vec3::new(0.63, 0.74, 0.05),
            rgb(0xe97025),
        );
        for dx in [-0.22, 0.0, 0.22] {
            block(
                parts,
                Vec3::new(x + dx, 0.62, -3.73),
                Vec3::new(0.055, 0.80, 0.10),
                p.dark,
            );
        }
    }
    for x in [5.65, 10.35] {
        for (r, h, y, col) in [
            (0.21, 2.5, 1.25, p.stone),
            (0.30, 0.16, 2.57, p.accent),
            (0.26, 0.45, 2.86, p.dark),
            (0.31, 0.14, 3.14, p.accent),
        ] {
            parts.push((Cyl(r, h), col, Transform::from_xyz(x, y, -4.83)));
        }
    }
}

fn tower(parts: &mut Vec<Part>, at: Vec3) {
    let p = Theme::Clockwork.palette();
    for (r, h, y, col) in [
        (0.67, 0.28, 0.14, p.dark),
        (0.52, 2.78, 1.56, p.stone),
        (0.61, 0.17, 0.65, p.accent),
        (0.61, 0.17, 2.45, p.accent),
        (0.69, 0.26, 3.01, p.dark),
    ] {
        parts.push((
            Cyl(r, h),
            col,
            Transform::from_translation(at + Vec3::Y * y),
        ));
    }
    for dx in [-0.46, 0.0, 0.46] {
        block(
            parts,
            at + Vec3::new(dx, 3.26, 0.0),
            Vec3::new(0.21, 0.30, 0.68),
            p.accent,
        );
    }
    for y in [1.1, 1.9] {
        block(
            parts,
            at + Vec3::new(0.0, y, 0.51),
            Vec3::new(0.17, 0.37, 0.06),
            rgb(0xcf9750),
        );
    }
}

fn crane(parts: &mut Vec<Part>, at: Vec3, side: f32) {
    let p = Theme::Clockwork.palette();
    parts.push((
        Cyl(0.56, 0.22),
        p.dark,
        Transform::from_translation(at + Vec3::Y * 0.11),
    ));
    block(
        parts,
        at + Vec3::Y * 1.22,
        Vec3::new(0.27, 2.1, 0.31),
        p.accent,
    );
    block(
        parts,
        at + Vec3::new(side * 0.56, 2.30, 0.0),
        Vec3::new(1.75, 0.20, 0.27),
        p.accent,
    );
    for x in [side * 0.15, side * 0.57, side * 1.0] {
        block(
            parts,
            at + Vec3::new(x, 2.30, 0.17),
            Vec3::new(0.08, 0.10, 0.06),
            p.dark,
        );
    }
    parts.push((
        Cyl(0.024, 1.26),
        p.dark,
        Transform::from_translation(at + Vec3::new(side * 1.20, 1.58, 0.0)),
    ));
    block(
        parts,
        at + Vec3::new(side * 1.20, 0.78, 0.0),
        Vec3::new(0.48, 0.40, 0.44),
        p.wood,
    );
    block(
        parts,
        at + Vec3::new(side * 1.20, 0.78, 0.23),
        Vec3::new(0.07, 0.43, 0.035),
        p.accent,
    );
}

fn boiler(parts: &mut Vec<Part>, at: Vec3) {
    let p = Theme::Clockwork.palette();
    parts.push((
        Cyl(0.33, 0.62),
        p.stone,
        Transform::from_translation(at + Vec3::Y * 0.33),
    ));
    for y in [0.1, 0.58] {
        parts.push((
            Cyl(0.38, 0.09),
            p.accent,
            Transform::from_translation(at + Vec3::Y * y),
        ));
    }
    parts.push((
        Cyl(0.10, 0.46),
        p.accent,
        Transform::from_translation(at + Vec3::new(0.30, 0.40, 0.0))
            .with_rotation(Quat::from_rotation_z(std::f32::consts::FRAC_PI_2)),
    ));
    block(
        parts,
        at + Vec3::new(0.0, 0.32, 0.32),
        Vec3::new(0.25, 0.17, 0.06),
        p.dark,
    );
}
