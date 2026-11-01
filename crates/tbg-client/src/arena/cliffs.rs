//! Faceted gray stone strata, following the meadow's irregular perimeter.

use super::{
    geometry::{outline, Surface, SEGMENTS},
    theme::Theme,
};
use bevy::prelude::*;

pub(super) fn mesh(theme: Theme) -> Mesh {
    let rim = outline();
    let center = Vec3::new(8.0, 0.0, 0.0);
    let mut surface = Surface::default();
    for band in 0..4 {
        for i in 0..SEGMENTS {
            let j = (i + 1) % SEGMENTS;
            let point = |k: usize, level: usize| {
                let crag = (k as f32 * 1.9).sin() * 0.013 + (k as f32 * 3.1).cos() * 0.009;
                let scale =
                    [1.0, 1.032, 1.006, 1.030, 0.975][level] + if level == 0 { 0.0 } else { crag };
                let mut p = center + (rim[k] - center) * scale;
                p.y = [-0.045, -0.30, -0.68, -1.08, -1.58][level];
                if level > 0 && level < 4 {
                    p.y += (k as f32 * 2.1).sin() * 0.065;
                }
                p
            };
            let value = [0.73, 0.66, 0.55, 0.43][band] + (i % 5) as f32 * 0.018;
            let (light, shade) = if theme == Theme::Forest {
                (
                    Color::srgb(value, value * 0.97, value * 0.92),
                    Color::srgb(value * 0.88, value * 0.86, value * 0.85),
                )
            } else {
                let color = theme.palette().cliff[[0, 0, 1, 2][band]].to_srgba();
                let k = 1.0 + (i % 5) as f32 * 0.025;
                (
                    Color::srgb(color.red * k, color.green * k, color.blue * k),
                    Color::srgb(color.red * 0.85, color.green * 0.85, color.blue * 0.85),
                )
            };
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
