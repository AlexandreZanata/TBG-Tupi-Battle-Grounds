//! Low-poly tier-2/3 additions, offshore platforms and plank bridges for the
//! War Auxiliary precincts; every builder reads its colors from the Arena palette.

use super::models::team_trim;
use super::palette::AuxPalette;
use crate::arena::{painted, water, Part};
use crate::art::mesh::Shape::*;
use bevy::prelude::*;

/// Distance between bridge planks along the bridge axis.
pub(super) const PLANK_STEP: f32 = 0.15;
const PLANK_WIDTH: f32 = 0.46;

/// Stilted plank deck covering a pad (`center`, `half` in precinct-local XZ).
pub(super) fn platform(pal: AuxPalette, center: Vec2, half: Vec2) -> Mesh {
    let mut parts: Vec<Part> = Vec::with_capacity(40);
    let rows = ((half.y * 2.0) / 0.16).ceil() as usize;
    for i in 0..rows {
        let z = center.y - half.y + 0.08 + i as f32 * 0.16;
        // Alternate two wood tones so the deck reads as planks, not a slab.
        let wood = if i % 2 == 0 {
            pal.pier_wood
        } else {
            pal.timber_beam
        };
        parts.push((
            Box(half.x * 2.0, 0.05, 0.14),
            wood,
            Transform::from_xyz(center.x, -0.025, z),
        ));
    }
    let depth = -water::LEVEL + 0.05;
    for dx in [-1.0, 0.0, 1.0] {
        for dz in [-1.0, 1.0] {
            parts.push((
                Cyl(0.06, depth),
                pal.pier_piling,
                Transform::from_xyz(
                    center.x + dx * (half.x - 0.08),
                    -depth * 0.5 - 0.05,
                    center.y + dz * (half.y - 0.08),
                ),
            ));
        }
    }
    // Edge beams frame the deck on every side.
    for dz in [-1.0, 1.0] {
        parts.push((
            Box(half.x * 2.0 + 0.06, 0.07, 0.06),
            pal.pier_piling,
            Transform::from_xyz(center.x, -0.07, center.y + dz * half.y),
        ));
    }
    painted(parts)
}

/// One bridge plank, its axis along local X.
pub(super) fn bridge_plank(pal: AuxPalette, index: usize) -> Mesh {
    let wood = if index.is_multiple_of(2) {
        pal.pier_wood
    } else {
        pal.timber_beam
    };
    painted(vec![
        (
            Box(0.12, 0.04, PLANK_WIDTH),
            wood,
            Transform::from_xyz(0.0, -0.02, 0.0),
        ),
        // A short stilt under every third plank, down into the water.
        if index % 3 == 1 {
            (
                Cyl(0.04, -water::LEVEL),
                pal.pier_piling,
                Transform::from_xyz(0.0, water::LEVEL * 0.5, 0.0),
            )
        } else {
            (
                Box(0.13, 0.02, 0.05),
                pal.pier_piling,
                Transform::from_xyz(0.0, -0.05, 0.0),
            )
        },
    ])
}

/// Posts and rope hand rails along a bridge of `length`, axis along local X.
pub(super) fn bridge_rails(pal: AuxPalette, length: f32) -> Mesh {
    let mut parts: Vec<Part> = Vec::with_capacity(16);
    let posts = (length / 0.45).ceil().max(1.0) as usize;
    let rope = pal.sack_cloth;
    for side in [-1.0, 1.0] {
        let z = side * (PLANK_WIDTH * 0.5 + 0.02);
        for i in 0..=posts {
            let x = length * i as f32 / posts as f32;
            parts.push((
                Box(0.05, 0.34, 0.05),
                pal.pier_piling,
                Transform::from_xyz(x, 0.15, z),
            ));
        }
        parts.push((
            Box(length, 0.025, 0.025),
            rope,
            Transform::from_xyz(length * 0.5, 0.28, z),
        ));
    }
    painted(parts)
}

/// Tier 2 Miner winch: A-frame with a crank drum (the drum spins separately).
pub(super) fn winch_frame(pal: AuxPalette) -> Mesh {
    let mut parts: Vec<Part> = Vec::with_capacity(8);
    for dz in [-0.16, 0.16] {
        for lean in [-0.32, 0.32] {
            parts.push((
                Box(0.06, 0.78, 0.06),
                pal.timber_post,
                Transform::from_xyz(lean * 0.35, 0.36, dz)
                    .with_rotation(Quat::from_rotation_z(lean)),
            ));
        }
    }
    parts.push((
        Box(0.08, 0.08, 0.44),
        pal.timber_beam,
        Transform::from_xyz(0.0, 0.72, 0.0),
    ));
    parts.push((
        Box(0.5, 0.05, 0.42),
        pal.timber_beam,
        Transform::from_xyz(0.0, 0.03, 0.0),
    ));
    painted(parts)
}

pub(super) fn winch_drum(pal: AuxPalette) -> Mesh {
    painted(vec![
        (
            Cyl(0.09, 0.3),
            pal.timber_beam,
            Transform::from_rotation(Quat::from_rotation_x(std::f32::consts::FRAC_PI_2)),
        ),
        (
            Box(0.03, 0.26, 0.03),
            pal.iron_rail,
            Transform::from_xyz(0.0, 0.0, 0.17),
        ),
        (
            Ring(0.012, 0.09, 360.0),
            pal.iron_rail,
            Transform::from_xyz(0.0, 0.0, 0.1),
        ),
    ])
}

/// Bucket hanging from the winch rope; it rises and falls while the drum turns.
pub(super) fn ore_bucket(pal: AuxPalette) -> Mesh {
    painted(vec![
        (
            Box(0.015, 0.4, 0.015),
            pal.sack_cloth,
            Transform::from_xyz(0.0, 0.2, 0.0),
        ),
        (
            Frustum(0.09, 0.07, 0.12),
            pal.timber_post,
            Transform::from_xyz(0.0, -0.02, 0.0),
        ),
        (
            Ball(0.05),
            pal.ore_crystal,
            Transform::from_xyz(0.0, 0.05, 0.0),
        ),
    ])
}

/// Glowing heap of mined ore beside the rails.
pub(super) fn ore_pile(pal: AuxPalette) -> Mesh {
    let mut parts: Vec<Part> = vec![(
        Rock(0.2, 71),
        pal.rock_inner,
        Transform::from_xyz(0.0, 0.08, 0.0).with_scale(Vec3::new(1.2, 0.6, 1.0)),
    )];
    for (dx, dz, s) in [
        (0.05, 0.02, 1.0),
        (-0.08, 0.06, 0.8),
        (0.02, -0.08, 0.7),
        (0.1, -0.02, 0.6),
    ] {
        parts.push((
            Cone(0.05 * s, 0.14 * s),
            pal.ore_crystal,
            Transform::from_xyz(dx, 0.14, dz).with_rotation(Quat::from_rotation_z(dx * 4.0)),
        ));
    }
    painted(parts)
}

/// Tier 3 Miner headframe: tall timber tower over the shaft.
pub(super) fn headframe(pal: AuxPalette) -> Mesh {
    let mut parts: Vec<Part> = Vec::with_capacity(14);
    for (dx, dz) in [(-0.18, -0.18), (0.18, -0.18), (-0.18, 0.18), (0.18, 0.18)] {
        parts.push((
            Box(0.07, 1.3, 0.07),
            pal.timber_post,
            Transform::from_xyz(dx, 0.65, dz),
        ));
    }
    for y in [0.4, 0.85] {
        for dz in [-0.18, 0.18] {
            parts.push((
                Box(0.42, 0.05, 0.05),
                pal.timber_beam,
                Transform::from_xyz(0.0, y, dz),
            ));
        }
        for dx in [-0.18, 0.18] {
            parts.push((
                Box(0.05, 0.05, 0.42),
                pal.timber_beam,
                Transform::from_xyz(dx, y, 0.0),
            ));
        }
    }
    parts.push((
        Box(0.5, 0.08, 0.5),
        pal.timber_beam,
        Transform::from_xyz(0.0, 1.32, 0.0),
    ));
    parts.push((
        Box(0.04, 0.5, 0.04),
        pal.iron_rail,
        Transform::from_xyz(0.0, 1.05, 0.0),
    ));
    painted(parts)
}

/// Spoked sheave wheel at the headframe top, spinning about local Z.
pub(super) fn sheave(pal: AuxPalette) -> Mesh {
    let mut parts: Vec<Part> = vec![(
        Ring(0.025, 0.2, 360.0),
        pal.iron_rail,
        Transform::from_rotation(Quat::from_rotation_x(std::f32::consts::FRAC_PI_2)),
    )];
    for i in 0..4 {
        parts.push((
            Box(0.4, 0.025, 0.025),
            pal.iron_rail,
            Transform::from_rotation(Quat::from_rotation_z(
                i as f32 * std::f32::consts::FRAC_PI_4,
            )),
        ));
    }
    parts.push((
        Cyl(0.04, 0.08),
        pal.trim_gold,
        Transform::from_rotation(Quat::from_rotation_x(1.57)),
    ));
    painted(parts)
}

/// Team banner on a pole; it sways in the wind from its base.
pub(super) fn banner(pal: AuxPalette, blue: bool) -> Mesh {
    painted(vec![
        (
            Cyl(0.025, 1.0),
            pal.timber_post,
            Transform::from_xyz(0.0, 0.5, 0.0),
        ),
        (
            Ball(0.04),
            pal.trim_gold,
            Transform::from_xyz(0.0, 1.02, 0.0),
        ),
        (
            Box(0.02, 0.34, 0.24),
            team_trim(blue),
            Transform::from_xyz(0.0, 0.78, 0.13),
        ),
        (
            Box(0.025, 0.04, 0.25),
            pal.trim_gold,
            Transform::from_xyz(0.0, 0.6, 0.13),
        ),
    ])
}

/// Tier 3 crystal crown on top of the outcrop; it pulses while the mine works.
pub(super) fn crystal_crown(pal: AuxPalette) -> Mesh {
    let mut parts: Vec<Part> = Vec::with_capacity(6);
    for (dx, dz, h, tilt) in [
        (0.0, 0.0, 0.34, 0.0),
        (0.1, 0.06, 0.24, -0.4),
        (-0.09, 0.05, 0.22, 0.45),
        (0.02, -0.1, 0.2, 0.3),
        (-0.05, -0.06, 0.16, -0.5),
    ] {
        parts.push((
            Prism(0.05, h),
            pal.ore_crystal,
            Transform::from_xyz(dx, h * 0.5, dz)
                .with_rotation(Quat::from_rotation_z(tilt) * Quat::from_rotation_x(tilt * 0.5)),
        ));
    }
    painted(parts)
}

/// Tier 2 Merchant goods: stacked crates, a barrel and grain sacks.
pub(super) fn goods_stack(pal: AuxPalette) -> Mesh {
    painted(vec![
        (
            Box(0.2, 0.18, 0.2),
            pal.crate_wood,
            Transform::from_xyz(0.0, 0.09, 0.0),
        ),
        (
            Box(0.16, 0.15, 0.16),
            pal.crate_wood,
            Transform::from_xyz(0.02, 0.26, 0.01).with_rotation(Quat::from_rotation_y(0.4)),
        ),
        (
            Box(0.21, 0.025, 0.21),
            pal.pier_piling,
            Transform::from_xyz(0.0, 0.18, 0.0),
        ),
        (
            Cyl(0.08, 0.2),
            pal.timber_beam,
            Transform::from_xyz(0.2, 0.1, -0.1),
        ),
        (
            Ring(0.01, 0.08, 360.0),
            pal.iron_rail,
            Transform::from_xyz(0.2, 0.15, -0.1),
        ),
        (
            Ball(0.08),
            pal.sack_cloth,
            Transform::from_xyz(-0.17, 0.06, 0.1).with_scale(Vec3::new(1.0, 0.75, 1.0)),
        ),
        (
            Ball(0.07),
            pal.sack_cloth,
            Transform::from_xyz(-0.15, 0.14, -0.05).with_scale(Vec3::new(1.0, 0.75, 1.0)),
        ),
    ])
}

/// Small team pennant for pier posts and the pavilion peak.
pub(super) fn pennant(pal: AuxPalette, blue: bool) -> Mesh {
    painted(vec![
        (
            Cyl(0.018, 0.6),
            pal.timber_post,
            Transform::from_xyz(0.0, 0.3, 0.0),
        ),
        (
            Prism(0.07, 0.02),
            team_trim(blue),
            Transform::from_xyz(0.0, 0.52, 0.07)
                .with_rotation(Quat::from_rotation_x(std::f32::consts::FRAC_PI_2)),
        ),
    ])
}

/// Tier 3 Merchant pavilion: striped market tent with a gold-trimmed peak.
pub(super) fn pavilion(pal: AuxPalette, blue: bool) -> Mesh {
    let mut parts: Vec<Part> = Vec::with_capacity(14);
    for (dx, dz) in [(-0.26, -0.26), (0.26, -0.26), (-0.26, 0.26), (0.26, 0.26)] {
        parts.push((
            Cyl(0.03, 0.55),
            pal.timber_post,
            Transform::from_xyz(dx, 0.27, dz),
        ));
    }
    parts.push((
        Cone(0.48, 0.38),
        pal.awning_cream,
        Transform::from_xyz(0.0, 0.72, 0.0),
    ));
    for i in 0..4 {
        parts.push((
            Box(0.04, 0.36, 0.5),
            if i % 2 == 0 {
                pal.awning_red
            } else {
                team_trim(blue)
            },
            Transform::from_xyz(0.0, 0.72, 0.0)
                .with_rotation(Quat::from_rotation_y(
                    i as f32 * std::f32::consts::FRAC_PI_4,
                ))
                .with_scale(Vec3::new(1.0, 1.0, 0.02)),
        ));
    }
    parts.push((
        Ring(0.02, 0.47, 360.0),
        pal.trim_gold,
        Transform::from_xyz(0.0, 0.54, 0.0),
    ));
    parts.push((
        Ball(0.05),
        pal.trim_gold,
        Transform::from_xyz(0.0, 0.94, 0.0),
    ));
    parts.push((
        Box(0.5, 0.16, 0.24),
        pal.crate_wood,
        Transform::from_xyz(0.0, 0.08, -0.08),
    ));
    parts.push((
        Box(0.52, 0.03, 0.26),
        pal.trim_gold,
        Transform::from_xyz(0.0, 0.17, -0.08),
    ));
    painted(parts)
}

pub(super) fn chest(pal: AuxPalette) -> Mesh {
    painted(vec![
        (
            Box(0.24, 0.13, 0.16),
            pal.crate_wood,
            Transform::from_xyz(0.0, 0.065, 0.0),
        ),
        (
            Box(0.25, 0.03, 0.17),
            pal.trim_gold,
            Transform::from_xyz(0.0, 0.11, 0.0),
        ),
        (
            Box(0.04, 0.06, 0.02),
            pal.trim_gold,
            Transform::from_xyz(0.0, 0.1, 0.085),
        ),
        (
            Ball(0.05),
            pal.ore_crystal,
            Transform::from_xyz(0.03, 0.13, 0.0),
        ),
    ])
}

/// Chest lid hinged at its back edge (local origin), opening about local X.
pub(super) fn chest_lid(pal: AuxPalette) -> Mesh {
    painted(vec![
        (
            Box(0.25, 0.06, 0.17),
            pal.crate_wood,
            Transform::from_xyz(0.0, 0.03, 0.085),
        ),
        (
            Box(0.26, 0.02, 0.04),
            pal.trim_gold,
            Transform::from_xyz(0.0, 0.06, 0.085),
        ),
    ])
}

/// Coins that fly out of the chest on every Merchant payout.
pub(super) fn coins(pal: AuxPalette) -> Mesh {
    let mut parts: Vec<Part> = Vec::with_capacity(6);
    for i in 0..6 {
        let a = i as f32 * std::f32::consts::TAU / 6.0;
        parts.push((
            Cyl(0.035, 0.012),
            pal.trim_gold,
            Transform::from_xyz(a.cos() * 0.09, (i % 3) as f32 * 0.05, a.sin() * 0.09)
                .with_rotation(Quat::from_rotation_x(a)),
        ));
    }
    painted(parts)
}

/// Golden ring that sweeps outward when a tier Evolution is bought.
pub(super) fn tier_ring(pal: AuxPalette) -> Mesh {
    painted(vec![(
        Ring(0.03, 0.7, 360.0),
        pal.trim_gold,
        Transform::from_xyz(0.0, 0.05, 0.0),
    )])
}
