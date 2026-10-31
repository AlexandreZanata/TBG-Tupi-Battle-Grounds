//! Drillnest's low tracked foundry, honeycomb rack and five distinct feed/barrel forms.
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
    body::plan(false, 0.85, t)
}
pub fn parent(n: Node) -> Option<Node> {
    match n {
        Body(Chest) | Body(Head) => Some(Body(Hips)),
        Prop(Socket::WeaponR) => Some(Body(Chest)),
        Secondary(_) => Some(Body(Hips)),
        _ => n.parent(),
    }
}
pub fn rest(n: Node, p: &Proportions) -> Vec3 {
    match n {
        Body(Hips) => Vec3::new(0.0, 0.24, 0.0),
        Body(Chest) => Vec3::new(0.0, 0.15, 0.0),
        Body(Head) => Vec3::new(0.0, 0.29, 0.05),
        Prop(Socket::WeaponR) => Vec3::new(0.0, 0.05, 0.18),
        Secondary(_) => Vec3::ZERO,
        _ => rig::rest(n, p),
    }
}
pub fn spec(t: u8, b: u8) -> FigureSpec {
    let f = Form::of(t, b);
    let p = plan(t);
    let mut v = vec![
        part(
            Body(Hips),
            Box(0.63, 0.27, 0.65),
            if f.a {
                IVORY
            } else if f.b {
                CHITIN
            } else {
                IRON
            },
            [0.0, 0.0, 0.0],
        ),
        part(
            Body(Chest),
            Box(if f.a { 0.50 } else { 0.39 }, 0.22, 0.40),
            OAK,
            [0.0, 0.04, -0.03],
        ),
        part(Body(Head), Box(0.20, 0.08, 0.16), BRASS, [0.0, 0.0, -0.12]),
        part(
            Prop(Socket::WeaponR),
            Cyl(if f.b { 0.10 } else { 0.07 }, 0.35),
            IRON,
            [0.0, 0.0, 0.16],
        )
        .rot([90.0, 0.0, 0.0]),
        part(
            Prop(Socket::WeaponR),
            Ring(0.018, 0.085, 360.0),
            BRASS,
            [0.0, 0.0, 0.33],
        )
        .rot([90.0, 0.0, 0.0]),
    ];
    for s in [-1.0, 1.0] {
        v.push(part(
            Body(Hips),
            Box(0.13, 0.23, 0.79),
            CHITIN,
            [s * 0.34, -0.03, 0.0],
        ));
        v.push(part(
            Body(Hips),
            Box(0.035, 0.13, 0.35),
            ALLY,
            [s * 0.415, 0.01, 0.0],
        ));
        for z in [-0.30, -0.15, 0.0, 0.15, 0.30] {
            v.push(
                part(Body(Hips), Cyl(0.07, 0.15), BRASS, [s * 0.34, -0.03, z])
                    .rot([0.0, 0.0, 90.0]),
            );
        }
        for z in [-0.33, -0.16, 0.01, 0.18, 0.35] {
            v.push(part(
                Secondary(if s < 0.0 {
                    Spring::Tabard
                } else {
                    Spring::HeadGear
                }),
                Box(0.14, 0.025, 0.08),
                IRON,
                [s * 0.34, -0.15, z],
            ));
        }
        v.push(part(
            Secondary(Spring::Back),
            Cyl(0.035, 0.26),
            BRASS,
            [s * 0.22, 0.36, -0.25],
        ));
        if f.a && f.t2 {
            v.push(part(
                Body(Chest),
                Box(0.15, 0.22, 0.27),
                IVORY,
                [s * 0.26, 0.21, -0.06],
            ));
        }
        if f.b && f.t2 {
            v.push(part(
                Prop(Socket::WeaponR),
                Box(0.09, 0.24, 0.26),
                BRASS,
                [s * 0.12, 0.0, 0.24],
            ));
        }
    }
    for (x, y) in [
        (-0.13, 0.10),
        (0.0, 0.10),
        (0.13, 0.10),
        (-0.065, 0.22),
        (0.065, 0.22),
    ] {
        v.push(part(Body(Chest), Cyl(0.045, 0.16), CHITIN, [x, y, -0.07]).rot([90.0, 0.0, 0.0]));
        v.push(part(Body(Chest), Cone(0.035, 0.10), BRASS, [x, y, 0.06]).rot([90.0, 0.0, 0.0]));
    }
    FigureSpec { plan: p, parts: v }
}
