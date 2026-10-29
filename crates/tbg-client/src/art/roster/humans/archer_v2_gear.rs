//! Candidate Archer gear per form: the bow (oak longbow, longer yew longbow,
//! gilded recurve), the quiver, and the evolution pieces. Every bow keeps its
//! string line `STRING_LINE` behind the grip so `bow.rs` can draw it.

use crate::art::{
    archer_v2::{bow_half, STRING_LINE},
    figure::{part, PartSpec},
    mesh::Shape::*,
    palette::*,
    rig::{
        Joint::*,
        Node::{Body, Prop, Secondary},
        Socket, Spring,
    },
};

/// Grip height above the socket; the string tips sit at `GRIP_Y ± half`.
const GRIP_Y: f32 = 0.08;
/// How far the bow belly bulges in front of the string line at the grip.
const BELLY: f32 = 0.288;

pub fn bow(parts: &mut Vec<PartSpec>, tier: u8) {
    let node = Prop(Socket::OffhandL);
    let half = bow_half(tier);
    let (wood, wrap, tip) = match tier {
        0 => (OAK, DARK_LEATHER, LINEN),
        1 => (TIMBER, TRIM, LINEN),
        _ => (RUST, GOLD, GOLD),
    };
    let z = -STRING_LINE;
    parts.extend([
        part(node, Ring(0.02, half, 180.0), wood, [0.0, GRIP_Y, z])
            .scale([1.0, 1.0, BELLY / half])
            .rot([0.0, 0.0, 90.0]),
        part(
            node,
            Box(0.048, 0.16, 0.062),
            wrap,
            [0.0, GRIP_Y, z + BELLY],
        ),
    ]);
    for s in [1.0, -1.0] {
        parts.push(part(
            node,
            Box(0.028, 0.05, 0.028),
            tip,
            [0.0, GRIP_Y + s * half, z],
        ));
        if tier >= 2 {
            // Recurve: the tips curl away from the archer, gold-capped.
            parts.push(
                part(
                    node,
                    Box(0.024, 0.13, 0.024),
                    wood,
                    [0.0, GRIP_Y + s * (half + 0.05), z + 0.04],
                )
                .rot([s * 38.0, 0.0, 0.0]),
            );
            parts.push(part(
                node,
                Box(0.034, 0.022, 0.034),
                GOLD,
                [0.0, GRIP_Y + s * half * 0.55, z + 0.25],
            ));
        }
    }
}

pub fn quiver(parts: &mut Vec<PartSpec>, tier: u8) {
    let node = Prop(Socket::Back);
    let rim = if tier >= 2 { GOLD } else { OAK };
    parts.extend([
        part(node, Cyl(0.065, 0.34), DARK_LEATHER, [0.04, 0.02, -0.03]).rot([-18.0, 0.0, -18.0]),
        part(node, Ring(0.009, 0.066, 360.0), rim, [0.08, 0.18, -0.07]).rot([-18.0, 0.0, -18.0]),
        // Baldric across the chest.
        part(
            Body(Chest),
            Box(0.045, 0.37, 0.02),
            DARK_LEATHER,
            [0.015, 0.18, 0.13],
        )
        .rot([0.0, 0.0, -32.0]),
    ]);
    let arrows: &[f32] = if tier == 0 {
        &[-0.025, 0.02, 0.065]
    } else {
        &[-0.04, -0.005, 0.03, 0.065, 0.10]
    };
    for &x in arrows {
        parts.push(
            part(node, Box(0.009, 0.29, 0.009), TIMBER, [x, 0.21, -0.07]).rot([-12.0, 0.0, -10.0]),
        );
        parts.push(
            part(
                node,
                Box(0.015, 0.07, 0.055),
                ALLY,
                [x + 0.012, 0.34, -0.10],
            )
            .rot([-12.0, 0.0, -10.0]),
        );
    }
}

/// Longbow (tier 1) and above: a ranger's feather and a bow-side pauldron.
pub fn ranger(parts: &mut Vec<PartSpec>) {
    parts.extend([
        part(
            Body(Head),
            Box(0.014, 0.22, 0.05),
            LINEN,
            [0.135, 0.16, -0.03],
        )
        .rot([-28.0, 0.0, -24.0]),
        part(
            Body(Head),
            Box(0.016, 0.08, 0.055),
            ALLY,
            [0.185, 0.27, -0.085],
        )
        .rot([-28.0, 0.0, -24.0]),
        // Layered leather pauldron (boxes: a sphere costs ~10x the triangles).
        part(
            Body(UpperArmL),
            Box(0.15, 0.055, 0.135),
            DARK_LEATHER,
            [0.0, 0.0, 0.0],
        )
        .rot([0.0, 0.0, -12.0]),
        part(
            Body(UpperArmL),
            Box(0.155, 0.014, 0.14),
            OAK,
            [0.0, -0.035, 0.0],
        )
        .rot([0.0, 0.0, -12.0]),
    ]);
}

/// Marksman (tier 2): team half-cape with a gold hem, steel arm guard, studs.
pub fn marksman(parts: &mut Vec<PartSpec>) {
    let cape = Secondary(Spring::Back);
    parts.extend([
        part(cape, Box(0.27, 0.40, 0.02), ALLY, [-0.04, -0.13, -0.14]).rot([6.0, 0.0, 5.0]),
        part(cape, Box(0.27, 0.03, 0.026), GOLD, [-0.055, -0.33, -0.16]).rot([6.0, 0.0, 5.0]),
        part(
            Body(Chest),
            Box(0.10, 0.03, 0.12),
            GOLD,
            [-0.15, 0.30, -0.02],
        ),
        part(
            Body(ForearmL),
            Box(0.135, 0.10, 0.135),
            STEEL,
            [0.0, -0.09, 0.0],
        ),
        part(
            Body(ForearmL),
            Box(0.14, 0.02, 0.14),
            GOLD,
            [0.0, -0.04, 0.0],
        ),
    ]);
    for y in [0.22, 0.12] {
        parts.push(part(
            Body(Chest),
            Box(0.024, 0.024, 0.012),
            GOLD,
            [0.07, y, 0.125],
        ));
    }
}
