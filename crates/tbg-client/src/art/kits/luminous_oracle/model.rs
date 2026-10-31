//! Luminous Oracle's empty hood, geometric robe/cape and articulated sun-ring staff in five forms.
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
    let mut p = body::plan(false, 0.99, t);
    p.torso.x *= 0.82;
    p
}
pub fn parent(n: Node) -> Option<Node> {
    match n {
        Secondary(Spring::HeadGear) => Some(Prop(Socket::WeaponR)),
        _ => n.parent(),
    }
}
pub fn rest(n: Node, p: &Proportions) -> Vec3 {
    match n {
        Secondary(Spring::HeadGear) => Vec3::new(0.0, 0.55, 0.0),
        _ => rig::rest(n, p),
    }
}
pub fn muzzle(t: u8, b: u8) -> Vec3 {
    Vec3::new(0.0, 0.55 + if t == 2 && b == 0 { 0.10 } else { 0.0 }, 0.08)
}
pub fn spec(t: u8, b: u8) -> FigureSpec {
    let p = plan(t);
    let f = Form::of(t, b);
    let mut v = Vec::new();
    body::humanoid(
        &mut v,
        &p,
        &body::Look {
            skin: NAVY,
            cloth: IVORY,
            legs: NAVY,
            boots: GOLD,
        },
    );
    v.extend([
        part(
            Body(Hips),
            Frustum(0.15, 0.24, 0.40),
            if f.b { NAVY } else { IVORY },
            [0.0, -0.13, 0.0],
        ),
        part(Body(Head), Cone(0.22, 0.35), IVORY, [0.0, 0.12, -0.04]),
        part(Body(Head), Box(0.18, 0.17, 0.04), NAVY, [0.0, 0.015, 0.18]),
        part(Body(Head), Box(0.20, 0.025, 0.045), GOLD, [0.0, 0.10, 0.19]),
        part(
            Secondary(Spring::Back),
            Box(0.32, 0.43, 0.025),
            NAVY,
            [0.0, 0.01, -0.16],
        ),
        part(
            Secondary(Spring::Back),
            Box(0.24, 0.36, 0.018),
            ALLY,
            [0.0, 0.02, -0.18],
        ),
        part(
            Prop(Socket::WeaponR),
            Cyl(0.022, 0.90),
            GOLD,
            [0.0, 0.12, 0.0],
        ),
        part(
            Secondary(Spring::HeadGear),
            Ring(0.018, if f.a && f.t2 { 0.17 } else { 0.12 }, 360.0),
            GOLD,
            [0.0, if f.a && f.t2 { 0.10 } else { 0.0 }, 0.0],
        )
        .rot([90.0, 0.0, 0.0]),
        part(
            Secondary(Spring::HeadGear),
            Ball(0.04),
            IVORY,
            [0.0, if f.a && f.t2 { 0.10 } else { 0.0 }, 0.0],
        )
        .glow(),
    ]);
    for s in [-1.0, 1.0] {
        v.push(part(
            Body(Chest),
            Box(if f.a { 0.10 } else { 0.05 }, 0.35, 0.025),
            if f.a { IVORY } else { NAVY },
            [s * if f.a { 0.10 } else { 0.08 }, 0.10, 0.13],
        ));
        v.push(part(
            Body(Hips),
            Box(0.025, 0.36, 0.025),
            GOLD,
            [s * 0.13, -0.13, 0.18],
        ));
        if f.b {
            v.push(
                part(Body(Chest), Cone(0.08, 0.23), NAVY, [s * 0.20, 0.16, 0.0]).rot([
                    0.0,
                    0.0,
                    -s * 55.0,
                ]),
            );
        }
    }
    if f.a && f.t2 {
        v.push(part(
            Secondary(Spring::HeadGear),
            Cone(0.03, 0.16),
            GOLD,
            [0.0, 0.34, 0.0],
        ));
    }
    if f.b && f.t2 {
        for s in [-1.0, 1.0] {
            v.push(
                part(
                    Secondary(Spring::HeadGear),
                    Box(0.025, 0.28, 0.03),
                    IVORY,
                    [s * 0.15, 0.04, 0.0],
                )
                .rot([0.0, 0.0, -s * 15.0]),
            );
        }
    }
    FigureSpec { plan: p, parts: v }
}
