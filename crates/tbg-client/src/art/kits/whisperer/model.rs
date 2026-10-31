//! Whisperer's floating bone bell, single eye, short tendrils and branch crests.

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
const BELL: &[[f32; 2]] = &[
    [0.12, -0.08],
    [0.29, 0.0],
    [0.31, 0.16],
    [0.23, 0.32],
    [0.08, 0.40],
    [0.0, 0.42],
];
pub fn plan(tier: u8) -> Proportions {
    body::plan(false, 1.0, tier)
}
pub fn parent(n: Node) -> Option<Node> {
    match n {
        Body(ThighL | ThighR) | Prop(WeaponR | OffhandL) => Some(Body(Chest)),
        Prop(Back) => Some(Body(Head)),
        _ => n.parent(),
    }
}
pub fn rest(n: Node, p: &Proportions) -> Vec3 {
    let v = match n {
        Body(Hips) => [0.0, 0.92, 0.0],
        Body(Head) => [0.0, 0.18, 0.23],
        Body(UpperArmL) => [0.16, 0.01, 0.15],
        Body(UpperArmR) => [-0.16, 0.01, 0.15],
        Body(ThighL) => [0.16, 0.01, -0.15],
        Body(ThighR) => [-0.16, 0.01, -0.15],
        Body(ForearmL | ForearmR | ShinL | ShinR) => [0.0, -0.17, 0.0],
        Body(HandL | HandR | FootL | FootR) => [0.0, -0.18, 0.0],
        Prop(WeaponR) => [0.29, 0.04, 0.0],
        Prop(OffhandL) => [-0.29, 0.04, 0.0],
        Prop(Back) => [0.0, 0.0, 0.0],
        _ => return rig::rest(n, p),
    };
    Vec3::from(v)
}
pub fn spec(tier: u8, branch: u8) -> FigureSpec {
    let f = Form::of(tier, branch);
    let mut v = vec![
        part(
            Body(Chest),
            Lathe(BELL),
            if f.a { IVORY } else { CHARCOAL },
            [0.0, 0.02, 0.0],
        ),
        part(
            Body(Chest),
            Ring(0.025, 0.28, 360.0),
            IVORY,
            [0.0, 0.045, 0.0],
        ),
        part(Body(Head), Ball(0.15), DARK, [0.0, 0.0, 0.005]).scale([1.25, 0.95, 0.55]),
        part(
            Body(Head),
            Ring(0.035, if f.b && f.t2 { 0.17 } else { 0.13 }, 360.0),
            IVORY,
            [0.0, 0.0, 0.06],
        )
        .rot([90.0, 0.0, 0.0]),
        part(Prop(Back), Ball(0.10), TURQUOISE, [0.0, 0.0, 0.065])
            .scale([1.0, 0.85, 0.65])
            .glow(),
        part(Prop(Back), Box(0.025, 0.11, 0.025), DARK, [0.0, 0.0, 0.13]),
        part(
            Secondary(Spring::Tabard),
            Box(0.09, 0.28, 0.018),
            ALLY,
            [0.0, -0.22, -0.04],
        )
        .rot([-8.0, 0.0, 8.0]),
        part(
            Secondary(Spring::Tabard),
            Box(0.10, 0.025, 0.024),
            LINEN,
            [-0.02, -0.35, -0.02],
        ),
    ];
    for (upper, lower, tip, s) in [
        (UpperArmL, ForearmL, HandL, 1.0),
        (UpperArmR, ForearmR, HandR, -1.0),
        (ThighL, ShinL, FootL, 1.0),
        (ThighR, ShinR, FootR, -1.0),
    ] {
        v.extend([
            part(
                Body(upper),
                Pill(0.045, 0.11),
                CHARCOAL,
                [s * 0.015, -0.08, 0.0],
            )
            .rot([0.0, 0.0, s * 10.0]),
            part(Body(lower), Pill(0.032, 0.12), DARK, [0.0, -0.09, 0.02]).rot([-12.0, 0.0, 0.0]),
            part(Body(tip), Cone(0.037, 0.15), IVORY, [0.0, -0.05, 0.04]).rot([150.0, 0.0, 0.0]),
            part(Body(lower), Ball(0.033), TURQUOISE, [0.0, -0.10, 0.04]).glow(),
        ]);
    }
    for (n, s) in [(WeaponR, 1.0), (OffhandL, -1.0)] {
        v.push(
            part(Prop(n), Pill(0.04, 0.22), CHARCOAL, [s * 0.04, -0.14, 0.0]).rot([
                0.0,
                0.0,
                s * 22.0,
            ]),
        );
        v.push(
            part(Prop(n), Cone(0.035, 0.15), IVORY, [s * 0.10, -0.29, 0.0]).rot([180.0, 0.0, 0.0]),
        );
    }
    crests(&mut v, f);
    FigureSpec {
        plan: plan(tier),
        parts: v,
    }
}
fn crests(v: &mut Vec<PartSpec>, f: Form) {
    for s in [-1.0, 1.0] {
        v.push(
            part(
                Body(Head),
                Cone(0.055, if f.a { 0.32 } else { 0.15 }),
                IVORY,
                [s * 0.12, 0.16, -0.01],
            )
            .rot([0.0, 0.0, s * -18.0]),
        );
        if f.a && f.t2 {
            v.push(
                part(
                    Body(Head),
                    Box(0.10, 0.30, 0.07),
                    IVORY,
                    [s * 0.22, -0.005, 0.01],
                )
                .rot([0.0, 0.0, s * -15.0]),
            );
        }
        if f.b {
            v.push(
                part(Body(Chest), Pill(0.05, 0.23), IVORY, [s * 0.12, 0.25, 0.08]).rot([
                    0.0,
                    0.0,
                    s * -20.0,
                ]),
            );
        }
    }
    if f.b {
        for a in [0.0_f32, 60.0, 120.0, 180.0, 240.0, 300.0] {
            let r = a.to_radians();
            v.push(
                part(
                    Body(Chest),
                    Box(0.045, 0.23, 0.06),
                    GUNMETAL,
                    [r.cos() * 0.27, 0.14, r.sin() * 0.27],
                )
                .rot([0.0, -a, 0.0]),
            );
        }
    }
}
