//! Trigger's orange feline silhouette, two revolvers, curved tail and branch-specific mercenary gear.
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
pub const MUZZLE: Vec3 = Vec3::new(0.0, 0.055, 0.25);
pub fn plan(t: u8) -> Proportions {
    let mut p = body::plan(false, 0.78, t);
    p.torso.x *= 0.85;
    p.arm = 0.18;
    p
}
pub fn parent(n: Node) -> Option<Node> {
    match n {
        Secondary(Spring::Back) => Some(Body(Hips)),
        Prop(Socket::Back) => Some(Secondary(Spring::Back)),
        _ => n.parent(),
    }
}
pub fn rest(n: Node, p: &Proportions) -> Vec3 {
    match n {
        Secondary(Spring::Back) => Vec3::new(0.0, -0.02, -0.14),
        Prop(Socket::Back) => Vec3::new(0.0, 0.04, -0.30),
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
            skin: ORANGE,
            cloth: if f.b { DARK_LEATHER } else { OAK },
            legs: CHARCOAL,
            boots: DARK_LEATHER,
        },
    );
    v.extend([
        part(Body(Head), Ball(0.175), ORANGE, [0.0, 0.025, 0.0]).scale([1.1, 0.9, 1.0]),
        part(Body(Head), Ball(0.085), LINEN, [0.0, -0.045, 0.15]).scale([1.0, 0.65, 0.75]),
        part(Body(Head), Cone(0.025, 0.035), DARK, [0.0, -0.005, 0.21]).rot([90.0, 0.0, 0.0]),
        part(
            Secondary(Spring::HeadGear),
            Cyl(if f.a { 0.34 } else { 0.30 }, 0.025),
            DARK_LEATHER,
            [0.0, 0.15, 0.0],
        )
        .rot([if f.a { 9.0 } else { 0.0 }, 0.0, -8.0]),
        part(
            Secondary(Spring::HeadGear),
            Frustum(0.17, 0.13, 0.19),
            OAK,
            [0.0, 0.25, -0.025],
        ),
        part(
            Secondary(Spring::HeadGear),
            Cyl(0.172, 0.035),
            GOLD,
            [0.0, 0.18, -0.025],
        ),
        part(Body(Chest), Box(0.22, 0.055, 0.25), ALLY, [0.0, 0.27, 0.0]),
        part(Body(Chest), Cone(0.08, 0.18), ALLY, [0.08, 0.15, 0.14]).rot([180.0, 0.0, -15.0]),
        part(
            Secondary(Spring::Back),
            Pill(0.04, 0.21),
            ORANGE,
            [0.0, 0.0, -0.14],
        )
        .rot([90.0, 0.0, 0.0]),
        part(
            Prop(Socket::Back),
            Pill(0.04, 0.18),
            ORANGE,
            [0.0, 0.12, -0.03],
        )
        .rot([-25.0, 0.0, 0.0]),
        part(Prop(Socket::Back), Ball(0.05), LINEN, [0.0, 0.25, 0.03]),
    ]);
    for s in [-1.0, 1.0] {
        v.push(
            part(
                Body(Head),
                Cone(0.075, 0.16),
                ORANGE,
                [s * 0.14, 0.17, -0.015],
            )
            .rot([0.0, 0.0, -s * 15.0]),
        );
        v.push(part(
            Body(Head),
            Ball(0.026),
            LEAF_DARK,
            [s * 0.075, 0.02, 0.148],
        ));
        v.push(part(
            Body(Head),
            Ball(0.012),
            DARK,
            [s * 0.075, 0.02, 0.172],
        ));
        v.push(
            part(
                Body(Hips),
                Box(0.08, 0.12, 0.09),
                DARK_LEATHER,
                [s * 0.16, -0.06, 0.03],
            )
            .rot([0.0, 0.0, s * 15.0]),
        );
    }
    for (socket, foot) in [(Socket::WeaponR, FootR), (Socket::OffhandL, FootL)] {
        pistol(&mut v, socket, f);
        v.push(part(
            Body(foot),
            Box(0.12, 0.09, 0.20),
            DARK_LEATHER,
            [0.0, 0.015, 0.04],
        ));
        if f.a && f.t2 {
            v.push(part(Body(foot), Ball(0.035), GOLD, [0.0, 0.03, -0.12]));
        }
    }
    if f.a {
        for s in [-1.0, 1.0] {
            v.push(
                part(
                    Secondary(Spring::Tabard),
                    Box(0.10, 0.31, 0.035),
                    ALLY,
                    [s * 0.10, -0.12, -0.13],
                )
                .rot([0.0, 0.0, s * 8.0]),
            );
        }
        if f.t2 {
            v.push(part(
                Prop(Socket::Back),
                Box(0.09, 0.14, 0.065),
                GOLD,
                [0.0, 0.19, 0.0],
            ));
        }
    }
    FigureSpec { plan: p, parts: v }
}
fn pistol(v: &mut Vec<PartSpec>, s: Socket, f: Form) {
    let w = Prop(s);
    v.extend([
        part(w, Box(0.045, 0.12, 0.055), OAK, [0.0, -0.025, 0.0]).rot([-12.0, 0.0, 0.0]),
        part(w, Cyl(0.038, 0.075), IRON, [0.0, 0.055, 0.035]).rot([90.0, 0.0, 0.0]),
        part(w, Cyl(0.022, 0.17), IRON, [0.0, 0.055, 0.15]).rot([90.0, 0.0, 0.0]),
        part(w, Cyl(0.024, 0.012), DARK, [0.0, 0.055, 0.24]).rot([90.0, 0.0, 0.0]),
        part(w, Ring(0.008, 0.04, 310.0), GOLD, [0.0, -0.015, 0.05]).rot([0.0, 0.0, 90.0]),
    ]);
    if f.b {
        v.push(part(w, Cyl(0.045, 0.075), BRASS, [0.0, 0.055, 0.20]).rot([90.0, 0.0, 0.0]));
        if f.t2 {
            v.push(part(w, Box(0.025, 0.065, 0.10), IRON, [0.0, 0.11, 0.15]).rot([15.0, 0.0, 0.0]));
        }
    }
}
