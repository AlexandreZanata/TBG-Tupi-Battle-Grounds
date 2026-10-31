//! Ravager's six articulated supports, anterior scythes and five predator forms.

use crate::art::{
    figure::{part, FigureSpec, PartSpec},
    kits::common::{body, Form},
    mesh::Shape::*,
    palette::*,
    rig::{
        self,
        Joint::*,
        Node,
        Node::{Body, Prop},
        Proportions,
        Socket::*,
    },
};
use bevy::prelude::*;

pub fn plan(tier: u8) -> Proportions {
    let mut p = body::plan(false, 1.0, tier);
    p.leg = 0.45;
    p
}

pub fn parent(node: Node) -> Option<Node> {
    match node {
        Prop(WeaponR | OffhandL) => Some(Body(Hips)),
        Prop(Back) => Some(Prop(WeaponR)),
        Prop(StringTop) => Some(Prop(OffhandL)),
        Prop(StringBottom) => Some(Prop(Back)),
        Prop(Nock) => Some(Prop(StringTop)),
        _ => node.parent(),
    }
}

pub fn rest(node: Node, p: &Proportions) -> Vec3 {
    let v = match node {
        Body(Hips) => [0.0, 0.51, -0.15],
        Body(Spine) => [0.0, 0.0, 0.08],
        Body(Chest) => [0.0, 0.0, 0.27],
        Body(Head) => [0.0, -0.06, 0.30],
        Body(UpperArmL) => [0.24, 0.0, 0.15],
        Body(UpperArmR) => [-0.24, 0.0, 0.15],
        Body(ForearmL) => [0.18, -0.20, 0.08],
        Body(ForearmR) => [-0.18, -0.20, 0.08],
        Body(HandL) => [0.08, -0.27, 0.09],
        Body(HandR) => [-0.08, -0.27, 0.09],
        Body(ThighL) => [0.21, 0.0, -0.02],
        Body(ThighR) => [-0.21, 0.0, -0.02],
        Body(ShinL) => [0.17, -0.20, 0.0],
        Body(ShinR) => [-0.17, -0.20, 0.0],
        Body(FootL) => [0.09, -0.28, 0.02],
        Body(FootR) => [-0.09, -0.28, 0.02],
        Prop(WeaponR) => [0.20, 0.0, -0.27],
        Prop(OffhandL) => [-0.20, 0.0, -0.27],
        Prop(Back) => [0.17, -0.20, -0.08],
        Prop(StringTop) => [-0.17, -0.20, -0.08],
        Prop(StringBottom) => [0.09, -0.28, -0.06],
        Prop(Nock) => [-0.09, -0.28, -0.06],
        _ => return rig::rest(node, p),
    };
    Vec3::from(v)
}

pub fn spec(tier: u8, branch: u8) -> FigureSpec {
    let f = Form::of(tier, branch);
    let mut v = vec![
        part(Body(Hips), Ball(0.26), CHARCOAL, [0.0, 0.015, -0.10]).scale([0.85, 0.75, 1.6]),
        part(
            Body(Chest),
            Ball(0.25),
            if f.b { DARK } else { CHARCOAL },
            [0.0, 0.0, 0.02],
        )
        .scale([1.0, 0.8, 1.5]),
        part(Body(Spine), Box(0.27, 0.04, 0.40), ALLY, [0.0, 0.23, -0.05]),
        part(
            Body(Spine),
            Box(0.29, 0.05, 0.06),
            LINEN,
            [0.0, 0.23, -0.23],
        ),
        part(
            Body(Hips),
            Cone(0.12, if f.a && f.t2 { 0.52 } else { 0.28 }),
            IVORY,
            [0.0, 0.15, -0.40],
        )
        .rot([-110.0, 0.0, 0.0]),
    ];
    let tripods = [
        (Body(UpperArmL), Body(ForearmL), Body(HandL), 1.0, true),
        (Body(UpperArmR), Body(ForearmR), Body(HandR), -1.0, true),
        (Body(ThighL), Body(ShinL), Body(FootL), 1.0, false),
        (Body(ThighR), Body(ShinR), Body(FootR), -1.0, false),
        (Prop(WeaponR), Prop(Back), Prop(StringBottom), 1.0, false),
        (Prop(OffhandL), Prop(StringTop), Prop(Nock), -1.0, false),
    ];
    for (upper, lower, foot, s, front) in tripods {
        limb(&mut v, upper, lower, foot, s, front, f);
    }
    skull(&mut v, f);
    for z in [-0.32, -0.12, 0.08] {
        for s in [-1.0, 1.0] {
            v.push(
                part(Body(Spine), Cone(0.055, 0.16), IVORY, [s * 0.17, 0.22, z]).rot([
                    -30.0,
                    0.0,
                    -35.0 * s,
                ]),
            );
        }
    }
    FigureSpec {
        plan: plan(tier),
        parts: v,
    }
}

fn limb(v: &mut Vec<PartSpec>, upper: Node, lower: Node, foot: Node, s: f32, front: bool, f: Form) {
    v.extend([
        part(upper, Pill(0.07, 0.17), CHARCOAL, [s * 0.09, -0.10, 0.02]).rot([0.0, 0.0, s * 42.0]),
        part(lower, Pill(0.055, 0.19), CHARCOAL, [s * 0.045, -0.13, 0.01]).rot([
            0.0,
            0.0,
            s * 17.0,
        ]),
        part(foot, Cone(0.07, 0.19), IVORY, [0.0, 0.04, 0.06]).rot([60.0, 0.0, 0.0]),
        part(
            upper,
            Ball(0.09),
            if f.a { IVORY } else { GUNMETAL },
            [0.0, 0.0, 0.0],
        )
        .scale([1.2, 0.65, 1.1]),
    ]);
    if f.a {
        v.push(
            part(lower, Cone(0.07, 0.29), IVORY, [s * 0.025, -0.10, -0.04]).rot([
                -20.0,
                0.0,
                s * 14.0,
            ]),
        );
    }
    if front {
        let w = if f.b { 0.105 } else { 0.075 };
        v.extend([
            part(lower, Cone(w, 0.38), IVORY, [0.0, -0.06, 0.17]).rot([-55.0, 0.0, s * -20.0]),
            part(lower, Cone(w * 0.7, 0.26), IVORY, [s * -0.07, -0.02, 0.37]).rot([
                -105.0,
                0.0,
                s * -35.0,
            ]),
            part(
                lower,
                Box(0.025, 0.05, 0.20),
                TURQUOISE,
                [s * -0.045, -0.02, 0.19],
            )
            .rot([-45.0, 0.0, 0.0])
            .glow(),
        ]);
    }
}

fn skull(v: &mut Vec<PartSpec>, f: Form) {
    let h = Body(Head);
    v.extend([
        part(h, Ball(0.19), DARK, [0.0, 0.0, 0.04]).scale([1.15, 0.65, 1.4]),
        part(h, Box(0.22, 0.045, 0.18), IVORY, [0.0, 0.09, 0.14]).rot([-15.0, 0.0, 0.0]),
    ]);
    for s in [-1.0, 1.0] {
        v.extend([
            part(h, Ball(0.047), TURQUOISE, [s * 0.14, 0.035, 0.19])
                .scale([1.0, 0.45, 0.8])
                .glow(),
            part(h, Cone(0.07, 0.28), IVORY, [s * 0.13, -0.06, 0.22]).rot([80.0, 0.0, s * -40.0]),
        ]);
        if f.b && f.t2 {
            v.push(
                part(h, Box(0.065, 0.13, 0.30), IVORY, [s * 0.20, -0.08, 0.11]).rot([
                    0.0,
                    s * -25.0,
                    0.0,
                ]),
            );
            for z in [0.02, 0.14, 0.26] {
                v.push(
                    part(h, Cone(0.038, 0.11), IVORY, [s * 0.20, -0.01, z]).rot([
                        0.0,
                        0.0,
                        s * -15.0,
                    ]),
                );
            }
        }
    }
}
