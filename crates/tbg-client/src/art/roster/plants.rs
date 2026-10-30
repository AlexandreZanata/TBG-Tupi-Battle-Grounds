//! Plants: leaf greens and bark with blossom and mushroom accents.

pub mod beetling_v2;
pub mod bloomkeeper_v2;
pub mod nester_v2;
pub mod rootcaller_v2;

use super::super::legacy::{part, BodyPlan, FigureSpec, Joint::*};
use super::super::mesh::Shape::*;
use super::super::palette::*;
use super::{base_body, Form, Look};

pub fn bloomkeeper(f: Form) -> FigureSpec {
    let plan = BodyPlan {
        scale: 0.92,
        ..BodyPlan::SMALL
    };
    let mut p = base_body(&plan, &Look::new(LEAF, LEAF_DARK, LEAF, BARK));
    let petal = if f.a { 0.09 } else { 0.07 };
    for i in 0..6 {
        let a = i as f32 / 6.0 * std::f32::consts::TAU;
        p.push(part(
            Head,
            Ball(petal),
            BLOSSOM,
            [a.cos() * 0.17, 0.1 + a.sin() * 0.17, -0.02],
        ));
    }
    p.push(part(Head, Ball(0.07), GOLD, [0.0, 0.1, 0.05]));
    p.push(part(Torso, Box(0.36, 0.42, 0.02), LEAF, [0.0, 0.16, -0.14]).rot([12.0, 0.0, 0.0]));
    p.push(part(HandR, Cyl(0.018, 0.75), BARK, [0.0, 0.15, 0.06]));
    p.push(part(HandR, Ball(0.07), BLOSSOM, [0.0, 0.54, 0.06]));
    if f.a && f.t2 {
        p.push(
            part(Head, Ring(0.03, 0.3, 360.0), BLOSSOM, [0.0, 0.1, -0.02]).rot([90.0, 0.0, 0.0]),
        );
    }
    if f.b {
        for y in [0.08, 0.24] {
            p.push(part(Torso, Box(0.4, 0.1, 0.28), BARK, [0.0, y, 0.0]));
        }
    }
    if f.b && f.t2 {
        for (x, y) in [
            (0.15, 0.3),
            (-0.15, 0.3),
            (0.0, 0.38),
            (0.18, 0.12),
            (-0.18, 0.12),
        ] {
            p.push(part(Torso, Cone(0.03, 0.12), BARK, [x, y, 0.16]).rot([90.0, 0.0, 0.0]));
        }
    }
    FigureSpec { plan, parts: p }
}

pub fn rootcaller(f: Form) -> FigureSpec {
    let plan = BodyPlan::SMALL;
    let mut p = base_body(&plan, &Look::new(BARK, BARK, LEAF_DARK, BARK));
    p.push(part(Head, Cone(0.05, 0.16), LEAF, [0.07, 0.17, 0.0]).rot([0.0, 0.0, -25.0]));
    p.push(part(Head, Cone(0.05, 0.16), LEAF, [-0.07, 0.17, 0.0]).rot([0.0, 0.0, 25.0]));
    let whip = if f.a { 0.7 } else { 0.5 };
    for hand in [HandL, HandR] {
        p.push(
            part(hand, Pill(0.03, whip), LEAF_DARK, [0.0, -0.05, whip / 2.0]).rot([90.0, 0.0, 0.0]),
        );
        p.push(part(hand, Cone(0.04, 0.1), LEAF, [0.0, -0.05, whip + 0.05]).rot([90.0, 0.0, 0.0]));
    }
    if f.a && f.t2 {
        for arm in [ArmL, ArmR] {
            p.push(part(arm, Ring(0.02, 0.09, 360.0), LEAF, [0.0, -0.08, 0.0]));
        }
    }
    if f.b {
        let cap = if f.t2 { 0.3 } else { 0.24 };
        p.push(part(Head, Ball(cap), MUSHROOM, [0.0, 0.14, 0.0]).scale([1.0, 0.45, 1.0]));
        for (x, z) in [(0.1, 0.08), (-0.12, 0.02), (0.02, -0.12)] {
            p.push(part(Head, Ball(0.03), BONE, [x, 0.25, z]));
        }
    }
    if f.b && f.t2 {
        for (x, y) in [(0.2, 0.4), (-0.22, 0.32), (0.05, 0.5)] {
            p.push(part(Torso, Ball(0.04), MUSHROOM, [x, y, 0.1]));
        }
    }
    FigureSpec { plan, parts: p }
}
