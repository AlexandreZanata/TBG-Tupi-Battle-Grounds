//! A low terrestrial turtle: beak, four thick feet, soft inner shell and four detachable plates.
use crate::art::{
    figure::{part, FigureSpec, PartSpec},
    kits::common::{body, Form},
    mesh::Shape::*,
    palette::*,
    rig::{Joint::*, Node::Body, Proportions},
};
use bevy::prelude::*;
const HIDE: Color = Color::srgb(0.42, 0.49, 0.25);
const SLATE: Color = Color::srgb(0.46, 0.49, 0.37);
pub fn plan(tier: u8) -> Proportions {
    let mut p = body::plan(true, 0.88, tier);
    p.leg *= 0.60;
    p.arm *= 0.75;
    p.torso.x *= 1.35;
    p.torso.z *= 1.9;
    p.head *= 0.85;
    p
}
pub fn tip() -> Vec3 {
    Vec3::new(0.0, 0.0, 0.24)
}
pub fn spec(tier: u8, branch: u8) -> FigureSpec {
    let p = plan(tier);
    let f = Form::of(tier, branch);
    let mut v = Vec::new();
    v.push(part(Body(Chest), Ball(0.38), BARK, [0.0, 0.09, -0.1]).scale([1.35, 0.65, 1.65]));
    v.push(
        part(
            Body(Chest),
            Ring(0.025, 0.41, 360.0),
            ALLY,
            [0.0, 0.03, -0.1],
        )
        .rot([90.0, 0.0, 0.0])
        .scale([1.2, 1.4, 1.0]),
    );
    for joint in [ThighL, ThighR, UpperArmL, UpperArmR] {
        v.push(part(Body(joint), Ball(0.12), HIDE, [0.0, -0.08, 0.0]).scale([1.0, 1.45, 1.0]));
    }
    for joint in [FootL, FootR, HandL, HandR] {
        v.push(part(
            Body(joint),
            Box(0.18, 0.12, 0.24),
            HIDE,
            [0.0, -0.03, 0.06],
        ));
        for x in [-0.055, 0.0, 0.055] {
            v.push(
                part(Body(joint), Cone(0.022, 0.08), BONE, [x, -0.05, 0.2]).rot([90.0, 0.0, 0.0]),
            );
        }
    }
    v.push(part(Body(Head), Ball(p.head * 1.1), HIDE, [0.0, 0.0, 0.08]).scale([1.0, 0.75, 1.25]));
    v.push(part(Body(Head), Cone(0.11, 0.18), BONE, [0.0, -0.03, 0.24]).rot([90.0, 0.0, 0.0]));
    body::glowing_eyes(&mut v, &p, if f.b { ORANGE } else { SAP });
    v.push(part(Body(Hips), Cone(0.07, 0.25), HIDE, [0.0, -0.03, -0.35]).rot([-110.0, 0.0, 0.0]));
    if f.a {
        v.push(
            part(
                Body(Chest),
                Ring(0.02, 0.2, 360.0),
                BRASS,
                [0.0, 0.15, 0.25],
            )
            .rot([65.0, 0.0, 0.0]),
        );
    }
    if f.b {
        v.push(part(
            Body(Chest),
            Box(0.05, 0.08, 0.18),
            ORANGE,
            [0.0, 0.15, 0.27],
        ));
    }
    if f.t2 {
        v.push(part(
            Body(Head),
            Box(0.1, 0.05, 0.12),
            if f.a { BRASS } else { IRON },
            [0.0, p.head * 0.6, 0.04],
        ));
    }
    FigureSpec { plan: p, parts: v }
}
/// Each plate has its own shared mesh and entity, rather than being merged into the shell.
pub fn plate(tier: u8, branch: u8, index: usize) -> FigureSpec {
    let f = Form::of(tier, branch);
    let x = if index.is_multiple_of(2) { -0.22 } else { 0.22 };
    let z = if index < 2 { 0.14 } else { -0.27 };
    let mut v: Vec<PartSpec> = vec![
        part(Body(Chest), Ball(0.25), SLATE, [x, 0.26, z]).scale([1.0, 0.6, 1.25]),
        part(
            Body(Chest),
            Ball(0.21),
            if f.b { LEAF_DARK } else { LEAF },
            [x, 0.35, z],
        )
        .scale([1.0, 0.22, 1.2]),
    ];
    if f.a {
        for offset in [-0.08, 0.08] {
            v.push(part(
                Body(Chest),
                Cone(0.04, if f.t2 { 0.23 } else { 0.15 }),
                BONE,
                [x + offset, 0.46, z],
            ));
        }
    }
    if f.b {
        v.push(
            part(Body(Chest), Box(0.024, 0.018, 0.32), ORANGE, [x, 0.40, z]).rot([0.0, 25.0, 0.0]),
        );
    }
    FigureSpec {
        plan: plan(tier),
        parts: v,
    }
}
