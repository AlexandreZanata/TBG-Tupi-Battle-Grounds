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
    for i in 0..SEGMENTS {
        let j = (i + 1) % SEGMENTS;
        let inner = center.lerp(rim[i], 0.56);
        let next = center.lerp(rim[j], 0.56);
        surface.triangle(center, next, inner, greens[(i / 5) % 2]);
        surface.quad([inner, next, rim[j], rim[i]], greens[(i / 3 + 1) % 2]);
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
        // A thin sunlit lip makes each seam legible without a texture.
        parts.push((
            Box(0.025, 0.038, 3.04),
            lip,
            Transform::from_xyz(x - 0.47, 0.003, 0.0),
        ));
    }
    painted(parts)
}
