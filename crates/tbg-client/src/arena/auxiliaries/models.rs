//! Low-poly 3D mesh builders for the `Mineiro` and `Mercador` War Auxiliaries,
//! their off-lane precincts, piers, minecarts, and Merchant Caravel ships (P74).

use super::palette::AuxPalette;
use crate::arena::{painted, Part};
use crate::art::mesh::Shape::*;
use bevy::prelude::*;

const SKIN: Color = Color::srgb(0.92, 0.72, 0.56);
const BEARD: Color = Color::srgb(0.36, 0.22, 0.14);
const BOOT_DARK: Color = Color::srgb(0.24, 0.16, 0.11);
const BELT_LEATHER: Color = Color::srgb(0.42, 0.27, 0.15);
const HARDHAT_YELLOW: Color = Color::srgb(0.98, 0.78, 0.14);
const PICKAXE_STEEL: Color = Color::srgb(0.76, 0.80, 0.86);

pub(super) fn team_trim(blue: bool) -> Color {
    if blue {
        Color::srgb(0.22, 0.52, 0.88)
    } else {
        Color::srgb(0.86, 0.26, 0.24)
    }
}

/// Low-poly `Mineiro` figure (hard-hat, bearded head, worker tunic, tool belt, boots, left stabilizer arm).
pub(super) fn miner_body(blue: bool) -> Mesh {
    let mut parts: Vec<Part> = Vec::with_capacity(18);
    let vest = Color::srgb(0.56, 0.38, 0.22);
    let shirt = Color::srgb(0.84, 0.72, 0.52);
    let trousers = Color::srgb(0.38, 0.42, 0.48);
    let badge = team_trim(blue);

    // Boots and legs
    for z in [-0.07, 0.07] {
        parts.push((
            Box(0.11, 0.08, 0.09),
            BOOT_DARK,
            Transform::from_xyz(0.01, 0.04, z),
        ));
        parts.push((
            Cyl(0.048, 0.18),
            trousers,
            Transform::from_xyz(0.0, 0.16, z),
        ));
    }
    // Torso, vest, and tool belt
    parts.push((
        Box(0.20, 0.26, 0.24),
        shirt,
        Transform::from_xyz(0.0, 0.36, 0.0),
    ));
    parts.push((
        Box(0.21, 0.24, 0.25),
        vest,
        Transform::from_xyz(-0.01, 0.36, 0.0),
    ));
    parts.push((
        Box(0.22, 0.05, 0.26),
        BELT_LEATHER,
        Transform::from_xyz(0.0, 0.24, 0.0),
    ));
    parts.push((
        Box(0.04, 0.06, 0.07),
        HARDHAT_YELLOW,
        Transform::from_xyz(0.10, 0.24, 0.0),
    ));
    // Team shoulder badge
    parts.push((
        Box(0.08, 0.05, 0.27),
        badge,
        Transform::from_xyz(0.0, 0.46, 0.0),
    ));
    // Left arm
    parts.push((
        Pill(0.042, 0.14),
        shirt,
        Transform::from_xyz(0.03, 0.36, -0.16).with_rotation(Quat::from_rotation_z(-0.35)),
    ));
    parts.push((Ball(0.045), SKIN, Transform::from_xyz(0.08, 0.28, -0.17)));
    // Head and beard
    parts.push((Ball(0.115), SKIN, Transform::from_xyz(0.0, 0.58, 0.0)));
    parts.push((
        Box(0.11, 0.11, 0.18),
        BEARD,
        Transform::from_xyz(0.05, 0.53, 0.0),
    ));
    // Yellow safety hard-hat with brim, ridge, and front lamp
    parts.push((
        Cyl(0.155, 0.03),
        HARDHAT_YELLOW,
        Transform::from_xyz(0.01, 0.65, 0.0),
    ));
    parts.push((
        Frustum(0.095, 0.135, 0.11),
        HARDHAT_YELLOW,
        Transform::from_xyz(0.0, 0.71, 0.0),
    ));
    parts.push((
        Box(0.18, 0.035, 0.045),
        HARDHAT_YELLOW,
        Transform::from_xyz(0.0, 0.76, 0.0),
    ));
    parts.push((
        Cyl(0.038, 0.03),
        Color::srgb(0.98, 0.96, 0.82),
        Transform::from_xyz(0.125, 0.70, 0.0).with_rotation(Quat::from_rotation_z(1.57)),
    ));
    painted(parts)
}

/// Right arm + double-pointed steel pickaxe pivoted at the Miner's right shoulder `(0.0, 0.44, 0.16)`.
pub(super) fn miner_pickaxe() -> Mesh {
    let mut parts: Vec<Part> = Vec::with_capacity(8);
    let handle = Color::srgb(0.58, 0.38, 0.20);
    // Upper arm + hand extending forward/up along +X/+Y
    parts.push((
        Pill(0.042, 0.15),
        SKIN,
        Transform::from_xyz(0.09, -0.02, 0.0).with_rotation(Quat::from_rotation_z(1.35)),
    ));
    // Pickaxe wooden shaft
    parts.push((
        Cyl(0.024, 0.46),
        handle,
        Transform::from_xyz(0.19, 0.12, 0.0).with_rotation(Quat::from_rotation_z(-0.25)),
    ));
    // Pickaxe steel collar + two curved/angled blades
    parts.push((
        Box(0.07, 0.06, 0.06),
        PICKAXE_STEEL,
        Transform::from_xyz(0.24, 0.31, 0.0),
    ));
    parts.push((
        Cone(0.042, 0.22),
        PICKAXE_STEEL,
        Transform::from_xyz(0.34, 0.29, 0.0).with_rotation(Quat::from_rotation_z(-1.35)),
    ));
    parts.push((
        Cone(0.038, 0.18),
        PICKAXE_STEEL,
        Transform::from_xyz(0.14, 0.33, 0.0).with_rotation(Quat::from_rotation_z(1.42)),
    ));
    painted(parts)
}

/// Faceted rocky hillside outcrop and dark tunnel portal arch.
pub(super) fn mine_outcrop(pal: AuxPalette) -> Mesh {
    let mut parts: Vec<Part> = Vec::with_capacity(14);
    // Main rocky crag clusters (strictly compact within local [-0.65, 0.55])
    parts.push((
        Rock(0.58, 17),
        pal.rock_outer,
        Transform::from_xyz(-0.18, 0.42, 0.0).with_scale(Vec3::new(0.95, 1.12, 1.15)),
    ));
    parts.push((
        Rock(0.42, 31),
        pal.rock_inner,
        Transform::from_xyz(-0.08, 0.34, -0.34),
    ));
    parts.push((
        Rock(0.40, 47),
        pal.rock_inner,
        Transform::from_xyz(-0.06, 0.32, 0.34),
    ));
    parts.push((
        Rock(0.32, 59),
        pal.rock_outer,
        Transform::from_xyz(-0.26, 0.76, 0.05),
    ));
    // Recessed dark tunnel portal mouth facing +X
    parts.push((
        Box(0.16, 0.64, 0.52),
        pal.tunnel_void,
        Transform::from_xyz(0.14, 0.32, 0.0),
    ));
    // Exposed glowing crystal vein inside the rock face
    for (dy, dz, rot) in [(0.24, -0.14, 0.35), (0.38, 0.12, -0.40), (0.18, 0.05, 0.15)] {
        parts.push((
            Cone(0.055, 0.16),
            pal.ore_crystal,
            Transform::from_xyz(0.21, dy, dz).with_rotation(Quat::from_rotation_z(-1.1 + rot)),
        ));
    }
    painted(parts)
}

/// Timber portal shoring: vertical posts, overhead lintel beam, and 45-degree corner braces.
pub(super) fn mine_shoring(pal: AuxPalette) -> Mesh {
    let mut parts: Vec<Part> = Vec::with_capacity(8);
    let x = 0.22;
    for z in [-0.28, 0.28] {
        parts.push((
            Box(0.09, 0.72, 0.09),
            pal.timber_post,
            Transform::from_xyz(x, 0.36, z),
        ));
        parts.push((
            Box(0.07, 0.24, 0.07),
            pal.timber_beam,
            Transform::from_xyz(x, 0.58, z * 0.72)
                .with_rotation(Quat::from_rotation_x(if z > 0.0 { 0.78 } else { -0.78 })),
        ));
    }
    // Top lintel crossbeam
    parts.push((
        Box(0.11, 0.10, 0.72),
        pal.timber_beam,
        Transform::from_xyz(x, 0.72, 0.0),
    ));
    painted(parts)
}

/// Narrow-gauge iron rails on wooden sleepers extending from the portal mouth (`x = 0.15 .. 0.92`).
pub(super) fn mine_rails(pal: AuxPalette) -> Mesh {
    let mut parts: Vec<Part> = Vec::with_capacity(10);
    for i in 0..5 {
        let x = 0.22 + i as f32 * 0.16;
        parts.push((
            Box(0.07, 0.03, 0.42),
            pal.timber_post,
            Transform::from_xyz(x, 0.015, 0.0),
        ));
    }
    for z in [-0.14, 0.14] {
        parts.push((
            Box(0.76, 0.035, 0.035),
            pal.iron_rail,
            Transform::from_xyz(0.54, 0.038, z),
        ));
    }
    painted(parts)
}

/// Timber-and-iron ore minecart on four flanged wheels.
pub(super) fn mine_cart(pal: AuxPalette) -> Mesh {
    let mut parts: Vec<Part> = Vec::with_capacity(10);
    // Flared hopper box
    parts.push((
        Box(0.36, 0.20, 0.28),
        pal.timber_beam,
        Transform::from_xyz(0.0, 0.17, 0.0),
    ));
    parts.push((
        Box(0.38, 0.035, 0.30),
        pal.iron_rail,
        Transform::from_xyz(0.0, 0.26, 0.0),
    ));
    parts.push((
        Box(0.38, 0.035, 0.30),
        pal.iron_rail,
        Transform::from_xyz(0.0, 0.10, 0.0),
    ));
    // 4 iron wheels
    for dx in [-0.12, 0.12] {
        for dz in [-0.15, 0.15] {
            parts.push((
                Cyl(0.055, 0.03),
                pal.iron_rail,
                Transform::from_xyz(dx, 0.055, dz).with_rotation(Quat::from_rotation_x(1.57)),
            ));
        }
    }
    painted(parts)
}

/// Glowing crystal/gold ore pile inside the minecart (scales up as ore accumulates).
pub(super) fn mine_ore(pal: AuxPalette) -> Mesh {
    let mut parts: Vec<Part> = Vec::with_capacity(6);
    for (dx, dy, dz, r) in [
        (-0.07, 0.28, -0.04, 0.075),
        (0.06, 0.29, 0.03, 0.080),
        (0.0, 0.32, -0.02, 0.070),
        (-0.03, 0.27, 0.06, 0.065),
    ] {
        parts.push((
            Rock(r, 11),
            pal.ore_crystal,
            Transform::from_xyz(dx, dy, dz),
        ));
    }
    painted(parts)
}

/// Swinging warm gold lantern suspended from the mine portal lintel.
pub(super) fn mine_lantern(pal: AuxPalette) -> Mesh {
    let parts: Vec<Part> = vec![
        (
            Cyl(0.012, 0.10),
            pal.iron_rail,
            Transform::from_xyz(0.0, -0.05, 0.0),
        ),
        (
            Cyl(0.052, 0.025),
            pal.iron_rail,
            Transform::from_xyz(0.0, -0.10, 0.0),
        ),
        (
            Cyl(0.042, 0.08),
            pal.lantern_glow,
            Transform::from_xyz(0.0, -0.14, 0.0),
        ),
        (
            Cyl(0.048, 0.02),
            pal.iron_rail,
            Transform::from_xyz(0.0, -0.18, 0.0),
        ),
    ];
    painted(parts)
}

/// Burst of crystal/gold sparks on pickaxe strike.
pub(super) fn mine_sparks(pal: AuxPalette) -> Mesh {
    let mut parts: Vec<Part> = Vec::with_capacity(4);
    for (dx, dy, dz) in [(0.0, 0.05, -0.06), (0.04, 0.09, 0.05), (-0.03, 0.03, 0.04)] {
        parts.push((
            Cone(0.032, 0.09),
            pal.ore_crystal,
            Transform::from_xyz(dx, dy, dz),
        ));
    }
    painted(parts)
}

/// Low-poly `Mercador` figure (crimson wide-brimmed merchant hat with gold band, coat, beard).
pub(super) fn merchant_body(blue: bool) -> Mesh {
    let mut parts: Vec<Part> = Vec::with_capacity(18);
    let coat = Color::srgb(0.76, 0.22, 0.18);
    let sleeve = Color::srgb(0.88, 0.60, 0.40);
    let trim = Color::srgb(0.94, 0.76, 0.28);
    let badge = team_trim(blue);

    for z in [-0.07, 0.07] {
        parts.push((
            Box(0.11, 0.08, 0.09),
            BOOT_DARK,
            Transform::from_xyz(0.01, 0.04, z),
        ));
        parts.push((
            Cyl(0.048, 0.18),
            Color::srgb(0.28, 0.22, 0.20),
            Transform::from_xyz(0.0, 0.16, z),
        ));
    }
    // Merchant coat, gold lapel trim, and belt with gold buckle
    parts.push((
        Box(0.21, 0.28, 0.25),
        coat,
        Transform::from_xyz(0.0, 0.36, 0.0),
    ));
    parts.push((
        Box(0.22, 0.26, 0.06),
        trim,
        Transform::from_xyz(0.01, 0.36, 0.0),
    ));
    parts.push((
        Box(0.23, 0.05, 0.26),
        BELT_LEATHER,
        Transform::from_xyz(0.0, 0.25, 0.0),
    ));
    parts.push((
        Box(0.04, 0.06, 0.07),
        trim,
        Transform::from_xyz(0.10, 0.25, 0.0),
    ));
    parts.push((
        Box(0.08, 0.05, 0.27),
        badge,
        Transform::from_xyz(0.0, 0.47, 0.0),
    ));
    // Left arm resting on hip
    parts.push((
        Pill(0.042, 0.14),
        sleeve,
        Transform::from_xyz(0.02, 0.36, -0.16).with_rotation(Quat::from_rotation_x(0.45)),
    ));
    parts.push((Ball(0.045), SKIN, Transform::from_xyz(0.04, 0.27, -0.14)));
    // Head and beard
    parts.push((Ball(0.115), SKIN, Transform::from_xyz(0.0, 0.58, 0.0)));
    parts.push((
        Box(0.11, 0.12, 0.18),
        BEARD,
        Transform::from_xyz(0.05, 0.53, 0.0),
    ));
    // Wide-brimmed crimson merchant hat with gold ribbon
    parts.push((Cyl(0.21, 0.025), coat, Transform::from_xyz(0.0, 0.66, 0.0)));
    parts.push((Cyl(0.135, 0.03), trim, Transform::from_xyz(0.0, 0.68, 0.0)));
    parts.push((
        Frustum(0.095, 0.13, 0.11),
        coat,
        Transform::from_xyz(0.0, 0.73, 0.0),
    ));
    painted(parts)
}

/// Right arm + extendable brass spyglass pivoted at the Merchant's shoulder `(0.0, 0.44, 0.16)`.
pub(super) fn merchant_spyglass_arm(_blue: bool) -> Mesh {
    let mut parts: Vec<Part> = Vec::with_capacity(6);
    let sleeve = Color::srgb(0.88, 0.60, 0.40);
    let brass = Color::srgb(0.96, 0.78, 0.22);
    parts.push((
        Pill(0.042, 0.14),
        sleeve,
        Transform::from_xyz(0.08, 0.04, 0.0).with_rotation(Quat::from_rotation_z(1.25)),
    ));
    parts.push((Ball(0.044), SKIN, Transform::from_xyz(0.16, 0.09, -0.04)));
    // Brass spyglass telescope barrels
    parts.push((
        Frustum(0.022, 0.038, 0.24),
        brass,
        Transform::from_xyz(0.20, 0.13, -0.10).with_rotation(Quat::from_rotation_z(-1.45)),
    ));
    parts.push((
        Cyl(0.042, 0.03),
        Color::srgb(0.98, 0.90, 0.52),
        Transform::from_xyz(0.31, 0.14, -0.10).with_rotation(Quat::from_rotation_z(-1.45)),
    ));
    painted(parts)
}

/// Timber pier pilings + water-surface foam rings extending into the sea.
pub(super) fn merchant_pier_pilings(pal: AuxPalette) -> Mesh {
    let mut parts: Vec<Part> = Vec::with_capacity(14);
    let foam = Color::srgb(0.86, 0.96, 0.95);
    for dz in [0.45, 0.95, 1.45] {
        for dx in [-0.28, 0.28] {
            parts.push((
                Cyl(0.065, 1.45),
                pal.pier_piling,
                Transform::from_xyz(dx, -0.92, dz),
            ));
            parts.push((Cyl(0.11, 0.025), foam, Transform::from_xyz(dx, -1.38, dz)));
        }
    }
    painted(parts)
}

/// Planked timber pier deck + mooring bollards (`dz = 0.35 .. 1.55`).
pub(super) fn merchant_pier_deck(pal: AuxPalette) -> Mesh {
    let mut parts: Vec<Part> = Vec::with_capacity(14);
    // Longitudinal stringers
    for dx in [-0.26, 0.26] {
        parts.push((
            Box(0.07, 0.07, 1.25),
            pal.pier_piling,
            Transform::from_xyz(dx, -0.22, 0.95),
        ));
    }
    // Cross planks
    for i in 0..8 {
        let z = 0.40 + i as f32 * 0.155;
        parts.push((
            Box(0.68, 0.045, 0.12),
            pal.pier_wood,
            Transform::from_xyz(0.0, -0.17, z),
        ));
    }
    // Mooring bollards at the seaward end
    for dx in [-0.28, 0.28] {
        parts.push((
            Cyl(0.055, 0.18),
            pal.pier_piling,
            Transform::from_xyz(dx, -0.07, 1.48),
        ));
    }
    painted(parts)
}

/// Market stall frame: stone/timber base, 4 corner posts, and merchant counter table.
pub(super) fn merchant_stall_frame(pal: AuxPalette) -> Mesh {
    let mut parts: Vec<Part> = Vec::with_capacity(10);
    // Timber base platform
    parts.push((
        Box(0.96, 0.08, 0.76),
        pal.pier_wood,
        Transform::from_xyz(0.0, 0.04, 0.0),
    ));
    // 4 corner posts
    for dx in [-0.42, 0.42] {
        for dz in [-0.32, 0.32] {
            parts.push((
                Box(0.07, 0.78, 0.07),
                pal.timber_post,
                Transform::from_xyz(dx, 0.43, dz),
            ));
        }
    }
    // Front merchant counter table
    parts.push((
        Box(0.84, 0.32, 0.24),
        pal.crate_wood,
        Transform::from_xyz(0.0, 0.22, 0.20),
    ));
    parts.push((
        Box(0.90, 0.045, 0.30),
        pal.pier_wood,
        Transform::from_xyz(0.0, 0.39, 0.20),
    ));
    painted(parts)
}

/// Striped red-and-cream market awning + crimson/gold balance-scales trade banner.
pub(super) fn merchant_stall_awning(pal: AuxPalette) -> Mesh {
    let mut parts: Vec<Part> = Vec::with_capacity(16);
    // 5 alternating red and cream striped roof panels
    for i in 0..5 {
        let dx = -0.36 + i as f32 * 0.18;
        let color = if i % 2 == 0 {
            pal.awning_red
        } else {
            pal.awning_cream
        };
        parts.push((
            Box(0.185, 0.05, 0.88),
            color,
            Transform::from_xyz(dx, 0.84, 0.02).with_rotation(Quat::from_rotation_x(0.18)),
        ));
        // Front valance tab
        parts.push((
            Box(0.175, 0.09, 0.04),
            color,
            Transform::from_xyz(dx, 0.74, 0.44),
        ));
    }
    // Trade banner with gold balance scales emblem on the side post
    parts.push((
        Box(0.04, 0.42, 0.24),
        pal.awning_red,
        Transform::from_xyz(-0.46, 0.56, 0.0),
    ));
    parts.push((
        Box(0.05, 0.03, 0.16),
        pal.trim_gold,
        Transform::from_xyz(-0.47, 0.62, 0.0),
    ));
    parts.push((
        Cyl(0.035, 0.02),
        pal.trim_gold,
        Transform::from_xyz(-0.47, 0.54, -0.06).with_rotation(Quat::from_rotation_z(1.57)),
    ));
    parts.push((
        Cyl(0.035, 0.02),
        pal.trim_gold,
        Transform::from_xyz(-0.47, 0.54, 0.06).with_rotation(Quat::from_rotation_z(1.57)),
    ));
    painted(parts)
}

/// Stacked wooden crates, grain sacks, and brass scales at the Merchant stall.
pub(super) fn merchant_stall_cargo(pal: AuxPalette) -> Mesh {
    let mut parts: Vec<Part> = Vec::with_capacity(8);
    parts.push((
        Box(0.24, 0.22, 0.24),
        pal.crate_wood,
        Transform::from_xyz(-0.28, 0.19, -0.14),
    ));
    parts.push((
        Box(0.18, 0.18, 0.18),
        pal.crate_wood,
        Transform::from_xyz(-0.26, 0.39, -0.14).with_rotation(Quat::from_rotation_y(0.25)),
    ));
    parts.push((
        Pill(0.09, 0.10),
        pal.sack_cloth,
        Transform::from_xyz(0.28, 0.18, -0.12),
    ));
    parts.push((
        Pill(0.08, 0.08),
        pal.sack_cloth,
        Transform::from_xyz(0.14, 0.16, -0.16),
    ));
    // Brass scale on the counter
    parts.push((
        Cyl(0.02, 0.12),
        pal.trim_gold,
        Transform::from_xyz(0.16, 0.47, 0.20),
    ));
    parts.push((
        Box(0.14, 0.02, 0.02),
        pal.trim_gold,
        Transform::from_xyz(0.16, 0.53, 0.20),
    ));
    painted(parts)
}

/// Low-poly wooden Merchant Caravel ship (hull, stern castle, bowsprit, mast, square sail, cargo).
pub(super) fn merchant_ship(pal: AuxPalette, blue: bool) -> Mesh {
    let mut parts: Vec<Part> = Vec::with_capacity(16);
    let badge = team_trim(blue);
    // Main hull body and tapered bow/stern
    parts.push((
        Box(0.92, 0.26, 0.44),
        pal.hull_wood,
        Transform::from_xyz(0.0, 0.14, 0.0),
    ));
    parts.push((
        Cone(0.22, 0.38),
        pal.hull_wood,
        Transform::from_xyz(0.58, 0.16, 0.0).with_rotation(Quat::from_rotation_z(-1.57)),
    ));
    // Raised stern castle with gold trim
    parts.push((
        Box(0.32, 0.24, 0.42),
        pal.pier_wood,
        Transform::from_xyz(-0.34, 0.32, 0.0),
    ));
    parts.push((
        Box(0.34, 0.04, 0.44),
        pal.trim_gold,
        Transform::from_xyz(-0.34, 0.44, 0.0),
    ));
    // Bowsprit spar
    parts.push((
        Cyl(0.025, 0.36),
        pal.timber_post,
        Transform::from_xyz(0.74, 0.28, 0.0).with_rotation(Quat::from_rotation_z(-1.22)),
    ));
    // Central mast and yardarm
    parts.push((
        Cyl(0.038, 1.05),
        pal.timber_post,
        Transform::from_xyz(0.06, 0.68, 0.0),
    ));
    parts.push((
        Cyl(0.025, 0.68),
        pal.timber_post,
        Transform::from_xyz(0.06, 0.86, 0.0).with_rotation(Quat::from_rotation_x(1.57)),
    ));
    // Billowing square main sail + crimson/team stripe
    parts.push((
        Box(0.08, 0.56, 0.64),
        pal.sail_cloth,
        Transform::from_xyz(0.11, 0.72, 0.0),
    ));
    parts.push((
        Box(0.09, 0.12, 0.65),
        pal.awning_red,
        Transform::from_xyz(0.115, 0.72, 0.0),
    ));
    // Masthead pennant in team color
    parts.push((
        Box(0.22, 0.07, 0.02),
        badge,
        Transform::from_xyz(-0.04, 1.18, 0.0),
    ));
    // Deck cargo crates
    parts.push((
        Box(0.18, 0.16, 0.18),
        pal.crate_wood,
        Transform::from_xyz(0.24, 0.32, -0.06),
    ));
    painted(parts)
}

/// Delivered cargo crate parcel that transfers from the docked Caravel onto the pier.
pub(super) fn merchant_crate_parcel(pal: AuxPalette) -> Mesh {
    let mut parts: Vec<Part> = Vec::with_capacity(3);
    parts.push((
        Box(0.22, 0.20, 0.22),
        pal.crate_wood,
        Transform::from_xyz(0.0, 0.10, 0.0),
    ));
    parts.push((
        Box(0.24, 0.04, 0.24),
        pal.trim_gold,
        Transform::from_xyz(0.0, 0.14, 0.0),
    ));
    painted(parts)
}
