//! Two-tone meadow and a sand-tile BattleLine with recessed grout and border.

use super::{
    geometry::{outline, Surface, SEGMENTS},
    painted,
    theme::Theme,
};
use crate::art::mesh::Shape::*;
use bevy::prelude::*;

pub(super) fn meadow(theme: Theme) -> Mesh {
    let rim = outline();
    let center = Vec3::new(8.0, -0.04, 0.0);
    let mut surface = Surface::default();
    let greens = theme.palette().ground;
    for band in 0..3 {
        for i in 0..SEGMENTS {
            let j = (i + 1) % SEGMENTS;
            let ring = |k: usize, r: f32| {
                let mut at = center.lerp(rim[k], r);
                if r < 0.99 {
                    at.y += (k as f32 * 2.3 + r * 4.0).sin() * 0.007;
                }
                at
            };
            let lo = [0.0, 0.56, 0.80][band];
            let hi = [0.56, 0.80, 1.0][band];
            let a = ring(i, lo);
            let b = ring(j, lo);
            let c = ring(j, hi);
            let d = ring(i, hi);
            let col = greens[(i / 4 + band) % 2];
            if band == 0 {
                surface.triangle(center, c, d, col);
            } else {
                surface.quad([a, b, c, d], col);
            }
        }
    }
    surface.finish()
}

pub(super) fn lane(theme: Theme) -> Mesh {
    let [border, tile_color, variation, lip] = theme.palette().lane;
    let mut parts = vec![(
        Box(18.2, 0.035, 3.34),
        border,
        Transform::from_xyz(8.0, -0.012, 0.0),
    )];
    for tile in 0..18 {
        let x = -0.5 + tile as f32;
        let sand = if tile % 3 == 0 { variation } else { tile_color };
        parts.push((
            Box(0.974, 0.036, 3.08),
            sand,
            Transform::from_xyz(x, 0.0, 0.0),
        ));
        if theme == Theme::Volcanic && tile % 3 == 1 {
            for (z, angle) in [(-0.65, 0.6), (0.15, -0.35), (0.85, 0.3)] {
                parts.push((
                    Box(0.016, 0.003, 0.85),
                    super::theme::rgb(0x645a65),
                    Transform::from_xyz(x + 0.1, 0.020, z)
                        .with_rotation(Quat::from_rotation_y(angle)),
                ));
            }
        }
        // A thin sunlit lip makes each seam legible without a texture.
        parts.push((
            Box(0.025, 0.038, 3.04),
            lip,
            Transform::from_xyz(x - 0.47, 0.003, 0.0),
        ));
    }
    painted(parts)
}
