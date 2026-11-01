//! Trident's shark head/jaw, dorsal fin, rusted three-tine weapon and five armored forms.
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
    let mut p = body::plan(true, 0.96, t);
    p.arm = 0.24;
    p.torso.x *= 1.05;
    p
}
pub fn parent(n: Node) -> Option<Node> {
    n.parent()
}
pub fn rest(n: Node, p: &Proportions) -> Vec3 {
    match n {
        Secondary(Spring::HeadGear) => Vec3::new(0.0, -0.10, 0.13),
        Secondary(Spring::Back) => Vec3::new(0.0, 0.28, -0.18),
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
            skin: GUNMETAL,
            cloth: IRON,
            legs: GUNMETAL,
            boots: RUST,
        },
    );
    v.extend([
        part(Body(Head), Ball(0.23), GUNMETAL, [0.0, 0.02, 0.02]).scale([1.28, 0.85, 1.10]),
        part(Body(Head), Cone(0.17, 0.26), GUNMETAL, [0.0, 0.01, 0.24])
            .rot([90.0, 0.0, 0.0])
            .scale([1.25, 1.0, 0.7]),
        part(
            Secondary(Spring::HeadGear),
            Ball(0.15),
            if f.a { BONE } else { IRON },
            [0.0, 0.0, 0.11],
        )
        .scale([1.40, 0.40, 1.20]),
        part(Body(Head), Box(0.30, 0.035, 0.06), DARK, [0.0, -0.07, 0.26]),
        part(
            Secondary(Spring::Back),
            Cone(
                if f.a && f.t2 { 0.25 } else { 0.20 },
                if f.a && f.t2 { 0.62 } else { 0.36 },
            ),
            if f.a && f.t2 { BONE } else { GUNMETAL },
            [0.0, if f.a && f.t2 { 0.28 } else { 0.12 }, -0.07],
        )
        .scale([0.28, 1.0, 1.0])
        .rot([-20.0, 0.0, 0.0]),
        part(
            Body(Chest),
            Ball(0.21),
            if f.a { BONE } else { STONE },
            [0.0, 0.17, 0.17],
        )
        .scale([1.15, 1.3, 0.40]),
        part(
            Body(Chest),
            Box(0.35, 0.08, 0.30),
            if f.b { RED_CLOTH } else { ALLY },
            [0.0, 0.18, 0.0],
        )
        .rot([0.0, 0.0, -20.0]),
    ]);
    for s in [-1.0, 1.0] {
        v.push(part(Body(Head), Ball(0.026), DARK, [s * 0.21, 0.075, 0.13]));
        v.push(part(Body(Head), Ball(0.012), AMBER, [s * 0.22, 0.08, 0.15]));
        for x in [0.04, 0.10] {
            v.push(
                part(Body(Head), Cone(0.019, 0.055), BONE, [s * x, -0.07, 0.28])
                    .rot([180.0, 0.0, 0.0]),
            );
        }
        for z in [-0.09, -0.04, 0.01] {
            v.push(
                part(
                    Body(Head),
                    Box(0.012, 0.09, 0.012),
                    DARK,
                    [s * 0.23, -0.015, z],
                )
                .rot([15.0, 0.0, s * 10.0]),
            );
        }
        v.push(part(
            Body(Hips),
            Box(0.035, 0.22, 0.04),
            ALLY,
            [s * 0.12, -0.06, 0.19],
        ));
    }
    for x in [-0.12, 0.0, 0.12] {
        v.push(
            part(
                Body(Chest),
                Ring(0.012, 0.055, 360.0),
                RUST,
                [x, 0.28, 0.22],
            )
            .rot([90.0, 0.0, 0.0]),
        );
    }
    if f.a {
        for j in [UpperArmL, UpperArmR] {
            v.push(part(Body(j), Pill(0.115, 0.04), BONE, [0.0, 0.0, 0.0]));
            v.push(part(
                Body(j),
                Ring(0.012, 0.11, 240.0),
                RUST,
                [0.0, 0.02, 0.0],
            ));
        }
    }
    if f.b {
        v.push(part(
            Body(Hips),
            Box(0.42, 0.10, 0.30),
            RED_CLOTH,
            [0.0, 0.03, 0.0],
        ));
    }
    weapon(&mut v, f);
    FigureSpec { plan: p, parts: v }
}
fn weapon(v: &mut Vec<PartSpec>, f: Form) {
    let w = Prop(Socket::WeaponR);
    let width = if f.b { 0.25 } else { 0.18 };
    v.extend([
        part(w, Cyl(0.025, 0.95), BARK, [0.0, 0.0, 0.18]).rot([90.0, 0.0, 0.0]),
        part(w, Box(width * 2.0, 0.045, 0.06), RUST, [0.0, 0.0, 0.61]),
        part(w, Cyl(0.032, 0.14), RUST, [0.0, 0.0, 0.50]).rot([90.0, 0.0, 0.0]),
    ]);
    for x in [-width, 0.0, width] {
        v.push(part(w, Cyl(0.018, 0.18), RUST, [x, 0.0, 0.70]).rot([90.0, 0.0, 0.0]));
        v.push(part(w, Cone(0.038, 0.13), BONE, [x, 0.0, 0.85]).rot([90.0, 0.0, 0.0]));
    }
    if f.b && f.t2 {
        for s in [-1.0, 1.0] {
            v.push(
                part(w, Ring(0.018, 0.12, 220.0), RUST, [s * 0.24, 0.0, 0.69]).rot([
                    90.0,
                    0.0,
                    s * 50.0,
                ]),
            );
        }
    }
}
