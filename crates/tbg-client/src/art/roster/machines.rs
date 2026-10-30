//! Machines: gunmetal and iron with brass and hot-orange accents.

pub mod rampart_v2;
pub mod sawbot_v2;
pub mod siege_walker_v2;

use bevy::prelude::Vec3;

use super::super::legacy::{part, BodyPlan, FigureSpec, Joint::*};
use super::super::mesh::Shape::*;
use super::super::palette::*;
use super::{base_body, Form, Look};

pub fn sawbot(f: Form) -> FigureSpec {
    let plan = BodyPlan {
        leg: 0.22,
        torso: Vec3::new(0.46, 0.34, 0.32),
        head: 0.15,
        ..BodyPlan::SMALL
    };
    let look = Look {
        eyes: ORANGE,
        ..Look::new(GUNMETAL, GUNMETAL, IRON, IRON)
    };
    let mut p = base_body(&plan, &look);
    p.push(part(Head, Box(0.26, 0.2, 0.24), GUNMETAL, [0.0, 0.0, 0.0]));
    p.push(part(Head, Box(0.22, 0.06, 0.03), ORANGE, [0.0, 0.02, 0.12]));
    p.push(part(Torso, Ball(0.07), BRASS, [0.0, 0.2, 0.17]));
    let saw = |p: &mut Vec<_>, hand, r: f32| {
        p.push(part(hand, Cyl(r, 0.03), IRON, [0.0, -0.04, 0.16]).rot([0.0, 0.0, 90.0]));
        p.push(part(hand, Ring(0.02, r, 360.0), ORANGE, [0.0, -0.04, 0.16]).rot([0.0, 0.0, 90.0]));
    };
    let r = if f.b && f.t2 { 0.27 } else { 0.21 };
    saw(&mut p, HandR, r);
    if f.a {
        saw(&mut p, HandL, r);
    }
    if f.a && f.t2 {
        for x in [-0.1, 0.1] {
            p.push(part(Torso, Cyl(0.04, 0.3), IRON, [x, 0.42, -0.17]));
            p.push(part(Torso, Cyl(0.045, 0.05), ORANGE, [x, 0.58, -0.17]));
        }
    }
    if f.b {
        p.push(part(Torso, Box(0.5, 0.26, 0.06), IRON, [0.0, 0.17, 0.18]));
        p.push(part(Torso, Box(0.5, 0.26, 0.06), IRON, [0.0, 0.17, -0.18]));
    }
    if f.b && f.t2 {
        p.push(part(Head, Box(0.3, 0.06, 0.28), IRON, [0.0, 0.12, 0.0]));
    }
    FigureSpec { plan, parts: p }
}

pub fn siege_walker(f: Form) -> FigureSpec {
    let plan = BodyPlan {
        head: 0.12,
        leg: 0.34,
        ..BodyPlan::BIG
    };
    let look = Look {
        eyes: ORANGE,
        ..Look::new(GUNMETAL, GUNMETAL, IRON, IRON)
    };
    let mut p = base_body(&plan, &look);
    p.push(part(Head, Box(0.22, 0.14, 0.2), BRASS, [0.0, 0.0, 0.0]));
    p.push(part(Head, Box(0.18, 0.05, 0.03), ORANGE, [0.0, 0.02, 0.1]));
    let len = if f.a { 1.25 } else { 0.95 };
    let barrels: &[f32] = if f.b { &[0.13, 0.23, 0.33] } else { &[0.23] };
    for x in barrels {
        let r = if f.b { 0.055 } else { 0.09 };
        p.push(part(Torso, Cyl(r, len), IRON, [*x, 0.42, len / 2.0 - 0.15]).rot([90.0, 0.0, 0.0]));
    }
    let muzzle = if f.a && f.t2 { 0.15 } else { 0.1 };
    p.push(
        part(
            Torso,
            Ring(0.025, muzzle, 360.0),
            BRASS,
            [0.23, 0.42, len - 0.15],
        )
        .rot([90.0, 0.0, 0.0]),
    );
    p.push(part(
        Torso,
        Box(0.2, 0.18, 0.3),
        GUNMETAL,
        [0.23, 0.42, -0.1],
    ));
    if f.b && f.t2 {
        p.push(part(Torso, Cyl(0.13, 0.18), BRASS, [0.23, 0.42, -0.3]).rot([90.0, 0.0, 0.0]));
    }
    FigureSpec { plan, parts: p }
}
