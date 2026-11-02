//! Build-time spacing for independent scenery assets around authored landmarks.

use super::Part;
use crate::art::mesh;
use bevy::{mesh::VertexAttributeValues, prelude::*};

pub(super) fn bounds((shape, _, transform): &Part) -> (Vec3, Vec3) {
    let mesh = mesh::build(*shape);
    let Some(VertexAttributeValues::Float32x3(vertices)) = mesh.attribute(Mesh::ATTRIBUTE_POSITION)
    else {
        return (Vec3::ZERO, Vec3::ZERO);
    };
    vertices.iter().fold(
        (Vec3::splat(f32::INFINITY), Vec3::splat(f32::NEG_INFINITY)),
        |(min, max), p| {
            let p = transform.transform_point(Vec3::from(*p));
            (min.min(p), max.max(p))
        },
    )
}

/// Omit an occupied candidate as a whole; never split or distort a compound prop.
pub(super) fn spaced(
    parts: &[Part],
    glow: &[Part],
    candidates: Vec<(usize, Vec3, f32)>,
    radius_scale: f32,
) -> Vec<(usize, Vec3, f32)> {
    let mut occupied: Vec<_> = parts
        .iter()
        .chain(glow)
        .map(bounds)
        .filter(|(_, max)| max.y > 0.02)
        .map(|(min, max)| (min.xz(), max.xz()))
        .collect();
    let mut accepted = Vec::new();
    for candidate @ (_, at, scale) in candidates {
        let radius = Vec2::splat(scale * radius_scale + 0.10);
        let min = at.xz() - radius;
        let max = at.xz() + radius;
        if occupied
            .iter()
            .any(|(lo, hi)| min.cmplt(*hi).all() && max.cmpgt(*lo).all())
        {
            continue;
        }
        occupied.push((min, max));
        accepted.push(candidate);
    }
    accepted
}
