//! Snow-loaded firs, ice spires, glowing crystals, floes and Nordic cairns for Frozen Tundra.

use super::{back_row, boulder, contact, scatter, shared, Kit};
use crate::{
    arena::{
        camp::BRAZIERS,
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
            0 => shared::crystal(&mut kit.props, at, s, p.accent, TIP),
            1 => snow_rock(&mut kit.props, at, s),
            2 => boulder(&mut kit.props, at, s, rgb(0xe9f3f3)),
            3 => frost_grass(&mut kit.props, at),
            _ => shared::crystal(&mut kit.props, at, s * 0.5, rgb(0xc7e9f1), TIP),
        }
    }
    for (x, z, s) in [
        (0.0, 3.0, 0.9),
        (4.4, 4.0, 1.0),
        (10.2, 4.5, 1.2),
        (16.7, 3.5, 0.9),
    ] {
        snow_rock(&mut kit.props, Vec3::new(x, 0.0, z), s);
        shared::crystal(
            &mut kit.props,
            Vec3::new(x + 0.5, 0.0, z - 0.3),
            s * 0.65,
            p.accent,
            TIP,
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
    // Pale rock spires rise between the back firs.
    for (x, z, h) in [
        (1.8, -3.7, 2.4),
        (5.0, -4.8, 3.2),
        (11.2, -4.8, 3.0),
        (14.4, -3.8, 2.3),
    ] {
        spire(&mut kit.props, Vec3::new(x, 0.0, z), h);
    }
    // Lit crystal clusters by the braziers and the front corners.
    for (i, (x, z)) in BRAZIERS.into_iter().take(4).enumerate() {
        let side = if z > 0.0 { 0.55 } else { -0.55 };
        let at = Vec3::new(x + 0.5, 0.0, z + side);
        shared::crystal(
            &mut kit.glow,
            at,
            0.5 + (i % 2) as f32 * 0.2,
            GLOW,
            GLOW_TIP,
        );
    }
    for (x, z, s) in [(-1.0, 3.7, 1.0), (17.2, 3.8, 1.1), (8.0, 4.4, 0.7)] {
        shared::crystal(&mut kit.glow, Vec3::new(x, 0.0, z), s, GLOW, GLOW_TIP);
    }
    // Ice floes drift around the island, some carrying crystals or snowy rock.
    for (i, (at, size)) in shared::water_sites(0x1ce5, 15, 2.4).into_iter().enumerate() {
        shared::floe(
            &mut kit.props,
            at,
            0.8 + size * 1.3,
            rgb(0xcde4ee),
            rgb(0xf1fafc),
        );
        let top = at + Vec3::Y * 0.22;
        match i % 4 {
            0 => shared::crystal(&mut kit.props, top, 0.45 * size + 0.2, p.accent, TIP),
            2 => snow_rock(&mut kit.props, top, 0.35 * size + 0.2),
            _ => {}
        }
    }
}

/// Crystal point colors: lit prisms and the unlit glowing ones.
const TIP: Color = Color::srgb(0.78, 0.93, 0.96);
const GLOW: Color = Color::srgb(0.45, 0.85, 0.95);
const GLOW_TIP: Color = Color::srgb(0.88, 0.98, 1.0);

/// A snow-capped column of pale blue rock.
fn spire(parts: &mut Vec<Part>, at: Vec3, h: f32) {
    let p = Theme::Tundra.palette();
    contact(parts, at, 0.9, rgb(0x9cb9c8));
    parts.push((
        Frustum(0.4, 0.85, h * 0.62),
        p.stone,
        Transform::from_translation(at + Vec3::Y * h * 0.31),
    ));
    parts.push((
        Frustum(0.1, 0.42, h * 0.5),
        rgb(0xb9d4e2),
        Transform::from_translation(at + Vec3::Y * h * 0.8)
            .with_rotation(Quat::from_rotation_z(0.06)),
    ));
    parts.push((
        Cone(0.34, 0.5),
        rgb(0xeef8fa),
        Transform::from_translation(at + Vec3::Y * (h * 1.05 + 0.05)),
    ));
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
