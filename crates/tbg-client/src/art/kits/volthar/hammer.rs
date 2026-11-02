//! The rune hammer: grip at the origin, head along weapon-local +Z, sparks on their own socket.

use crate::art::{
    figure::{part, PartSpec},
    kits::common::Form,
    mesh::Shape::*,
    palette::*,
    rig::{Node::Prop, Socket},
};
use bevy::prelude::*;

pub(super) const REAR: f32 = -0.16;
pub(super) const HEAD_Z: f32 = 0.66;
/// The off hand's second grip, below the head on the same axis.
pub(super) const SUPPORT: Vec3 = Vec3::new(0.0, 0.0, 0.26);
/// Where bolts leave: the head's striking face.
pub(super) const MUZZLE: Vec3 = Vec3::new(0.0, 0.0, HEAD_Z + 0.12);

fn head_size(f: Form) -> Vec3 {
    let grow = if f.t2 { 1.12 } else { 1.0 };
    let base = if f.b {
        Vec3::new(0.36, 0.22, 0.22)
    } else {
        Vec3::new(0.34, 0.21, 0.21)
    };
    base * grow
}

pub(super) fn build(v: &mut Vec<PartSpec>, f: Form) {
    let w = Prop(Socket::WeaponR);
    let head = head_size(f);
    let iron = if f.b { GRAPHITE } else { STEEL };
    v.extend([
        part(
            w,
            Cyl(0.03, HEAD_Z - REAR),
            DARK_LEATHER,
            [0.0, 0.0, (HEAD_Z + REAR) * 0.5],
        )
        .rot([90.0, 0.0, 0.0]),
        part(w, Cyl(0.045, 0.06), GOLD, [0.0, 0.0, REAR]).rot([90.0, 0.0, 0.0]),
        part(w, Cyl(0.04, 0.05), GOLD, [0.0, 0.0, HEAD_Z - head.z * 0.62]).rot([90.0, 0.0, 0.0]),
        // The head lies across the shaft, striking faces along ±X.
        part(
            w,
            Bevel(head.x, head.y, head.z, 0.02),
            iron,
            [0.0, 0.0, HEAD_Z],
        ),
        part(
            w,
            Bevel(head.x * 0.25, head.y * 1.08, head.z * 1.08, 0.012),
            GOLD,
            [0.0, 0.0, HEAD_Z],
        ),
    ]);
    for s in [-1.0f32, 1.0] {
        v.push(part(
            w,
            Bevel(0.035, head.y * 1.1, head.z * 1.1, 0.008),
            GOLD,
            [s * head.x * 0.47, 0.0, HEAD_Z],
        ));
        // Lightning rune on both broad faces.
        v.push(
            part(
                w,
                Prism(0.03, head.y * 0.75),
                STORM_BLUE,
                [s * head.x * 0.22, s * head.y * 0.52, HEAD_Z],
            )
            .rot([0.0, 0.0, 25.0])
            .glow(),
        );
    }
    // Grip wraps mark both hand anchors.
    for z in [0.0, SUPPORT.z] {
        v.push(part(w, Cyl(0.038, 0.08), BRASS, [0.0, 0.0, z]).rot([90.0, 0.0, 0.0]));
    }
    if f.a {
        v.push(
            part(
                w,
                Ring(0.012, head.y * 0.62, 360.0),
                STORM_BLUE,
                [0.0, 0.0, HEAD_Z + head.z * 0.62],
            )
            .glow(),
        );
    }
    if f.b {
        for s in [-1.0f32, 1.0] {
            v.push(
                part(
                    w,
                    Cone(0.05, 0.14),
                    GOLD,
                    [s * (head.x * 0.5 + 0.06), 0.0, HEAD_Z],
                )
                .rot([0.0, 0.0, -s * 90.0]),
            );
        }
        if f.t2 {
            v.push(
                part(
                    w,
                    Cone(0.05, 0.16),
                    GOLD,
                    [0.0, 0.0, HEAD_Z + head.z * 0.55 + 0.07],
                )
                .rot([90.0, 0.0, 0.0]),
            );
        }
    }
    sparks(v, f);
}

/// A crackling cluster around the head; motion spins and jitters it.
fn sparks(v: &mut Vec<PartSpec>, f: Form) {
    let s = Prop(Socket::OffhandL);
    let count = if f.t2 { 4 } else { 3 };
    for i in 0..count {
        let a = i as f32 / count as f32 * std::f32::consts::TAU;
        v.push(
            part(
                s,
                Prism(0.012, 0.11),
                STORM_BLUE,
                [a.cos() * 0.27, a.sin() * 0.17, 0.0],
            )
            .rot([0.0, 0.0, a.to_degrees() + 60.0])
            .glow(),
        );
    }
}
