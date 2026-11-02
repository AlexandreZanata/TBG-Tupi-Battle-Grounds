//! An ivory-and-gold citadel, crystal sanctuaries and waterfall islets above blue water.

use super::{
    boulder,
    monuments::{block, column, guardian, planter, steps},
    shared, Kit,
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
    let p = Theme::Celestial.palette();
    let parts = &mut kit.props;
    block(
        parts,
        Vec3::new(8.0, 1.32, -4.66),
        Vec3::new(4.6, 2.64, 0.70),
        p.stone,
    );
    block(
        parts,
        Vec3::new(8.0, 1.50, -4.28),
        Vec3::new(1.22, 2.02, 0.10),
        rgb(0x4986a2),
    );
    for x in [5.8, 7.1, 8.9, 10.2] {
        column(parts, Vec3::new(x, 0.0, -4.04), 3.05, p.stone, p.accent);
    }
    for (y, w, d, col) in [
        (2.95, 5.3, 1.15, p.accent),
        (3.12, 5.6, 1.25, p.stone),
        (3.30, 4.9, 1.06, p.stone),
    ] {
        block(parts, Vec3::new(8.0, y, -4.45), Vec3::new(w, 0.16, d), col);
    }
    steps(parts, Vec3::new(8.0, 0.0, -3.10), 2.8, p.stone);
    for (x, z, h) in [
        (1.3, -3.5, 2.7),
        (3.0, -4.2, 3.35),
        (13.0, -4.2, 3.35),
        (14.7, -3.5, 2.7),
    ] {
        column(parts, Vec3::new(x, 0.0, z), h, p.stone, p.accent);
        parts.push((
            Cone(0.40, 0.62),
            p.accent,
            Transform::from_xyz(x, h + 0.27, z),
        ));
        shared::crystal(
            &mut kit.glow,
            Vec3::new(x, h + 0.12, z),
            0.55,
            rgb(0x36b9df),
            rgb(0xb8f3ff),
        );
    }
    for x in [4.35, 11.65] {
        guardian(parts, Vec3::new(x, 0.0, -3.30), 1.02, p.stone, p.accent);
        planter(parts, Vec3::new(x + 0.65, 0.0, -3.26), p.stone, p.plant);
    }
    shared::crystal(
        &mut kit.glow,
        Vec3::new(8.0, 3.40, -4.54),
        1.20,
        rgb(0x2fbbe5),
        rgb(0xb1f1ff),
    );
    for (x, z) in [
        (0.8, 3.6),
        (3.0, 4.5),
        (13.0, 4.5),
        (15.2, 3.6),
        (0.2, -3.8),
        (15.8, -3.8),
    ] {
        sanctuary(parts, &mut kit.glow, Vec3::new(x, 0.0, z));
    }
    for (x, z) in [(5.0, 3.8), (11.0, 3.8), (3.6, -4.8), (12.4, -4.8)] {
        planter(parts, Vec3::new(x, 0.0, z), p.stone, p.plant);
    }
    for (x, z) in [(-3.5, -4.5), (19.5, -4.5), (2.0, -7.4), (14.0, -7.4)] {
        islet(parts, &mut kit.glow, Vec3::new(x, -1.15, z));
    }
}

fn sanctuary(parts: &mut Vec<Part>, glow: &mut Vec<Part>, at: Vec3) {
    let p = Theme::Celestial.palette();
    parts.push((
        Cyl(0.48, 0.16),
        p.stone,
        Transform::from_translation(at + Vec3::Y * 0.08),
    ));
    parts.push((
        Frustum(0.31, 0.38, 0.25),
        p.accent,
        Transform::from_translation(at + Vec3::Y * 0.26),
    ));
    shared::crystal(
        glow,
        at + Vec3::Y * 0.38,
        0.68,
        rgb(0x42bee4),
        rgb(0xb9f4ff),
    );
}

fn islet(parts: &mut Vec<Part>, glow: &mut Vec<Part>, at: Vec3) {
    let p = Theme::Celestial.palette();
    boulder(parts, at, 1.35, p.stone);
    planter(parts, at + Vec3::Y * 0.83, p.stone, p.plant);
    shared::crystal(
        glow,
        at + Vec3::new(-0.30, 1.0, -0.12),
        0.72,
        rgb(0x45c5e9),
        rgb(0xc9f6ff),
    );
    // Layered ribbons reach the sea and stay outside the playable island.
    for (dx, width, col) in [(-0.17, 0.24, rgb(0x55bcdf)), (0.10, 0.14, rgb(0xabe9f3))] {
        block(
            glow,
            at + Vec3::new(dx, 0.35, 0.45),
            Vec3::new(width, 1.18, 0.07),
            col,
        );
    }
}
