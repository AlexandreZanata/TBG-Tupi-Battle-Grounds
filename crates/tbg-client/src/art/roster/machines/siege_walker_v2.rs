//! Native rig-v2 Siege Walker: Large war engine, gunmetal chassis, brass fittings
//! and orange optics.
//! - Base: Large body plan, geared great-bow on right arm with brass muzzle ring.
//! - Branch A (Double-Limb Ballista): twin-tiered composite bow limbs, brass pulleys.
//! - Branch B (Hammer-Head Battery): heavy multi-rail bolt launcher, brass ammo drum.

use crate::art::{
    figure::{part, FigureSpec},
    mesh::Shape::*,
    palette::*,
    rig::{BodyPlan, Joint::*, Node::Body, Proportions},
};
use bevy::prelude::Vec3;

pub fn plan(tier: u8) -> Proportions {
    let mut plan = BodyPlan::Large.proportions();
    plan.torso = Vec3::new(0.50, 0.40, 0.32);
    plan.leg = 0.38;
    plan.arm = 0.24;
    plan.head = 0.15;

    let evo_scale = match tier {
        1 => 1.08,
        2 => 1.16,
        _ => 1.00,
    };
    // Large body plan (1.50x) + match-camera lane readability (1.20x) * tier evolution
    plan.scale = 1.50 * 1.20 * evo_scale;
    plan
}

pub fn spec(tier: u8, branch: u8) -> FigureSpec {
    let plan = plan(tier);
    let mut parts = super::super::body::parts(&plan, GUNMETAL, IRON, GUNMETAL);

    // Mechanical armor plates on limbs
    for p in &mut parts {
        match p.node {
            Body(ThighL | ThighR | ShinL | ShinR) => {
                p.scale = Vec3::new(1.30, 1.0, 1.30);
                p.color = GUNMETAL;
            }
            Body(FootL | FootR) => {
                p.color = IRON;
            }
            Body(UpperArmL | UpperArmR | ForearmL | ForearmR) => {
                p.scale = Vec3::new(1.25, 1.0, 1.25);
                p.color = IRON;
            }
            _ => {}
        }
    }

    // Angular brass and gunmetal head casing
    parts.push(part(
        Body(Head),
        Box(0.24, 0.16, 0.22),
        BRASS,
        [0.0, 0.02, 0.0],
    ));
    // Horizontal glowing optical sensor slit
    parts.push(part(
        Body(Head),
        Box(0.20, 0.05, 0.04),
        ORANGE,
        [0.0, 0.03, 0.11],
    ));

    // Torso power generator housing on chest
    parts.push(part(
        Body(Chest),
        Box(0.42, 0.24, 0.12),
        GUNMETAL,
        [0.0, 0.18, 0.16],
    ));
    parts.push(part(Body(Chest), Ball(0.07), BRASS, [0.0, 0.18, 0.23]));

    // Geared Great-Bow / Ballista on Right Arm / HandR
    let stock_len = if tier >= 2 { 1.35 } else { 1.10 };

    // Launch rail / stock along aiming axis (HandR)
    parts.push(
        part(
            Body(HandR),
            Box(0.10, 0.10, stock_len),
            IRON,
            [0.0, 0.08, stock_len * 0.40],
        )
        .rot([0.0, 0.0, 0.0]),
    );

    // Brass gear / winch mechanism
    parts.push(part(Body(HandR), Cyl(0.08, 0.06), BRASS, [0.0, 0.15, 0.12]).rot([0.0, 0.0, 90.0]));

    let muzzle_z = stock_len * 0.88;
    let muzzle_r = if tier >= 2 { 0.15 } else { 0.10 };

    // Brass muzzle ring at tip of launch rail
    parts.push(
        part(
            Body(HandR),
            Ring(0.025, muzzle_r, 360.0),
            BRASS,
            [0.0, 0.08, muzzle_z],
        )
        .rot([0.0, 0.0, 0.0]),
    );

    if branch == 0 {
        // Branch A (Double-Limb Ballista)
        let limb_span = if tier >= 2 { 1.35 } else { 1.05 };
        // Primary cross-limb
        parts.push(
            part(
                Body(HandR),
                Box(limb_span, 0.07, 0.06),
                IRON,
                [0.0, 0.08, 0.32],
            )
            .rot([0.0, 0.0, 0.0]),
        );
        // Tension pulleys at tips
        for s in [-1.0f32, 1.0] {
            parts.push(part(
                Body(HandR),
                Cyl(0.045, 0.03),
                BRASS,
                [s * limb_span * 0.48, 0.08, 0.32],
            ));
        }

        if tier >= 1 {
            // Secondary reinforced bow-limb
            parts.push(
                part(
                    Body(HandR),
                    Box(limb_span * 0.75, 0.06, 0.05),
                    BRASS,
                    [0.0, 0.15, 0.28],
                )
                .rot([0.0, 0.0, 0.0]),
            );
        }
        if tier >= 2 {
            // High-tension steel cables along rail
            for s in [-0.08f32, 0.08] {
                parts.push(part(
                    Body(HandR),
                    Cyl(0.012, stock_len * 0.70),
                    STEEL,
                    [s, 0.04, stock_len * 0.42],
                ));
            }
        }
    } else {
        // Branch B (Hammer-Head / Multi-Bolt Battery)
        let barrels: &[f32] = if tier >= 1 {
            &[-0.09, 0.0, 0.09]
        } else {
            &[0.0]
        };

        for &x in barrels {
            parts.push(
                part(
                    Body(HandR),
                    Cyl(0.05, stock_len * 0.85),
                    IRON,
                    [x, 0.14, stock_len * 0.45],
                )
                .rot([90.0, 0.0, 0.0]),
            );
            parts.push(
                part(
                    Body(HandR),
                    Ring(0.018, 0.06, 360.0),
                    BRASS,
                    [x, 0.14, stock_len * 0.85],
                )
                .rot([0.0, 0.0, 0.0]),
            );
        }

        if tier >= 2 {
            // Rotating cylindrical brass ammunition magazine drum on shoulder
            parts.push(
                part(Body(Chest), Cyl(0.14, 0.22), BRASS, [0.26, 0.36, -0.18])
                    .rot([90.0, 0.0, 0.0]),
            );
            parts.push(part(Body(Chest), Ball(0.05), ORANGE, [0.26, 0.36, -0.30]));
        }
    }

    FigureSpec { plan, parts }
}
