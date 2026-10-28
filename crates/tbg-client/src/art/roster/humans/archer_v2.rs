//! Candidate Archer (owner review, `?archer=v2`): the base form refined to the
//! owner's reference sheet, and the A-branch evolutions Longbow (tier 1: green
//! hood, feather, yew longbow) and Marksman (tier 2: gilded recurve bow,
//! team half-cape, gold trim, 7 % larger). Gear lives in `archer_v2_gear`.

use super::super::body;
use super::archer_v2_gear as gear;
use crate::art::{
    figure::{part, FigureSpec, PartSpec},
    mesh::Shape::*,
    palette::*,
    rig::{Joint::*, Node::Body, Node::Secondary, Spring},
};

pub fn spec(tier: u8) -> FigureSpec {
    let mut plan = crate::art::archer_v2::plan();
    // The client's 20 % lane-readability scale; the Marksman stands taller.
    plan.scale *= 1.2 * if tier >= 2 { 1.07 } else { 1.0 };
    let hood = if tier == 0 { OAK } else { HOOD_GREEN };
    let mut parts = body::parts(&plan, LINEN, SKIN, DARK_LEATHER);
    for p in &mut parts {
        if matches!(p.node, Body(ThighL | ThighR)) {
            p.color = RUST;
        }
    }
    tunic(&mut parts);
    head(&mut parts, hood, tier);
    gear::bow(&mut parts, tier);
    gear::quiver(&mut parts, tier);
    if tier >= 1 {
        gear::ranger(&mut parts);
    }
    if tier >= 2 {
        gear::marksman(&mut parts);
    }
    FigureSpec { plan, parts }
}

fn tunic(parts: &mut Vec<PartSpec>) {
    parts.extend([
        // Skirt with a darker hem, belt, buckle, and a side pouch.
        part(
            Body(Hips),
            Frustum(0.16, 0.21, 0.20),
            LINEN,
            [0.0, -0.08, 0.0],
        )
        .scale([1.0, 1.0, 0.75]),
        part(
            Body(Hips),
            Frustum(0.212, 0.218, 0.032),
            TRIM,
            [0.0, -0.17, 0.0],
        )
        .scale([1.0, 1.0, 0.77]),
        part(
            Body(Hips),
            Box(0.34, 0.045, 0.255),
            DARK_LEATHER,
            [0.0, 0.04, 0.0],
        ),
        part(
            Body(Hips),
            Box(0.055, 0.055, 0.015),
            GOLD,
            [0.0, 0.04, 0.14],
        ),
        part(
            Body(Hips),
            Box(0.075, 0.085, 0.05),
            DARK_LEATHER,
            [-0.15, -0.025, 0.07],
        ),
        // Team tabard with a gold hem and a heraldic lozenge.
        part(
            Secondary(Spring::Tabard),
            Box(0.145, 0.34, 0.018),
            ALLY,
            [0.0, 0.09, 0.025],
        ),
        part(
            Secondary(Spring::Tabard),
            Box(0.12, 0.02, 0.022),
            GOLD,
            [0.0, -0.075, 0.025],
        ),
        part(
            Secondary(Spring::Tabard),
            Box(0.05, 0.05, 0.022),
            GOLD,
            [0.0, 0.16, 0.026],
        )
        .rot([0.0, 0.0, 45.0]),
        // Bracer on the bow arm, glove on the string hand, cuffs at the elbows.
        part(
            Body(ForearmL),
            Box(0.125, 0.13, 0.125),
            DARK_LEATHER,
            [0.0, -0.09, 0.0],
        ),
        part(
            Body(ForearmR),
            Box(0.115, 0.055, 0.12),
            DARK_LEATHER,
            [0.0, -0.11, 0.0],
        ),
    ]);
    for (fore, shin) in [(ForearmL, ShinL), (ForearmR, ShinR)] {
        // Boxes, not frustums: four cuffs at ~48 triangles each break the budget.
        parts.push(part(
            Body(fore),
            Box(0.118, 0.03, 0.118),
            OAK,
            [0.0, -0.012, 0.0],
        ));
        parts.push(part(
            Body(shin),
            Box(0.142, 0.036, 0.142),
            OAK,
            [0.0, -0.02, 0.0],
        ));
    }
}

fn head(parts: &mut Vec<PartSpec>, hood: bevy::prelude::Color, tier: u8) {
    parts.extend([
        // Rounded hood, forehead brim, capelet, and the cowl point (spring).
        part(Body(Head), Ball(0.163), hood, [0.0, 0.035, -0.055]).scale([1.04, 1.05, 1.0]),
        part(
            Body(Head),
            Box(0.25, 0.038, 0.07),
            if tier >= 2 { GOLD } else { DARK_LEATHER },
            [0.0, 0.10, 0.095],
        )
        .rot([-12.0, 0.0, 0.0]),
        part(
            Body(Chest),
            Frustum(0.16, 0.21, 0.10),
            hood,
            [0.0, 0.32, -0.015],
        )
        .scale([1.0, 1.0, 0.8]),
        part(
            Secondary(Spring::HeadGear),
            Cone(0.10, 0.18),
            hood,
            [0.0, 0.13, -0.14],
        )
        .rot([-45.0, 0.0, 0.0]),
        // Face: nose and a short dark beard along the jaw.
        part(
            Body(Head),
            Box(0.027, 0.045, 0.04),
            SKIN,
            [0.0, -0.005, 0.139],
        ),
        part(Body(Head), Box(0.16, 0.04, 0.05), RUST, [0.0, -0.07, 0.10]),
    ]);
    for x in [-1.0, 1.0] {
        parts.push(part(
            Body(Head),
            Box(0.035, 0.15, 0.10),
            hood,
            [x * 0.125, -0.015, 0.055],
        ));
        parts.push(part(
            Body(Head),
            Box(0.018, 0.026, 0.012),
            INK,
            [x * 0.049, 0.025, 0.125],
        ));
        parts.push(part(
            Body(Head),
            Box(0.04, 0.01, 0.012),
            RUST,
            [x * 0.05, 0.055, 0.126],
        ));
    }
}
