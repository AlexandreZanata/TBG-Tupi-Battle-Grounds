//! Native rig-v2 Rootcaller: small treant archetype, bark wood, dark leaf tendrils,
//! and fungal mushroom accents.
//! - Base: leaf-horned treant head, root claws on both hands.
//! - Branch A (Strangle): extended root tendril whips, leaf cuffs, thorn lashers.
//! - Branch B (Overgrowth): vibrant red mushroom cap canopy with white spore spots.

use crate::art::{
    figure::{part, FigureSpec},
    mesh::Shape::*,
    palette::*,
    rig::{BodyPlan, Joint::*, Node::Body, Proportions},
};
use bevy::prelude::Vec3;

pub fn plan(tier: u8) -> Proportions {
    let mut plan = BodyPlan::Humanoid.proportions();
    plan.torso = Vec3::new(0.40, 0.34, 0.28);
    plan.leg = 0.32;
    plan.arm = 0.22;
    plan.head = 0.14;

    let evo_scale = match tier {
        1 => 1.08,
        2 => 1.16,
        _ => 1.00,
    };
    plan.scale = 1.15 * evo_scale;
    plan
}

pub fn spec(tier: u8, branch: u8) -> FigureSpec {
    let plan = plan(tier);
    let mut parts = super::super::body::parts(&plan, BARK, BARK, LEAF_DARK);

    // Make treant limbs rough and woody
    for p in &mut parts {
        match p.node {
            Body(ThighL | ThighR | ShinL | ShinR) => {
                p.color = BARK;
            }
            Body(UpperArmL | UpperArmR | ForearmL | ForearmR) => {
                p.color = BARK;
            }
            _ => {}
        }
    }

    // Horned leaf branches sprouting from head
    parts.push(part(Body(Head), Cone(0.05, 0.18), LEAF, [0.08, 0.16, 0.0]).rot([0.0, 0.0, -25.0]));
    parts.push(part(Body(Head), Cone(0.05, 0.18), LEAF, [-0.08, 0.16, 0.0]).rot([0.0, 0.0, 25.0]));

    // Root claws / tendril whips on hands
    let claw_len = if branch == 0 {
        if tier >= 2 {
            0.78
        } else if tier >= 1 {
            0.65
        } else {
            0.52
        }
    } else {
        0.44
    };

    for hand in [HandL, HandR] {
        parts.push(
            part(
                Body(hand),
                Pill(0.03, claw_len),
                LEAF_DARK,
                [0.0, -0.04, claw_len * 0.50],
            )
            .rot([90.0, 0.0, 0.0]),
        );
        parts.push(
            part(
                Body(hand),
                Cone(0.04, 0.12),
                LEAF,
                [0.0, -0.04, claw_len + 0.05],
            )
            .rot([90.0, 0.0, 0.0]),
        );
    }

    if branch == 0 {
        // Branch A (Strangle): Leaf cuffs and grasping tendrils
        if tier >= 1 {
            // Leaf cuffs around forearms
            for arm in [ForearmL, ForearmR] {
                parts.push(part(
                    Body(arm),
                    Ring(0.022, 0.095, 360.0),
                    LEAF,
                    [0.0, -0.08, 0.0],
                ));
            }
        }
        if tier >= 2 {
            // Twisting root spurs along torso
            for (x, s) in [(-0.16f32, -1.0f32), (0.16, 1.0)] {
                parts.push(
                    part(Body(Chest), Cone(0.03, 0.20), LEAF_DARK, [x, 0.15, 0.12]).rot([
                        0.0,
                        0.0,
                        s * 45.0,
                    ]),
                );
            }
        }
    } else {
        // Branch B (Overgrowth): Red mushroom canopy
        let cap_r = if tier >= 2 { 0.32 } else { 0.25 };
        parts.push(
            part(Body(Head), Ball(cap_r), MUSHROOM, [0.0, 0.15, 0.0]).scale([1.0, 0.45, 1.0]),
        );

        // White fungal spore spots on head mushroom cap
        for (x, z) in [(0.10f32, 0.08f32), (-0.12, 0.02), (0.02, -0.12)] {
            parts.push(part(Body(Head), Ball(0.035), BONE, [x, 0.24, z]));
        }

        if tier >= 1 {
            // Shoulder mushroom caps
            for (arm, s) in [(UpperArmL, 1.0f32), (UpperArmR, -1.0f32)] {
                parts.push(
                    part(Body(arm), Ball(0.09), MUSHROOM, [s * 0.06, 0.06, 0.0])
                        .scale([1.0, 0.50, 1.0]),
                );
            }
        }
        if tier >= 2 {
            // Sprouting mushroom cluster across back and chest
            for (x, y) in [(0.18f32, 0.36f32), (-0.18, 0.32), (0.0, 0.42)] {
                parts.push(part(Body(Chest), Ball(0.05), MUSHROOM, [x, y, 0.12]));
            }
        }
    }

    FigureSpec { plan, parts }
}
