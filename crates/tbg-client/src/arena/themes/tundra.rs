//! Snow-loaded firs, ice shards, snowdrifts and Nordic cairns for Frozen Tundra.

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
    let p = Theme::Tundra.palette();
    for (at, s) in back_row() {
        fir(&mut kit.props, at, s);
    }
    for (i, at, s) in scatter(0x1ce) {
        match i % 5 {
            0 => crystal(&mut kit.props, at, s, p.accent),
            1 => snow_rock(&mut kit.props, at, s),
            2 => boulder(&mut kit.props, at, s, rgb(0xe9f3f3)),
            3 => frost_grass(&mut kit.props, at),
            _ => crystal(&mut kit.props, at, s * 0.5, rgb(0xc7e9f1)),
        }
    }
    for (x, z, s) in [
        (0.0, 3.0, 0.9),
        (4.4, 4.0, 1.0),
        (10.2, 4.5, 1.2),
        (16.7, 3.5, 0.9),
    ] {
        snow_rock(&mut kit.props, Vec3::new(x, 0.0, z), s);
        crystal(
            &mut kit.props,
            Vec3::new(x + 0.5, 0.0, z - 0.3),
            s * 0.65,
            p.accent,
        );
    }
    for x in [-2.1, 18.1] {
        for (y, size) in [(0.14, 0.7), (0.42, 0.53), (0.65, 0.35)] {
            kit.props.push((
                Box(size, 0.27, size * 0.8),
                p.stone,
                Transform::from_xyz(x, y, 1.5),
            ));
        }
        kit.props.push((
            Box(0.76, 0.07, 0.6),
            rgb(0xf0f8f8),
            Transform::from_xyz(x, 0.3, 1.5),
        ));
        frost_grass(&mut kit.props, Vec3::new(x, 0.0, -1.2));
    }
}

fn fir(parts: &mut Vec<Part>, at: Vec3, s: f32) {
    let p = Theme::Tundra.palette();
    contact(parts, at, s * 0.5, rgb(0x9cb9c8));
    parts.push((
        Cyl(0.11 * s, 0.7 * s),
        p.wood,
        Transform::from_translation(at + Vec3::Y * s * 0.3),
    ));
    for (r, h, y) in [(0.69, 1.08, 0.92), (0.54, 0.96, 1.46), (0.35, 0.85, 1.96)] {
        parts.push((
            Cone(r * s, h * s),
            p.plant,
            Transform::from_translation(at + Vec3::Y * y * s),
        ));
        parts.push((
            Cone(r * s * 0.94, h * s * 0.76),
            rgb(0xe6f3f5),
            Transform::from_translation(at + Vec3::Y * (y + h * 0.15) * s),
        ));
    }
}

fn snow_rock(parts: &mut Vec<Part>, at: Vec3, s: f32) {
    contact(parts, at, s * 0.8, rgb(0x91b0c1));
    boulder(parts, at, s, rgb(0x6d8b9e));
    parts.push((
        Ball(1.0),
        rgb(0xe5f2f3),
        Transform::from_translation(at + Vec3::Y * s * 0.59)
            .with_scale(Vec3::new(0.78, 0.21, 0.65) * s),
    ));
}

pub(super) fn crystal(parts: &mut Vec<Part>, at: Vec3, s: f32, color: Color) {
    for (dx, dz, h, tilt) in [
        (0.0, 0.0, 1.0, 0.12),
        (0.23, 0.08, 0.65, -0.28),
        (-0.2, 0.1, 0.5, 0.3),
    ] {
        let rot = Quat::from_rotation_z(tilt);
        let origin = at + Vec3::new(dx * s, 0.0, dz * s);
        parts.push((
            Cyl(0.16 * s, h * s * 0.6),
            color,
            Transform::from_translation(origin + rot * Vec3::Y * h * s * 0.3).with_rotation(rot),
        ));
        parts.push((
            Cone(0.16 * s, h * s * 0.4),
            rgb(0xc8edf5),
            Transform::from_translation(origin + rot * Vec3::Y * h * s * 0.8).with_rotation(rot),
        ));
    }
}

fn frost_grass(parts: &mut Vec<Part>, at: Vec3) {
    for i in 0..3 {
        parts.push((
            Cone(0.035, 0.3),
            rgb(0x8ab6ca),
            Transform::from_translation(at + Vec3::new(i as f32 * 0.08, 0.12, 0.0))
                .with_rotation(Quat::from_rotation_z((i as f32 - 1.0) * 0.35)),
        ));
    }
}
