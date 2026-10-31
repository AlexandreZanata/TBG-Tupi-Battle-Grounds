//! Weathered colonnades, a gemmed arch, fallen masonry, statues, moss, pines, stairs
//! to the water and lily pads for Ancient Ruins.

use super::{back_row, boulder, contact, scatter, shared, Kit};
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
        match i % 3 {
            0 => column(&mut kit.props, at, s),
            1 => pine(&mut kit.props, at, s * 0.8),
            _ => shrub(&mut kit.props, at, s * 0.5),
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
    ]);
    // The keystone gem glows; moss and bushes crown the lintel.
    kit.glow.push((
        Box(0.72, 0.72, 0.14),
        rgb(0x76ece4),
        Transform::from_xyz(8.0, 2.46, -3.68).with_rotation(Quat::from_rotation_z(0.785)),
    ));
    for x in [6.2, 8.0, 9.8] {
        shrub(&mut kit.props, Vec3::new(x, 2.78, -4.1), 0.42);
    }
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
    stairs(&mut kit.props, Vec3::new(9.6, 0.0, 5.1));
    // Lily pads cover the water, mossy rocks and a few blooms among them.
    for (i, (at, size)) in shared::water_sites(0x1117, 24, 1.9).into_iter().enumerate() {
        let leaf = if i % 2 == 0 {
            rgb(0x5aa447)
        } else {
            rgb(0x6fb852)
        };
        let bloom = (i % 4 == 0).then(|| rgb(0xf6efe0));
        shared::lily_pad(&mut kit.props, at, 0.35 + size * 0.45, leaf, bloom);
        if i % 5 == 0 {
            boulder(
                &mut kit.props,
                at + Vec3::new(0.5, 0.0, 0.2),
                0.5 + size * 0.4,
                rgb(0x8c9086),
            );
            boulder(
                &mut kit.props,
                at + Vec3::new(0.5, 0.2, 0.2),
                0.3 + size * 0.2,
                rgb(0x6f9a4a),
            );
        }
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

fn pine(parts: &mut Vec<Part>, at: Vec3, s: f32) {
    let p = Theme::Ruins.palette();
    contact(parts, at, 0.45 * s, rgb(0x6f8a4f));
    parts.push((
        Cyl(0.09 * s, 0.5 * s),
        p.wood,
        Transform::from_translation(at + Vec3::Y * 0.22 * s),
    ));
    for (i, (r, h, y)) in [(0.52, 0.8, 0.7), (0.4, 0.72, 1.15), (0.26, 0.62, 1.55)]
        .into_iter()
        .enumerate()
    {
        parts.push((
            Cone(r * s, h * s),
            if i % 2 == 0 {
                rgb(0x4f8a45)
            } else {
                rgb(0x5c9a4c)
            },
            Transform::from_translation(at + Vec3::Y * y * s),
        ));
    }
}

/// Four stone steps that leave the island's front edge down to the water.
fn stairs(parts: &mut Vec<Part>, at: Vec3) {
    let p = Theme::Ruins.palette();
    for i in 0..4 {
        let top = -0.04 - 0.36 * i as f32;
        parts.push((
            Box(1.6, 0.4, 0.56),
            if i % 2 == 0 { p.stone } else { p.dark },
            Transform::from_xyz(at.x, top - 0.2, at.z + 0.5 * i as f32),
        ));
    }
}
