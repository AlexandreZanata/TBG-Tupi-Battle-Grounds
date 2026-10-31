//! Capybara's broad belly, rounded muzzle, leaf necklace and five bark/stone/fiber forms.
use crate::art::{
    figure::{part, FigureSpec},
    kits::common::{body, Form},
    mesh::Shape::*,
    palette::*,
    rig::{
        self,
        Joint::*,
        Node,
        Node::{Body, Secondary},
        Proportions, Spring,
    },
};
use bevy::prelude::*;
pub fn plan(t: u8) -> Proportions {
    let mut p = body::plan(true, 1.05, t);
    p.leg = 0.25;
    p.arm = 0.18;
    p.torso.y = 0.32;
    p.torso.x = 0.48;
    p
}
pub fn parent(n: Node) -> Option<Node> {
    match n {
        Secondary(Spring::Tabard) => Some(Body(Chest)),
        _ => n.parent(),
    }
}
pub fn rest(n: Node, p: &Proportions) -> Vec3 {
    match n {
        Body(Head) => Vec3::new(0.0, 0.38, 0.17),
        Secondary(Spring::Tabard) => Vec3::new(0.0, 0.13, 0.14),
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
            skin: if f.a { RUST } else { LEATHER },
            cloth: LEATHER,
            legs: LEATHER,
            boots: RUST,
        },
    );
    v.extend([
        part(
            Body(Chest),
            Ball(0.34),
            if f.a { RUST } else { LEATHER },
            [0.0, 0.16, 0.0],
        )
        .scale([1.2, 1.1, 0.95]),
        part(
            Secondary(Spring::Tabard),
            Ball(0.29),
            if f.a { STONE } else { TRIM },
            [0.0, 0.0, 0.12],
        )
        .scale([1.12, 1.10, 0.65]),
        part(Body(Head), Ball(0.205), LEATHER, [0.0, 0.02, 0.0]).scale([1.1, 0.9, 1.1]),
        part(Body(Head), Pill(0.13, 0.12), TRIM, [0.0, -0.045, 0.20]).rot([0.0, 0.0, 90.0]),
        part(
            Body(Head),
            Box(0.055, 0.055, 0.025),
            LINEN,
            [0.0, -0.14, 0.26],
        ),
        part(
            Body(Head),
            Box(0.09, 0.025, 0.025),
            DARK_LEATHER,
            [0.0, -0.01, 0.32],
        ),
        part(
            Body(Hips),
            Frustum(0.30, 0.34, 0.19),
            if f.b { LINEN } else { MOSS },
            [0.0, -0.06, 0.0],
        ),
        part(
            Secondary(Spring::Back),
            Box(0.25, 0.22, 0.10),
            BARK,
            [0.0, 0.12, -0.15],
        ),
        part(
            Secondary(Spring::Back),
            Box(0.18, 0.14, 0.02),
            ALLY,
            [0.0, 0.12, -0.21],
        ),
    ]);
    for s in [-1.0, 1.0] {
        v.push(part(Body(Head), Ball(0.06), LEATHER, [s * 0.16, 0.17, 0.0]));
        v.push(part(Body(Head), Ball(0.03), RUST, [s * 0.16, 0.18, 0.04]));
        v.push(part(Body(Head), Ball(0.025), DARK, [s * 0.13, 0.11, 0.16]));
        v.push(
            part(Body(Chest), Ball(0.07), LEAF_DARK, [s * 0.20, 0.33, 0.18])
                .scale([0.7, 1.0, 0.35])
                .rot([0.0, 0.0, s * 30.0]),
        );
        for x in [0.05, 0.13, 0.21] {
            v.push(part(
                Body(Hips),
                Box(0.025, 0.20, 0.025),
                if f.b { LINEN } else { BARK },
                [s * x, -0.10, 0.25],
            ));
        }
    }
    if f.a && f.t2 {
        for s in [-1.0, 1.0] {
            v.push(
                part(Body(Chest), Cone(0.10, 0.25), LEAF, [s * 0.24, 0.36, 0.05]).rot([
                    0.0,
                    0.0,
                    -s * 60.0,
                ]),
            );
        }
    }
    if f.b {
        for y in [-0.05, 0.10] {
            v.push(part(
                Secondary(Spring::Tabard),
                Box(0.56, 0.04, 0.06),
                BARK,
                [0.0, y, 0.27],
            ));
        }
        if f.t2 {
            v.push(part(Body(Hips), Cyl(0.38, 0.08), BARK, [0.0, 0.04, 0.0]));
        }
    }
    FigureSpec { plan: p, parts: v }
}
