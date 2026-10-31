//! Wind-bent conifers, banner-flying conductor spires, a broken dock, dead trees, floes
//! and low offshore squalls for Storm Valley.

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
    let p = Theme::Storm.palette();
    for (i, (at, s)) in back_row().into_iter().enumerate() {
        if i % 7 == 0 {
            shared::dead_tree(&mut kit.props, at, s, rgb(0x454552));
        } else {
            bent_tree(&mut kit.props, at, s * 0.83);
        }
    }
    for (x, z, s) in [
        (0.6, -3.6, 1.35),
        (4.9, -4.5, 1.65),
        (11.1, -4.5, 1.65),
        (15.4, -3.6, 1.35),
    ] {
        let at = Vec3::new(x, 0.0, z);
        conductor(&mut kit.props, &mut kit.glow, at, s);
        banner(&mut kit.props, at, s);
    }
    shared::dead_tree(
        &mut kit.props,
        Vec3::new(8.0, 0.0, -3.8),
        1.55,
        rgb(0x4a4245),
    );
    for (i, at, s) in scatter(0x5704).into_iter().filter(|(i, _, _)| i % 4 != 1) {
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
    // Overlapping dark banks form a distant horizon, outside the playable footprint.
    for layer in 0..2 {
        for i in 0..10 {
            let x = -6.0 + i as f32 * 3.0;
            let z = -10.8 - layer as f32 * 2.4 - (i % 3) as f32 * 0.55;
            kit.props.push((
                Ball(1.0),
                if (i + layer) % 2 == 0 {
                    rgb(0x263b53)
                } else {
                    rgb(0x1e3049)
                },
                Transform::from_xyz(x, -0.25 + layer as f32 * 0.45, z).with_scale(Vec3::new(
                    2.55,
                    0.45 + (i % 3) as f32 * 0.14,
                    1.2,
                )),
            ));
        }
    }
    // A broken dock leaves the left end; a few posts and planks remain on the right.
    let (dock, dark) = (rgb(0x6b5a48), rgb(0x3d3430));
    shared::walkway(
        &mut kit.props,
        Vec3::new(-2.5, 0.02, 2.4),
        -1.0,
        8,
        0b0010_1000,
        dock,
        dark,
    );
    shared::walkway(
        &mut kit.props,
        Vec3::new(18.5, 0.02, -2.1),
        1.0,
        4,
        0b1010,
        dock,
        dark,
    );
    for (x, z, s) in [
        (-1.3, -3.3, 1.1),
        (17.3, -3.4, 1.0),
        (0.6, 3.9, 0.8),
        (15.4, 4.0, 0.9),
    ] {
        shared::dead_tree(&mut kit.props, Vec3::new(x, 0.0, z), s, rgb(0x3f3a40));
    }
    // Slate islets belong to the storm coast; white ice slabs remain a Tundra motif.
    for (i, (at, size)) in shared::water_sites(0x57a0, 10, 3.0).into_iter().enumerate() {
        boulder(&mut kit.props, at, 0.9 + size, rgb(0x698298));
        kit.props.push((
            Frustum(0.09, 0.42, 1.3),
            rgb(0x7d94a8),
            Transform::from_translation(at + Vec3::new(0.1, 0.5, 0.0))
                .with_rotation(Quat::from_rotation_z(-0.18)),
        ));
        if i % 3 == 0 {
            shared::dead_tree(&mut kit.props, at + Vec3::Y * 0.75, 0.65, rgb(0x414553));
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
            rgb(0xb0c1d1),
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
                .with_rotation(Quat::from_rotation_x(std::f32::consts::FRAC_PI_2))
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

/// A blue banner snapping from a conductor spire.
fn banner(parts: &mut Vec<Part>, at: Vec3, s: f32) {
    parts.push((
        Box(0.36 * s, 0.55 * s, 0.03),
        rgb(0x4f86d6),
        Transform::from_translation(at + Vec3::new(0.32 * s, 1.2 * s, 0.1))
            .with_rotation(Quat::from_rotation_z(-0.12)),
    ));
}
