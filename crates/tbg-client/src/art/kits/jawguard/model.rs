//! Jawguard's broad beetle shell, paired mandibles, horn and five carapace/jaw forms.
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
pub fn plan(t: u8) -> Proportions {
    let mut p = body::plan(true, 1.02, t);
    p.leg = 0.28;
    p.arm = 0.18;
    p.torso.y = 0.30;
    p
}
pub fn parent(n: Node) -> Option<Node> {
    match n {
        Prop(Socket::WeaponR) | Prop(Socket::OffhandL) => Some(Body(Head)),
        _ => n.parent(),
    }
}
pub fn rest(n: Node, p: &Proportions) -> Vec3 {
    match n {
        Prop(Socket::WeaponR) => Vec3::new(-0.13, -0.08, 0.16),
        Prop(Socket::OffhandL) => Vec3::new(0.13, -0.08, 0.16),
        Secondary(Spring::Back) => Vec3::new(0.0, 0.23, -0.08),
        Body(Head) => Vec3::new(0.0, 0.30, 0.16),
        _ => rig::rest(n, p),
    }
}
pub fn spec(t: u8, b: u8) -> FigureSpec {
    let p = plan(t);
    let f = Form::of(t, b);
    let mut v = Vec::new();
    body::humanoid(
        &mut v,
        &p,
        &body::Look {
            skin: CHITIN,
            cloth: MOSS,
            legs: CHITIN,
            boots: MOSS,
        },
    );
    v.extend([
        part(
            Secondary(Spring::Back),
            Ball(0.34),
            if f.a { IVORY } else { MOSS },
            [0.0, 0.12, -0.07],
        )
        .scale([1.25, 1.1, 0.85]),
        part(
            Secondary(Spring::Back),
            Box(0.035, 0.48, 0.07),
            CHITIN,
            [0.0, 0.13, -0.31],
        ),
        part(
            Body(Head),
            Ball(0.20),
            if f.b { CHARCOAL } else { MOSS },
            [0.0, 0.0, 0.0],
        )
        .scale([1.1, 0.8, 1.0]),
        part(
            Secondary(Spring::HeadGear),
            Cone(0.075, 0.25),
            AMBER,
            [0.0, 0.17, 0.05],
        )
        .rot([20.0, 0.0, 0.0]),
    ]);
    for s in [-1.0, 1.0] {
        v.push(part(
            Body(Head),
            Ball(0.035),
            AMBER,
            [s * 0.13, 0.025, 0.15],
        ));
        v.push(part(
            Body(Chest),
            Box(0.035, 0.25, 0.29),
            ALLY,
            [s * 0.29, 0.15, 0.0],
        ));
        let n = if s < 0.0 {
            Socket::WeaponR
        } else {
            Socket::OffhandL
        };
        v.push(
            part(
                Prop(n),
                Ring(0.035, if f.b { 0.17 } else { 0.12 }, 150.0),
                AMBER,
                [0.0, 0.0, 0.04],
            )
            .rot([90.0, 0.0, if s < 0.0 { 50.0 } else { 130.0 }]),
        );
        v.push(
            part(
                Prop(n),
                Cone(0.04, if f.b { 0.22 } else { 0.15 }),
                CHITIN,
                [s * 0.04, 0.0, if f.b { 0.19 } else { 0.13 }],
            )
            .rot([90.0, 0.0, s * 15.0]),
        );
        if f.b && f.t2 {
            v.push(
                part(
                    Prop(n),
                    Box(0.06, 0.035, 0.22),
                    AMBER,
                    [s * 0.09, 0.0, 0.13],
                )
                .rot([0.0, -s * 18.0, 0.0]),
            );
        }
    }
    if f.a {
        for s in [-1.0, 1.0] {
            v.push(
                part(
                    Secondary(Spring::Back),
                    Box(0.13, 0.30, 0.08),
                    STONE,
                    [s * 0.22, 0.10, -0.24],
                )
                .rot([0.0, 0.0, -s * 12.0]),
            );
        }
        if f.t2 {
            v.push(
                part(
                    Secondary(Spring::Back),
                    Cone(0.16, 0.34),
                    IVORY,
                    [0.0, 0.43, -0.10],
                )
                .scale([0.35, 1.0, 1.0]),
            );
        }
    }
    FigureSpec { plan: p, parts: v }
}
