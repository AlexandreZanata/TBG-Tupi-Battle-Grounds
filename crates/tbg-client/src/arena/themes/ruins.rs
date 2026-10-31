//! An ivory temple colonnade, mossy masonry and lily-strewn shallows for Ancient Ruins.

use super::{boulder, contact, scatter, shared, Kit};
use crate::{
    arena::{
        theme::{rgb, Theme},
        Part,
    },
    art::mesh::Shape::*,
};
use bevy::prelude::*;

pub(super) fn build(kit: &mut Kit) {
    // A few legible architectural groups replace the evenly spaced forest of columns.
    for (x, z, s) in [
        (-1.4, -3.05, 1.05),
        (1.25, -3.75, 1.35),
        (4.0, -4.45, 1.10),
        (12.0, -4.45, 1.10),
        (14.75, -3.75, 1.35),
        (17.4, -3.05, 1.05),
    ] {
        column(&mut kit.props, Vec3::new(x, 0.0, z), s);
    }
    for x in [5.8, 10.2] {
        column(&mut kit.props, Vec3::new(x, 0.0, -4.05), 1.05);
    }
    let stone = Theme::Ruins.palette().stone;
    for (w, h, y, col) in [
        (5.5, 0.28, 2.66, rgb(0xe8dec0)),
        (5.9, 0.20, 2.90, stone),
        (5.45, 0.12, 3.06, rgb(0xf1e7ce)),
    ] {
        kit.props
            .push((block(w, h, 0.88), col, Transform::from_xyz(8.0, y, -4.05)));
    }
    kit.props.push((
        block(0.89, 0.89, 0.16),
        rgb(0xcac5a7),
        Transform::from_xyz(8.0, 2.75, -3.52)
            .with_rotation(Quat::from_rotation_z(std::f32::consts::FRAC_PI_4)),
    ));
    kit.glow.push((
        block(0.65, 0.65, 0.18),
        rgb(0x72d9cc),
        Transform::from_xyz(8.0, 2.75, -3.40)
            .with_rotation(Quat::from_rotation_z(std::f32::consts::FRAC_PI_4)),
    ));
    for (x, z, s) in [
        (0.1, -4.5, 0.95),
        (3.0, -4.7, 1.05),
        (6.7, -4.8, 0.70),
        (9.4, -4.8, 0.80),
        (13.0, -4.7, 1.15),
        (16.0, -4.5, 0.95),
        (1.2, 3.6, 0.62),
        (14.9, 3.7, 0.70),
    ] {
        pine(&mut kit.props, Vec3::new(x, 0.0, z), s);
    }
    for (i, at, s) in scatter(0xa6e5).into_iter().filter(|(i, _, _)| i % 3 != 2) {
        match i % 4 {
            0 => masonry(&mut kit.props, at, s * 1.3),
            1 => shrub(&mut kit.props, at, s * 1.1),
            2 => flowers(&mut kit.props, at, s),
            _ => boulder(&mut kit.props, at, s, rgb(0xa8ad8c)),
        }
    }
    for (x, z, s, angle) in [
        (2.0, 3.65, 1.0, -0.65),
        (5.3, 4.4, 1.1, 0.48),
        (13.5, 3.9, 0.9, -0.35),
    ] {
        fallen(&mut kit.props, Vec3::new(x, 0.0, z), s, angle);
    }
    for (x, z, scale) in [
        (0.8, -3.7, 0.65),
        (3.0, -4.0, 0.8),
        (6.1, -4.8, 0.75),
        (9.5, -4.8, 0.8),
        (13.6, -3.9, 0.75),
        (16.0, -3.1, 0.55),
        (1.7, 3.1, 0.7),
        (5.0, 3.8, 0.8),
        (11.8, 3.6, 0.7),
        (15.0, 3.2, 0.75),
    ] {
        let at = Vec3::new(x, 0.0, z);
        shrub(&mut kit.props, at, scale);
        flowers(&mut kit.props, at + Vec3::new(0.45, 0.0, 0.20), scale);
    }
    stairs(&mut kit.props);
    for x in [-2.7, 18.7] {
        column(&mut kit.props, Vec3::new(x, -0.18, 1.6), 0.60);
    }
    for (i, (at, size)) in shared::water_sites(0x1117, 32, 1.25)
        .into_iter()
        .enumerate()
    {
        shared::lily_pad(
            &mut kit.props,
            at,
            0.18 + size * 0.30,
            if i % 2 == 0 {
                rgb(0x69a747)
            } else {
                rgb(0x83b651)
            },
            (i % 6 == 0).then(|| rgb(0xffefbf)),
        );
        if i % 7 == 0 {
            boulder(
                &mut kit.props,
                at + Vec3::new(0.35, 0.0, 0.20),
                size * 0.8,
                rgb(0xaaa892),
            );
            shrub(
                &mut kit.props,
                at + Vec3::new(0.35, size * 0.5, 0.20),
                size * 0.4,
            );
        }
    }
}

fn column(parts: &mut Vec<Part>, at: Vec3, s: f32) {
    let p = Theme::Ruins.palette();
    contact(parts, at, 0.65 * s, rgb(0x7f9465));
    for (shape, y, col) in [
        (block(1.0, 0.17, 0.90), 0.08, rgb(0xa9a58b)),
        (block(0.84, 0.16, 0.78), 0.24, p.stone),
        (Frustum(0.31, 0.36, 1.95), 1.29, p.stone),
        (block(0.75, 0.14, 0.73), 2.31, rgb(0xe8dfc3)),
        (block(1.0, 0.24, 0.94), 2.49, p.stone),
        (block(0.94, 0.10, 0.89), 2.65, rgb(0xf2e8d0)),
    ] {
        parts.push((
            shape,
            col,
            Transform::from_translation(at + Vec3::Y * y * s).with_scale(Vec3::splat(s)),
        ));
    }
    for side in [-1.0, 1.0] {
        parts.push((
            block(0.045, 1.60, 0.02),
            rgb(0xb1ae91),
            Transform::from_translation(at + Vec3::new(side * 0.15, 1.30, 0.305) * s)
                .with_scale(Vec3::splat(s)),
        ));
    }
    shrub(parts, at + Vec3::new(0.28, 0.05, 0.20) * s, s * 0.38);
    shrub(parts, at + Vec3::new(-0.17, 2.72, -0.02) * s, s * 0.25);
    parts.push((
        block(0.12, 0.52, 0.08),
        rgb(0x6f9849),
        Transform::from_translation(at + Vec3::new(0.3, 2.39, 0.28) * s)
            .with_scale(Vec3::splat(s))
            .with_rotation(Quat::from_rotation_z(-0.12)),
    ));
}

fn masonry(parts: &mut Vec<Part>, at: Vec3, s: f32) {
    let p = Theme::Ruins.palette();
    contact(parts, at, s * 0.9, rgb(0x87956d));
    parts.push((
        block(1.15 * s, 0.42 * s, 0.65 * s),
        p.stone,
        Transform::from_translation(at + Vec3::Y * s * 0.20)
            .with_rotation(Quat::from_rotation_y(at.x)),
    ));
    parts.push((
        block(0.42 * s, 0.28 * s, 0.55 * s),
        rgb(0xeee3c8),
        Transform::from_translation(at + Vec3::new(-0.25, 0.47, -0.08) * s)
            .with_rotation(Quat::from_rotation_y(at.x + 0.25)),
    ));
    shrub(parts, at + Vec3::new(0.32, 0.38, 0.10) * s, s * 0.4);
}

fn fallen(parts: &mut Vec<Part>, at: Vec3, s: f32, angle: f32) {
    let p = Theme::Ruins.palette();
    contact(parts, at, s, rgb(0x8e996e));
    let rot = Quat::from_euler(EulerRot::XYZ, 0.0, angle, std::f32::consts::FRAC_PI_2);
    for (shape, offset, col) in [
        (block(0.55, 2.0, 0.55), Vec3::ZERO, p.stone),
        (block(0.77, 0.25, 0.76), Vec3::Y * 0.85, rgb(0xe8dec3)),
        (block(0.07, 1.75, 0.025), Vec3::Z * 0.28, rgb(0xb4b197)),
    ] {
        parts.push((
            shape,
            col,
            Transform::from_translation(at + Vec3::Y * 0.30 + rot * offset * s)
                .with_rotation(rot)
                .with_scale(Vec3::splat(s)),
        ));
    }
    shrub(parts, at + Vec3::new(0.35, 0.28, 0.20), s * 0.38);
}

fn shrub(parts: &mut Vec<Part>, at: Vec3, s: f32) {
    for i in 0..3 {
        boulder(
            parts,
            at + Vec3::new(i as f32 * 0.18 - 0.18, 0.0, (i % 2) as f32 * 0.15),
            s * 0.70,
            if i % 2 == 0 {
                rgb(0x448637)
            } else {
                rgb(0x70a83d)
            },
        );
    }
}

fn flowers(parts: &mut Vec<Part>, at: Vec3, s: f32) {
    for i in 0..3 {
        let at = at + Vec3::new((i % 2) as f32 * 0.17, 0.035, i as f32 * 0.12);
        parts.push((
            Cyl(0.085 * s, 0.025),
            rgb(0xf7eed3),
            Transform::from_translation(at).with_scale(Vec3::new(1.8, 0.6, 1.0)),
        ));
        parts.push((
            Cyl(0.03 * s, 0.035),
            rgb(0xe3bc60),
            Transform::from_translation(at + Vec3::Y * 0.025),
        ));
    }
}

fn pine(parts: &mut Vec<Part>, at: Vec3, s: f32) {
    parts.push((
        Cyl(0.10 * s, 0.6 * s),
        rgb(0x836c49),
        Transform::from_translation(at + Vec3::Y * 0.28 * s),
    ));
    for (r, h, y, col) in [
        (0.62, 1.02, 0.91, rgb(0x579143)),
        (0.45, 0.94, 1.52, rgb(0x72a948)),
        (0.28, 0.83, 2.04, rgb(0x90b54e)),
    ] {
        parts.push((
            Canopy(r * s, h * s, at.x.to_bits() ^ (at.z + y).to_bits()),
            col,
            Transform::from_translation(at + Vec3::Y * y * s),
        ));
    }
}

fn stairs(parts: &mut Vec<Part>) {
    let p = Theme::Ruins.palette();
    // The run projects beyond the existing cliff instead of being buried inside it.
    for i in 0..5 {
        let top = 0.08 - i as f32 * 0.24;
        let height = top + 1.50;
        parts.push((
            block(2.2, height, 0.34),
            p.stone,
            Transform::from_xyz(8.0, top - height * 0.5, 5.45 + i as f32 * 0.32),
        ));
    }
    for x in [6.65, 9.35] {
        parts.push((
            block(0.28, 0.58, 1.65),
            rgb(0xcec4a8),
            Transform::from_xyz(x, -0.23, 6.06),
        ));
    }
}

fn block(w: f32, h: f32, d: f32) -> crate::art::mesh::Shape {
    Bevel(w, h, d, 0.065)
}
