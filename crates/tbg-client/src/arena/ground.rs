//! Two-tone meadow and a sand-tile BattleLine with recessed grout and border.

use super::{
    geometry::{outline, Surface, SEGMENTS},
    painted,
};
use crate::art::{mesh::Shape::*, palette::*};
use bevy::prelude::*;

pub(super) fn meadow() -> Mesh {
    let rim = outline();
    let center = Vec3::new(8.0, -0.04, 0.0);
    let mut surface = Surface::default();
    let greens = [Color::srgb(0.47, 0.65, 0.20), Color::srgb(0.53, 0.70, 0.24)];
    for i in 0..SEGMENTS {
        let j = (i + 1) % SEGMENTS;
        let inner = center.lerp(rim[i], 0.56);
        let next = center.lerp(rim[j], 0.56);
        surface.triangle(center, next, inner, greens[(i / 5) % 2]);
        surface.quad([inner, next, rim[j], rim[i]], greens[(i / 3 + 1) % 2]);
    }
    surface.finish()
}

pub(super) fn lane() -> Mesh {
    let mut parts = vec![(
        Box(18.2, 0.035, 3.34),
        LANE_DARK,
        Transform::from_xyz(8.0, -0.012, 0.0),
    )];
    for tile in 0..18 {
        let x = -0.5 + tile as f32;
        let sand = if tile % 3 == 0 {
            Color::srgb(0.82, 0.64, 0.37)
        } else {
            LANE
        };
        parts.push((
            Box(0.974, 0.036, 3.08),
            sand,
            Transform::from_xyz(x, 0.0, 0.0),
        ));
        // A thin sunlit lip makes each seam legible without a texture.
        parts.push((
            Box(0.025, 0.038, 3.04),
            Color::srgb(0.89, 0.73, 0.46),
            Transform::from_xyz(x - 0.47, 0.003, 0.0),
        ));
    }
    painted(parts)
}
