//! Native rig-v2 Sawbot: mechanical chassis, iron and gunmetal with brass core
//! and orange sensor glow.
//! - Base: mechanical limbs, boxy sensor head, right forearm mounted circular saw.
//! - Branch A (Twin Shredder): dual spinning saws on both arms, back exhaust vents.
//! - Branch B (Heavy Dreadnought): riveted iron armor plating, tower helm crest, giant blade.

use crate::art::{
    figure::{part, FigureSpec},
    mesh::Shape::*,
    palette::*,
    rig::{BodyPlan, Joint::*, Node::Body, Node::Secondary, Proportions, Spring},
};
use bevy::prelude::Vec3;

pub fn plan(tier: u8) -> Proportions {
    let mut plan = BodyPlan::Humanoid.proportions();
    plan.torso = Vec3::new(0.48, 0.34, 0.34);
    plan.leg = 0.26;
    plan.arm = 0.22;
    plan.head = 0.15;

    let evo_scale = match tier {
        1 => 1.08,
        2 => 1.16,
        _ => 1.00,
    };
    // Match-camera lane readability factor (1.20x) * tier evolution
    plan.scale = 1.20 * evo_scale;
    plan
}

pub fn spec(tier: u8, branch: u8) -> FigureSpec {
    let plan = plan(tier);
    let mut parts = super::super::body::parts(&plan, GUNMETAL, IRON, GUNMETAL);

    // Replace organic limb feel with mechanical blocky plating
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

    // Boxy robot head
    parts.push(part(
        Body(Head),
        Box(0.28, 0.22, 0.26),
        GUNMETAL,
        [0.0, 0.02, 0.0],
    ));
    // Horizontal glowing orange sensor visor slit
    parts.push(part(
        Body(Head),
        Box(0.24, 0.06, 0.04),
        ORANGE,
        [0.0, 0.04, 0.13],
    ));

    // Torso central brass boiler core with forge glow
    parts.push(part(Body(Chest), Ball(0.08), BRASS, [0.0, 0.16, 0.18]));
    parts.push(part(Body(Chest), Ball(0.045), ORANGE, [0.0, 0.16, 0.22]));

    // Helper to spawn a circular buzzsaw on a hand
    let saw = |parts: &mut Vec<_>, hand, r: f32| {
        parts.push(part(Body(hand), Cyl(0.04, 0.08), BRASS, [0.0, -0.04, 0.10]));
        parts.push(part(Body(hand), Cyl(r, 0.025), IRON, [0.0, -0.04, 0.18]).rot([0.0, 0.0, 90.0]));
        parts.push(
            part(Body(hand), Ring(0.02, r, 360.0), ORANGE, [0.0, -0.04, 0.18])
                .rot([0.0, 0.0, 90.0]),
        );
    };

    let saw_r = if branch == 1 && tier >= 2 {
        0.28
    } else if tier >= 1 {
        0.24
    } else {
        0.21
    };

    // Right hand always carries a buzzsaw
    saw(&mut parts, HandR, saw_r);

    if branch == 0 {
        // Branch A (Twin-Saw Shredder / Vents)
        if tier >= 1 {
            // Left hand also carries a buzzsaw
            saw(&mut parts, HandL, saw_r);
        }
        if tier >= 1 {
            // Twin vertical exhaust pipes on back
            for x in [-0.11f32, 0.11] {
                parts.push(part(Body(Chest), Cyl(0.038, 0.32), IRON, [x, 0.36, -0.16]));
                parts.push(part(
                    Body(Chest),
                    Cyl(0.044, 0.06),
                    ORANGE,
                    [x, 0.52, -0.16],
                ));
            }
        }
        if tier >= 2 {
            // Central high-pressure turbine on back spring
            parts.push(part(
                Secondary(Spring::Back),
                Cyl(0.06, 0.14),
                BRASS,
                [0.0, 0.22, -0.22],
            ));
            parts.push(part(
                Secondary(Spring::Back),
                Ball(0.04),
                ORANGE,
                [0.0, 0.22, -0.28],
            ));
        }
    } else {
        // Branch B (Ironclad / Heavy Dreadnought): Riveted plates & tower helm
        // Heavy front chest armor slab
        parts.push(part(
            Body(Chest),
            Box(0.52, 0.28, 0.06),
            IRON,
            [0.0, 0.16, 0.19],
        ));
        // Heavy rear reactor carapace
        parts.push(part(
            Body(Chest),
            Box(0.50, 0.26, 0.06),
            IRON,
            [0.0, 0.16, -0.19],
        ));

        // Brass corner rivets
        for x in [-0.22f32, 0.22] {
            for y in [0.06f32, 0.26] {
                parts.push(part(Body(Chest), Ball(0.025), BRASS, [x, y, 0.22]));
            }
        }

        if tier >= 1 {
            // Heavy tower helm crest plate on head
            parts.push(part(
                Body(Head),
                Box(0.32, 0.07, 0.30),
                IRON,
                [0.0, 0.14, 0.0],
            ));
        }
        if tier >= 2 {
            // Forward iron ram bumper on hips
            parts.push(part(
                Body(Hips),
                Box(0.48, 0.10, 0.12),
                IRON,
                [0.0, -0.04, 0.18],
            ));
        }
    }

    FigureSpec { plan, parts }
}
