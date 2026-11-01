//! Bonehorn's low quadruped silhouette, overlapping bone mantle and five forms.

use crate::art::{
    figure::{part, FigureSpec, PartSpec},
    kits::common::{body, Form},
    mesh::Shape::*,
    palette::*,
    rig::{self, Joint::*, Node, Node::Body, Proportions, Spring},
};
use bevy::prelude::*;

pub fn plan(tier: u8) -> Proportions {
    let mut p = body::plan(true, 0.82, tier);
    p.leg = 0.64;
    p.arm = 0.34;
    p
}

pub fn rest(node: Node, p: &Proportions) -> Vec3 {
    let v = match node {
        Body(Hips) => [0.0, 0.70, -0.34],
        Body(Spine) => [0.0, 0.02, 0.18],
        Body(Chest) => [0.0, 0.02, 0.38],
        Body(Head) => [0.0, 0.02, 0.47],
        Body(UpperArmL) => [0.29, -0.02, 0.03],
        Body(UpperArmR) => [-0.29, -0.02, 0.03],
        Body(ForearmL | ForearmR) => [0.0, -0.32, 0.0],
        Body(HandL | HandR) => [0.0, -0.32, 0.0],
        Body(ThighL) => [0.25, -0.02, -0.06],
        Body(ThighR) => [-0.25, -0.02, -0.06],
        Body(ShinL | ShinR) => [0.0, -0.30, 0.0],
        Body(FootL | FootR) => [0.0, -0.32, 0.0],
        Node::Secondary(Spring::Back) => [0.0, 0.0, 0.0],
        _ => return rig::rest(node, p),
    };
    Vec3::from(v)
}

pub fn spec(tier: u8, branch: u8) -> FigureSpec {
    let f = Form::of(tier, branch);
    let mut v = vec![
        part(Body(Hips), Ball(0.35), CHARCOAL, [0.0, 0.0, 0.0]).scale([1.0, 0.9, 1.15]),
        part(Body(Spine), Ball(0.39), CHARCOAL, [0.0, 0.0, 0.06]).scale([1.05, 0.95, 1.4]),
        part(
            Body(Chest),
            Ball(0.40),
            if f.b { DARK } else { CHARCOAL },
            [0.0, 0.01, 0.04],
        )
        .scale([1.12, 1.05, 1.1]),
        part(Body(Hips), Cone(0.09, 0.31), DARK, [0.0, -0.03, -0.42]).rot([-125.0, 0.0, 0.0]),
    ];
    legs(&mut v, f);
    skull(&mut v, f);
    mantle(&mut v, f);
    FigureSpec {
        plan: plan(tier),
        parts: v,
    }
}

fn legs(v: &mut Vec<PartSpec>, f: Form) {
    for (upper, lower, foot) in [
        (UpperArmL, ForearmL, HandL),
        (UpperArmR, ForearmR, HandR),
        (ThighL, ShinL, FootL),
        (ThighR, ShinR, FootR),
    ] {
        v.extend([
            part(Body(upper), Pill(0.14, 0.15), CHARCOAL, [0.0, -0.14, 0.0]),
            part(
                Body(lower),
                Frustum(0.12, 0.09, 0.29),
                DARK,
                [0.0, -0.14, 0.0],
            ),
            part(Body(lower), Ball(0.13), IVORY, [0.0, 0.01, 0.04]).scale([1.0, 0.65, 1.0]),
            part(
                Body(foot),
                Box(0.23, 0.13, 0.29),
                GUNMETAL,
                [0.0, 0.03, 0.06],
            ),
        ]);
        for x in [-0.07, 0.07] {
            v.push(
                part(Body(foot), Cone(0.045, 0.12), IVORY, [x, 0.03, 0.23]).rot([90.0, 0.0, 0.0]),
            );
        }
        if f.a {
            v.push(
                part(Body(upper), Ball(0.19), IVORY, [0.0, -0.05, 0.04]).scale([1.05, 0.7, 0.9]),
            );
        }
    }
}

fn skull(v: &mut Vec<PartSpec>, f: Form) {
    let h = Body(Head);
    v.extend([
        part(h, Ball(0.29), CHARCOAL, [0.0, 0.0, 0.04]).scale([1.05, 0.85, 1.3]),
        part(h, Box(0.40, 0.25, 0.47), CHARCOAL, [0.0, -0.10, 0.26]),
        part(h, Ball(0.22), DARK, [0.0, -0.08, 0.46]).scale([1.1, 0.65, 0.55]),
        part(h, Box(0.44, 0.09, 0.22), IVORY, [0.0, 0.12, 0.14]).rot([-12.0, 0.0, 0.0]),
        part(
            h,
            Cone(0.11, if f.a && f.t2 { 0.46 } else { 0.31 }),
            IVORY,
            [0.0, 0.18, 0.35],
        )
        .rot([32.0, 0.0, 0.0]),
    ]);
    for s in [-1.0, 1.0] {
        v.extend([
            part(
                h,
                Box(0.06, 0.035, 0.19),
                TURQUOISE,
                [s * 0.255, 0.015, 0.14],
            )
            .rot([0.0, s * -15.0, 0.0])
            .glow(),
            part(
                h,
                Cone(0.10, if f.b { 0.48 } else { 0.30 }),
                IVORY,
                [s * 0.28, -0.01, 0.40],
            )
            .rot([45.0, 0.0, s * -25.0]),
            part(h, Cone(0.085, 0.19), DARK, [s * 0.23, 0.16, -0.13]).rot([-22.0, 0.0, s * -40.0]),
            part(h, Ball(0.035), GUNMETAL, [s * 0.10, -0.03, 0.56]),
        ]);
        if f.b && f.t2 {
            v.push(
                part(h, Box(0.09, 0.18, 0.45), IVORY, [s * 0.27, -0.19, 0.25]).rot([
                    0.0,
                    s * -10.0,
                    s * -12.0,
                ]),
            );
        }
    }
}

fn mantle(v: &mut Vec<PartSpec>, f: Form) {
    let back = Node::Secondary(Spring::Back);
    for (i, z) in [-0.67, -0.42, -0.17, 0.08].into_iter().enumerate() {
        let width = 0.25 + i as f32 * 0.035 + if f.a { 0.06 } else { 0.0 };
        v.push(
            part(back, Ball(width), IVORY, [0.0, 0.35, z])
                .scale([1.4, 0.40, 0.8])
                .rot([-12.0, 0.0, 0.0]),
        );
        v.push(
            part(
                back,
                Cone(0.09, if f.a && f.t2 { 0.31 } else { 0.17 }),
                IVORY,
                [0.0, 0.46, z],
            )
            .rot([-20.0, 0.0, 0.0]),
        );
        if f.a && f.t2 {
            for s in [-1.0, 1.0] {
                v.push(
                    part(back, Cone(0.07, 0.21), IVORY, [s * width, 0.35, z]).rot([
                        -20.0,
                        0.0,
                        s * -55.0,
                    ]),
                );
            }
        }
    }
    for s in [-1.0, 1.0] {
        v.extend([
            part(
                Body(Spine),
                Box(0.035, 0.32, 0.46),
                ALLY,
                [s * 0.38, -0.02, -0.04],
            )
            .rot([0.0, 0.0, s * -10.0]),
            part(
                Body(Spine),
                Box(0.045, 0.04, 0.47),
                LINEN,
                [s * 0.40, -0.16, -0.04],
            ),
            part(Body(Spine), Ball(0.055), BRASS, [s * 0.405, 0.07, 0.05]).scale([0.35, 1.0, 1.0]),
        ]);
        if f.a {
            for z in [0.12, 0.31] {
                v.push(
                    part(Body(Chest), Ball(0.23), IVORY, [s * 0.26, 0.22, z])
                        .scale([1.0, 0.45, 0.8])
                        .rot([0.0, 0.0, s * -30.0]),
                );
            }
        }
    }
}
