//! The verdict lance: one weapon-local axis (+Z to the point) shared by geometry and grips.

use crate::art::{
    figure::{part, PartSpec},
    kits::common::Form,
    mesh::Shape::*,
    palette::*,
    rig::{Node::Prop, Socket},
};
use bevy::prelude::*;

/// Butt behind the rear hand; most of the long shaft trails the grip so the
/// thrust reaches the 2.2-tile target without spearing far past it.
pub(super) const REAR: f32 = -0.78;
pub(super) const SHAFT_FRONT: f32 = 0.52;
/// The leading hand, ahead of the rear grip at the origin.
pub(super) const SUPPORT: Vec3 = Vec3::new(0.0, 0.0, 0.20);

pub(super) fn blade(f: Form) -> f32 {
    match (f.a, f.t2) {
        (true, true) => 0.40,
        (true, false) => 0.34,
        _ if f.b && f.t2 => 0.36,
        _ => 0.28,
    }
}

/// The point, for impacts and the reach fit.
pub(super) fn tip(f: Form) -> Vec3 {
    Vec3::new(0.0, 0.0, SHAFT_FRONT + blade(f))
}

pub(super) fn build(v: &mut Vec<PartSpec>, f: Form) {
    let w = Prop(Socket::WeaponR);
    let metal = if f.b { CRIMSON } else { GOLD };
    let shaft = if f.b { GRAPHITE } else { IVORY };
    let len = blade(f);
    v.extend([
        part(
            w,
            Cyl(0.026, SHAFT_FRONT - REAR),
            shaft,
            [0.0, 0.0, (SHAFT_FRONT + REAR) * 0.5],
        )
        .rot([90.0, 0.0, 0.0]),
        part(w, Cone(0.05, 0.12), GOLD, [0.0, 0.0, REAR - 0.04]).rot([-90.0, 0.0, 0.0]),
        part(w, Cyl(0.045, 0.07), GOLD, [0.0, 0.0, SHAFT_FRONT - 0.02]).rot([90.0, 0.0, 0.0]),
        // A broad faceted leaf point: a flattened prism read edge-on from the lane.
        part(
            w,
            Cone(0.085, len),
            STEEL,
            [0.0, 0.0, SHAFT_FRONT + len * 0.5],
        )
        .rot([90.0, 0.0, 0.0])
        .scale([1.0, 1.0, 0.38]),
        part(
            w,
            Prism(0.03, len * 0.75),
            metal,
            [0.0, 0.0, SHAFT_FRONT + len * 0.42],
        )
        .rot([90.0, 0.0, 0.0])
        .scale([1.0, 1.0, 0.5]),
    ]);
    for z in [0.0, SUPPORT.z] {
        v.push(part(w, Cyl(0.034, 0.075), DARK_LEATHER, [0.0, 0.0, z]).rot([90.0, 0.0, 0.0]));
    }
    // Wings below the point grow with Evolution; A2 and B2 add a glowing core.
    let wing = if f.t2 {
        0.13
    } else if f.a || f.b {
        0.10
    } else {
        0.07
    };
    for s in [-1.0f32, 1.0] {
        v.push(
            part(
                w,
                Prism(0.022, wing),
                metal,
                [s * (0.05 + wing * 0.4), 0.0, SHAFT_FRONT + 0.03],
            )
            .rot([0.0, 0.0, -s * 60.0]),
        );
    }
    if f.t2 {
        let core = if f.b { RED_CLOTH } else { STORM_BLUE };
        v.push(part(w, Ball(0.03), core, [0.0, 0.0, SHAFT_FRONT + len * 0.30]).glow());
        v.push(
            part(
                w,
                Ring(0.01, 0.06, 360.0),
                metal,
                [0.0, 0.0, SHAFT_FRONT + 0.10],
            )
            .rot([90.0, 0.0, 0.0]),
        );
    }
}
