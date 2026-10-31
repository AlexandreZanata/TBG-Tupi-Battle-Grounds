//! Acidmaw's long braced neck, luminous throat sac and four independently opening jaw petals.

use crate::art::{
    figure::{part, FigureSpec, PartSpec},
    kits::common::{body, Form},
    mesh::Shape::*,
    palette::*,
    rig::{
        self,
        Joint::*,
        Node,
        Node::{Body, Prop, Secondary},
        Proportions,
        Socket::*,
        Spring,
    },
};
use bevy::prelude::*;

pub fn plan(tier: u8) -> Proportions {
    let mut p = body::plan(false, 1.0, tier);
    p.torso.x = 0.25;
    p.leg = 0.48;
    p.arm = 0.12;
    p
}
pub fn parent(n: Node) -> Option<Node> {
    match n {
        Prop(WeaponR | OffhandL | Back | StringTop) => Some(Body(Head)),
        _ => n.parent(),
    }
}
pub fn rest(n: Node, p: &Proportions) -> Vec3 {
    match n {
        Body(Head) => Vec3::new(0.0, 0.70, 0.08),
        Secondary(Spring::Back) => Vec3::new(0.0, 0.42, 0.09),
        Prop(WeaponR | OffhandL | Back | StringTop) => Vec3::ZERO,
        _ => rig::rest(n, p),
    }
}

pub fn spec(tier: u8, branch: u8) -> FigureSpec {
    let f = Form::of(tier, branch);
    let p = plan(tier);
    let mut v = vec![
        part(Body(Hips), Ball(0.14), CHARCOAL, [0.0, 0.03, -0.02]).scale([1.0, 0.7, 1.2]),
        part(
            Body(Chest),
            Ball(0.19),
            if f.b { DARK } else { CHARCOAL },
            [0.0, 0.18, -0.03],
        )
        .scale([0.9, 1.15, 0.8]),
        part(Body(Chest), Pill(0.07, 0.46), CHARCOAL, [0.0, 0.44, 0.03]).rot([8.0, 0.0, 0.0]),
        part(
            Secondary(Spring::Back),
            Ball(if f.b { 0.17 } else { 0.13 }),
            TURQUOISE,
            [0.0, 0.0, 0.06],
        )
        .scale([1.0, 1.4, 0.9])
        .glow(),
        part(Body(Head), Ball(0.14), CHARCOAL, [0.0, 0.0, 0.02]).scale([1.2, 0.8, 1.1]),
        part(Body(Head), Ball(0.10), DARK, [0.0, 0.0, 0.17]).scale([1.0, 1.0, 0.4]),
    ];
    for (upper, lower, foot) in [(ThighL, ShinL, FootL), (ThighR, ShinR, FootR)] {
        v.extend([
            part(Body(upper), Pill(0.05, 0.17), CHARCOAL, [0.0, -0.11, 0.0]),
            part(Body(lower), Pill(0.037, 0.17), GUNMETAL, [0.0, -0.11, 0.0]),
            part(
                Body(foot),
                Box(0.09, 0.07, 0.19),
                CHARCOAL,
                [0.0, -0.02, 0.05],
            ),
        ]);
        for x in [-0.03, 0.03] {
            v.push(
                part(Body(foot), Cone(0.025, 0.09), IVORY, [x, -0.02, 0.17]).rot([90.0, 0.0, 0.0]),
            );
        }
    }
    for (upper, lower, hand, s) in [
        (UpperArmL, ForearmL, HandL, 1.0),
        (UpperArmR, ForearmR, HandR, -1.0),
    ] {
        v.extend([
            part(Body(upper), Pill(0.04, 0.08), CHARCOAL, [0.0, -0.06, 0.0]),
            part(Body(lower), Cone(0.05, 0.15), IVORY, [0.0, -0.06, 0.02]).rot([160.0, 0.0, 0.0]),
            part(Body(hand), Cone(0.035, 0.12), IVORY, [0.0, -0.02, 0.05]).rot([65.0, 0.0, 0.0]),
            part(
                Body(Chest),
                Box(0.025, 0.24, 0.16),
                ALLY,
                [s * 0.18, 0.27, -0.02],
            )
            .rot([0.0, 0.0, s * -12.0]),
            part(Body(Head), Ball(0.03), TURQUOISE, [s * 0.12, 0.045, 0.10]).glow(),
        ]);
    }
    jaws(&mut v);
    forms(&mut v, f);
    FigureSpec { plan: p, parts: v }
}

fn jaws(v: &mut Vec<PartSpec>) {
    for (socket, axis) in [
        (WeaponR, [1.0, 0.0]),
        (OffhandL, [-1.0, 0.0]),
        (Back, [0.0, 1.0]),
        (StringTop, [0.0, -1.0]),
    ] {
        let [x, y] = axis;
        v.extend([
            part(
                Prop(socket),
                Cone(0.065, 0.24),
                IVORY,
                [x * 0.095, y * 0.095, 0.17],
            )
            .rot([90.0, 0.0, 0.0]),
            part(
                Prop(socket),
                Cone(0.028, 0.12),
                IVORY,
                [x * 0.07, y * 0.07, 0.32],
            )
            .rot([90.0 + y * 22.0, x * -22.0, 0.0]),
        ]);
    }
}

fn forms(v: &mut Vec<PartSpec>, f: Form) {
    if f.a {
        for s in [-1.0, 1.0] {
            v.push(
                part(Body(Chest), Pill(0.035, 0.43), IVORY, [s * 0.09, 0.44, 0.0]).rot([
                    8.0,
                    0.0,
                    s * -5.0,
                ]),
            );
        }
        if f.t2 {
            v.push(
                part(Body(Head), Cone(0.06, 0.39), IVORY, [0.0, 0.14, -0.02]).rot([35.0, 0.0, 0.0]),
            );
        }
    }
    if f.b {
        for s in [-1.0, 1.0] {
            v.push(
                part(
                    Secondary(Spring::Back),
                    Ball(0.16),
                    IVORY,
                    [s * 0.12, -0.02, -0.01],
                )
                .scale([0.4, 1.2, 0.75]),
            );
        }
        if f.t2 {
            for a in [0.0_f32, 120.0, 240.0] {
                let r = a.to_radians();
                v.push(
                    part(
                        Secondary(Spring::Back),
                        Cone(0.07, 0.27),
                        IVORY,
                        [r.cos() * 0.16, -0.07, 0.03 + r.sin() * 0.11],
                    )
                    .rot([0.0, 0.0, r.cos() * -35.0]),
                );
            }
        }
    }
}
