//! Bulwark's stone dome, iron bands, compact walker and five fortress/tool forms.

use crate::art::{
    figure::{part, FigureSpec, PartSpec},
    kits::common::{body, Form},
    mesh::Shape::*,
    palette::*,
    rig::{
        Joint::*,
        Node::{Body, Prop},
        Proportions, Socket,
    },
};
use bevy::prelude::*;
const DOME: &[[f32; 2]] = &[
    [0.40, -0.15],
    [0.44, 0.0],
    [0.43, 0.20],
    [0.36, 0.38],
    [0.20, 0.50],
    [0.0, 0.55],
];
pub fn plan(tier: u8) -> Proportions {
    let mut p = body::plan(true, 0.80, tier);
    p.leg = 0.30;
    p.arm = 0.19;
    p.torso.y = 0.29;
    p
}
pub fn tip(tier: u8, branch: u8) -> Vec3 {
    Vec3::new(
        0.0,
        -0.22,
        if Form::of(tier, branch).b { 0.13 } else { 0.10 },
    )
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
            cloth: DARK_LEATHER,
            legs: IRON,
            boots: GUNMETAL,
        },
    );
    v.extend([
        part(
            Body(Chest),
            Lathe(DOME),
            if f.a { LINEN } else { STONE },
            [0.0, 0.23, -0.16],
        ),
        part(
            Body(Head),
            Box(0.22, 0.15, 0.10),
            GUNMETAL,
            [0.0, -0.03, 0.32],
        ),
        part(Body(Head), Box(0.23, 0.045, 0.12), IRON, [0.0, 0.045, 0.34]),
        part(
            Body(Head),
            Box(0.16, 0.025, 0.025),
            DARK,
            [0.0, -0.07, 0.38],
        ),
    ]);
    for s in [-1.0, 1.0] {
        v.push(part(Body(Head), Ball(0.023), ORANGE, [s * 0.055, -0.02, 0.38]).glow());
        v.extend([
            part(
                Body(Chest),
                Box(0.055, 0.42, 0.05),
                OAK,
                [s * 0.20, 0.26, 0.27],
            )
            .rot([8.0, 0.0, s * -8.0]),
            part(
                Body(Chest),
                Box(0.035, 0.34, 0.28),
                ALLY,
                [s * 0.43, 0.27, -0.12],
            )
            .rot([0.0, 0.0, s * -12.0]),
            part(
                Body(Chest),
                Box(0.045, 0.03, 0.29),
                LINEN,
                [s * 0.46, 0.10, -0.12],
            ),
        ]);
    }
    for (y, r) in [(0.16, 0.43), (0.61, 0.37)] {
        v.push(part(
            Body(Chest),
            Ring(0.027, r, 360.0),
            GUNMETAL,
            [0.0, y, -0.16],
        ));
        for a in [0.0_f32, 60.0, 120.0, 180.0, 240.0, 300.0] {
            let a = a.to_radians();
            v.push(part(
                Body(Chest),
                Ball(0.032),
                BRASS,
                [a.cos() * r, y, a.sin() * r - 0.16],
            ));
        }
    }
    upgrades(&mut v, f);
    tool(&mut v, f);
    FigureSpec { plan: p, parts: v }
}
fn upgrades(v: &mut Vec<PartSpec>, f: Form) {
    if f.a {
        for s in [-1.0, 1.0] {
            v.push(
                part(
                    Body(Chest),
                    Box(0.15, 0.36, 0.30),
                    STONE,
                    [s * 0.34, 0.06, -0.23],
                )
                .rot([0.0, 0.0, s * -12.0]),
            );
            v.push(part(
                Body(Chest),
                Box(0.16, 0.045, 0.32),
                IRON,
                [s * 0.36, -0.05, -0.23],
            ));
        }
        if f.t2 {
            for x in [-0.20, 0.0, 0.20] {
                v.push(part(
                    Body(Chest),
                    Box(0.14, 0.20, 0.17),
                    LINEN,
                    [x, 0.75, -0.20],
                ));
            }
            v.push(part(
                Body(Chest),
                Box(0.60, 0.08, 0.19),
                IRON,
                [0.0, 0.65, -0.20],
            ));
        }
    }
    if f.b {
        for j in [ForearmL, ForearmR] {
            v.push(part(Body(j), Cyl(0.105, 0.19), IRON, [0.0, -0.07, 0.0]));
            v.push(part(
                Body(j),
                Ring(0.016, 0.11, 360.0),
                BRASS,
                [0.0, -0.15, 0.0],
            ));
        }
    }
}
fn tool(v: &mut Vec<PartSpec>, f: Form) {
    let n = Prop(Socket::WeaponR);
    v.extend([
        part(n, Cyl(0.035, 0.47), OAK, [0.0, -0.06, 0.0]),
        part(
            n,
            Box(if f.b { 0.31 } else { 0.22 }, 0.15, 0.23),
            STONE,
            [0.0, -0.25, 0.04],
        ),
        part(n, Box(0.065, 0.17, 0.25), IRON, [0.0, -0.25, 0.04]),
        part(n, Cyl(0.045, 0.07), BRASS, [0.0, 0.18, 0.0]),
    ]);
    if f.b && f.t2 {
        v.push(part(n, Box(0.37, 0.045, 0.30), IRON, [0.0, -0.35, 0.07]));
    }
}
