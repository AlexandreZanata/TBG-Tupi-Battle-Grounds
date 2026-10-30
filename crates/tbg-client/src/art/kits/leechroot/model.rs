//! Leechroot figure: a hunched root-beast with a leech's round toothed maw
//! for a head, a skirt of rootlets and long whip-root hands. Branch A
//! (Insatiable) widens the maw, then swells blood sacs on its back;
//! branch B (Parasite) shows glowing veins, then a blood-bulb heart and
//! maws at the root tips.

use bevy::prelude::*;

use crate::art::{
    figure::{part, FigureSpec, PartSpec},
    kits::common::{body, gear, Form},
    mesh::Shape::*,
    palette::*,
    rig::{Joint::*, Node::Body, Node::Secondary, Proportions, Spring},
};

const ROOT: Color = Color::srgb(0.52, 0.35, 0.24);
const ROOT_DARK: Color = Color::srgb(0.36, 0.24, 0.16);
const GUM: Color = Color::srgb(0.66, 0.22, 0.24);
const BLOOD: Color = Color::srgb(0.85, 0.1, 0.12);

pub fn plan(tier: u8) -> Proportions {
    let mut p = body::plan(false, 0.98, tier);
    p.arm *= 1.3;
    p
}

/// Where the whip root strikes (right hand space).
pub fn tip(p: &Proportions) -> Vec3 {
    Vec3::new(0.0, -p.arm * 1.4, 0.06)
}

pub fn spec(tier: u8, branch: u8) -> FigureSpec {
    let f = Form::of(tier, branch);
    let p = plan(tier);
    let mut v = Vec::with_capacity(130);
    let look = body::Look {
        skin: ROOT,
        cloth: ROOT_DARK,
        legs: ROOT_DARK,
        boots: ROOT_DARK,
    };
    body::humanoid(&mut v, &p, &look);
    maw(&mut v, &p, f);
    roots(&mut v, &p, f);
    let t = p.torso;
    v.push(
        part(
            Body(Chest),
            Box(0.05, t.y * 0.9, t.z * 1.05),
            ALLY,
            [t.x * 0.2, t.y * 0.5, 0.0],
        )
        .rot([0.0, 0.0, -25.0]),
    );
    if f.a && f.t2 {
        let b = Secondary(Spring::Back);
        for (x, y, r) in [(-0.1, 0.05, 0.09), (0.09, 0.1, 0.07), (0.0, -0.08, 0.06)] {
            v.push(part(b, Ball(r), BLOOD, [x, y, -0.06]).glow());
        }
    }
    if f.b {
        for arm in [UpperArmL, UpperArmR, ForearmL, ForearmR] {
            v.push(
                part(
                    Body(arm),
                    Box(0.012, p.arm * 0.7, 0.012),
                    BLOOD,
                    [0.04, -p.arm * 0.45, 0.05],
                )
                .glow(),
            );
        }
        v.push(
            part(
                Body(Chest),
                Box(0.012, t.y * 0.7, 0.012),
                BLOOD,
                [-0.06, t.y * 0.5, t.z * 0.38],
            )
            .rot([0.0, 0.0, 15.0])
            .glow(),
        );
    }
    if f.b && f.t2 {
        v.extend([
            part(
                Body(Chest),
                Ball(0.075),
                BLOOD,
                [0.0, t.y * 0.62, t.z * 0.36],
            )
            .glow(),
            part(
                Body(Chest),
                Ring(0.012, 0.08, 360.0),
                ROOT,
                [0.0, t.y * 0.62, t.z * 0.36],
            )
            .rot([90.0, 0.0, 0.0]),
        ]);
    }
    FigureSpec { plan: p, parts: v }
}

/// The leech head: a long hood of root ending in a round maw ringed with
/// teeth, and a crest of leaves.
fn maw(v: &mut Vec<PartSpec>, p: &Proportions, f: Form) {
    let h = p.head;
    let r = h * if f.a { 0.8 } else { 0.64 };
    let (y, front) = (-h * 0.05, h * 1.5);
    let face = [90.0, 0.0, 0.0];
    v.extend([
        part(Body(Head), Ball(h * 1.1), ROOT, [0.0, 0.0, h * 0.35]).scale([0.95, 1.0, 1.45]),
        part(
            Body(Head),
            Cyl(r * 1.15, 0.06),
            ROOT_DARK,
            [0.0, y, front - 0.03],
        )
        .rot(face),
        part(Body(Head), Cyl(r, 0.04), DARK, [0.0, y, front]).rot(face),
        part(
            Body(Head),
            Ring(0.035, r, 360.0),
            GUM,
            [0.0, y, front + 0.01],
        )
        .rot(face),
        part(
            Body(Head),
            Ring(0.02, r * 0.5, 360.0),
            GUM,
            [0.0, y, front - 0.01],
        )
        .rot(face),
    ]);
    let teeth = if f.a { 12 } else { 8 };
    for i in 0..teeth {
        let a = i as f32 / teeth as f32 * std::f32::consts::TAU;
        let at = [a.cos() * r * 0.8, y + a.sin() * r * 0.8, front + 0.02];
        // Each tooth points at the maw's center.
        v.push(part(Body(Head), Cone(0.02, 0.08), BONE, at).rot([0.0, 0.0, a.to_degrees() + 90.0]));
    }
    for (x, rz) in [(-0.04, 30.0), (0.04, -30.0)] {
        v.push(
            part(
                Body(Head),
                Ball(h * 0.45),
                LEAF_DARK,
                [x, h * 0.95, -h * 0.1],
            )
            .scale([0.5, 1.4, 0.25])
            .rot([-20.0, 0.0, rz]),
        );
    }
}

/// The rootlet skirt and the whip roots trailing from both hands.
fn roots(v: &mut Vec<PartSpec>, p: &Proportions, f: Form) {
    let t = p.torso;
    for i in 0..7 {
        let a = (i as f32 / 7.0) * std::f32::consts::TAU;
        let at = [a.sin() * t.x * 0.42, -0.08, a.cos() * t.z * 0.5];
        v.push(part(Body(Hips), Cone(0.035, 0.16), ROOT_DARK, at).rot([
            180.0 - 20.0 * a.cos(),
            0.0,
            20.0 * a.sin(),
        ]));
    }
    let a = p.arm;
    for hand in [HandL, HandR] {
        let mid = [0.0, -a * 0.6, 0.04];
        let end = [0.0, -a * 1.3, 0.07];
        v.extend([
            gear::strut(Body(hand), [0.0, -0.04, 0.0], mid, 0.03, ROOT),
            gear::strut(Body(hand), mid, end, 0.02, ROOT),
            part(
                Body(hand),
                Cone(0.02, 0.1),
                ROOT_DARK,
                [0.0, -a * 1.38, 0.08],
            )
            .rot([180.0, 0.0, 0.0]),
        ]);
        if f.b && f.t2 {
            v.push(part(
                Body(hand),
                Ring(0.012, 0.035, 360.0),
                GUM,
                [0.0, -a * 1.32, 0.075],
            ));
        }
    }
}
