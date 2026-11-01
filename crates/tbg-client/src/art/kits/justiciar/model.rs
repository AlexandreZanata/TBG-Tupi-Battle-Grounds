//! Justiciar's rigid segmented construct, compact chest sun and five bright/dark judgment forms.
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
    let mut p = body::plan(true, 0.92, t);
    p.arm = 0.22;
    p.torso.x *= 1.1;
    p
}
pub fn parent(n: Node) -> Option<Node> {
    n.parent()
}
pub fn rest(n: Node, p: &Proportions) -> Vec3 {
    match n {
        Secondary(Spring::Back) => Vec3::new(0.0, 0.19, 0.19),
        Secondary(Spring::HeadGear) => Vec3::new(0.0, -0.02, 0.0),
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
            skin: IVORY,
            cloth: NAVY,
            legs: NAVY,
            boots: GOLD,
        },
    );
    v.extend([
        part(
            Body(Chest),
            Box(if f.a { 0.47 } else { 0.39 }, 0.30, 0.14),
            IVORY,
            [0.0, 0.15, 0.12],
        ),
        part(Body(Head), Box(0.23, 0.20, 0.18), IVORY, [0.0, 0.02, 0.0]),
        part(Body(Head), Box(0.16, 0.045, 0.04), NAVY, [0.0, 0.03, 0.11]),
        part(
            Secondary(Spring::Back),
            Cyl(0.085, 0.035),
            GOLD,
            [0.0, 0.0, 0.0],
        )
        .rot([90.0, 0.0, 0.0]),
        part(Secondary(Spring::Back), Ball(0.04), IVORY, [0.0, 0.0, 0.03]).glow(),
        part(
            Body(UpperArmL),
            Box(0.19, 0.08, 0.17),
            ALLY,
            [0.0, 0.0, 0.0],
        ),
    ]);
    for j in [ForearmL, ForearmR] {
        v.push(part(Body(j), Cyl(0.026, 0.23), GUNMETAL, [0.0, -0.08, 0.0]));
        for y in [-0.02, -0.10, -0.18] {
            v.push(part(
                Body(j),
                Box(0.15, 0.055, 0.14),
                if f.b { CHARCOAL } else { IVORY },
                [0.0, y, 0.0],
            ));
            v.push(part(
                Body(j),
                Box(0.16, 0.015, 0.15),
                GOLD,
                [0.0, y + 0.025, 0.0],
            ));
        }
    }
    for i in 0..6 {
        let a = i as f32 * std::f32::consts::TAU / 6.0;
        v.push(
            part(
                Secondary(Spring::Back),
                Box(0.025, 0.065, 0.025),
                GOLD,
                [a.sin() * 0.10, a.cos() * 0.10, 0.0],
            )
            .rot([0.0, 0.0, -a.to_degrees()]),
        );
    }
    if f.a && f.t2 {
        v.push(
            part(
                Secondary(Spring::HeadGear),
                Ring(0.045, 0.24, 310.0),
                GOLD,
                [0.0, 0.0, 0.0],
            )
            .rot([90.0, 0.0, 0.0]),
        );
    }
    if f.b && f.t2 {
        for s in [-1.0, 1.0] {
            v.push(
                part(
                    Body(Chest),
                    Box(0.09, 0.30, 0.12),
                    IVORY,
                    [s * 0.28, 0.25, 0.0],
                )
                .rot([0.0, 0.0, -s * 25.0]),
            );
        }
    }
    FigureSpec { plan: p, parts: v }
}
