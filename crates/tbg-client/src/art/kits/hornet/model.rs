//! Hornet's banded abdomen, opaque faceted wing pairs, stinger and five venom/swoop forms.
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
pub fn muzzle(t: u8, b: u8) -> Vec3 {
    Vec3::new(0.0, 0.03, if t == 2 && b == 0 { 0.455 } else { 0.375 })
}
pub fn plan(t: u8) -> Proportions {
    let mut p = body::plan(false, 0.94, t);
    p.torso.x *= 0.76;
    p.arm = 0.19;
    p
}
pub fn parent(n: Node) -> Option<Node> {
    match n {
        Secondary(Spring::Back) => Some(Body(Hips)),
        Secondary(Spring::HeadGear) | Secondary(Spring::Tabard) => Some(Body(Chest)),
        _ => n.parent(),
    }
}
pub fn rest(n: Node, p: &Proportions) -> Vec3 {
    match n {
        Secondary(Spring::Back) => Vec3::new(0.0, 0.0, -0.14),
        Secondary(Spring::HeadGear) => Vec3::new(0.10, 0.26, -0.09),
        Secondary(Spring::Tabard) => Vec3::new(-0.10, 0.26, -0.09),
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
            skin: AMBER,
            cloth: if f.a {
                IVORY
            } else if f.b {
                CHITIN
            } else {
                MOSS
            },
            legs: CHITIN,
            boots: AMBER,
        },
    );
    v.extend([
        part(Body(Head), Ball(0.17), CHITIN, [0.0, 0.01, 0.0]).scale([1.0, 1.0, 0.8]),
        part(
            Secondary(Spring::Back),
            Pill(0.10, 0.15),
            AMBER,
            [0.0, -0.02, -0.11],
        )
        .rot([65.0, 0.0, 0.0]),
        part(
            Secondary(Spring::Back),
            Cone(0.06, 0.18),
            CHITIN,
            [0.0, -0.10, -0.30],
        )
        .rot([-70.0, 0.0, 0.0]),
        part(
            Secondary(Spring::Back),
            Box(0.13, 0.11, 0.025),
            ALLY,
            [0.0, 0.02, -0.27],
        ),
        part(
            Prop(Socket::WeaponR),
            Cyl(0.035, if f.a && f.t2 { 0.32 } else { 0.23 }),
            AMBER,
            [0.0, 0.03, if f.a && f.t2 { 0.15 } else { 0.11 }],
        )
        .rot([90.0, 0.0, 0.0]),
        part(
            Prop(Socket::WeaponR),
            Cone(0.025, 0.15),
            CHITIN,
            [0.0, 0.03, if f.a && f.t2 { 0.38 } else { 0.30 }],
        )
        .rot([90.0, 0.0, 0.0]),
    ]);
    for s in [-1.0, 1.0] {
        v.push(part(Body(Head), Ball(0.045), AMBER, [s * 0.10, 0.04, 0.12]));
        v.push(
            part(Body(Head), Cyl(0.008, 0.18), MOSS, [s * 0.07, 0.20, 0.01]).rot([
                0.0,
                0.0,
                -s * 20.0,
            ]),
        );
        let n = if s < 0.0 {
            Spring::Tabard
        } else {
            Spring::HeadGear
        };
        for (i, angle) in [(0, 35.0), (1, 70.0)] {
            v.push(
                part(
                    Secondary(n),
                    Frustum(0.035, 0.09, if f.b { 0.40 } else { 0.32 }),
                    IVORY,
                    [s * (0.12 + i as f32 * 0.05), 0.10 - i as f32 * 0.09, 0.0],
                )
                .scale([1.0, 1.0, 0.18])
                .rot([0.0, 0.0, -s * angle]),
            );
        }
        if f.a {
            v.push(part(Body(Chest), Ball(0.055), LEAF, [s * 0.13, 0.10, 0.10]).glow());
        }
        if f.b && f.t2 {
            v.push(
                part(
                    Secondary(n),
                    Box(0.05, 0.22, 0.025),
                    AMBER,
                    [s * 0.15, 0.28, 0.0],
                )
                .rot([0.0, 0.0, -s * 15.0]),
            );
        }
    }
    for z in [-0.04, -0.13, -0.20] {
        v.push(
            part(
                Secondary(Spring::Back),
                Cyl(0.105, 0.045),
                CHITIN,
                [0.0, -0.02, z],
            )
            .rot([90.0, 0.0, 0.0]),
        );
    }
    FigureSpec { plan: p, parts: v }
}
