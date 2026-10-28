//! Edge props placed from a fixed seed on the island rim, never on the
//! lane: trees on the far side, only low rocks/bushes/ferns on the camera side. Returned as parts to merge.

use bevy::prelude::*;
use tbg_core::Pcg32;

use crate::art::mesh::Shape::{self, *};
use crate::art::palette::*;

type Part = (Shape, Color, Transform);

pub(super) fn scatter() -> Vec<Part> {
    let mut rng = Pcg32::new(0x7ab1e, 3);
    let mut unit = move || rng.below(10_000) as f32 / 10_000.0;
    let mut parts = Vec::new();
    for i in 0..120 {
        let x = -3.5 + unit() * 23.0;
        let side = if i % 2 == 0 { 1.0 } else { -1.0 };
        let z = side * (2.3 + unit() * 3.4);
        // Keep inside the island ellipse (13 × 6.4 around x = 8).
        let (dx, dz) = ((x - 8.0) / 12.4, z / 6.0);
        if dx * dx + dz * dz > 1.0 {
            continue;
        }
        let s = 0.75 + unit() * 0.6;
        let at = Vec3::new(x, 0.0, z);
        if side > 0.0 {
            // Camera side: only low props, so nothing hides the lane.
            match i % 3 {
                0 => rock(&mut parts, at, s, unit()),
                1 => bush(&mut parts, at, s),
                _ => fern(&mut parts, at, s),
            }
            continue;
        }
        match i % 4 {
            0 | 1 => pine(&mut parts, at, s),
            2 => round_tree(&mut parts, at, s),
            _ => rock(&mut parts, at, s, unit()),
        }
    }
    parts
}

fn t(at: Vec3, y: f32) -> Transform {
    Transform::from_translation(at + Vec3::Y * y)
}

fn pine(parts: &mut Vec<Part>, at: Vec3, s: f32) {
    parts.push((Cyl(0.09 * s, 0.5 * s), BARK, t(at, 0.25 * s)));
    parts.push((Cone(0.55 * s, 0.9 * s), PINE, t(at, 0.85 * s)));
    parts.push((Cone(0.4 * s, 0.7 * s), PINE, t(at, 1.3 * s)));
}

fn round_tree(parts: &mut Vec<Part>, at: Vec3, s: f32) {
    parts.push((Cyl(0.08 * s, 0.6 * s), BARK, t(at, 0.3 * s)));
    parts.push((Ball(0.5 * s), LEAF, t(at, 0.95 * s)));
    parts.push((
        Ball(0.32 * s),
        LEAF_DARK,
        t(at + Vec3::new(0.25, 0.0, 0.15) * s, 0.75 * s),
    ));
}

fn rock(parts: &mut Vec<Part>, at: Vec3, s: f32, r: f32) {
    let tr = t(at, 0.08 * s)
        .with_scale(Vec3::new(1.3, 0.7, 1.0) * 0.35 * s)
        .with_rotation(Quat::from_rotation_y(r * 6.0));
    parts.push((Ball(1.0), ROCK, tr));
}

fn bush(parts: &mut Vec<Part>, at: Vec3, s: f32) {
    parts.push((
        Ball(0.28 * s),
        LEAF,
        t(at, 0.14 * s).with_scale(Vec3::new(1.4, 0.8, 1.1)),
    ));
    parts.push((
        Ball(0.2 * s),
        LEAF_DARK,
        t(at + Vec3::new(0.2, 0.0, 0.1) * s, 0.1 * s),
    ));
}

fn fern(parts: &mut Vec<Part>, at: Vec3, s: f32) {
    for k in 0..3 {
        let off = Vec3::new(k as f32 * 0.12 - 0.12, 0.0, (k % 2) as f32 * 0.1);
        parts.push((Cone(0.12 * s, 0.35 * s), LEAF_DARK, t(at + off, 0.17 * s)));
    }
}
