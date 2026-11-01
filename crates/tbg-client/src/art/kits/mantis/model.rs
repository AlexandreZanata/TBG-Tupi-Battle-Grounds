//! Mantis's inverted triangular face, folded scythes, narrow abdomen and five leg/blade forms.
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
    let mut p = body::plan(false, 1.01, t);
    p.torso.x *= 0.68;
    p.leg = 0.36;
    p.arm = 0.23;
    p
}
pub fn parent(n: Node) -> Option<Node> {
    match n {
        Secondary(Spring::Back) => Some(Body(Hips)),
        _ => n.parent(),
    }
}
pub fn rest(n: Node, p: &Proportions) -> Vec3 {
    match n {
        Secondary(Spring::Back) => Vec3::new(0.0, 0.02, -0.12),
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
            skin: MOSS,
            cloth: CHITIN,
            legs: MOSS,
            boots: CHITIN,
        },
    );
    v.extend([
        part(Body(Head), Cone(0.18, 0.26), MOSS, [0.0, 0.02, 0.0])
            .rot([180.0, 0.0, 0.0])
            .scale([1.25, 1.0, 0.5]),
        part(Secondary(Spring::Back), Ball(0.11), MOSS, [0.0, 0.0, -0.06]).scale([0.8, 1.15, 1.6]),
        part(
            Secondary(Spring::Back),
            Cyl(0.11, 0.05),
            ALLY,
            [0.0, 0.0, -0.07],
        ),
        part(Body(Chest), Box(0.12, 0.05, 0.025), ALLY, [0.0, 0.14, 0.10]),
    ]);
    for (s, hand, elbow, shin) in [
        (-1.0, Socket::WeaponR, ForearmR, ShinR),
        (1.0, Socket::OffhandL, ForearmL, ShinL),
    ] {
        v.push(part(Body(Head), Ball(0.038), AMBER, [s * 0.12, 0.08, 0.10]));
        v.push(part(
            Body(Head),
            Ball(0.018),
            CHITIN,
            [s * 0.12, 0.08, 0.13],
        ));
        v.push(
            part(
                Secondary(Spring::HeadGear),
                Cyl(0.009, if f.a && f.t2 { 0.39 } else { 0.24 }),
                AMBER,
                [s * 0.10, if f.a && f.t2 { 0.32 } else { 0.25 }, 0.0],
            )
            .rot([0.0, 0.0, -s * 25.0]),
        );
        v.push(
            part(
                Prop(hand),
                Ring(
                    if f.b { 0.035 } else { 0.024 },
                    if f.b { 0.18 } else { 0.15 },
                    135.0,
                ),
                if f.b { CHITIN } else { AMBER },
                [0.0, -0.08, 0.08],
            )
            .rot([90.0, 0.0, s * 45.0]),
        );
        v.push(
            part(
                Prop(hand),
                Cone(0.035, 0.18),
                if f.b { CHARCOAL } else { AMBER },
                [0.0, -0.04, 0.23],
            )
            .rot([90.0, 0.0, 0.0]),
        );
        if f.a {
            v.push(part(
                Body(shin),
                Pill(0.06, 0.14),
                IVORY,
                [0.0, -0.08, 0.025],
            ));
        }
        if f.b && f.t2 {
            for y in [-0.04, -0.10, -0.16] {
                v.push(
                    part(Body(elbow), Cone(0.028, 0.10), AMBER, [s * 0.08, y, 0.0]).rot([
                        0.0,
                        0.0,
                        -s * 70.0,
                    ]),
                );
            }
        }
    }
    if f.a && f.t2 {
        v.push(
            part(
                Secondary(Spring::HeadGear),
                Cone(0.045, 0.22),
                IVORY,
                [0.0, 0.27, -0.02],
            )
            .rot([-25.0, 0.0, 0.0]),
        );
    }
    FigureSpec { plan: p, parts: v }
}
