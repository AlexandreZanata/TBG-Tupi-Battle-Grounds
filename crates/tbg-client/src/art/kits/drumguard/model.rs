//! Drumguard figure: a marching drummer in a red coat with white straps
//! and a team shako, a snare drum at the hip and two sticks. Branch A
//! drives the march (a plumed shako and a bigger drum, then a grand gilded
//! bass drum and a team standard on the back); branch B holds the line
//! (iron bands and a steel helmet, then plate, pauldrons and glowing
//! rhythm runes).

use bevy::prelude::*;

use crate::art::{
    figure::{part, FigureSpec, PartSpec},
    kits::common::{body, body::Helm, Form},
    mesh::Shape::*,
    palette::*,
    rig::{Joint::*, Node::Body, Node::Prop, Node::Secondary, Proportions, Socket, Spring},
};

pub fn plan(tier: u8) -> Proportions {
    body::plan(false, 1.0, tier)
}

pub fn spec(tier: u8, branch: u8) -> FigureSpec {
    let f = Form::of(tier, branch);
    let p = plan(tier);
    let mut v = Vec::with_capacity(110);
    let look = body::Look {
        skin: SKIN,
        cloth: RED_CLOTH,
        legs: LINEN,
        boots: DARK_LEATHER,
    };
    body::humanoid(&mut v, &p, &look);
    body::tunic(&mut v, &p, RED_CLOTH, GOLD, LINEN);
    // White cross straps and gold buttons.
    for s in [-1.0f32, 1.0] {
        v.push(
            part(
                Body(Chest),
                Box(0.05, p.torso.y * 1.1, 0.02),
                LINEN,
                [0.06 * s, p.torso.y * 0.5, p.torso.z * 0.44],
            )
            .rot([0.0, 0.0, 30.0 * s]),
        );
    }
    head(&mut v, &p, f);
    drum(&mut v, f);
    sticks(&mut v);
    if f.b {
        body::pauldrons(&mut v, STEEL, 0.08);
        v.push(part(
            Body(Chest),
            Box(p.torso.x * 0.85, p.torso.y * 0.5, 0.05),
            STEEL,
            [0.0, p.torso.y * 0.62, p.torso.z * 0.44],
        ));
    }
    if f.a && f.t2 {
        let b = Secondary(Spring::Back);
        v.push(part(b, Cyl(0.015, 0.9), WOOD, [0.12, 0.25, -0.06]));
        v.push(part(b, Box(0.02, 0.22, 0.3), ALLY, [0.12, 0.55, -0.22]));
        v.push(part(b, Ball(0.03), GOLD, [0.12, 0.72, -0.06]));
    }
    FigureSpec { plan: p, parts: v }
}

fn head(v: &mut Vec<PartSpec>, p: &Proportions, f: Form) {
    let h = p.head;
    if f.b {
        body::helmet(v, p, Helm::Cap(STEEL));
        if f.t2 {
            body::helmet(v, p, Helm::Plumed(STEEL, ALLY));
        }
        return;
    }
    // Shako: a tall team cylinder with a gold plate and a peak.
    let tall = if f.a { 0.26 } else { 0.2 };
    v.extend([
        part(
            Body(Head),
            Cyl(h * 0.85, tall),
            ALLY,
            [0.0, h * 0.55 + tall * 0.5, -h * 0.05],
        ),
        part(
            Body(Head),
            Box(h * 0.6, h * 0.5, 0.02),
            GOLD,
            [0.0, h * 0.6 + tall * 0.4, h * 0.8],
        ),
        part(
            Body(Head),
            Box(h * 1.3, 0.02, h * 0.5),
            DARK,
            [0.0, h * 0.55, h * 0.75],
        ),
    ]);
    if f.a {
        v.push(part(
            Body(Head),
            Cone(h * 0.25, h * 1.2),
            LINEN,
            [0.0, h * 0.6 + tall + h * 0.5, 0.0],
        ));
    }
}

/// The drum hangs on a sling at the left hip (on the hips, so it bobs with
/// the march).
fn drum(v: &mut Vec<PartSpec>, f: Form) {
    let (r, depth) = if f.a && f.t2 {
        (0.2, 0.18)
    } else if f.a {
        (0.16, 0.15)
    } else {
        (0.14, 0.13)
    };
    let at = [0.08, 0.02, 0.2];
    let shell = if f.a && f.t2 { GOLD } else { ALLY };
    let rim = if f.b { IRON } else { BRASS };
    v.extend([
        part(Body(Hips), Cyl(r, depth), shell, at),
        part(
            Body(Hips),
            Cyl(r * 1.02, 0.015),
            LINEN,
            [at[0], at[1] + depth * 0.5, at[2]],
        ),
        part(
            Body(Hips),
            Cyl(r * 1.05, 0.025),
            rim,
            [at[0], at[1] + depth * 0.5, at[2]],
        ),
        part(
            Body(Hips),
            Cyl(r * 1.05, 0.025),
            rim,
            [at[0], at[1] - depth * 0.5, at[2]],
        ),
        part(Body(Chest), Box(0.04, 0.5, 0.02), LINEN, [0.02, 0.15, 0.16]).rot([0.0, 0.0, 40.0]),
    ]);
    for i in 0..6 {
        let a = i as f32 / 6.0 * std::f32::consts::TAU;
        v.push(
            part(
                Body(Hips),
                Box(0.012, depth * 0.95, 0.012),
                LINEN,
                [at[0] + a.cos() * r, at[1], at[2] + a.sin() * r],
            )
            .rot([0.0, 0.0, 25.0]),
        );
    }
    if f.b && f.t2 {
        for i in 0..4 {
            let a = (i as f32 + 0.5) / 4.0 * std::f32::consts::TAU;
            v.push(
                part(
                    Body(Hips),
                    Box(0.05, 0.03, 0.012),
                    SAP,
                    [
                        at[0] + a.cos() * r * 1.01,
                        at[1],
                        at[2] + a.sin() * r * 1.01,
                    ],
                )
                .glow(),
            );
        }
    }
}

/// A stick in each fist, along +Z.
fn sticks(v: &mut Vec<PartSpec>) {
    for s in [Socket::WeaponR, Socket::OffhandL] {
        v.push(part(Prop(s), Cyl(0.012, 0.26), WOOD, [0.0, 0.0, 0.1]).rot([90.0, 0.0, 0.0]));
        v.push(part(Prop(s), Ball(0.022), LINEN, [0.0, 0.0, 0.23]));
    }
}
