//! Bloomkeeper figure: a light-wood dryad in a leaf dress and petal skirt,
//! a crown of blossoms in her leafy hair, a staff topped with a flower.
//! Branch A (Blossom) blooms on her shoulders and lights the flower, then
//! carries a blossoming tree on her back; branch B (Bramble) grows bark
//! armor and thorns, then a thorn crown, roots at her feet and an amber
//! thorn staff.

use bevy::prelude::*;

use crate::art::{
    figure::{part, FigureSpec, PartSpec},
    kits::common::{body, gear, Form},
    mesh::Shape::*,
    palette::*,
    rig::{Joint::*, Node, Node::Body, Node::Prop, Node::Secondary, Proportions, Socket, Spring},
};

const DRYAD: Color = Color::srgb(0.8, 0.68, 0.5);
const THORN: Color = Color::srgb(0.24, 0.16, 0.1);
const AMBER: Color = Color::srgb(1.0, 0.72, 0.25);
/// Staff length (socket space, +Y).
const STAFF: f32 = 1.05;

pub fn plan(tier: u8) -> Proportions {
    body::plan(false, 0.96, tier)
}

/// The petals leave the staff's flower (weapon space).
pub fn muzzle() -> Vec3 {
    Vec3::from(gear::staff_top(STAFF)) + Vec3::Y * 0.06
}

pub fn spec(tier: u8, branch: u8) -> FigureSpec {
    let f = Form::of(tier, branch);
    let p = plan(tier);
    let mut v = Vec::with_capacity(150);
    let look = body::Look {
        skin: DRYAD,
        cloth: LEAF,
        legs: BARK,
        boots: BARK,
    };
    body::humanoid(&mut v, &p, &look);
    body::tunic(&mut v, &p, LEAF, LEAF_DARK, BARK);
    let t = p.torso;
    for i in 0..6 {
        let a = i as f32 / 6.0 * std::f32::consts::TAU;
        v.push(
            part(
                Body(Hips),
                Ball(0.07),
                BLOSSOM,
                [a.sin() * t.x * 0.5, -0.12, a.cos() * t.z * 0.6],
            )
            .scale([1.0, 1.6, 0.5])
            .rot([0.0, a.to_degrees(), 0.0]),
        );
    }
    v.push(
        part(
            Body(Chest),
            Box(t.x * 1.0, 0.05, t.z * 0.9),
            ALLY,
            [0.0, t.y * 0.4, 0.02],
        )
        .rot([0.0, 0.0, 15.0]),
    );
    hair(&mut v, &p, f);
    staff(&mut v, f);
    if f.a {
        for arm in [UpperArmL, UpperArmR] {
            flower(&mut v, Body(arm), [0.0, 0.08, 0.0], 0.8, f.t2);
        }
    }
    if f.a && f.t2 {
        let b = Secondary(Spring::Back);
        v.push(part(b, Cyl(0.035, 0.5), BARK, [0.0, 0.25, -0.08]));
        for (x, y, r) in [(0.0, 0.58, 0.14), (-0.12, 0.48, 0.1), (0.12, 0.5, 0.1)] {
            v.push(part(b, Ball(r), BLOSSOM, [x, y, -0.08]));
        }
        v.push(part(b, Ball(0.03), SAP, [0.06, 0.62, 0.0]).glow());
    }
    if f.b {
        body::pauldrons(&mut v, BARK, 0.1);
        for (node, at) in [
            (UpperArmL, [0.06, 0.06, 0.0]),
            (UpperArmR, [-0.06, 0.06, 0.0]),
            (ForearmL, [0.05, -0.06, 0.0]),
            (ForearmR, [-0.05, -0.06, 0.0]),
        ] {
            v.push(part(Body(node), Cone(0.022, 0.09), THORN, at).rot([
                0.0,
                0.0,
                -70.0 * at[0].signum(),
            ]));
        }
    }
    if f.b && f.t2 {
        for foot in [FootL, FootR] {
            for x in [-0.06, 0.06] {
                v.push(
                    part(Body(foot), Cone(0.025, 0.12), BARK, [x, -0.04, 0.12]).rot([
                        100.0,
                        0.0,
                        30.0 * x.signum(),
                    ]),
                );
            }
        }
    }
    FigureSpec { plan: p, parts: v }
}

/// Leafy hair with a crown of blossoms; Thornwarden's crown is thorns.
fn hair(v: &mut Vec<PartSpec>, p: &Proportions, f: Form) {
    let h = p.head;
    v.extend([
        part(
            Body(Head),
            Ball(h * 1.1),
            LEAF_DARK,
            [0.0, h * 0.2, -h * 0.2],
        )
        .scale([1.05, 0.9, 1.0]),
        part(
            Secondary(Spring::HeadGear),
            Ball(h * 0.6),
            LEAF_DARK,
            [0.0, -h * 0.4, -h * 0.9],
        )
        .scale([1.0, 1.6, 0.5]),
    ]);
    for a in [-60.0f32, -30.0, 0.0, 30.0, 60.0] {
        let r = a.to_radians();
        let at = [r.sin() * h * 0.95, h * 0.7, r.cos() * h * 0.7];
        if f.b && f.t2 {
            v.push(part(Body(Head), Cone(h * 0.12, h * 0.5), THORN, at).rot([
                r.cos() * 20.0,
                0.0,
                -r.sin() * 30.0,
            ]));
        } else {
            flower(v, Body(Head), at, 0.55, false);
        }
    }
}

/// A five-petal flower with a sap heart, glowing when `lit`.
fn flower(v: &mut Vec<PartSpec>, at: Node, [x, y, z]: [f32; 3], s: f32, lit: bool) {
    for i in 0..5 {
        let a = i as f32 / 5.0 * std::f32::consts::TAU;
        v.push(
            part(
                at,
                Ball(0.035 * s),
                BLOSSOM,
                [x + a.sin() * 0.04 * s, y, z + a.cos() * 0.04 * s],
            )
            .scale([1.0, 0.5, 1.0]),
        );
    }
    let heart = part(at, Ball(0.022 * s), SAP, [x, y + 0.01 * s, z]);
    v.push(if lit { heart.glow() } else { heart });
}

/// The flower staff; Bramble winds a thorny vine round it.
fn staff(v: &mut Vec<PartSpec>, f: Form) {
    let w = Prop(Socket::WeaponR);
    v.extend(gear::staff(w, STAFF, BARK));
    let top = gear::staff_top(STAFF);
    let [x, y, z] = top;
    if f.b && f.t2 {
        v.extend([
            part(w, Ball(0.06), AMBER, [x, y + 0.04, z]).glow(),
            part(w, Ring(0.012, 0.07, 360.0), THORN, [x, y + 0.04, z]),
        ]);
        for a in [0.0f32, 120.0, 240.0] {
            let r = a.to_radians();
            v.push(
                part(
                    w,
                    Cone(0.02, 0.1),
                    THORN,
                    [x + r.sin() * 0.07, y + 0.08, z + r.cos() * 0.07],
                )
                .rot([r.cos() * 30.0, 0.0, -r.sin() * 30.0]),
            );
        }
    } else {
        flower(v, w, [x, y + 0.04, z], if f.a { 1.6 } else { 1.3 }, f.a);
    }
    if f.b {
        for k in [0.2f32, 0.4, 0.6] {
            v.push(
                part(
                    w,
                    Ring(0.01, 0.035, 360.0),
                    LEAF_DARK,
                    [x, STAFF * k - STAFF * 0.2, z],
                )
                .rot([15.0, 0.0, 10.0]),
            );
            v.push(
                part(
                    w,
                    Cone(0.012, 0.05),
                    THORN,
                    [x + 0.035, STAFF * k - STAFF * 0.2, z],
                )
                .rot([0.0, 0.0, -90.0]),
            );
        }
    }
}
