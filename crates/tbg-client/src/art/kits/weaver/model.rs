//! Weaver's eight individually grouped legs, low eyes, spinner and five web/nozzle silhouettes.
use crate::art::{
    figure::{part, FigureSpec},
    kits::common::{body, Form},
    mesh::Shape::*,
    palette::*,
    rig::{
        self,
        Joint::*,
        Node,
        Node::{Body, Prop, Secondary},
        Proportions, Socket, Spring,
    },
};
use bevy::prelude::*;
pub const LEGS: [(rig::Joint, f32, f32); 8] = [
    (UpperArmL, 1.0, 0.20),
    (ForearmL, 1.0, 0.06),
    (ThighL, 1.0, -0.08),
    (ShinL, 1.0, -0.22),
    (UpperArmR, -1.0, 0.20),
    (ForearmR, -1.0, 0.06),
    (ThighR, -1.0, -0.08),
    (ShinR, -1.0, -0.22),
];
pub fn plan(t: u8) -> Proportions {
    body::plan(false, 0.75, t)
}
pub fn parent(n: Node) -> Option<Node> {
    match n {
        Body(j) if LEGS.iter().any(|p| p.0 == j) => Some(Body(Hips)),
        Body(Chest) | Body(Head) => Some(Body(Hips)),
        Prop(Socket::WeaponR) => Some(Body(Head)),
        Secondary(_) => Some(Body(Chest)),
        _ => n.parent(),
    }
}
pub fn rest(n: Node, p: &Proportions) -> Vec3 {
    if let Body(j) = n {
        if let Some((_, s, z)) = LEGS.iter().find(|v| v.0 == j) {
            return Vec3::new(s * 0.14, 0.0, *z);
        }
    }
    match n {
        Body(Hips) => Vec3::new(0.0, 0.31, 0.0),
        Body(Chest) => Vec3::new(0.0, 0.06, -0.18),
        Body(Head) => Vec3::new(0.0, -0.04, 0.25),
        Prop(Socket::WeaponR) => Vec3::new(0.0, 0.0, 0.11),
        Secondary(_) => Vec3::ZERO,
        _ => rig::rest(n, p),
    }
}
pub fn spec(t: u8, b: u8) -> FigureSpec {
    let f = Form::of(t, b);
    let p = plan(t);
    let mut v = vec![
        part(
            Body(Chest),
            Ball(0.25),
            if f.b { CHITIN } else { MOSS },
            [0.0, 0.0, -0.08],
        )
        .scale([1.0, 0.8, 1.2]),
        part(Body(Head), Ball(0.14), CHITIN, [0.0, 0.0, 0.0]).scale([1.2, 0.6, 1.0]),
        part(
            Body(Chest),
            Box(0.18, 0.035, 0.16),
            ALLY,
            [0.0, 0.20, -0.08],
        ),
        part(
            Prop(Socket::WeaponR),
            Cyl(if f.a { 0.045 } else { 0.03 }, 0.14),
            AMBER,
            [0.0, 0.0, 0.07],
        )
        .rot([90.0, 0.0, 0.0]),
    ];
    for (j, s, z) in LEGS {
        let spread = if f.b && z > 0.05 { 0.36 } else { 0.28 };
        v.push(
            part(
                Body(j),
                Box(spread, 0.035, 0.035),
                MOSS,
                [s * spread * 0.45, 0.04, 0.0],
            )
            .rot([0.0, -s * z * 100.0, s * 15.0]),
        );
        v.push(
            part(
                Body(j),
                Box(0.03, 0.34, 0.03),
                if f.a { IVORY } else { CHITIN },
                [s * spread, -0.10, -z * 0.3],
            )
            .rot([0.0, 0.0, s * 25.0]),
        );
    }
    for s in [-1.0, 1.0] {
        for x in [0.04, 0.10] {
            v.push(part(Body(Head), Ball(0.025), LEAF, [s * x, 0.02, 0.12]).glow());
        }
        v.push(
            part(
                Body(Head),
                Cone(0.025, 0.10),
                AMBER,
                [s * 0.07, -0.06, 0.14],
            )
            .rot([-45.0, 0.0, s * 15.0]),
        );
        if f.a {
            for z in [-0.13, 0.0, 0.13] {
                v.push(
                    part(
                        Body(Chest),
                        Box(0.035, 0.03, 0.09),
                        IVORY,
                        [s * 0.16, 0.16, z - 0.08],
                    )
                    .rot([0.0, 0.0, -s * 30.0]),
                );
            }
        }
        if f.a && f.t2 {
            v.push(
                part(
                    Secondary(Spring::Back),
                    Cyl(0.012, 0.30),
                    AMBER,
                    [s * 0.12, 0.28, -0.12],
                )
                .rot([0.0, 0.0, s * 45.0]),
            );
        }
        if f.b && f.t2 {
            v.push(
                part(
                    Prop(Socket::WeaponR),
                    Box(0.02, 0.06, 0.15),
                    AMBER,
                    [s * 0.065, 0.0, 0.12],
                )
                .rot([0.0, s * 15.0, 0.0]),
            );
        }
    }
    if f.a && f.t2 {
        v.push(part(
            Secondary(Spring::Back),
            Box(0.30, 0.025, 0.025),
            IVORY,
            [0.0, 0.30, -0.12],
        ));
    }
    FigureSpec { plan: p, parts: v }
}
