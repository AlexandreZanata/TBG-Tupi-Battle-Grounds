//! Shared ogre anatomy and material progression; each kit owns its equipment silhouette.
pub(super) use super::super::common::{body, Form};
pub(super) use crate::art::{
    figure::{part, FigureSpec, PartSpec},
    mesh::Shape::*,
    palette::*,
    rig::{
        Joint::*,
        Node::{Body, Prop, Secondary},
        Proportions, Socket, Spring,
    },
};
pub(super) use bevy::prelude::*;
pub fn plan(t: u8, size: f32, leg: f32, width: f32) -> Proportions {
    let mut p = body::plan(false, size, t);
    p.leg = leg;
    p.torso.x = width;
    p.arm = 0.22;
    p.head = 0.15;
    p
}
pub fn ogre(p: &Proportions, skin: Color, cloth: Color) -> Vec<PartSpec> {
    let mut v = Vec::new();
    body::humanoid(
        &mut v,
        p,
        &body::Look {
            skin,
            cloth,
            legs: DARK_LEATHER,
            boots: DARK_LEATHER,
        },
    );
    v.push(part(
        Body(Head),
        Bevel(0.29, 0.15, 0.17, 0.025),
        skin,
        [0.0, -0.055, 0.09],
    ));
    v.push(part(
        Body(Head),
        Bevel(0.065, 0.07, 0.065, 0.015),
        skin,
        [0.0, 0.045, 0.17],
    ));
    for (upper, fore, hand) in [(UpperArmL, ForearmL, HandL), (UpperArmR, ForearmR, HandR)] {
        v.push(part(
            Body(upper),
            Bevel(0.15, 0.19, 0.15, 0.035),
            skin,
            [0.0, -0.085, 0.0],
        ));
        v.push(part(
            Body(fore),
            Bevel(0.15, 0.16, 0.14, 0.03),
            skin,
            [0.0, -0.075, 0.01],
        ));
        v.push(part(
            Body(hand),
            Bevel(0.13, 0.10, 0.14, 0.025),
            skin,
            [0.0, -0.015, 0.02],
        ));
    }
    for s in [-1.0, 1.0] {
        v.push(
            part(
                Body(Head),
                Bevel(0.08, 0.035, 0.05, 0.01),
                skin,
                [s * 0.065, 0.075, 0.14],
            )
            .rot([0.0, 0.0, s * 12.0]),
        );
        v.push(
            part(
                Body(Head),
                Cone(0.03, 0.12),
                IVORY,
                [s * 0.105, 0.005, 0.20],
            )
            .rot([0.0, 0.0, s * 12.0]),
        );
        v.push(
            part(Body(Head), Cone(0.055, 0.15), skin, [s * 0.19, 0.035, 0.0]).rot([
                0.0,
                0.0,
                s * -75.0,
            ]),
        );
    }
    v
}
pub fn cloth(f: Form, base: Color) -> Color {
    if f.a {
        LINEN
    } else if f.b {
        CHARCOAL
    } else {
        base
    }
}
pub fn tier(v: &mut Vec<PartSpec>, f: Form, at: [f32; 3], shape: crate::art::mesh::Shape) {
    if f.a || f.b {
        v.push(part(
            Body(UpperArmL),
            Bevel(0.17, 0.09, 0.17, 0.02),
            if f.a { STONE } else { RUST },
            [0.0, -0.03, 0.0],
        ));
    }
    if f.t2 {
        v.push(part(Body(Chest), shape, if f.a { GOLD } else { AMBER }, at).glow());
    }
}
