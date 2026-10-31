//! Rattle & Tick: one heavy bruiser and an articulated shoulder brother sharing one figure.
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
        Proportions, Socket, Spring,
    },
};
use bevy::prelude::*;
pub fn plan(t: u8) -> Proportions {
    let mut p = body::plan(true, 0.95, t);
    p.torso.x *= 1.10;
    p.head *= 0.90;
    p
}
pub fn parent(n: Node) -> Option<Node> {
    match n {
        Secondary(Spring::HeadGear) | Prop(Socket::Back) | Prop(Socket::StringTop) => {
            Some(Secondary(Spring::Back))
        }
        Prop(Socket::Nock) => Some(Prop(Socket::StringTop)),
        Secondary(Spring::Tabard) => Some(Body(Chest)),
        _ => n.parent(),
    }
}
pub fn rest(n: Node, p: &Proportions) -> Vec3 {
    match n {
        Secondary(Spring::Back) => Vec3::new(0.28, 0.42, -0.05),
        Secondary(Spring::HeadGear) => Vec3::new(0.0, 0.22, 0.0),
        Prop(Socket::Back) => Vec3::new(0.09, 0.12, 0.0),
        Prop(Socket::StringTop) => Vec3::new(-0.09, 0.12, 0.0),
        Prop(Socket::Nock) => Vec3::new(0.0, -0.15, 0.0),
        Secondary(Spring::Tabard) => Vec3::new(0.0, 0.20, -0.22),
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
            skin: SKIN,
            cloth: LINEN,
            legs: OAK,
            boots: DARK_LEATHER,
        },
    );
    v.extend([
        part(
            Body(Chest),
            Box(p.torso.x * 1.03, 0.08, p.torso.z * 1.03),
            ALLY,
            [0.0, 0.16, 0.0],
        )
        .rot([0.0, 0.0, -24.0]),
        part(
            Body(Head),
            Box(0.31, 0.08, 0.12),
            DARK_LEATHER,
            [0.0, 0.12, 0.06],
        ),
        part(Body(Head), Box(0.19, 0.045, 0.08), SKIN, [0.0, -0.08, 0.18]),
        part(
            Body(Head),
            Box(0.10, 0.025, 0.025),
            DARK,
            [0.0, -0.08, 0.225],
        ),
        part(
            Body(Chest),
            Box(0.12, 0.31, 0.30),
            DARK_LEATHER,
            [-0.15, 0.16, 0.0],
        )
        .rot([0.0, 0.0, -15.0]),
        part(
            Secondary(Spring::Tabard),
            Box(if f.b { 0.34 } else { 0.25 }, 0.30, 0.13),
            DARK_LEATHER,
            [0.0, 0.0, 0.0],
        ),
        part(
            Secondary(Spring::Back),
            Box(0.14, 0.20, 0.11),
            if f.b { ALLY } else { LEATHER },
            [0.0, 0.08, 0.0],
        ),
        part(
            Secondary(Spring::HeadGear),
            Ball(0.075),
            SKIN,
            [0.0, 0.0, 0.0],
        ),
        part(
            Secondary(Spring::HeadGear),
            Cone(0.09, 0.10),
            OAK,
            [0.0, 0.08, 0.0],
        )
        .rot([0.0, 0.0, -15.0]),
        part(
            Prop(Socket::Back),
            Pill(0.025, 0.10),
            SKIN,
            [0.0, -0.07, 0.0],
        ),
        part(
            Prop(Socket::StringTop),
            Pill(0.025, 0.10),
            SKIN,
            [0.0, -0.07, 0.0],
        ),
        part(Prop(Socket::Nock), Ball(0.07), CHARCOAL, [0.0, 0.0, 0.0]),
        part(
            Prop(Socket::Nock),
            Cyl(0.009, 0.06),
            LINEN,
            [0.0, 0.07, 0.0],
        ),
    ]);
    for s in [-1.0, 1.0] {
        v.push(part(
            Secondary(Spring::HeadGear),
            Ball(0.012),
            DARK,
            [s * 0.025, 0.0, 0.064],
        ));
        v.push(
            part(
                Secondary(Spring::Back),
                Pill(0.025, 0.13),
                OAK,
                [s * 0.045, -0.07, 0.03],
            )
            .rot([-70.0, 0.0, 0.0]),
        );
    }
    club(&mut v, f);
    if f.a {
        v.push(
            part(
                Body(Chest),
                Box(0.18, 0.34, 0.34),
                DARK_LEATHER,
                [0.17, 0.16, 0.0],
            )
            .rot([0.0, 0.0, 10.0]),
        );
        if f.t2 {
            v.push(part(
                Body(UpperArmL),
                Box(0.28, 0.09, 0.25),
                IRON,
                [0.0, 0.0, 0.0],
            ));
        }
    }
    if f.b {
        for y in [0.02, 0.09, 0.16] {
            v.push(part(
                Secondary(Spring::Back),
                Box(0.145, 0.025, 0.12),
                LINEN,
                [0.0, y, 0.0],
            ));
        }
        if f.t2 {
            for s in [-1.0, 1.0] {
                v.push(part(
                    Secondary(Spring::Tabard),
                    Ball(0.075),
                    CHARCOAL,
                    [s * 0.13, 0.12, -0.08],
                ));
                v.push(part(
                    Secondary(Spring::Tabard),
                    Box(0.06, 0.22, 0.04),
                    IRON,
                    [s * 0.13, 0.0, -0.13],
                ));
            }
        }
    }
    FigureSpec { plan: p, parts: v }
}
fn club(v: &mut Vec<PartSpec>, f: Form) {
    let w = Prop(Socket::WeaponR);
    v.push(part(w, Cyl(0.035, 0.47), OAK, [0.0, -0.11, 0.0]));
    v.push(part(
        w,
        Box(if f.a { 0.23 } else { 0.18 }, 0.29, 0.19),
        OAK,
        [0.0, -0.40, 0.0],
    ));
    for y in [-0.32, -0.48] {
        v.push(part(w, Box(0.21, 0.045, 0.21), IRON, [0.0, y, 0.0]));
        for s in [-1.0, 1.0] {
            v.push(part(w, Cone(0.025, 0.08), IRON, [s * 0.14, y, 0.0]).rot([0.0, 0.0, s * 90.0]));
        }
    }
}
