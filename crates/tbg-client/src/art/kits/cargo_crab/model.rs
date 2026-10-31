//! Cargo Crab figure: a robust forge-orange machine crab of the Ironworks, on
//! thick piston legs with heavy plated pincers, and a passenger saddle on its
//! back. Branch A (Armored Carrier) adds iron plates, spikes, and a glowing
//! shield dome over the saddle; branch B (Turbo Carrier) adds a racing stripe,
//! rear exhausts, leg servos, and a rear spoiler fin. Machines move on rigid
//! pivots and pistons only (ART-DIRECTION §3).

use bevy::prelude::*;

use crate::art::{
    figure::{part, FigureSpec, PartSpec},
    kits::common::{body, gear, Form},
    mesh::Shape::*,
    palette::*,
    rig::{Joint::*, Node::Body, Node::Secondary, Proportions, Spring},
};

const HOT: Color = Color::srgb(1.0, 0.55, 0.18);
/// Carapace forge orange: more saturated than the palette's forge glow, so the
/// crab reads as a machine at match-camera distance.
const FORGE: Color = Color::srgb(1.0, 0.42, 0.10);

pub fn plan(tier: u8) -> Proportions {
    let mut p = body::plan(true, 0.84, tier);
    p.leg = 0.28;
    p.torso = Vec3::new(0.70, 0.22, 0.56);
    p.head = 0.09;
    p.arm = 0.24;
    p
}

/// The saddle cushion's top above the crab's ground, in figure units: hips at
/// 0.34, chest 0.02 above them, cushion 0.245 above the chest (see `saddle`).
const SEAT: f32 = 0.605;

/// The passenger seat's height above the crab's ground, in world units. The
/// figure root carries the form's scale, so growth raises the seat with it.
pub fn seat(tier: u8) -> f32 {
    SEAT * plan(tier).scale
}

/// Where the right pincer closes (hand space).
pub fn tip() -> Vec3 {
    Vec3::new(0.0, -0.28, 0.0)
}

pub fn spec(tier: u8, branch: u8) -> FigureSpec {
    let f = Form::of(tier, branch);
    let p = plan(tier);
    let mut v = Vec::with_capacity(180);
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
    // Carapace dome over a deep gunmetal belly, rimmed in brass.
    v.extend([
        part(h, Ball(0.38), FORGE, [0.0, 0.07, 0.0]).scale([1.18, 0.46, 0.96]),
        part(h, Ball(0.34), GUNMETAL, [0.0, -0.07, 0.0]).scale([1.16, 0.44, 0.94]),
        part(h, Ring(0.024, 0.39, 360.0), BRASS, [0.0, 0.04, 0.0]).scale([1.14, 1.0, 0.94]),
        // Front armour plate with its grille
        part(h, Box(0.34, 0.09, 0.07), FORGE, [0.0, 0.02, 0.36]),
        part(h, Box(0.34, 0.035, 0.04), BRASS, [0.0, 0.05, 0.39]),
        part(h, Box(0.07, 0.07, 0.05), DARK, [-0.09, 0.0, 0.40]),
        part(h, Box(0.07, 0.07, 0.05), DARK, [0.09, 0.0, 0.40]),
    ]);
    // Sturdy mechanical eye-stalks with glowing amber lenses
    for s in [-1.0f32, 1.0] {
        v.extend([
            gear::strut(
                h,
                [s * 0.09, 0.09, 0.28],
                [s * 0.13, 0.27, 0.33],
                0.024,
                BRASS,
            ),
            part(h, Ball(0.05), FORGE, [s * 0.13, 0.29, 0.33]).glow(),
        ]);
    }
    // Brass rivets along the carapace rim
    for (x, z) in [
        (-0.24, 0.16),
        (0.24, 0.16),
        (-0.32, -0.06),
        (0.32, -0.06),
        (0.0, 0.27),
        (-0.16, -0.22),
        (0.16, -0.22),
    ] {
        v.push(part(h, Ball(0.024), BRASS, [x, 0.16, z]));
    }
    if f.a {
        // Armored Carrier: iron carapace plates and defense spikes
        for (z, w) in [(0.18, 0.64), (-0.02, 0.74), (-0.22, 0.62)] {
            v.push(part(h, Box(w, 0.06, 0.18), IRON, [0.0, 0.18, z]).rot([-8.0, 0.0, 0.0]));
        }
        for s in [-1.0f32, 1.0] {
            v.push(
                part(h, Cone(0.05, 0.16), STEEL, [s * 0.44, 0.06, 0.05]).rot([0.0, 0.0, -80.0 * s]),
            );
        }
    }
    if f.b {
        // Turbo Carrier: high-contrast racing stripe down the shell
        v.push(part(h, Box(0.09, 0.025, 0.68), ALLY, [0.0, 0.185, 0.0]));
    }
}

/// The passenger saddle mounted on the chassis; Emergency Shield domes it in A2.
fn saddle(v: &mut Vec<PartSpec>, f: Form) {
    let c = Body(Chest);
    v.extend([
        part(c, Box(0.40, 0.05, 0.34), WOOD, [0.0, 0.16, -0.03]),
        part(c, Box(0.28, 0.07, 0.24), RED_CLOTH, [0.0, 0.21, -0.04]),
        part(c, Box(0.28, 0.14, 0.05), RED_CLOTH, [0.0, 0.27, -0.15]),
    ]);
    for s in [-1.0f32, 1.0] {
        // Safety guardrails
        v.push(gear::strut(
            c,
            [s * 0.20, 0.16, 0.12],
            [s * 0.20, 0.16, -0.15],
            0.016,
            BRASS,
        ));
        v.push(gear::strut(
            c,
            [s * 0.20, 0.16, 0.12],
            [s * 0.20, 0.30, 0.12],
            0.016,
            BRASS,
        ));
        // Stirrup bracket
        v.push(gear::strut(
            c,
            [s * 0.18, 0.16, 0.02],
            [s * 0.18, 0.02, 0.02],
            0.014,
            IRON,
        ));
    }
    let b = Secondary(Spring::Back);
    v.extend([
        part(b, Cyl(0.014, 0.55), WOOD, [-0.18, 0.20, 0.05]),
        part(b, Box(0.018, 0.14, 0.20), ALLY, [-0.18, 0.40, -0.03]),
    ]);
    if f.a && f.t2 {
        // A2 Fortress Crab: protective shield ring and energy emitter
        v.extend([
            part(c, Ring(0.018, 0.30, 360.0), ALLY, [0.0, 0.22, -0.03]).glow(),
            part(c, Ball(0.05), ALLY, [0.0, 0.34, -0.17]).glow(),
            part(c, Box(0.44, 0.035, 0.38), GOLD, [0.0, 0.14, -0.03]),
        ]);
    }
}

/// Three thick piston legs per side: a hip cylinder, a brass knee joint, and a
/// shin piston ending in a broad iron pad.
fn legs(v: &mut Vec<PartSpec>, p: &Proportions, f: Form, s: f32) {
    let thigh = Body(if s > 0.0 { ThighL } else { ThighR });
    let ground = -(p.leg + 0.04);
    for z in [-0.18f32, 0.0, 0.18] {
        let knee = [s * 0.33, 0.11, z * 1.15];
        let foot = [s * 0.44, ground, z * 1.35];
        v.extend([
            gear::strut(thigh, [s * 0.07, 0.0, z * 0.5], knee, 0.046, GUNMETAL),
            part(thigh, Ball(0.058), BRASS, knee),
            gear::strut(thigh, knee, foot, 0.038, BRASS),
            part(
                thigh,
                Cyl(0.068, 0.03),
                IRON,
                [foot[0], foot[1] + 0.01, foot[2]],
            ),
        ]);
        if f.b {
            // Turbo Carrier: hydraulic servo sleeve along the shin
            v.push(gear::strut(
                thigh,
                [s * 0.11, 0.06, z * 0.6],
                [s * 0.38, -0.05, z * 1.25],
                0.02,
                STEEL,
            ));
        }
    }
}

/// A heavy pincer: a piston arm, a plated palm, and two broad jaws, the lower
/// one set a little open and both toothed. Blocky on purpose, so it reads from
/// the match camera.
fn claw(v: &mut Vec<PartSpec>, p: &Proportions, f: Form, s: f32) {
    let (upper, fore, hand) = if s > 0.0 {
        (UpperArmL, ForearmL, HandL)
    } else {
        (UpperArmR, ForearmR, HandR)
    };
    let big = if f.a { 1.34 } else { 1.22 };
    v.extend([
        gear::strut(
            Body(upper),
            [0.0, 0.0, 0.0],
            [0.0, -p.arm, 0.0],
            0.054,
            GUNMETAL,
        ),
        part(
            Body(fore),
            Cyl(0.068, p.arm * 0.85),
            BRASS,
            [0.0, -p.arm * 0.45, 0.0],
        ),
        // Palm joint and its forge-orange plate
        part(Body(hand), Ball(0.12 * big), GUNMETAL, [0.0, -0.08, 0.0]).scale([0.90, 1.10, 0.95]),
        part(
            Body(hand),
            Box(0.16 * big, 0.12 * big, 0.12 * big),
            FORGE,
            [0.0, -0.10 * big, 0.04 * big],
        ),
        // Broad upper jaw and the lower jaw, opened a little
        part(
            Body(hand),
            Box(0.12 * big, 0.24 * big, 0.06 * big),
            IRON,
            [0.0, -0.22 * big, 0.04],
        ),
        part(
            Body(hand),
            Box(0.11 * big, 0.22 * big, 0.05 * big),
            BRASS,
            [0.0, -0.21 * big, -0.05],
        )
        .rot([-10.0, 0.0, 0.0]),
        part(Body(hand), Ball(0.024), BRASS, [0.0, -0.12, 0.0]),
    ]);
    // Teeth on the inner faces of both jaws
    for y in [-0.16, -0.26] {
        v.push(part(
            Body(hand),
            Box(0.025, 0.035, 0.03 * big),
            BRASS,
            [0.0, y * big, 0.01],
        ));
    }
    if f.a && f.t2 {
        v.push(part(
            Body(hand),
            Box(0.14, 0.025, 0.14),
            GOLD,
            [0.0, -0.02, 0.0],
        ));
    }
}

/// Turbo Carrier's exhausts, high-velocity burners, and rear spoiler fin.
fn exhausts(v: &mut Vec<PartSpec>, f: Form) {
    if !f.b {
        return;
    }
    let h = Body(Hips);
    for x in [-0.14, 0.14] {
        v.push(part(h, Cyl(0.05, 0.22), IRON, [x, 0.13, -0.36]).rot([-70.0, 0.0, 0.0]));
        if f.t2 {
            v.push(
                part(h, Cone(0.045, 0.16), HOT, [x, 0.08, -0.50])
                    .rot([-110.0, 0.0, 0.0])
                    .glow(),
            );
        }
    }
    if f.t2 {
        v.push(part(h, Box(0.03, 0.16, 0.25), ALLY, [0.0, 0.27, -0.24]).rot([-15.0, 0.0, 0.0]));
    }
}
