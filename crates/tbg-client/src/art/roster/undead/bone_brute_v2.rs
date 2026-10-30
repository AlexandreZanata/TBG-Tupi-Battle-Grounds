//! Native rig-v2 Bone Brute: Large hunched giant, cleaver, ribcage armor,
//! Branch A (Ossified / Bone Wall) with bone plates and skull/rib shield,
//! Branch B (Butcher / Reaper) with twin cleavers or two-handed scythe.

use crate::art::{
    figure::{part, FigureSpec},
    mesh::Shape::*,
    palette::*,
    rig::{BodyPlan, Joint::*, Node::Body, Proportions},
};
use bevy::prelude::Vec3;

pub fn plan(tier: u8) -> Proportions {
    let mut plan = BodyPlan::Large.proportions();
    plan.torso = Vec3::new(0.48, 0.38, 0.28);
    plan.leg = 0.40;
    plan.arm = 0.22;
    plan.head = 0.16;

    let evo_scale = match tier {
        1 => 1.08,
        2 => 1.16,
        _ => 1.00,
    };
    // Large body plan (1.50x) + match-camera lane readability scale (1.20x)
    plan.scale = 1.50 * 1.20 * evo_scale;
    plan
}

pub fn spec(tier: u8, branch: u8) -> FigureSpec {
    let plan = plan(tier);
    let mut parts = super::super::body::parts(&plan, VIOLET, BONE, DARK);

    // Thick hunched skeletal limbs
    for p in &mut parts {
        match p.node {
            Body(ThighL | ThighR | ShinL | ShinR) => {
                p.scale = Vec3::new(1.25, 1.0, 1.25);
                p.color = BONE;
            }
            Body(UpperArmL | UpperArmR | ForearmL | ForearmR) => {
                p.scale = Vec3::new(1.30, 1.0, 1.30);
                p.color = BONE;
            }
            _ => {}
        }
    }

    // Hunched hump on back
    parts.push(part(Body(Chest), Ball(0.18), VIOLET, [0.0, 0.26, -0.12]).scale([1.6, 0.85, 1.1]));

    // Ribcage armor across torso
    for i in 0..4 {
        parts.push(part(
            Body(Chest),
            Box(0.44, 0.04, 0.04),
            BONE,
            [0.0, 0.08 + 0.07 * i as f32, 0.18],
        ));
    }

    // Protruding heavy jaw and glowing green eye sockets
    parts.push(part(
        Body(Head),
        Box(0.18, 0.06, 0.14),
        BONE,
        [0.0, -0.12, 0.06],
    ));
    for s in [-1.0f32, 1.0] {
        parts.push(part(
            Body(Head),
            Box(0.04, 0.04, 0.03),
            GLOW,
            [s * 0.06, 0.04, 0.15],
        ));
    }

    // Armor and weapons per branch
    if branch == 0 && tier >= 1 {
        // Branch A (Ossified / Bone Wall): Heavy bone shoulder plates
        for (arm, s) in [(UpperArmL, 1.0), (UpperArmR, -1.0)] {
            parts.push(part(
                Body(arm),
                Box(0.20, 0.10, 0.22),
                BONE,
                [s * 0.03, 0.02, 0.0],
            ));
        }
        // Left hand: Ribcage / skull shield
        let shield_h = if tier >= 2 { 0.60 } else { 0.48 };
        for i in 0..4 {
            parts.push(part(
                Body(HandL),
                Box(0.05, shield_h, 0.05),
                BONE,
                [-0.15 + 0.10 * i as f32, 0.06, 0.14],
            ));
        }
        parts.push(part(Body(HandL), Ball(0.10), BONE, [0.0, 0.06, 0.18]));
    }

    // Weapon on HandR (and HandL for Branch B)
    if branch == 1 {
        if tier >= 2 {
            // Branch B Tier 2 (Reaper): Long Two-Handed Scythe
            parts.push(
                part(Body(HandR), Cyl(0.026, 1.25), WOOD, [0.0, 0.25, 0.0]).rot([90.0, 0.0, 0.0]),
            );
            parts.push(
                part(Body(HandR), Box(0.04, 0.12, 0.65), IRON, [0.0, 0.82, 0.24])
                    .rot([-15.0, 0.0, 0.0]),
            );
        } else {
            // Branch B Tier 1 (Butcher): Twin iron cleavers
            parts.push(part(
                Body(HandR),
                Box(0.06, 0.32, 0.44),
                IRON,
                [0.0, 0.0, 0.24],
            ));
            parts.push(part(
                Body(HandL),
                Box(0.06, 0.32, 0.44),
                IRON,
                [0.0, 0.0, 0.24],
            ));
        }
    } else {
        // Base and Branch A: Heavy iron cleaver
        parts.push(part(
            Body(HandR),
            Box(0.06, 0.32, 0.46),
            IRON,
            [0.0, 0.0, 0.26],
        ));
    }

    FigureSpec { plan, parts }
}
