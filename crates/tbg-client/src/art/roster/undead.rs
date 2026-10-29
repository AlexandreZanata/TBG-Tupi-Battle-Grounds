//! Undead: bone and dark violet with a spectral green glow.

use super::super::legacy::{part, BodyPlan, FigureSpec, Joint::*};
use super::super::mesh::Shape::*;
use super::super::palette::*;
use super::{base_body, Form, Look};

pub fn bone_brute(f: Form) -> FigureSpec {
    let plan = BodyPlan {
        head: 0.13,
        ..BodyPlan::BIG
    };
    let look = Look {
        eyes: GLOW,
        ..Look::new(BONE, VIOLET, BONE, DARK)
    };
    let mut p = base_body(&plan, &look);
    // Ribs over the torso, a jaw, hunched shoulders.
    for i in 0..3 {
        p.push(part(
            Torso,
            Box(0.4, 0.04, 0.03),
            BONE,
            [0.0, 0.12 + 0.08 * i as f32, 0.16],
        ));
    }
    p.push(part(Head, Box(0.16, 0.05, 0.12), BONE, [0.0, -0.11, 0.05]));
    p.push(part(Torso, Ball(0.15), VIOLET, [0.0, 0.42, -0.06]).scale([1.6, 0.8, 1.0]));
    if f.b {
        if f.t2 {
            p.push(part(HandR, Cyl(0.025, 1.15), WOOD, [0.0, 0.25, 0.0]));
            p.push(
                part(HandR, Box(0.04, 0.09, 0.55), IRON, [0.0, 0.8, 0.22]).rot([-15.0, 0.0, 0.0]),
            );
        } else {
            p.push(part(HandR, Box(0.06, 0.32, 0.42), IRON, [0.0, 0.0, 0.25]));
        }
    } else {
        p.push(
            part(HandR, Frustum(0.13, 0.05, 0.65), WOOD, [0.0, 0.0, 0.33]).rot([90.0, 0.0, 0.0]),
        );
        p.push(part(HandR, Ball(0.08), BONE, [0.07, 0.0, 0.58]));
    }
    if f.a {
        p.push(part(Torso, Box(0.5, 0.18, 0.08), BONE, [0.0, 0.22, 0.19]));
        for (arm, s) in [(ArmL, 1.0), (ArmR, -1.0)] {
            p.push(part(arm, Box(0.18, 0.08, 0.2), BONE, [s * 0.02, 0.02, 0.0]));
        }
    }
    if f.a && f.t2 {
        for i in 0..4 {
            p.push(part(
                HandL,
                Box(0.04, 0.5, 0.04),
                BONE,
                [-0.15 + 0.1 * i as f32, 0.05, 0.12],
            ));
        }
        p.push(part(HandL, Box(0.42, 0.06, 0.05), BONE, [0.0, 0.25, 0.12]));
    }
    FigureSpec { plan, parts: p }
}

pub fn hexer(f: Form) -> FigureSpec {
    let plan = BodyPlan::SMALL;
    let look = Look {
        eyes: GLOW,
        ..Look::new(BONE, VIOLET, VIOLET, DARK)
    };
    let mut p = base_body(&plan, &look);
    p.push(part(Hips, Cone(0.3, 0.5), VIOLET, [0.0, -0.18, 0.0]).rot([180.0, 0.0, 0.0]));
    p.push(part(Head, Cone(0.21, 0.34), VIOLET, [0.0, 0.11, -0.04]));
    p.push(part(HandR, Cyl(0.022, 1.0), WOOD, [0.0, 0.2, 0.06]));
    let orb = if f.a && f.t2 { 0.12 } else { 0.085 };
    let top = if f.a { BONE } else { GLOW };
    p.push(part(HandR, Ball(orb), top, [0.0, 0.74, 0.06]));
    if f.a {
        p.push(part(HandR, Ball(0.05), GLOW, [0.05, 0.74, 0.14]));
        p.push(part(HandR, Ball(0.05), GLOW, [-0.05, 0.74, 0.14]));
    }
    if f.a && f.t2 {
        p.push(
            part(HandR, Ring(0.015, 0.18, 360.0), GLOW, [0.0, 0.74, 0.06]).rot([90.0, 0.0, 0.0]),
        );
    }
    if f.b {
        p.push(part(HandL, Ball(0.09), IRON, [0.0, -0.12, 0.05]));
        p.push(part(HandL, Cyl(0.006, 0.12), IRON, [0.0, -0.02, 0.05]));
        let spores: &[[f32; 3]] = if f.t2 {
            &[
                [0.1, -0.05, 0.1],
                [-0.12, 0.02, 0.06],
                [0.04, 0.1, 0.14],
                [-0.04, -0.2, 0.14],
                [0.14, 0.12, 0.0],
            ]
        } else {
            &[[0.1, -0.05, 0.1], [-0.1, 0.02, 0.08]]
        };
        for s in spores {
            p.push(part(HandL, Ball(0.04), GLOW, *s));
        }
    }
    FigureSpec { plan, parts: p }
}
