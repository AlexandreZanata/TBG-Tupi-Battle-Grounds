//! Cached, nose-forward fish geometry and cosmetic Evolution size; no trajectory or hit logic.

use crate::art::{mesh::Shape, palette::*};
use bevy::prelude::*;

pub(super) fn scale(tier: u8) -> f32 {
    [1.0, 1.35, 1.7][usize::from(tier.min(2))]
}

pub(super) fn parts() -> Vec<(Shape, Color, Transform)> {
    use Shape::*;
    let mut parts = vec![
        (
            Ball(0.085),
            FISH_BACK,
            Transform::from_scale(Vec3::new(0.75, 1.0, 1.65)),
        ),
        (
            Ball(0.075),
            FISH_SILVER,
            Transform::from_xyz(0.0, -0.025, 0.025).with_scale(Vec3::new(0.82, 0.75, 1.65)),
        ),
        (
            Ball(0.073),
            FISH_SILVER,
            Transform::from_xyz(0.0, 0.0, 0.105).with_scale(Vec3::new(0.86, 0.95, 0.9)),
        ),
        (
            Ball(0.038),
            FISH_BACK,
            Transform::from_xyz(0.0, 0.0, -0.15).with_scale(Vec3::new(0.8, 0.9, 1.3)),
        ),
        (
            Ring(0.008, 0.025, 360.0),
            DARK,
            Transform::from_xyz(0.0, -0.006, 0.174)
                .with_rotation(Quat::from_rotation_x(std::f32::consts::FRAC_PI_2)),
        ),
        (
            Cone(0.06, 0.10),
            FISH_BACK,
            Transform::from_xyz(0.0, 0.095, -0.02)
                .with_rotation(Quat::from_rotation_x(-25.0_f32.to_radians()))
                .with_scale(Vec3::new(0.2, 1.0, 1.35)),
        ),
    ];
    for side in [-1.0_f32, 1.0] {
        // Two eyes and gill covers remain readable when the flight crosses either seat.
        parts.extend([
            (
                Ball(0.023),
                IVORY,
                Transform::from_xyz(side * 0.055, 0.022, 0.12),
            ),
            (
                Ball(0.012),
                DARK,
                Transform::from_xyz(side * 0.074, 0.024, 0.125),
            ),
            (
                Cyl(0.038, 0.012),
                FISH_BACK,
                Transform::from_xyz(side * 0.064, -0.005, 0.066)
                    .with_rotation(Quat::from_rotation_z(std::f32::consts::FRAC_PI_2)),
            ),
            (
                Cone(0.043, 0.09),
                AMBER,
                Transform::from_xyz(side * 0.075, -0.03, 0.01)
                    .with_rotation(Quat::from_rotation_z(side * -65.0_f32.to_radians()))
                    .with_scale(Vec3::new(0.75, 1.0, 0.35)),
            ),
            (
                Cone(0.07, 0.14),
                FISH_BACK,
                Transform::from_xyz(0.0, side * 0.053, -0.21)
                    .with_rotation(Quat::from_rotation_x(if side > 0.0 {
                        -25.0_f32.to_radians()
                    } else {
                        205.0_f32.to_radians()
                    }))
                    .with_scale(Vec3::new(0.23, 1.0, 0.65)),
            ),
        ]);
    }
    parts
}
