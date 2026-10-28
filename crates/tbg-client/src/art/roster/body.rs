//! Shared v2 blockout anatomy; final characters supply their own costume and props.

use crate::art::{
    figure::{part, PartSpec},
    mesh::Shape::*,
    rig::{Joint::*, Node::Body, Proportions},
};
use bevy::prelude::*;

pub fn parts(p: &Proportions, cloth: Color, skin: Color, boots: Color) -> Vec<PartSpec> {
    let t = p.torso;
    let mut parts = vec![
        part(
            Body(Hips),
            Box(t.x * 0.87, 0.13, t.z),
            cloth,
            [0.0, 0.0, 0.0],
        ),
        part(
            Body(Chest),
            Frustum(t.x * 0.53, t.x * 0.43, t.y),
            cloth,
            [0.0, t.y * 0.5, 0.0],
        )
        .scale([1.0, 1.0, 0.72]),
        part(Body(Head), Ball(p.head), skin, [0.0, 0.0, 0.0]),
    ];
    for (arm, fore, hand, thigh, shin, foot) in [
        (UpperArmL, ForearmL, HandL, ThighL, ShinL, FootL),
        (UpperArmR, ForearmR, HandR, ThighR, ShinR, FootR),
    ] {
        parts.extend([
            part(
                Body(arm),
                Frustum(0.07, 0.055, p.arm),
                cloth,
                [0.0, -p.arm * 0.5, 0.0],
            ),
            part(
                Body(fore),
                Frustum(0.055, 0.045, p.arm * 0.9),
                cloth,
                [0.0, -p.arm * 0.45, 0.0],
            ),
            part(Body(hand), Box(0.10, 0.11, 0.12), skin, [0.0, -0.025, 0.0]),
            part(
                Body(thigh),
                Box(0.12, p.leg * 0.5, 0.14),
                cloth,
                [0.0, -p.leg * 0.25, 0.0],
            ),
            part(
                Body(shin),
                Frustum(0.065, 0.055, p.leg * 0.5),
                boots,
                [0.0, -p.leg * 0.25, 0.0],
            ),
            part(
                Body(foot),
                Box(0.14, 0.08, 0.22),
                boots,
                [0.0, -0.02, 0.045],
            ),
        ]);
    }
    parts
}
