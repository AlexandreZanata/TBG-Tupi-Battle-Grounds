//! Squat faceted boulders with dark contact patches.

use super::{trees, Part};
use crate::art::{mesh::Shape::Rock, palette::ROCK};
use bevy::prelude::*;

pub(super) fn boulder(parts: &mut Vec<Part>, at: Vec3, s: f32, angle: f32) {
    trees::ao(parts, at, s * 0.8);
    parts.push((
        Rock(1.0, at.x.to_bits() ^ at.z.to_bits().rotate_left(13)),
        ROCK,
        Transform::from_translation(at + Vec3::Y * s * 0.30)
            .with_scale(Vec3::new(0.85, 0.72, 0.65) * s)
            .with_rotation(Quat::from_euler(EulerRot::XYZ, 0.17, angle, 0.13)),
    ));
}
