//! Trident geometry and grip points share one weapon-local longitudinal axis.

use crate::art::{
    figure::{part, PartSpec},
    kits::common::Form,
    mesh::Shape::*,
    palette::*,
    rig::{Node::Prop, Socket},
};
use bevy::prelude::*;

pub(super) const REAR: f32 = -0.40;
pub(super) const SHAFT_FRONT: f32 = 0.85;
pub(super) const SUPPORT: Vec3 = Vec3::new(0.0, 0.0, 0.18);
pub(super) const TIP: Vec3 = Vec3::new(0.0, 0.0, 1.12);

pub(super) fn width(f: Form) -> f32 {
    if f.b {
        0.23
    } else {
        0.17
    }
}

pub(super) fn build(v: &mut Vec<PartSpec>, f: Form) {
    let w = Prop(Socket::WeaponR);
    let width = width(f);
    v.extend([
        part(
            w,
            Cyl(0.025, SHAFT_FRONT - REAR),
            BARK,
            [0.0, 0.0, (SHAFT_FRONT + REAR) * 0.5],
        )
        .rot([90.0, 0.0, 0.0]),
        part(w, Cyl(0.035, 0.08), RUST, [0.0, 0.0, REAR + 0.04]).rot([90.0, 0.0, 0.0]),
        part(w, Box(width * 2.0, 0.055, 0.075), RUST, [0.0, 0.0, 0.80]),
        part(w, Cyl(0.045, 0.13), BRASS, [0.0, 0.0, 0.72]).rot([90.0, 0.0, 0.0]),
    ]);
    // The wrapped sections identify the two real hand anchors, including in profile.
    for z in [0.0, SUPPORT.z] {
        v.push(part(w, Cyl(0.034, 0.075), DARK_LEATHER, [0.0, 0.0, z]).rot([90.0, 0.0, 0.0]));
    }
    for x in [-width, 0.0, width] {
        v.push(part(w, Cyl(0.020, 0.20), RUST, [x, 0.0, 0.91]).rot([90.0, 0.0, 0.0]));
        v.push(part(w, Cone(0.040, 0.18), BONE, [x, 0.0, 1.03]).rot([90.0, 0.0, 0.0]));
    }
    if f.a && f.t2 {
        v.push(
            part(w, Ring(0.012, 0.046, 360.0), AMBER, [0.0, 0.0, 0.64])
                .rot([90.0, 0.0, 0.0])
                .glow(),
        );
    }
    if f.b && f.t2 {
        for s in [-1.0, 1.0] {
            v.push(
                part(w, Cone(0.032, 0.12), BRASS, [s * width, 0.0, 0.77]).rot([-90.0, 0.0, 0.0]),
            );
        }
    }
}
