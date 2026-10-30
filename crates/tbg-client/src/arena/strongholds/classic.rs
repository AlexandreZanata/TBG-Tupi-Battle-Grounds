//! Classic: the reference round tower the tests and the balance lab use.
//! It has no Fortification forms; its crown hides at <= 66 % HP and its
//! middle at <= 33 %.

use super::parts::{whole, PartDef, Piece};
use crate::arena::Part;
use crate::art::{mesh::Shape::*, palette::*};
use bevy::prelude::*;

const BASE_PROFILE: &[[f32; 2]] = &[[1.0, 0.0], [1.0, 0.18], [0.82, 0.24], [0.76, 0.95]];
const MID_PROFILE: &[[f32; 2]] = &[[0.76, 0.95], [0.7, 1.65], [0.84, 1.75], [0.84, 1.85]];

static BASE: PartDef = PartDef {
    name: "classic base",
    build: base,
};
static MID: PartDef = PartDef {
    name: "classic middle",
    build: mid,
};
static CROWN: PartDef = PartDef {
    name: "classic crown",
    build: crown,
};

pub(super) fn pieces() -> Vec<Piece> {
    vec![whole(&BASE), whole(&MID).lost(2), whole(&CROWN).lost(1)]
}

fn at(y: f32) -> Transform {
    Transform::from_xyz(0.0, y, 0.0)
}

fn base(_team: Color) -> Vec<Part> {
    let mut parts = vec![
        (Lathe(BASE_PROFILE), STONE, at(0.0)),
        (Cyl(0.76, 0.02), STONE, at(0.95)),
        (
            Box(0.32, 0.5, 0.06),
            WOOD,
            Transform::from_xyz(0.78, 0.25, 0.0).with_rotation(Quat::from_rotation_y(1.57)),
        ),
    ];
    for y in [0.24, 0.59, 0.94] {
        parts.push((
            Ring(0.016, if y < 0.3 { 0.83 } else { 0.77 }, 360.0),
            ROCK,
            at(y),
        ));
    }
    parts
}

fn mid(team: Color) -> Vec<Part> {
    vec![
        (Lathe(MID_PROFILE), STONE, at(0.0)),
        (Cyl(0.84, 0.02), ROCK, at(1.85)),
        (
            Box(0.44, 0.65, 0.035),
            team,
            Transform::from_xyz(0.0, 1.25, 0.77),
        ),
    ]
}

fn crown(_team: Color) -> Vec<Part> {
    let mut parts = vec![(
        Box(0.13, 0.13, 0.025),
        LINEN,
        Transform::from_xyz(0.0, 1.30, 0.80).with_rotation(Quat::from_rotation_z(0.785)),
    )];
    for i in 0..8 {
        let a = i as f32 / 8.0 * std::f32::consts::TAU;
        parts.push((
            Box(0.22, 0.24, 0.22),
            STONE,
            Transform::from_xyz(a.cos() * 0.72, 1.97, a.sin() * 0.72),
        ));
    }
    parts.push((Cyl(0.03, 0.9), WOOD, at(2.3)));
    parts
}
