//! Turquoise sea, shallow coastal bands, and ten offshore rocks with foam rings.

use super::{
    geometry::{outline, Surface, SEGMENTS},
    painted,
};
use crate::art::{mesh::Shape::*, palette::*};
use bevy::prelude::*;

pub(super) fn sea() -> Mesh {
    painted(vec![(
        Box(180.0, 0.02, 180.0),
        WATER,
        Transform::from_xyz(8.0, -1.40, 0.0),
    )])
}

pub(super) fn coast() -> Mesh {
    let rim = outline();
    let center = Vec3::new(8.0, 0.0, 0.0);
    let mut surface = Surface::default();
    for (inside, outside, color) in [
        (0.97, 1.045, Color::srgb(0.72, 0.91, 0.85)),
        (1.045, 1.085, Color::srgb(0.35, 0.75, 0.74)),
        (1.085, 1.13, Color::srgb(0.26, 0.69, 0.72)),
    ] {
        for i in 0..SEGMENTS {
            let j = (i + 1) % SEGMENTS;
            let point = |k: usize, s: f32| {
                let mut p = center + (rim[k] - center) * s;
                p.y = -1.375;
                p
            };
            surface.quad(
                [
                    point(i, inside),
                    point(j, inside),
                    point(j, outside),
                    point(i, outside),
                ],
                color,
            );
        }
    }
    surface.finish()
}

pub(super) fn rocks() -> Mesh {
    let mut parts = Vec::new();
    for i in 0..10 {
        let a = (i as f32 + 0.35) * std::f32::consts::TAU / 10.0;
        let p = Vec3::new(8.0 + a.cos() * 13.0, -1.30, a.sin() * 6.6);
        let s = 0.35 + (i % 3) as f32 * 0.16;
        parts.push((
            Ball(1.0),
            ROCK,
            Transform::from_translation(p)
                .with_scale(Vec3::new(s, s * 1.3, s * 0.9))
                .with_rotation(Quat::from_rotation_y(a)),
        ));
        parts.push((
            Ring(0.028, s * 1.18, 360.0),
            Color::srgb(0.76, 0.93, 0.89),
            Transform::from_translation(p - Vec3::Y * 0.065).with_scale(Vec3::new(1.0, 0.3, 0.9)),
        ));
    }
    painted(parts)
}
