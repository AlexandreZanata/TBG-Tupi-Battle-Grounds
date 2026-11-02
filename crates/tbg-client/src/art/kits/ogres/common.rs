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
    p.arm = 0.235;
    p.head = 0.145;
    p
}
pub fn ogre(p: &Proportions, skin: Color, cloth: Color) -> Vec<PartSpec> {
    super::anatomy::parts(p, skin, cloth)
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
