//! Demolisher's hydraulic cuffs, near-ground fists, pressure tank and five industrial forms.

use crate::art::{
    figure::{part, FigureSpec, PartSpec},
    kits::common::{body, Form},
    mesh::Shape::*,
    palette::*,
    rig::{
        Joint::*,
        Node::{Body, Secondary},
        Proportions, Spring,
    },
};
use bevy::prelude::*;
pub fn plan(tier: u8) -> Proportions {
    let mut p = body::plan(true, 0.80, tier);
    p.leg = 0.32;
    p.arm = 0.25;
    p.torso.y = 0.34;
    p
}
pub fn spec(tier: u8, branch: u8) -> FigureSpec {
    let p = plan(tier);
    let f = Form::of(tier, branch);
    let mut v = Vec::new();
    body::humanoid(
        &mut v,
        &p,
        &body::Look {
            skin: GUNMETAL,
            cloth: IRON,
            legs: DARK_LEATHER,
            boots: GUNMETAL,
        },
    );
    v.extend([
        part(
            Body(Chest),
            Box(0.35, 0.18, 0.08),
            GUNMETAL,
            [0.0, 0.20, 0.16],
        ),
        part(Body(Chest), Box(0.14, 0.21, 0.025), ALLY, [0.0, 0.14, 0.21]),
        part(
            Body(Hips),
            Box(0.29, 0.06, 0.24),
            DARK_LEATHER,
            [0.0, 0.02, 0.0],
        ),
        part(Body(Head), Box(0.24, 0.13, 0.11), IRON, [0.0, -0.025, 0.08]),
        part(
            Body(Head),
            Box(0.25, 0.045, 0.09),
            GUNMETAL,
            [0.0, 0.035, 0.13],
        ),
        part(
            Body(Head),
            Box(0.15, 0.025, 0.02),
            ORANGE,
            [0.0, -0.01, 0.145],
        )
        .glow(),
        part(
            Secondary(Spring::Back),
            Cyl(0.13, 0.34),
            RUST,
            [0.0, 0.05, -0.09],
        ),
        part(
            Secondary(Spring::Back),
            Cyl(0.14, 0.045),
            BRASS,
            [0.0, 0.22, -0.09],
        ),
        part(
            Secondary(Spring::Back),
            Cyl(0.14, 0.045),
            BRASS,
            [0.0, -0.12, -0.09],
        ),
    ]);
    for (upper, fore, hand, s) in [
        (UpperArmL, ForearmL, HandL, 1.0),
        (UpperArmR, ForearmR, HandR, -1.0),
    ] {
        v.push(part(Body(upper), Ball(0.16), GUNMETAL, [0.0, -0.01, 0.0]).scale([1.1, 0.75, 1.0]));
        cuff(&mut v, fore, hand, s, f);
    }
    if f.b && f.t2 {
        for y in [-0.02, 0.05, 0.12, 0.19] {
            v.push(part(
                Secondary(Spring::Back),
                Ring(0.017, 0.085, 360.0),
                BRASS,
                [-0.22, y, -0.09],
            ));
        }
        v.extend([
            part(
                Secondary(Spring::Back),
                Cyl(0.05, 0.34),
                IRON,
                [-0.22, 0.08, -0.09],
            ),
            part(
                Secondary(Spring::Back),
                Ball(0.05),
                ORANGE,
                [-0.22, 0.28, -0.09],
            )
            .glow(),
        ]);
    }
    FigureSpec { plan: p, parts: v }
}
fn cuff(
    v: &mut Vec<PartSpec>,
    fore: crate::art::rig::Joint,
    hand: crate::art::rig::Joint,
    s: f32,
    f: Form,
) {
    v.extend([
        part(
            Body(fore),
            Frustum(0.15, 0.13, 0.23),
            GUNMETAL,
            [0.0, -0.10, 0.0],
        ),
        part(Body(fore), Cyl(0.16, 0.045), BRASS, [0.0, 0.02, 0.0]),
        part(Body(fore), Cyl(0.145, 0.04), IRON, [0.0, -0.22, 0.0]),
        part(
            Body(hand),
            Box(0.28, 0.25, 0.29),
            GUNMETAL,
            [0.0, -0.08, 0.03],
        ),
        part(Body(hand), Box(0.29, 0.07, 0.12), BRASS, [0.0, -0.04, 0.19]),
    ]);
    for x in [-0.09, 0.09] {
        v.push(part(Body(fore), Cyl(0.022, 0.24), IRON, [x, -0.09, 0.13]));
        v.push(part(Body(hand), Ball(0.029), BRASS, [x, -0.04, 0.25]));
    }
    if f.a {
        v.push(part(
            Body(hand),
            Box(0.32, 0.18, 0.07),
            IRON,
            [0.0, -0.08, 0.22],
        ));
        if f.t2 {
            v.push(part(
                Body(hand),
                Box(0.37, 0.10, 0.38),
                GUNMETAL,
                [0.0, -0.20, 0.10],
            ));
        }
    }
    if f.b {
        v.extend([
            part(
                Body(fore),
                Box(0.06, 0.19, 0.20),
                RUST,
                [s * 0.16, -0.10, 0.0],
            ),
            part(
                Body(fore),
                Ring(0.025, 0.075, 360.0),
                ORANGE,
                [s * 0.20, -0.10, 0.0],
            )
            .rot([0.0, 0.0, 90.0]),
            part(
                Body(fore),
                Box(0.04, 0.025, 0.12),
                BRASS,
                [s * 0.22, -0.10, 0.0],
            ),
        ]);
    }
}
