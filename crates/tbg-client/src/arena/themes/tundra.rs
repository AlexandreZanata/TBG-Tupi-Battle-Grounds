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
    for (i, (at, s)) in back_row().into_iter().enumerate() {
        if i % 4 != 2 {
            fir(&mut kit.props, at, s * 0.9);
        }
    }
    for (i, at, s) in scatter(0x1ce).into_iter().filter(|(i, _, _)| i % 3 != 1) {
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
    for (x, z, s) in [
        (2.3, -4.4, 1.25),
        (6.5, -4.85, 1.45),
        (10.2, -4.9, 1.30),
        (14.1, -4.25, 1.10),
    ] {
        ridge(&mut kit.props, Vec3::new(x, 0.0, z), s);
    }
    for (x, z, s) in [
        (0.7, 3.6, 0.8),
        (4.8, 4.5, 0.75),
        (12.6, 4.35, 0.8),
        (16.0, 3.6, 0.65),
    ] {
        fir(&mut kit.props, Vec3::new(x, 0.0, z), s * 0.65);
        crystal(
            &mut kit.props,
            Vec3::new(x + 0.7, 0.0, z + 0.1),
            s,
            rgb(0x36aad5),
        );
    }
    for (i, at) in crate::arena::geometry::outline().into_iter().enumerate() {
        if at.z > 2.5 && i % 3 == 0 {
            kit.props.push((
                Cone(0.13, 0.65 + (i % 4) as f32 * 0.12),
                rgb(0x90d8ef),
                Transform::from_xyz(at.x, -0.70, at.z * 1.012)
                    .with_rotation(Quat::from_rotation_z(std::f32::consts::PI)),
            ));
        }
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
            Canopy(r * s, h * s, at.x.to_bits() ^ (at.z + y).to_bits()),
            p.plant,
            Transform::from_translation(at + Vec3::Y * y * s),
        ));
        parts.push((
            Canopy(
                r * s * 0.94,
                h * s * 0.76,
                at.x.to_bits() ^ (at.z + y).to_bits(),
            ),
            rgb(0xf0f8ff),
            Transform::from_translation(at + Vec3::Y * (y + h * 0.15) * s),
        ));
    }
}

fn snow_rock(parts: &mut Vec<Part>, at: Vec3, s: f32) {
    contact(parts, at, s * 0.8, rgb(0x91b0c1));
    let seed = at.x.to_bits() ^ at.z.to_bits().rotate_left(13);
    let transform = Transform::from_translation(at + Vec3::Y * s * 0.32)
        .with_scale(Vec3::new(0.95, 0.85, 0.80) * s)
        .with_rotation(Quat::from_rotation_y(at.x));
    parts.push((Rock(1.0, seed), rgb(0x6d8b9e), transform));
    parts.push((SnowCap(1.0, seed), rgb(0xeaf5fc), transform));
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

/// A common mineral silhouette keeps the fitted snow crust attached to every ridge.
fn ridge(parts: &mut Vec<Part>, at: Vec3, s: f32) {
    for (dx, dz, h, r) in [
        (0.0, 0.0, 2.1, 0.9),
        (-0.7, 0.1, 1.3, 0.65),
        (0.7, 0.15, 1.6, 0.7),
    ] {
        let base = at + Vec3::new(dx, 0.0, dz) * s;
        let seed = base.x.to_bits() ^ base.z.to_bits().rotate_left(13);
        let transform = Transform::from_translation(base + Vec3::Y * h * s * 0.44)
            .with_scale(Vec3::new(r, h * 0.9, r * 0.8) * s)
            .with_rotation(Quat::from_rotation_y(dx * 0.8));
        parts.push((Rock(1.0, seed), rgb(0x6f94b0), transform));
        parts.push((SnowCap(1.0, seed), rgb(0xdeedf9), transform));
    }
    crystal(
        parts,
        at + Vec3::new(0.6, 0.0, 0.55) * s,
        0.75 * s,
        rgb(0x46abd1),
    );
}
