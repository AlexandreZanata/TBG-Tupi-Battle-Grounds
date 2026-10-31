//! Native rig-v2 Bloomkeeper: Support/Healer archetype, leaf greens, bark, and blossom pink.
//! - Base: petal head wreath, draped leaf cloak on back, flowering wooden staff.
//! - Branch A (Grove Bloom): floating blossom halo ring, radiant lotus staff head.
//! - Branch B (Thornwarden): heavy bark cuirass with briar thorn spikes, bramble offhand shield.

use crate::art::{
    figure::{part, FigureSpec},
    mesh::Shape::*,
    palette::*,
    rig::{BodyPlan, Joint::*, Node::Body, Node::Secondary, Proportions, Spring},
};
use bevy::prelude::Vec3;

pub fn plan(tier: u8) -> Proportions {
    let mut plan = BodyPlan::Humanoid.proportions();
    plan.torso = Vec3::new(0.34, 0.36, 0.24);
    plan.leg = 0.36;
    plan.arm = 0.18;
    plan.head = 0.14;

    let evo_scale = match tier {
        1 => 1.08,
        2 => 1.16,
        _ => 1.00,
    };
    // Match-camera lane readability factor (1.15x) * tier evolution
    plan.scale = 1.15 * evo_scale;
    plan
}

pub fn spec(tier: u8, branch: u8) -> FigureSpec {
    let plan = plan(tier);
    let mut parts = super::super::body::parts(&plan, LEAF, LEAF_DARK, BARK);

    // Blossom petal halo/wreath around head
    let petal_r = if tier >= 1 { 0.08 } else { 0.065 };
    for i in 0..6 {
        let a = i as f32 / 6.0 * std::f32::consts::TAU;
        parts.push(part(
            Body(Head),
            Ball(petal_r),
            BLOSSOM,
            [a.cos() * 0.17, 0.10 + a.sin() * 0.17, -0.02],
        ));
    }
    // Golden center pistil on forehead
    parts.push(part(Body(Head), Ball(0.065), GOLD, [0.0, 0.10, 0.06]));

    // Leaf cloak draped on back spring
    parts.push(
        part(
            Secondary(Spring::Back),
            Box(0.38, 0.44, 0.02),
            LEAF,
            [0.0, 0.02, -0.12],
        )
        .rot([10.0, 0.0, 0.0]),
    );

    // Flowering wooden staff on HandR
    let staff_len = if tier >= 2 { 1.05 } else { 0.88 };
    parts.push(
        part(
            Body(HandR),
            Cyl(0.018, staff_len),
            BARK,
            [0.0, staff_len * 0.20, 0.06],
        )
        .rot([90.0, 0.0, 0.0]),
    );
    let staff_top = staff_len * 0.70;

    // Blossom bulb at top of staff
    parts.push(part(
        Body(HandR),
        Ball(0.08),
        BLOSSOM,
        [0.0, staff_top, 0.06],
    ));

    if branch == 0 {
        // Branch A (Grove Bloom): Halo ring and radiant petals
        if tier >= 1 {
            // Golden pollen aura surrounding staff head
            for s in [-0.07f32, 0.07] {
                parts.push(part(
                    Body(HandR),
                    Ball(0.035),
                    GOLD,
                    [s, staff_top + 0.06, 0.06],
                ));
            }
        }
        if tier >= 2 {
            // Radiant floral halo ring behind head
            parts.push(
                part(
                    Secondary(Spring::HeadGear),
                    Ring(0.025, 0.28, 360.0),
                    BLOSSOM,
                    [0.0, 0.10, -0.06],
                )
                .rot([90.0, 0.0, 0.0]),
            );
        }
    } else {
        // Branch B (Thornwarden): Bark cuirass, briar thorns, and bramble shield
        for y in [0.08f32, 0.24] {
            parts.push(part(
                Body(Chest),
                Box(0.42, 0.10, 0.28),
                BARK,
                [0.0, y, 0.0],
            ));
        }

        // Briar thorn spikes protruding from shoulders
        for (x, y) in [
            (0.16f32, 0.28f32),
            (-0.16, 0.28),
            (0.0, 0.36),
            (0.18, 0.12),
            (-0.18, 0.12),
        ] {
            parts.push(
                part(Body(Chest), Cone(0.03, 0.14), BARK, [x, y, 0.16]).rot([90.0, 0.0, 0.0]),
            );
        }

        if tier >= 1 {
            // Bramble shield on HandL
            parts.push(part(
                Body(HandL),
                Box(0.24, 0.36, 0.06),
                BARK,
                [0.0, 0.04, 0.14],
            ));
            parts.push(
                part(
                    Body(HandL),
                    Ring(0.02, 0.16, 360.0),
                    LEAF_DARK,
                    [0.0, 0.04, 0.17],
                )
                .rot([90.0, 0.0, 0.0]),
            );
        }
    }

    FigureSpec { plan, parts }
}
