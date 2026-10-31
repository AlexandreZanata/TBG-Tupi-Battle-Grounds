//! Four procedural environment kits assembled with the original arena mesh pipeline.

use super::{theme::Theme, Part};
use bevy::prelude::*;
use tbg_core::Pcg32;

mod ruins;
mod shared;
mod storm;
pub(super) mod structures;
mod tundra;
mod volcanic;

#[derive(Default)]
pub(super) struct Kit {
    pub props: Vec<Part>,
    pub glow: Vec<Part>,
}

pub(super) fn build(theme: Theme) -> Kit {
    let mut kit = Kit::default();
    match theme {
        Theme::Tundra => tundra::build(&mut kit),
        Theme::Volcanic => volcanic::build(&mut kit),
        Theme::Ruins => ruins::build(&mut kit),
        Theme::Storm => storm::build(&mut kit),
        Theme::Forest => return kit,
    }
    shore_rocks(&mut kit, theme);
    structures::fittings(&mut kit.props, theme);
    kit
}

/// The same rear-tree envelope and foreground clearance as the base diorama.
pub(super) fn back_row() -> Vec<(Vec3, f32)> {
    let mut rng = Pcg32::new(0x16a_f07e57, 3);
    let mut unit = || rng.below(10_000) as f32 / 10_000.0;
    (0..46)
        .filter_map(|i| {
            let x = -3.0 + (i % 16) as f32 * 1.42 + unit() * 0.65;
            let z = -2.9 - (i / 16) as f32 * 0.92 - unit() * 0.45;
            inside(x, z).then(|| (Vec3::new(x, 0.0, z), 0.75 + unit() * 0.8))
        })
        .collect()
}

pub(super) fn scatter(seed: u64) -> Vec<(usize, Vec3, f32)> {
    let mut rng = Pcg32::new(seed, 11);
    let mut unit = || rng.below(10_000) as f32 / 10_000.0;
    (0..100)
        .filter_map(|i| {
            let x = -3.7 + unit() * 23.4;
            let z = (if i % 2 == 0 { 1.0 } else { -1.0 }) * (2.9 + unit() * 2.3);
            inside(x, z).then(|| (i, Vec3::new(x, 0.0, z), 0.25 + unit() * 0.35))
        })
        .collect()
}

fn inside(x: f32, z: f32) -> bool {
    ((x - 8.0) / 11.3).powi(2) + (z / 5.25).powi(2) < 1.0
}

pub(super) fn contact(parts: &mut Vec<Part>, at: Vec3, radius: f32, color: Color) {
    parts.push((
        crate::art::mesh::Shape::Cyl(radius, 0.008),
        color,
        Transform::from_translation(at - Vec3::Y * 0.025).with_scale(Vec3::new(1.0, 1.0, 0.72)),
    ));
}

pub(super) fn boulder(parts: &mut Vec<Part>, at: Vec3, s: f32, color: Color) {
    parts.push((
        crate::art::mesh::Shape::Rock(1.0, at.x.to_bits() ^ at.z.to_bits().rotate_left(13)),
        color,
        Transform::from_translation(at + Vec3::Y * s * 0.28)
            .with_scale(Vec3::new(0.9, 0.65, 0.73) * s)
            .with_rotation(Quat::from_euler(EulerRot::XYZ, 0.17, at.x, 0.13)),
    ));
}

/// Low coastal groups create geological continuity without entering the BattleLine.
fn shore_rocks(kit: &mut Kit, theme: Theme) {
    use crate::art::mesh::Shape::{Rock, SnowCap};
    let p = theme.palette();
    let rim = super::geometry::outline();
    let center = Vec3::new(8.0, 0.0, 0.0);
    for i in [7, 12, 19, 25, 39, 45, 51, 57] {
        let at = center + (rim[i] - center) * 0.93;
        for (j, (dx, dz, s)) in [(-0.35, 0.0, 0.53), (0.30, 0.10, 0.37), (0.0, -0.20, 0.72)]
            .into_iter()
            .enumerate()
        {
            let at = at + Vec3::new(dx, 0.0, dz);
            let seed = at.x.to_bits() ^ at.z.to_bits().rotate_left(13);
            let transform = Transform::from_translation(at + Vec3::Y * s * 0.20)
                .with_scale(Vec3::new(s, s * 0.82, s * 0.88))
                .with_rotation(Quat::from_rotation_y(i as f32));
            kit.props.push((
                Rock(1.0, seed),
                if j == 1 { p.dark } else { p.stone },
                transform,
            ));
            if theme == Theme::Tundra || theme == Theme::Storm {
                kit.props.push((
                    SnowCap(1.0, seed),
                    if theme == Theme::Tundra {
                        super::theme::rgb(0xe7f2fa)
                    } else {
                        super::theme::rgb(0x9bb0c5)
                    },
                    transform,
                ));
            }
        }
    }
}
