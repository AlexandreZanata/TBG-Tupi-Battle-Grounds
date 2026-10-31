//! Native rig-v2 Hexer: Slender Control archetype, hooded cowl, skull staff,
//! Branch A (Withering / Doom) with bone crown, tall staff, and occult halo ring,
//! Branch B (Plague / Pestilence) with hanging iron censer and spectral green spores.

use crate::art::{
    figure::{part, FigureSpec},
    mesh::Shape::*,
    palette::*,
    rig::{BodyPlan, Joint::*, Node::Body, Node::Secondary, Proportions, Spring},
};
use bevy::prelude::Vec3;

pub fn plan(tier: u8) -> Proportions {
    let mut plan = BodyPlan::Humanoid.proportions();
    plan.torso = Vec3::new(0.32, 0.34, 0.22);
    plan.leg = 0.38;
    plan.arm = 0.18;
    plan.head = 0.1365;

    let evo_scale = match tier {
        1 => 1.08,
        2 => 1.16,
        _ => 1.00,
    };
    plan.scale = 1.20 * evo_scale;
    plan
}

pub fn spec(tier: u8, branch: u8) -> FigureSpec {
    let plan = plan(tier);
    let mut parts = super::super::body::parts(&plan, VIOLET, BONE, DARK);

    // Violet hooded cowl
    parts.push(part(
        Body(Head),
        Cone(0.22, 0.36),
        VIOLET,
        [0.0, 0.12, -0.04],
    ));
    // Glowing spectral green eyes
    for s in [-1.0f32, 1.0] {
        parts.push(part(
            Body(Head),
            Box(0.03, 0.03, 0.02),
            GLOW,
            [s * 0.05, 0.02, 0.14],
        ));
    }

    // Robe skirt on Hips
    parts.push(
        part(
            Body(Hips),
            Frustum(0.24, 0.34, 0.42),
            VIOLET,
            [0.0, -0.21, 0.0],
        )
        .scale([1.0, 1.0, 0.85]),
    );

    // Right Hand: Staff
    let staff_len = if tier >= 2 { 1.25 } else { 1.05 };
    parts.push(
        part(
            Body(HandR),
            Cyl(0.022, staff_len),
            WOOD,
            [0.0, staff_len * 0.22, 0.06],
        )
        .rot([90.0, 0.0, 0.0]),
    );
    let head_y = staff_len * 0.72;

    if branch == 0 {
        // Skull on staff
        parts.push(part(Body(HandR), Ball(0.09), BONE, [0.0, head_y, 0.06]));
        parts.push(part(Body(HandR), Ball(0.04), GLOW, [0.04, head_y, 0.14]));
        parts.push(part(Body(HandR), Ball(0.04), GLOW, [-0.04, head_y, 0.14]));

        if tier >= 1 {
            // Bone crown on head
            parts.push(part(
                Body(Head),
                Box(0.20, 0.05, 0.20),
                BONE,
                [0.0, 0.18, 0.0],
            ));
        }
        if tier >= 2 {
            // Glowing occult halo ring behind staff
            parts.push(
                part(
                    Body(HandR),
                    Ring(0.016, 0.20, 360.0),
                    GLOW,
                    [0.0, head_y, 0.06],
                )
                .rot([90.0, 0.0, 0.0]),
            );
        }
    } else {
        // Branch B (Plague / Pestilence): Left hand iron censer on chain
        parts.push(part(Body(HandR), Ball(0.08), GLOW, [0.0, head_y, 0.06]));

        // Hanging censer
        parts.push(part(
            Body(HandL),
            Cyl(0.006, 0.16),
            IRON,
            [0.0, -0.06, 0.05],
        ));
        parts.push(part(
            Body(HandL),
            Frustum(0.09, 0.05, 0.12),
            IRON,
            [0.0, -0.16, 0.05],
        ));

        // Floating spectral green mist particles on Spring::Back / HandL
        let spores: &[[f32; 3]] = if tier >= 2 {
            &[
                [0.10, -0.08, 0.10],
                [-0.12, -0.04, 0.08],
                [0.05, 0.06, 0.14],
                [-0.06, -0.22, 0.12],
                [0.12, 0.10, 0.02],
            ]
        } else {
            &[[0.10, -0.08, 0.10], [-0.10, -0.02, 0.08]]
        };
        for s in spores {
            parts.push(part(Body(HandL), Ball(0.04), GLOW, *s));
        }

        // Tattered back cowl extension
        parts.push(part(
            Secondary(Spring::HeadGear),
            Box(0.18, 0.24, 0.02),
            VIOLET,
            [0.0, -0.10, -0.10],
        ));
    }

    FigureSpec { plan, parts }
}
