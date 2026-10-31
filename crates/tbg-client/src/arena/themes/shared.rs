//! Pieces the four themed kits share: open-water sites, ice floes, lily pads, plank
//! walkways, dead trees and crystal clusters. Everything is a vertex-colored part that
//! merges with the kit, so none of it adds a draw call.

use super::contact;
use crate::arena::{water::LEVEL, Part};
use crate::art::mesh::Shape::*;
use bevy::prelude::*;
use tbg_core::Pcg32;

/// Deterministic open-water sites around the island that stay inside the camera
/// frame: `(position at the water level, size 0.5..1.0)`, at least `gap` apart.
pub(super) fn water_sites(seed: u64, count: usize, gap: f32) -> Vec<(Vec3, f32)> {
    let mut rng = Pcg32::new(seed, 5);
    let mut unit = || rng.below(10_000) as f32 / 10_000.0;
    let mut sites: Vec<(Vec3, f32)> = Vec::with_capacity(count);
    for _ in 0..600 {
        if sites.len() == count {
            break;
        }
        let x = -6.5 + unit() * 29.0;
        let z = -9.0 + unit() * 17.5;
        // Beyond the island, its cliff and the coastal bands.
        let ring = ((x - 8.0) / 12.6).powi(2) + (z / 6.2).powi(2);
        let at = Vec3::new(x, LEVEL, z);
        let size = 0.5 + unit() * 0.5;
        if ring >= 1.45 && sites.iter().all(|(o, _)| o.distance(at) >= gap) {
            sites.push((at, size));
        }
    }
    sites
}

fn yaw(at: Vec3) -> Quat {
    Quat::from_rotation_y(at.x * 1.3 + at.z * 0.7)
}

/// A flat ice slab with a pale top, `r` across.
pub(super) fn floe(parts: &mut Vec<Part>, at: Vec3, r: f32, body: Color, top: Color) {
    let flat = Vec3::new(1.0, 1.0, 0.74);
    parts.push((
        Cyl(r, 0.24),
        body,
        Transform::from_translation(at + Vec3::Y * 0.05)
            .with_scale(flat)
            .with_rotation(yaw(at)),
    ));
    parts.push((
        Cyl(r * 0.9, 0.06),
        top,
        Transform::from_translation(at + Vec3::Y * 0.19)
            .with_scale(flat)
            .with_rotation(yaw(at)),
    ));
}

/// A floating leaf, with an optional pale bloom.
pub(super) fn lily_pad(parts: &mut Vec<Part>, at: Vec3, r: f32, leaf: Color, bloom: Option<Color>) {
    parts.push((
        Cyl(r, 0.03),
        leaf,
        Transform::from_translation(at + Vec3::Y * 0.03)
            .with_scale(Vec3::new(1.0, 1.0, 0.86))
            .with_rotation(yaw(at)),
    ));
    if let Some(color) = bloom {
        parts.push((
            Cone(r * 0.2, r * 0.45),
            color,
            Transform::from_translation(at + Vec3::new(r * 0.15, 0.2, 0.0)),
        ));
    }
}

/// Three leaning crystal prisms, `tip` on the points.
pub(super) fn crystal(parts: &mut Vec<Part>, at: Vec3, s: f32, color: Color, tip: Color) {
    for (dx, dz, h, tilt) in [
        (0.0, 0.0, 1.0, 0.12),
        (0.23, 0.08, 0.65, -0.28),
        (-0.2, 0.1, 0.5, 0.3),
    ] {
        let rot = Quat::from_rotation_z(tilt);
        let origin = at + Vec3::new(dx * s, 0.0, dz * s);
        parts.push((
            Cyl(0.16 * s, h * s * 0.6),
            color,
            Transform::from_translation(origin + rot * Vec3::Y * h * s * 0.3).with_rotation(rot),
        ));
        parts.push((
            Cone(0.16 * s, h * s * 0.4),
            tip,
            Transform::from_translation(origin + rot * Vec3::Y * h * s * 0.8).with_rotation(rot),
        ));
    }
}

/// A bare trunk with two forked branches.
pub(super) fn dead_tree(parts: &mut Vec<Part>, at: Vec3, s: f32, wood: Color) {
    contact(parts, at, s * 0.45, Color::srgb(0.12, 0.1, 0.13));
    parts.push((
        Frustum(0.06 * s, 0.2 * s, 1.6 * s),
        wood,
        Transform::from_translation(at + Vec3::Y * 0.75 * s),
    ));
    for (side, h) in [(-1.0f32, 0.9f32), (1.0, 1.15)] {
        parts.push((
            Cone(0.09 * s, 0.8 * s),
            wood,
            Transform::from_translation(at + Vec3::new(side * 0.25, h, 0.0) * s)
                .with_rotation(Quat::from_rotation_z(-side * 0.7)),
        ));
        parts.push((
            Cone(0.05 * s, 0.45 * s),
            wood,
            Transform::from_translation(at + Vec3::new(side * 0.52, h + 0.42, 0.04) * s)
                .with_rotation(Quat::from_rotation_z(-side * 1.1)),
        ));
    }
}

/// A plank walkway running along x from `from`: `dir` is 1 or -1, `broken` is a bit
/// mask of planks that are missing, and the pilings reach down to the water.
pub(super) fn walkway(
    parts: &mut Vec<Part>,
    from: Vec3,
    dir: f32,
    planks: u32,
    broken: u32,
    wood: Color,
    dark: Color,
) {
    for i in 0..planks {
        let x = from.x + dir * 0.52 * i as f32;
        if broken >> i & 1 == 1 {
            continue;
        }
        parts.push((
            Box(0.46, 0.07, 1.1),
            wood,
            Transform::from_xyz(x, from.y, from.z)
                .with_rotation(Quat::from_rotation_y((i % 3) as f32 * 0.03 - 0.03)),
        ));
        if i % 2 == 0 {
            for side in [-0.52f32, 0.52] {
                parts.push((
                    Box(0.08, 0.5, 0.08),
                    dark,
                    Transform::from_xyz(x, from.y + 0.25, from.z + side),
                ));
                parts.push((
                    Cyl(0.09, from.y - LEVEL + 0.1),
                    dark,
                    Transform::from_xyz(x, (from.y + LEVEL) * 0.5, from.z + side),
                ));
            }
        }
    }
    let len = 0.52 * planks as f32;
    for side in [-0.52f32, 0.52] {
        parts.push((
            Box(len, 0.05, 0.05),
            wood,
            Transform::from_xyz(
                from.x + dir * (len - 0.52) * 0.5,
                from.y + 0.46,
                from.z + side,
            ),
        ));
    }
}
