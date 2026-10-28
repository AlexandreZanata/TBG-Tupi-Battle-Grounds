//! Humans: disciplined guards — leather, trim, steel, gold.

pub mod archer;
pub mod archer_v2;
mod archer_v2_gear;

use super::super::legacy::{part, BodyPlan, FigureSpec, Joint::*};
use super::super::mesh::Shape::*;
use super::super::palette::*;
use super::{base_body, Form, Look};

pub fn shieldbearer(f: Form) -> FigureSpec {
    let plan = BodyPlan::SMALL;
    let mut p = base_body(&plan, &Look::new(SKIN, LEATHER, TRIM, WOOD));
    // Helmet with nose guard; Fortress adds a red crest.
    p.push(part(
        Head,
        Frustum(0.13, 0.19, 0.13),
        STEEL,
        [0.0, 0.09, 0.0],
    ));
    p.push(part(Head, Box(0.04, 0.13, 0.03), STEEL, [0.0, -0.01, 0.17]));
    p.push(part(
        Torso,
        Box(0.38, 0.05, 0.26),
        if f.t2 && f.a { GOLD } else { WOOD },
        [0.0, 0.02, 0.0],
    ));
    if f.a && f.t2 {
        p.push(part(
            Head,
            Box(0.04, 0.12, 0.26),
            RED_CLOTH,
            [0.0, 0.21, -0.02],
        ));
        p.push(part(HandL, Box(0.46, 0.6, 0.03), GOLD, [0.06, 0.08, 0.1]));
        p.push(part(
            HandL,
            Box(0.42, 0.56, 0.06),
            STEEL,
            [0.06, 0.08, 0.13],
        ));
    } else {
        let r = if f.a {
            0.29
        } else if f.b {
            0.18
        } else {
            0.23
        };
        p.push(part(HandL, Cyl(r, 0.05), WOOD, [0.06, 0.0, 0.11]).rot([90.0, 0.0, 0.0]));
        p.push(part(HandL, Ball(0.055), STEEL, [0.06, 0.0, 0.15]));
        if f.a {
            p.push(
                part(HandL, Ring(0.022, r, 360.0), GOLD, [0.06, 0.0, 0.13]).rot([90.0, 0.0, 0.0]),
            );
        }
    }
    if f.b {
        let len = if f.t2 { 1.05 } else { 0.85 };
        p.push(part(HandR, Cyl(0.022, len), WOOD, [0.0, 0.0, 0.12]).rot([90.0, 0.0, 0.0]));
        p.push(
            part(
                HandR,
                Cone(0.055, 0.16),
                STEEL,
                [0.0, 0.0, 0.12 + len / 2.0 + 0.08],
            )
            .rot([90.0, 0.0, 0.0]),
        );
    } else {
        p.push(part(HandR, Box(0.06, 0.035, 0.44), STEEL, [0.0, 0.0, 0.25]));
        p.push(part(HandR, Box(0.17, 0.04, 0.04), GOLD, [0.0, 0.0, 0.05]));
    }
    if f.b && f.t2 {
        for (arm, s) in [(ArmL, 1.0), (ArmR, -1.0)] {
            p.push(part(arm, Ball(0.1), STEEL, [s * 0.02, 0.0, 0.0]).scale([1.3, 0.7, 1.3]));
        }
        p.push(part(Torso, Cyl(0.015, 0.8), WOOD, [0.0, 0.38, -0.15]));
        p.push(part(
            Torso,
            Box(0.02, 0.22, 0.28),
            RED_CLOTH,
            [0.0, 0.64, -0.29],
        ));
    }
    FigureSpec { plan, parts: p }
}

pub fn archer(f: Form) -> FigureSpec {
    let plan = BodyPlan::SMALL;
    let mut p = base_body(&plan, &Look::new(SKIN, HOOD_GREEN, TRIM, WOOD));
    p.push(part(Head, Cone(0.2, 0.3), HOOD_GREEN, [0.0, 0.11, -0.03]));
    let bow = if f.a { 0.38 } else { 0.3 };
    let tips = if f.a && f.t2 { GOLD } else { WOOD };
    p.push(part(HandL, Ring(0.018, bow, 180.0), tips, [0.0, 0.0, 0.05]).rot([0.0, 90.0, 90.0]));
    p.push(part(
        HandL,
        Box(0.008, bow * 2.0, 0.008),
        TRIM,
        [0.0, 0.0, 0.05],
    ));
    let quivers: &[f32] = if f.b { &[0.09, -0.09] } else { &[0.08] };
    for x in quivers {
        p.push(part(Torso, Cyl(0.06, 0.32), WOOD, [*x, 0.26, -0.16]).rot([20.0, 0.0, 0.0]));
        p.push(part(Torso, Cone(0.05, 0.08), RED_CLOTH, [*x, 0.46, -0.1]));
    }
    if f.a {
        p.push(
            part(Head, Cone(0.035, 0.22), RED_CLOTH, [0.1, 0.22, -0.08]).rot([-30.0, 0.0, -20.0]),
        );
    }
    if f.b && f.t2 {
        p.push(part(
            Torso,
            Box(0.4, 0.07, 0.28),
            RED_CLOTH,
            [0.0, 0.36, 0.0],
        ));
        p.push(part(
            Torso,
            Box(0.08, 0.22, 0.02),
            RED_CLOTH,
            [0.1, 0.24, -0.14],
        ));
    }
    FigureSpec { plan, parts: p }
}
