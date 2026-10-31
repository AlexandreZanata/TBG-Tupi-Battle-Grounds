//! Native rig-v2 Nester: hive druid archetype, dark bark, chitin lacquer,
//! golden amber honey and woven hive structures.
//! - Base: insectoid antennae crest, hollow-tree hive on back, honey crook staff.
//! - Branch A (Swarm Brood): towering multi-tier hive nest, golden honey crown.
//! - Branch B (Chitin Warden): heavy beetle carapace plates, mandible horns.

use crate::art::{
    figure::{part, FigureSpec},
    mesh::Shape::*,
    rig::{BodyPlan, Joint::*, Node::Body, Node::Secondary, Proportions, Spring},
};
use bevy::prelude::{Color, Vec3};

const NESTER_AMBER: Color = Color::srgb(0.92, 0.72, 0.25);
const NESTER_DARK_CHITIN: Color = Color::srgb(0.22, 0.17, 0.12);
const NESTER_WOOD: Color = Color::srgb(0.35, 0.28, 0.19);

pub fn plan(tier: u8) -> Proportions {
    let mut plan = BodyPlan::Humanoid.proportions();
    plan.torso = Vec3::new(0.36, 0.36, 0.26);
    plan.leg = 0.36;
    plan.arm = 0.20;
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
    let mut parts = super::super::body::parts(&plan, NESTER_WOOD, NESTER_DARK_CHITIN, NESTER_WOOD);

    // Insectoid antennae / horn crest on head
    for s in [-1.0f32, 1.0] {
        parts.push(
            part(
                Body(Head),
                Cone(0.035, 0.18),
                NESTER_AMBER,
                [s * 0.08, 0.16, 0.06],
            )
            .rot([15.0, 0.0, s * 20.0]),
        );
    }

    // Honeycomb cells on chest
    for y in [0.10f32, 0.22] {
        parts.push(part(Body(Chest), Ball(0.05), NESTER_AMBER, [0.0, y, 0.15]));
    }

    // Woven hollow-tree hive nest on back spring
    let hive_h = if tier >= 2 { 0.52 } else { 0.42 };
    parts.push(
        part(
            Secondary(Spring::Back),
            Frustum(0.18, 0.12, hive_h),
            NESTER_WOOD,
            [0.0, 0.10, -0.16],
        )
        .rot([-10.0, 0.0, 0.0]),
    );

    // Honeycomb hive entrance cell
    parts.push(part(
        Secondary(Spring::Back),
        Ball(0.06),
        NESTER_AMBER,
        [0.0, 0.12, -0.22],
    ));

    // Hivekeeper staff on HandR
    let staff_len = if tier >= 2 { 1.05 } else { 0.88 };
    parts.push(
        part(
            Body(HandR),
            Cyl(0.018, staff_len),
            NESTER_WOOD,
            [0.0, staff_len * 0.20, 0.06],
        )
        .rot([90.0, 0.0, 0.0]),
    );
    let staff_top = staff_len * 0.70;

    // Glowing amber honey drop at staff tip
    parts.push(part(
        Body(HandR),
        Ball(0.075),
        NESTER_AMBER,
        [0.0, staff_top, 0.06],
    ));

    if branch == 0 {
        // Branch A (Swarm Brood): Giant multi-chamber hive and golden honey crown
        if tier >= 1 {
            // Golden queen honey crown
            parts.push(part(
                Body(Head),
                Ring(0.02, 0.16, 360.0),
                NESTER_AMBER,
                [0.0, 0.16, 0.0],
            ));
        }
        if tier >= 2 {
            // Additional upper hive chamber
            parts.push(part(
                Secondary(Spring::Back),
                Frustum(0.12, 0.08, 0.24),
                NESTER_WOOD,
                [0.0, hive_h * 0.65, -0.16],
            ));
            // Flying drone aura around staff head
            for s in [-0.08f32, 0.08] {
                parts.push(part(
                    Body(HandR),
                    Ball(0.03),
                    NESTER_AMBER,
                    [s, staff_top + 0.06, 0.06],
                ));
            }
        }
    } else {
        // Branch B (Chitin Warden): Beetle carapace plates and mandible horns
        // Heavy chitin shoulder carapaces
        for (arm, s) in [(UpperArmL, 1.0f32), (UpperArmR, -1.0f32)] {
            parts.push(part(
                Body(arm),
                Box(0.18, 0.24, 0.14),
                NESTER_DARK_CHITIN,
                [s * 0.04, 0.02, 0.0],
            ));
        }

        if tier >= 1 {
            // Mandible pincer horns on head
            for s in [-1.0f32, 1.0] {
                parts.push(
                    part(
                        Body(Head),
                        Cone(0.04, 0.20),
                        NESTER_DARK_CHITIN,
                        [s * 0.12, 0.04, 0.16],
                    )
                    .rot([90.0, s * 25.0, 0.0]),
                );
            }
        }
        if tier >= 2 {
            // Reinforced chitin chest breastplate
            parts.push(part(
                Body(Chest),
                Box(0.42, 0.32, 0.08),
                NESTER_DARK_CHITIN,
                [0.0, 0.16, 0.18],
            ));
        }
    }

    FigureSpec { plan, parts }
}
