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
    Vec3::new(0.0, 0.06, if t == 2 && b == 0 { 0.53 } else { 0.46 })
}
pub fn plan(t: u8) -> Proportions {
    let mut p = body::plan(false, 1.10, t);
    p.torso.x *= 0.88;
    p.arm = 0.23;
    p
}
pub fn parent(n: Node) -> Option<Node> {
    match n {
        Secondary(Spring::Back) => Some(Body(Hips)),
        Secondary(Spring::HeadGear) | Secondary(Spring::Tabard) => Some(Body(Chest)),
        Prop(Socket::OffhandL | Socket::Nock | Socket::StringTop | Socket::StringBottom) => {
            Some(Body(Root))
        }
        _ => n.parent(),
    }
}
pub fn rest(n: Node, p: &Proportions) -> Vec3 {
    match n {
        Secondary(Spring::Back) => Vec3::new(0.0, 0.0, -0.14),
        Secondary(Spring::HeadGear) => Vec3::new(0.10, 0.26, -0.09),
        Secondary(Spring::Tabard) => Vec3::new(-0.10, 0.26, -0.09),
        Prop(Socket::OffhandL | Socket::Nock | Socket::StringTop | Socket::StringBottom) => {
            Vec3::ZERO
        }
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
                AMBER
            },
            legs: CHITIN,
            boots: AMBER,
        },
    );
    v.extend([
        part(Body(Head), Ball(0.18), AMBER, [0.0, 0.01, 0.0]).scale([1.0, 1.0, 0.8]),
        part(
            Secondary(Spring::Back),
            Pill(0.14, 0.18),
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
            Prop(Socket::WeaponR),
            Box(0.16, 0.15, 0.29),
            CHARCOAL,
            [0.0, 0.055, 0.14],
        ),
        part(
            Prop(Socket::WeaponR),
            Box(0.18, 0.07, 0.23),
            AMBER,
            [0.0, 0.14, 0.13],
        ),
        part(
            Prop(Socket::WeaponR),
            Cyl(0.062, if f.a && f.t2 { 0.36 } else { 0.29 }),
            BRASS,
            [0.0, 0.06, if f.a && f.t2 { 0.33 } else { 0.295 }],
        )
        .rot([90.0, 0.0, 0.0]),
        part(
            Prop(Socket::WeaponR),
            Cyl(0.035, 0.018),
            DARK,
            [0.0, 0.06, if f.a && f.t2 { 0.515 } else { 0.445 }],
        )
        .rot([90.0, 0.0, 0.0]),
        part(
            Prop(Socket::WeaponR),
            Box(0.07, 0.13, 0.09),
            DARK_LEATHER,
            [0.0, -0.065, 0.03],
        ),
        part(Prop(Socket::WeaponR), Ball(0.065), LEAF, [0.09, 0.08, 0.13])
            .scale([0.6, 1.0, 1.4])
            .glow(),
        part(
            Secondary(Spring::Back),
            Ball(0.085),
            LEAF,
            [0.10, -0.035, -0.18],
        )
        .scale([0.6, 1.0, 1.1])
        .glow(),
    ]);
    for y in [0.06, 0.20] {
        v.push(part(
            Body(Chest),
            Box(0.19, 0.04, 0.17),
            CHITIN,
            [0.0, y, 0.0],
        ));
    }
    for s in [-1.0, 1.0] {
        v.push(part(
            Body(Head),
            Ball(0.065),
            CHARCOAL,
            [s * 0.10, 0.04, 0.12],
        ));
        v.push(
            part(Body(Head), Cyl(0.012, 0.20), CHITIN, [s * 0.07, 0.20, 0.01]).rot([
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
                    LINEN,
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
                Cyl(0.145, 0.045),
                CHITIN,
                [0.0, -0.02, z],
            )
            .rot([90.0, 0.0, 0.0]),
        );
    }
    for socket in super::hose::SOCKETS {
        v.push(part(
            Prop(socket),
            Cyl(0.022, 1.0),
            DARK_LEATHER,
            [0.0, 0.0, 0.0],
        ));
    }
    FigureSpec { plan: p, parts: v }
}
