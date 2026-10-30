//! Native rig-v2 Beetling: miniature beetle summon (0.55x scale), minimal mesh
//! budget; dark lacquer chitin and amber accents matching Nester's tier and branch.
//! - Base: compact beetle body, front mandibles/horns, dark lacquer shell.
//! - Branch A (Swarm Winged): amber wing covers on back, golden eyes.
//! - Branch B (Spiked Stag): large front stag horn, spiked chitin shell.

use crate::art::{
    figure::{part, FigureSpec},
    mesh::Shape::*,
    rig::{BodyPlan, Joint::*, Node::Body, Proportions},
};
use bevy::prelude::{Color, Vec3};

const BEETLE_CHITIN: Color = Color::srgb(0.20, 0.16, 0.12);
const BEETLE_AMBER: Color = Color::srgb(0.88, 0.68, 0.22);

pub fn plan(tier: u8) -> Proportions {
    let mut plan = BodyPlan::Humanoid.proportions();
    plan.torso = Vec3::new(0.32, 0.24, 0.30);
    plan.leg = 0.20;
    plan.arm = 0.16;
    plan.head = 0.12;

    let evo_scale = match tier {
        1 => 1.08,
        2 => 1.16,
        _ => 1.00,
    };
    // Miniature summon scale (0.55x) * tier evolution
    plan.scale = 0.55 * evo_scale;
    plan
}

pub fn spec(tier: u8, branch: u8) -> FigureSpec {
    let plan = plan(tier);
    let mut parts = super::super::body::parts(&plan, BEETLE_CHITIN, BEETLE_CHITIN, BEETLE_CHITIN);

    // Shiny beetle carapace shell on Chest
    parts.push(
        part(Body(Chest), Ball(0.18), BEETLE_CHITIN, [0.0, 0.10, -0.06]).scale([1.1, 0.85, 1.2]),
    );

    // Amber glowing insect eyes
    for s in [-1.0f32, 1.0] {
        parts.push(part(
            Body(Head),
            Ball(0.03),
            BEETLE_AMBER,
            [s * 0.05, 0.02, 0.10],
        ));
    }

    if branch == 0 {
        // Branch A (Swarm Winged): Wing covers and flight mandibles
        for s in [-1.0f32, 1.0] {
            parts.push(
                part(
                    Body(Head),
                    Cone(0.028, 0.14),
                    BEETLE_AMBER,
                    [s * 0.06, 0.04, 0.12],
                )
                .rot([90.0, s * 15.0, 0.0]),
            );
        }

        if tier >= 1 {
            // Amber elytra wing covers over back
            parts.push(part(
                Body(Chest),
                Box(0.24, 0.22, 0.04),
                BEETLE_AMBER,
                [0.0, 0.10, -0.14],
            ));
        }
    } else {
        // Branch B (Spiked Stag): Prominent stag horn and spiked shell
        // Central forward stag beetle horn
        let horn_len = if tier >= 2 { 0.26 } else { 0.18 };
        parts.push(
            part(
                Body(Head),
                Cone(0.035, horn_len),
                BEETLE_CHITIN,
                [0.0, 0.08, 0.14],
            )
            .rot([70.0, 0.0, 0.0]),
        );

        if tier >= 1 {
            // Spiked shell ridges on carapace
            for z in [-0.08f32, 0.04] {
                parts.push(
                    part(
                        Body(Chest),
                        Cone(0.025, 0.08),
                        BEETLE_CHITIN,
                        [0.0, 0.22, z],
                    )
                    .rot([0.0, 0.0, 0.0]),
                );
            }
        }
    }

    FigureSpec { plan, parts }
}
