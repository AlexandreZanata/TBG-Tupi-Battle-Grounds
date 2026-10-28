//! Faceted gray stone strata, following the meadow's irregular perimeter.

use super::geometry::{outline, Surface, SEGMENTS};
use bevy::prelude::*;

pub(super) fn mesh() -> Mesh {
    let rim = outline();
    let center = Vec3::new(8.0, 0.0, 0.0);
    let mut surface = Surface::default();
    for band in 0..3 {
        for i in 0..SEGMENTS {
            let j = (i + 1) % SEGMENTS;
            let point = |k: usize, level: usize| {
                let scale = [1.0, 1.025, 1.005, 0.975][level];
                let mut p = center + (rim[k] - center) * scale;
                p.y = [-0.045, -0.43, -0.94, -1.55][level];
                if level == 1 || level == 2 {
                    p.y += (k % 3) as f32 * 0.045;
                }
                p
            };
            let value = [0.68, 0.57, 0.43][band] + (i % 5) as f32 * 0.018;
            let light = Color::srgb(value, value * 0.97, value * 0.92);
            let shade = Color::srgb(value * 0.88, value * 0.86, value * 0.85);
            let [a, b, c, d] = [
                point(i, band),
                point(j, band),
                point(j, band + 1),
                point(i, band + 1),
            ];
            surface.triangle(a, b, c, light);
            surface.triangle(a, c, d, shade);
        }
    }
    surface.finish()
}
