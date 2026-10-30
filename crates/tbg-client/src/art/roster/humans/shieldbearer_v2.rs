//! Native rig-v2 Shieldbearer: base form with kite shield and flanged mace,
//! Branch A (Bulwark / Fortress) with tower shield, plate cuirass, and crest,
//! Branch B (Vanguard / Champion) with spear, fluted pauldrons, and back banner.

use crate::art::{
    figure::{part, FigureSpec, PartSpec},
    mesh::Shape::*,
    palette::*,
    rig::{BodyPlan, Joint::*, Node::Body, Node::Secondary, Proportions, Spring},
};
use bevy::prelude::Vec3;

/// Humanoid Tank proportions according to ART-DIRECTION Section 11.1:
/// - Shoulder : hip ratio 1.40 (torso x = 0.42)
/// - Limb thickness factor x1.25
/// - Prop scale >= 0.70 of body height
pub fn plan(tier: u8) -> Proportions {
    let mut plan = BodyPlan::Humanoid.proportions();
    plan.torso = Vec3::new(0.42, 0.34, 0.24);
    plan.leg = 0.38;
    plan.arm = 0.19;
    plan.head = 0.1365;

    // Strict evolution scaling: 1.00x Base, 1.08x Tier 1, 1.16x Tier 2
    let evo_scale = match tier {
        1 => 1.08,
        2 => 1.16,
        _ => 1.00,
    };
    // 20% match-camera lane readability scale
    plan.scale = 1.20 * evo_scale;
    plan
}

pub fn spec(tier: u8, branch: u8) -> FigureSpec {
    let plan = plan(tier);
    let mut parts = super::super::body::parts(&plan, LINEN, SKIN, DARK_LEATHER);

    // Thicker limbs for Tank archetype
    for p in &mut parts {
        match p.node {
            Body(ThighL | ThighR | ShinL | ShinR) => {
                p.scale = Vec3::new(1.22, 1.0, 1.22);
                p.color = STEEL;
            }
            Body(UpperArmL | UpperArmR | ForearmL | ForearmR) => {
                p.scale = Vec3::new(1.20, 1.0, 1.20);
                p.color = LEATHER;
            }
            _ => {}
        }
    }

    armor(&mut parts, tier, branch);
    helmet(&mut parts, tier, branch);
    shield(&mut parts, tier, branch);
    weapon(&mut parts, tier, branch);

    if branch == 1 && tier >= 1 {
        banner(&mut parts, tier);
    }

    FigureSpec { plan, parts }
}

fn armor(parts: &mut Vec<PartSpec>, tier: u8, branch: u8) {
    let chest_plate_mat = if tier >= 2 && branch == 0 {
        GOLD
    } else {
        STEEL
    };

    // Steel breastplate
    parts.push(
        part(
            Body(Chest),
            Frustum(0.24, 0.21, 0.30),
            chest_plate_mat,
            [0.0, 0.17, 0.02],
        )
        .scale([1.08, 1.0, 0.85]),
    );

    // Team tabard with gold hem on Spring::Tabard
    parts.extend([
        part(
            Secondary(Spring::Tabard),
            Box(0.18, 0.36, 0.02),
            ALLY,
            [0.0, 0.09, 0.03],
        ),
        part(
            Secondary(Spring::Tabard),
            Box(0.16, 0.03, 0.025),
            GOLD,
            [0.0, -0.08, 0.03],
        ),
    ]);

    // Pauldrons (shoulder plates)
    let pauldron_mat = if tier >= 2 { GOLD } else { STEEL };
    let pauldron_size = if branch == 1 { 0.13 } else { 0.11 };
    for (arm, s) in [(UpperArmL, 1.0), (UpperArmR, -1.0)] {
        parts.push(
            part(
                Body(arm),
                Frustum(pauldron_size, pauldron_size * 0.7, 0.09),
                pauldron_mat,
                [s * 0.03, 0.04, 0.0],
            )
            .scale([1.2, 1.0, 1.2]),
        );
    }
}

fn helmet(parts: &mut Vec<PartSpec>, tier: u8, branch: u8) {
    // Steel helmet base
    parts.push(part(
        Body(Head),
        Frustum(0.14, 0.20, 0.15),
        STEEL,
        [0.0, 0.08, 0.0],
    ));
    // Nasal / brow guard
    parts.push(part(
        Body(Head),
        Box(0.04, 0.14, 0.04),
        STEEL,
        [0.0, -0.01, 0.16],
    ));

    // Branch A Tier 2 (Fortress): Signature red horsehair crest
    if branch == 0 && tier >= 2 {
        parts.push(part(
            Body(Head),
            Box(0.04, 0.14, 0.30),
            RED_CLOTH,
            [0.0, 0.22, -0.02],
        ));
    } else if branch == 1 && tier >= 2 {
        // Branch B Tier 2 (Champion): Side plumage
        parts.push(
            part(Body(Head), Cone(0.04, 0.22), RED_CLOTH, [0.12, 0.16, -0.05])
                .rot([20.0, 0.0, 25.0]),
        );
    }
}

fn shield(parts: &mut Vec<PartSpec>, tier: u8, branch: u8) {
    if branch == 0 && tier >= 1 {
        // Branch A (Bulwark / Fortress): Large Tower Shield (Pavise)
        let w = if tier >= 2 { 0.48 } else { 0.44 };
        let h = if tier >= 2 { 0.72 } else { 0.64 };
        parts.push(part(
            Body(HandL),
            Box(w, h, 0.04),
            STEEL,
            [0.08, 0.06, 0.12],
        ));
        // Gold rim/cross emblem
        parts.push(part(
            Body(HandL),
            Box(w * 0.9, 0.06, 0.05),
            GOLD,
            [0.08, 0.06, 0.13],
        ));
        parts.push(part(
            Body(HandL),
            Box(0.06, h * 0.9, 0.05),
            GOLD,
            [0.08, 0.06, 0.13],
        ));
        parts.push(part(Body(HandL), Ball(0.065), GOLD, [0.08, 0.06, 0.16]));
    } else if branch == 1 {
        // Branch B (Vanguard / Champion): Steel heater shield / buckler
        let r = if tier >= 2 { 0.26 } else { 0.22 };
        parts.push(part(Body(HandL), Cyl(r, 0.04), STEEL, [0.08, 0.0, 0.12]).rot([90.0, 0.0, 0.0]));
        parts.push(part(Body(HandL), Ball(0.06), GOLD, [0.08, 0.0, 0.15]));
    } else {
        // Base (0/0): Kite shield (tapered downward)
        parts.push(part(
            Body(HandL),
            Box(0.38, 0.32, 0.035),
            STEEL,
            [0.08, 0.10, 0.12],
        ));
        parts.push(
            part(Body(HandL), Cone(0.19, 0.36), WOOD, [0.08, -0.16, 0.12]).rot([180.0, 0.0, 0.0]),
        );
        // Central steel boss
        parts.push(part(Body(HandL), Ball(0.06), STEEL, [0.08, 0.04, 0.15]));
    }
}

fn weapon(parts: &mut Vec<PartSpec>, tier: u8, branch: u8) {
    if branch == 1 {
        // Branch B: Spear / Halberd (longer reach)
        let len = if tier >= 2 { 1.15 } else { 0.95 };
        parts
            .push(part(Body(HandR), Cyl(0.022, len), WOOD, [0.0, 0.0, 0.18]).rot([90.0, 0.0, 0.0]));
        // Spearhead
        parts.push(
            part(
                Body(HandR),
                Cone(0.055, 0.20),
                STEEL,
                [0.0, 0.0, 0.18 + len / 2.0 + 0.10],
            )
            .rot([90.0, 0.0, 0.0]),
        );
        // Gold wing lugs on spear
        parts.push(part(
            Body(HandR),
            Box(0.14, 0.03, 0.03),
            GOLD,
            [0.0, 0.0, 0.18 + len / 2.0 - 0.04],
        ));
    } else {
        // Base and Branch A: Flanged Mace
        let shaft_len = if tier >= 2 { 0.62 } else { 0.55 };
        parts.push(
            part(Body(HandR), Cyl(0.024, shaft_len), WOOD, [0.0, 0.0, 0.18]).rot([90.0, 0.0, 0.0]),
        );
        let head_z = 0.18 + shaft_len / 2.0;
        let flange_mat = if tier >= 2 { GOLD } else { STEEL };

        // 4 orthogonal flanges radiating from mace head
        parts.push(part(
            Body(HandR),
            Box(0.13, 0.02, 0.14),
            flange_mat,
            [0.0, 0.0, head_z],
        ));
        parts.push(part(
            Body(HandR),
            Box(0.02, 0.13, 0.14),
            flange_mat,
            [0.0, 0.0, head_z],
        ));
        // Top crown spike
        parts.push(
            part(
                Body(HandR),
                Cone(0.035, 0.08),
                STEEL,
                [0.0, 0.0, head_z + 0.09],
            )
            .rot([90.0, 0.0, 0.0]),
        );
    }
}

fn banner(parts: &mut Vec<PartSpec>, tier: u8) {
    // Back pole on Spring::Back
    let pole_h = if tier >= 2 { 0.90 } else { 0.75 };
    parts.push(part(
        Secondary(Spring::Back),
        Cyl(0.016, pole_h),
        WOOD,
        [0.0, pole_h / 2.0, -0.14],
    ));
    // Fluttering team banner
    parts.push(part(
        Secondary(Spring::Back),
        Box(0.02, 0.28, 0.36),
        ALLY,
        [0.0, pole_h - 0.18, -0.32],
    ));
    parts.push(part(
        Secondary(Spring::Back),
        Box(0.025, 0.04, 0.38),
        GOLD,
        [0.0, pole_h - 0.04, -0.32],
    ));
}
