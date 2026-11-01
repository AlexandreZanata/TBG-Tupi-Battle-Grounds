//! Celestial Archer's angular gold bow, rigid feather ornaments, ivory armor and five forms.
use crate::art::{
    figure::{part, FigureSpec, PartSpec},
    kits::common::{body, gear, Form},
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
pub const MUZZLE: Vec3 = Vec3::new(0.0, 0.0, gear::BOW_TIP_Z + gear::ARROW_LEN);
pub fn plan(t: u8) -> Proportions {
    let mut p = body::plan(false, 0.95, t);
    p.torso.x *= 0.85;
    p.arm = 0.20;
    p
}
pub fn half(t: u8, b: u8) -> f32 {
    let f = Form::of(t, b);
    if f.a && f.t2 {
        0.64
    } else if f.a {
        0.60
    } else if f.b && f.t2 {
        0.58
    } else if f.b {
        0.54
    } else {
        0.50
    }
}
pub fn parent(n: Node) -> Option<Node> {
    match n {
        Secondary(Spring::Back) | Secondary(Spring::Tabard) | Prop(Socket::Back) => {
            Some(Body(Chest))
        }
        _ => n.parent(),
    }
}
pub fn rest(n: Node, p: &Proportions) -> Vec3 {
    match n {
        Secondary(Spring::Back) => Vec3::new(0.16, 0.28, -0.13),
        Secondary(Spring::Tabard) => Vec3::new(-0.16, 0.28, -0.13),
        Secondary(Spring::HeadGear) => Vec3::new(0.0, 0.18, -0.05),
        Prop(Socket::Back) => Vec3::new(0.0, 0.12, -0.15),
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
            cloth: IVORY,
            legs: NAVY,
            boots: GOLD,
        },
    );
    v.extend([
        part(Body(Head), Cone(0.17, 0.15), IVORY, [0.0, 0.14, -0.015]),
        part(Body(Head), Box(0.045, 0.12, 0.035), GOLD, [0.0, 0.18, 0.11]),
        part(
            Secondary(Spring::HeadGear),
            Ring(0.012, 0.19, 360.0),
            GOLD,
            [0.0, 0.02, -0.04],
        )
        .rot([90.0, 0.0, 0.0]),
        part(
            Body(Chest),
            Box(0.26, 0.23, 0.09),
            if f.b { NAVY } else { IVORY },
            [0.0, 0.14, 0.10],
        ),
        part(Body(Chest), Box(0.22, 0.045, 0.25), ALLY, [0.0, 0.20, 0.0]).rot([0.0, 0.0, -30.0]),
        part(Prop(Socket::Back), Cyl(0.065, 0.28), NAVY, [0.0, 0.02, 0.0]).rot([0.0, 0.0, -15.0]),
        part(
            Prop(Socket::Back),
            Box(0.07, 0.20, 0.02),
            ALLY,
            [0.0, 0.02, -0.07],
        ),
    ]);
    for (s, node) in [(1.0, Spring::Back), (-1.0, Spring::Tabard)] {
        for i in 0..3 {
            v.push(
                part(
                    Secondary(node),
                    Box(0.055, 0.24 - i as f32 * 0.04, 0.035),
                    IVORY,
                    [s * (0.04 + i as f32 * 0.045), 0.07 + i as f32 * 0.035, 0.0],
                )
                .rot([0.0, 0.0, -s * (30.0 + i as f32 * 10.0)]),
            );
            v.push(
                part(
                    Secondary(node),
                    Cone(0.028, 0.07),
                    GOLD,
                    [s * (0.10 + i as f32 * 0.05), 0.18 + i as f32 * 0.02, 0.0],
                )
                .rot([0.0, 0.0, -s * 45.0]),
            );
        }
    }
    if f.a {
        for j in [UpperArmL, UpperArmR] {
            v.push(part(Body(j), Pill(0.065, 0.10), IVORY, [0.0, -0.07, 0.0]));
        }
    }
    for x in [-0.03, 0.03] {
        v.push(part(
            Prop(Socket::Back),
            Cyl(0.008, 0.25),
            OAK,
            [x, 0.20, 0.0],
        ));
        v.push(part(
            Prop(Socket::Back),
            Box(0.05, 0.07, 0.015),
            IVORY,
            [x, 0.32, 0.0],
        ));
    }
    bow(&mut v, half(t, b), f);
    FigureSpec { plan: p, parts: v }
}
fn bow(v: &mut Vec<PartSpec>, half: f32, f: Form) {
    let w = Prop(Socket::OffhandL);
    v.push(part(w, Box(0.04, 0.13, 0.05), NAVY, [0.0, 0.0, 0.0]));
    for s in [-1.0, 1.0] {
        let mid = [0.0, s * half * 0.55, 0.06];
        let tip = [0.0, s * half, gear::BOW_TIP_Z];
        v.push(gear::strut(w, [0.0, 0.0, 0.0], mid, 0.019, GOLD));
        v.push(gear::strut(w, mid, tip, 0.019, GOLD));
        v.push(
            part(
                w,
                Cone(0.035, 0.09),
                IVORY,
                [0.0, s * half, gear::BOW_TIP_Z],
            )
            .rot([0.0, 0.0, if s < 0.0 { 180.0 } else { 0.0 }]),
        );
        if f.b {
            for x in [-0.04, 0.04] {
                v.push(part(
                    w,
                    Box(0.025, 0.12, 0.03),
                    GOLD,
                    [x, s * (half - 0.03), gear::BOW_TIP_Z],
                ));
            }
        }
    }
    let length = half.hypot(gear::BOW_DRAW);
    for (s, n) in [(-1.0, Socket::StringTop), (1.0, Socket::StringBottom)] {
        v.push(part(
            Prop(n),
            Cyl(0.004, length),
            LINEN,
            [0.0, s * length * 0.5, 0.0],
        ));
    }
    v.push(
        part(
            Prop(Socket::Nock),
            Cyl(0.006, gear::ARROW_LEN),
            IVORY,
            [0.0, 0.0, gear::ARROW_LEN * 0.5],
        )
        .rot([90.0, 0.0, 0.0]),
    );
    v.push(
        part(
            Prop(Socket::Nock),
            Cone(0.025, 0.07),
            GOLD,
            [0.0, 0.0, gear::ARROW_LEN - 0.025],
        )
        .rot([90.0, 0.0, 0.0]),
    );
    if f.a && f.t2 {
        v.push(part(w, Ring(0.012, 0.10, 360.0), GOLD, [0.0, 0.04, 0.12]).rot([90.0, 0.0, 0.0]));
    }
    if f.b && f.t2 {
        for x in [-0.08, 0.0, 0.08] {
            v.push(part(
                w,
                Cone(0.035, if x == 0.0 { 0.16 } else { 0.10 }),
                IVORY,
                [x, half + 0.09, gear::BOW_TIP_Z],
            ));
        }
    }
}
