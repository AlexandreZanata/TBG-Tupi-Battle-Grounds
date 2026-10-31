//! Cargo Crab figure: a brass-and-gunmetal walking crab of the Ironworks
//! with a passenger saddle on its shell, eyes on stalks, two pincers and
//! six legs that ride the hip joints so they scuttle with the gait.
//! Branch A (Armored Carrier) layers shell plates, then a glowing shield
//! dome over the saddle; branch B (Turbo Carrier) adds exhausts and leg
//! pistons, then lit afterburners and a fin.

use bevy::prelude::*;

use crate::art::{
    figure::{part, FigureSpec, PartSpec},
    kits::common::{body, gear, Form},
    mesh::Shape::*,
    palette::*,
    rig::{Joint::*, Node::Body, Node::Secondary, Proportions, Spring},
};

const HOT: Color = Color::srgb(1.0, 0.55, 0.18);

pub fn plan(tier: u8) -> Proportions {
    let mut p = body::plan(true, 0.75, tier);
    p.leg = 0.26;
    p.torso = Vec3::new(0.62, 0.18, 0.5);
    p.head = 0.08;
    p.arm = 0.2;
    p
}

/// Where the right pincer closes (hand space).
pub fn tip() -> Vec3 {
    Vec3::new(0.0, -0.26, 0.0)
}

pub fn spec(tier: u8, branch: u8) -> FigureSpec {
    let f = Form::of(tier, branch);
    let p = plan(tier);
    let mut v = Vec::with_capacity(130);
    shell(&mut v, f);
    saddle(&mut v, f);
    for s in [-1.0f32, 1.0] {
        legs(&mut v, &p, f, s);
        claw(&mut v, &p, f, s);
    }
    exhausts(&mut v, f);
    FigureSpec { plan: p, parts: v }
}

fn shell(v: &mut Vec<PartSpec>, f: Form) {
    let h = Body(Hips);
    v.extend([
        part(h, Ball(0.36), GUNMETAL, [0.0, 0.05, 0.0]).scale([1.15, 0.45, 0.95]),
        part(h, Ball(0.33), DARK, [0.0, -0.01, 0.0]).scale([1.12, 0.3, 0.9]),
        part(h, Ring(0.022, 0.37, 360.0), BRASS, [0.0, 0.03, 0.0]).scale([1.12, 1.0, 0.93]),
        part(h, Box(0.26, 0.07, 0.05), BRASS, [0.0, 0.0, 0.33]),
        part(h, Box(0.05, 0.05, 0.03), DARK, [-0.07, 0.0, 0.355]),
        part(h, Box(0.05, 0.05, 0.03), DARK, [0.07, 0.0, 0.355]),
    ]);
    for s in [-1.0f32, 1.0] {
        v.extend([
            gear::strut(
                h,
                [s * 0.08, 0.08, 0.26],
                [s * 0.12, 0.25, 0.31],
                0.016,
                BRASS,
            ),
            part(h, Ball(0.04), ORANGE, [s * 0.12, 0.27, 0.31]).glow(),
        ]);
    }
    for (x, z) in [
        (-0.2, 0.15),
        (0.2, 0.15),
        (-0.28, -0.05),
        (0.28, -0.05),
        (0.0, 0.25),
    ] {
        v.push(part(h, Ball(0.018), BRASS, [x, 0.15, z]));
    }
    if f.a {
        // Armored Carrier: overlapping iron shell plates with spikes.
        for (z, w) in [(0.16, 0.6), (-0.02, 0.7), (-0.2, 0.58)] {
            v.push(part(h, Box(w, 0.05, 0.16), IRON, [0.0, 0.17, z]).rot([-8.0, 0.0, 0.0]));
        }
        for s in [-1.0f32, 1.0] {
            v.push(
                part(h, Cone(0.035, 0.12), STEEL, [s * 0.4, 0.05, 0.05]).rot([0.0, 0.0, -80.0 * s]),
            );
        }
    }
    if f.b {
        // Turbo Carrier: a racing stripe down the shell.
        v.push(part(h, Box(0.08, 0.02, 0.62), ALLY, [0.0, 0.17, 0.0]));
    }
}

/// The passenger saddle on the chest; Emergency Shield domes it.
fn saddle(v: &mut Vec<PartSpec>, f: Form) {
    let c = Body(Chest);
    v.extend([
        part(c, Box(0.36, 0.04, 0.3), WOOD, [0.0, 0.15, -0.03]),
        part(c, Box(0.22, 0.06, 0.18), RED_CLOTH, [0.0, 0.19, -0.04]),
        part(c, Box(0.22, 0.12, 0.04), RED_CLOTH, [0.0, 0.24, -0.13]),
    ]);
    for s in [-1.0f32, 1.0] {
        v.push(gear::strut(
            c,
            [s * 0.17, 0.15, 0.1],
            [s * 0.17, 0.15, -0.14],
            0.012,
            BRASS,
        ));
        v.push(gear::strut(
            c,
            [s * 0.17, 0.15, 0.1],
            [s * 0.17, 0.26, 0.1],
            0.012,
            BRASS,
        ));
    }
    let b = Secondary(Spring::Back);
    v.extend([
        part(b, Cyl(0.01, 0.5), WOOD, [-0.16, 0.18, 0.05]),
        part(b, Box(0.015, 0.12, 0.18), ALLY, [-0.16, 0.36, -0.03]),
    ]);
    if f.a && f.t2 {
        v.extend([
            part(c, Ring(0.015, 0.24, 360.0), ALLY, [0.0, 0.2, -0.03]).glow(),
            part(c, Ball(0.04), ALLY, [0.0, 0.3, -0.16]).glow(),
            part(c, Box(0.38, 0.03, 0.32), GOLD, [0.0, 0.13, -0.03]),
        ]);
    }
}

/// Three legs per side on the thigh, so they swing with the stride.
fn legs(v: &mut Vec<PartSpec>, p: &Proportions, f: Form, s: f32) {
    let thigh = Body(if s > 0.0 { ThighL } else { ThighR });
    let ground = -(p.leg + 0.04);
    for z in [-0.17f32, 0.0, 0.17] {
        let knee = [s * 0.3, 0.1, z * 1.1];
        let foot = [s * 0.4, ground, z * 1.3];
        v.extend([
            gear::strut(thigh, [s * 0.06, 0.0, z * 0.5], knee, 0.026, GUNMETAL),
            part(thigh, Ball(0.03), BRASS, knee),
            gear::strut(thigh, knee, foot, 0.02, BRASS),
        ]);
        if f.b {
            v.push(gear::strut(
                thigh,
                [s * 0.1, 0.05, z * 0.6],
                [s * 0.36, -0.06, z * 1.2],
                0.01,
                STEEL,
            ));
        }
    }
}

/// A pincer on the hand: palm, fixed jaw and a hooked lower jaw.
fn claw(v: &mut Vec<PartSpec>, p: &Proportions, f: Form, s: f32) {
    let (upper, fore, hand) = if s > 0.0 {
        (UpperArmL, ForearmL, HandL)
    } else {
        (UpperArmR, ForearmR, HandR)
    };
    let big = if f.a { 1.15 } else { 1.0 };
    v.extend([
        gear::strut(
            Body(upper),
            [0.0, 0.0, 0.0],
            [0.0, -p.arm, 0.0],
            0.035,
            GUNMETAL,
        ),
        part(
            Body(fore),
            Cyl(0.05, p.arm * 0.85),
            BRASS,
            [0.0, -p.arm * 0.45, 0.0],
        ),
        part(Body(hand), Ball(0.09 * big), GUNMETAL, [0.0, -0.07, 0.0]).scale([0.85, 1.25, 0.95]),
        part(
            Body(hand),
            Cone(0.05 * big, 0.2 * big),
            GUNMETAL,
            [0.0, -0.22 * big, 0.035],
        )
        .rot([170.0, 0.0, 0.0]),
        part(
            Body(hand),
            Cone(0.04 * big, 0.16 * big),
            BRASS,
            [0.0, -0.2 * big, -0.05],
        )
        .rot([195.0, 0.0, 0.0]),
    ]);
    if f.a && f.t2 {
        v.push(part(
            Body(hand),
            Box(0.1, 0.02, 0.1),
            GOLD,
            [0.0, -0.02, 0.0],
        ));
    }
}

/// Turbo Carrier's exhausts; Overdrive lights them and adds a fin.
fn exhausts(v: &mut Vec<PartSpec>, f: Form) {
    if !f.b {
        return;
    }
    let h = Body(Hips);
    for x in [-0.12, 0.12] {
        v.push(part(h, Cyl(0.04, 0.2), IRON, [x, 0.12, -0.34]).rot([-70.0, 0.0, 0.0]));
        if f.t2 {
            v.push(
                part(h, Cone(0.035, 0.14), HOT, [x, 0.08, -0.47])
                    .rot([-110.0, 0.0, 0.0])
                    .glow(),
            );
        }
    }
    if f.t2 {
        v.push(part(h, Box(0.03, 0.14, 0.22), ALLY, [0.0, 0.25, -0.22]).rot([-15.0, 0.0, 0.0]));
    }
}
