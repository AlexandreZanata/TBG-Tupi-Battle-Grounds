//! Seeded meadow dressing, with tall silhouettes confined to the back rim.

use super::{camp, rocks, trees, Part};
use bevy::prelude::*;
use tbg_core::Pcg32;

pub(super) fn scatter() -> Vec<Part> {
    let mut rng = Pcg32::new(0x16a_f07e57, 3);
    let mut unit = || rng.below(10_000) as f32 / 10_000.0;
    let mut parts = Vec::new();
    for i in 0..46 {
        let x = -3.0 + (i % 16) as f32 * 1.42 + unit() * 0.65;
        let z = -2.9 - (i / 16) as f32 * 0.92 - unit() * 0.45;
        if !inside(x, z) {
            continue;
        }
        trees::pine(&mut parts, Vec3::new(x, 0.0, z), 0.75 + unit() * 0.8);
    }
    for (x, z, scale) in [
        (-2.5, -2.8, 1.0),
        (2.0, -3.4, 1.2),
        (6.6, -4.8, 1.4),
        (13.0, -3.6, 1.1),
        (18.2, -2.8, 1.0),
        (0.0, 3.0, 0.65),
        (4.4, 4.0, 1.0),
        (9.6, 4.8, 0.8),
        (16.7, 3.5, 0.9),
    ] {
        rocks::boulder(&mut parts, Vec3::new(x, 0.0, z), scale, unit() * 6.0);
    }
    for i in 0..150 {
        let x = -3.7 + unit() * 23.4;
        let z = (if i % 2 == 0 { 1.0 } else { -1.0 }) * (2.7 + unit() * 2.6);
        if !inside(x, z) {
            continue;
        }
        let at = Vec3::new(x, 0.0, z);
        match i % 8 {
            0 => trees::bush(&mut parts, at, 0.65 + unit() * 0.4),
            1 => rocks::boulder(&mut parts, at, 0.16 + unit() * 0.18, unit() * 6.0),
            2 => trees::flowers(&mut parts, at),
            3 if z > 3.5 => trees::pine(&mut parts, at, 0.24 + unit() * 0.12),
            _ => trees::grass(&mut parts, at, unit() * 6.0),
        }
    }
    for (x, z, s) in [
        (0.0, 3.0, 0.65),
        (4.4, 4.0, 1.0),
        (9.6, 4.8, 0.8),
        (16.7, 3.5, 0.9),
    ] {
        let at = Vec3::new(x, 0.0, z);
        trees::bush(&mut parts, at + Vec3::new(0.55, 0.0, 0.18), s * 0.58);
        rocks::boulder(&mut parts, at + Vec3::new(-0.50, 0.0, 0.24), s * 0.45, x);
        trees::flowers(&mut parts, at + Vec3::new(0.22, 0.0, 0.60));
    }
    camp::fences(&mut parts);
    camp::supplies(&mut parts);
    camp::log(&mut parts, Vec3::new(10.9, 0.0, 4.3));
    parts
}

fn inside(x: f32, z: f32) -> bool {
    ((x - 8.0) / 11.3).powi(2) + (z / 5.25).powi(2) < 1.0
}
