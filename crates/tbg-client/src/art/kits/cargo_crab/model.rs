//! Cargo Crab figure: a robust orange machine crab of the Ironworks
//! with heavy mechanical pincers, articulated legs, and a passenger
//! saddle mounted securely on its chassis.
//! Branch A (Armored Carrier) adds heavy iron plates, spikes, and a
//! glowing shield dome over the saddle; branch B (Turbo Carrier) adds
//! rear exhausts, leg boosters, afterburners, and a rear spoiler fin.

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
    let mut p = body::plan(true, 0.84, tier);
    p.leg = 0.28;
    p.torso = Vec3::new(0.70, 0.22, 0.56);
    p.head = 0.09;
    p.arm = 0.24;
    p
}

/// Where the right pincer closes (hand space).
pub fn tip() -> Vec3 {
    Vec3::new(0.0, -0.28, 0.0)
}

pub fn spec(tier: u8, branch: u8) -> FigureSpec {
    let f = Form::of(tier, branch);
    let p = plan(tier);
    let mut v = Vec::with_capacity(160);
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
    // Heavy machine chassis with orange carapace plates and brass trim
    v.extend([
        part(h, Ball(0.38), ORANGE, [0.0, 0.07, 0.0]).scale([1.18, 0.46, 0.96]),
        part(h, Ball(0.34), GUNMETAL, [0.0, -0.01, 0.0]).scale([1.14, 0.32, 0.92]),
        part(h, Ring(0.024, 0.39, 360.0), BRASS, [0.0, 0.04, 0.0]).scale([1.14, 1.0, 0.94]),
        part(h, Box(0.30, 0.08, 0.06), ORANGE, [0.0, 0.01, 0.35]),
        part(h, Box(0.32, 0.03, 0.04), BRASS, [0.0, 0.04, 0.36]),
        part(h, Box(0.06, 0.06, 0.04), DARK, [-0.08, 0.01, 0.37]),
        part(h, Box(0.06, 0.06, 0.04), DARK, [0.08, 0.01, 0.37]),
    ]);
    // Sturdy mechanical eye-stalks with glowing amber lenses
    for s in [-1.0f32, 1.0] {
        v.extend([
            gear::strut(
                h,
                [s * 0.09, 0.09, 0.28],
                [s * 0.13, 0.27, 0.33],
                0.018,
                BRASS,
            ),
            part(h, Ball(0.045), ORANGE, [s * 0.13, 0.29, 0.33]).glow(),
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
        v.push(part(h, Ball(0.02), BRASS, [x, 0.16, z]));
    }
    if f.a {
        // Armored Carrier: heavy iron carapace plating and defense spikes
        for (z, w) in [(0.18, 0.64), (-0.02, 0.74), (-0.22, 0.62)] {
            v.push(part(h, Box(w, 0.06, 0.18), IRON, [0.0, 0.18, z]).rot([-8.0, 0.0, 0.0]));
        }
        for s in [-1.0f32, 1.0] {
            v.push(
                part(h, Cone(0.04, 0.14), STEEL, [s * 0.44, 0.06, 0.05]).rot([0.0, 0.0, -80.0 * s]),
            );
        }
    }
    if f.b {
        // Turbo Carrier: high-contrast aerodynamic racing stripe down shell
        v.push(part(h, Box(0.09, 0.025, 0.68), ALLY, [0.0, 0.185, 0.0]));
    }
}

/// The passenger saddle mounted on the chassis; Emergency Shield domes it in A2.
fn saddle(v: &mut Vec<PartSpec>, f: Form) {
    let c = Body(Chest);
    v.extend([
        part(c, Box(0.40, 0.05, 0.34), WOOD, [0.0, 0.16, -0.03]),
        part(c, Box(0.26, 0.07, 0.22), RED_CLOTH, [0.0, 0.21, -0.04]),
        part(c, Box(0.26, 0.14, 0.05), RED_CLOTH, [0.0, 0.27, -0.14]),
    ]);
    for s in [-1.0f32, 1.0] {
        // Safety guardrails
        v.push(gear::strut(
            c,
            [s * 0.19, 0.16, 0.12],
            [s * 0.19, 0.16, -0.15],
            0.014,
            BRASS,
        ));
        v.push(gear::strut(
            c,
            [s * 0.19, 0.16, 0.12],
            [s * 0.19, 0.28, 0.12],
            0.014,
            BRASS,
        ));
        // Stirrup bracket
        v.push(gear::strut(
            c,
            [s * 0.18, 0.16, 0.02],
            [s * 0.18, 0.02, 0.02],
            0.012,
            IRON,
        ));
    }
    let b = Secondary(Spring::Back);
    v.extend([
        part(b, Cyl(0.012, 0.55), WOOD, [-0.18, 0.20, 0.05]),
        part(b, Box(0.018, 0.14, 0.20), ALLY, [-0.18, 0.40, -0.03]),
    ]);
    if f.a && f.t2 {
        // A2 Fortress Crab: protective shield ring and energy emitter
        v.extend([
            part(c, Ring(0.018, 0.28, 360.0), ALLY, [0.0, 0.22, -0.03]).glow(),
            part(c, Ball(0.05), ALLY, [0.0, 0.34, -0.16]).glow(),
            part(c, Box(0.42, 0.035, 0.36), GOLD, [0.0, 0.14, -0.03]),
        ]);
    }
}

/// Three articulated mechanical legs per side on the thigh.
fn legs(v: &mut Vec<PartSpec>, p: &Proportions, f: Form, s: f32) {
    let thigh = Body(if s > 0.0 { ThighL } else { ThighR });
    let ground = -(p.leg + 0.04);
    for z in [-0.18f32, 0.0, 0.18] {
        let knee = [s * 0.33, 0.11, z * 1.15];
        let foot = [s * 0.44, ground, z * 1.35];
        v.extend([
            gear::strut(thigh, [s * 0.07, 0.0, z * 0.5], knee, 0.028, GUNMETAL),
            part(thigh, Ball(0.034), BRASS, knee),
            gear::strut(thigh, knee, foot, 0.022, BRASS),
            part(thigh, Box(0.06, 0.02, 0.06), IRON, foot),
        ]);
        if f.b {
            // Turbo Carrier hydraulic leg servos
            v.push(gear::strut(
                thigh,
                [s * 0.11, 0.06, z * 0.6],
                [s * 0.38, -0.05, z * 1.25],
                0.012,
                STEEL,
            ));
        }
    }
}

/// Heavy mechanical pincers with iron/steel jaws and brass pistons.
fn claw(v: &mut Vec<PartSpec>, p: &Proportions, f: Form, s: f32) {
    let (upper, fore, hand) = if s > 0.0 {
        (UpperArmL, ForearmL, HandL)
    } else {
        (UpperArmR, ForearmR, HandR)
    };
    let big = if f.a { 1.18 } else { 1.0 };
    v.extend([
        gear::strut(
            Body(upper),
            [0.0, 0.0, 0.0],
            [0.0, -p.arm, 0.0],
            0.040,
            GUNMETAL,
        ),
        part(
            Body(fore),
            Cyl(0.055, p.arm * 0.85),
            BRASS,
            [0.0, -p.arm * 0.45, 0.0],
        ),
        // Palm joint
        part(Body(hand), Ball(0.10 * big), GUNMETAL, [0.0, -0.08, 0.0]).scale([0.88, 1.28, 0.95]),
        // Orange carapace shield plate on claw
        part(
            Body(hand),
            Box(0.10 * big, 0.14 * big, 0.04 * big),
            ORANGE,
            [0.0, -0.10 * big, 0.05 * big],
        ),
        // Heavy upper pincer jaw
        part(
            Body(hand),
            Cone(0.055 * big, 0.22 * big),
            IRON,
            [0.0, -0.24 * big, 0.04],
        )
        .rot([170.0, 0.0, 0.0]),
        // Serrated lower pincer jaw
        part(
            Body(hand),
            Cone(0.045 * big, 0.18 * big),
            BRASS,
            [0.0, -0.22 * big, -0.05],
        )
        .rot([195.0, 0.0, 0.0]),
        part(Body(hand), Ball(0.02), BRASS, [0.0, -0.12, 0.0]),
    ]);
    if f.a && f.t2 {
        v.push(part(
            Body(hand),
            Box(0.12, 0.025, 0.12),
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
        v.push(part(h, Cyl(0.045, 0.22), IRON, [x, 0.13, -0.36]).rot([-70.0, 0.0, 0.0]));
        if f.t2 {
            v.push(
                part(h, Cone(0.04, 0.16), HOT, [x, 0.08, -0.50])
                    .rot([-110.0, 0.0, 0.0])
                    .glow(),
            );
        }
    }
    if f.t2 {
        v.push(part(h, Box(0.03, 0.16, 0.25), ALLY, [0.0, 0.27, -0.24]).rot([-15.0, 0.0, 0.0]));
    }
}
