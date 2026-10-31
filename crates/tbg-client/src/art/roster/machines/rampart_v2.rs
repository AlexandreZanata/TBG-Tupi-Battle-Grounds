//! Native rig-v2 Rampart: wall-like Tank without human limbs; heavy mechanical
//! chassis, interlocking mantlet wall, gunmetal and iron with brass rivets and orange sensors.
//! - Base: Large body plan, bunker cupola, heavy mantlet shield plate across torso.
//! - Branch A (Spiked Bastion): iron ram spikes, double-thick reinforced mantlet plate.
//! - Branch B (Mobile Redoubt): fortified battlement crenellations, flanking barricade wings.

use crate::art::{
    figure::{part, FigureSpec},
    mesh::Shape::*,
    palette::*,
    rig::{BodyPlan, Joint::*, Node::Body, Proportions},
};
use bevy::prelude::Vec3;

pub fn plan(tier: u8) -> Proportions {
    let mut plan = BodyPlan::Large.proportions();
    plan.torso = Vec3::new(0.60, 0.42, 0.36);
    plan.leg = 0.28;
    plan.arm = 0.20;
    plan.head = 0.16;

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

    // Mechanical tread columns and strut limbs
    for p in &mut parts {
        match p.node {
            Body(ThighL | ThighR | ShinL | ShinR) => {
                p.scale = Vec3::new(1.40, 1.0, 1.40);
                p.color = GUNMETAL;
            }
            Body(FootL | FootR) => {
                p.scale = Vec3::new(1.35, 1.1, 1.50);
                p.color = IRON;
            }
            Body(UpperArmL | UpperArmR | ForearmL | ForearmR) => {
                p.scale = Vec3::new(1.30, 1.0, 1.30);
                p.color = IRON;
            }
            _ => {}
        }
    }

    // Armored bunker observation cupola on Head
    parts.push(part(
        Body(Head),
        Box(0.34, 0.22, 0.30),
        GUNMETAL,
        [0.0, 0.02, 0.0],
    ));
    // Narrow horizontal orange sensor vision slit
    parts.push(part(
        Body(Head),
        Box(0.28, 0.05, 0.04),
        ORANGE,
        [0.0, 0.04, 0.15],
    ));
    // Brass top periscope cap
    parts.push(part(Body(Head), Cyl(0.08, 0.06), BRASS, [0.0, 0.15, 0.0]));

    // Massive central mantlet wall plate on Chest
    let wall_w = if tier >= 2 { 0.95 } else { 0.82 };
    let wall_h = if tier >= 2 { 0.58 } else { 0.48 };

    parts.push(part(
        Body(Chest),
        Box(wall_w, wall_h, 0.10),
        IRON,
        [0.0, 0.12, 0.22],
    ));

    // Brass corner and edge reinforcement bolts on the wall plate
    for x in [-wall_w * 0.42, wall_w * 0.42] {
        for y in [-wall_h * 0.35 + 0.12, wall_h * 0.35 + 0.12] {
            parts.push(part(Body(Chest), Ball(0.035), BRASS, [x, y, 0.28]));
        }
    }

    // Side mantlet extensions mounted on HandL and HandR to form contiguous wall
    let side_w = if tier >= 1 { 0.26 } else { 0.18 };
    for (hand, s) in [(HandL, 1.0f32), (HandR, -1.0f32)] {
        parts.push(part(
            Body(hand),
            Box(side_w, wall_h * 0.85, 0.08),
            GUNMETAL,
            [s * side_w * 0.5, 0.04, 0.20],
        ));
    }

    if branch == 0 {
        // Branch A (Spiked Bastion): Heavy ram spikes & reinforced bumper
        for x in [-0.26f32, 0.0, 0.26] {
            parts.push(
                part(Body(Chest), Cone(0.06, 0.24), STEEL, [x, 0.12, 0.38]).rot([90.0, 0.0, 0.0]),
            );
        }

        if tier >= 1 {
            // Lower iron ramming bar on hips
            parts.push(part(
                Body(Hips),
                Box(wall_w * 0.90, 0.12, 0.14),
                IRON,
                [0.0, -0.06, 0.22],
            ));
        }
        if tier >= 2 {
            // Heavy steel armor slab strapped over central mantlet
            parts.push(part(
                Body(Chest),
                Box(wall_w * 0.65, wall_h * 0.65, 0.06),
                STEEL,
                [0.0, 0.12, 0.28],
            ));
        }
    } else {
        // Branch B (Mobile Redoubt): Crenellations and flanking wings
        // Battlements along top edge of mantlet
        for i in 0..4 {
            let cx = -wall_w * 0.38 + i as f32 * (wall_w * 0.76 / 3.0);
            parts.push(part(
                Body(Chest),
                Box(0.10, 0.10, 0.08),
                GUNMETAL,
                [cx, wall_h * 0.50 + 0.12, 0.22],
            ));
        }

        if tier >= 1 {
            // Side bunker wings angled forward on shoulders
            for (arm, s) in [(UpperArmL, 1.0f32), (UpperArmR, -1.0f32)] {
                parts.push(
                    part(
                        Body(arm),
                        Box(0.18, 0.32, 0.12),
                        IRON,
                        [s * 0.06, 0.0, 0.12],
                    )
                    .rot([0.0, s * 20.0, 0.0]),
                );
            }
        }
        if tier >= 2 {
            // Twin glowing orange hazard beacons on top corners
            for x in [-wall_w * 0.44, wall_w * 0.44] {
                parts.push(part(
                    Body(Chest),
                    Ball(0.045),
                    ORANGE,
                    [x, wall_h * 0.52 + 0.14, 0.22],
                ));
            }
        }
    }

    FigureSpec { plan, parts }
}
