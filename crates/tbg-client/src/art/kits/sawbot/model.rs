//! Sawbot figure: a lean Ironworks automaton with one great glowing eye, a
//! boiler chest, a clamp hand and a spinning saw disk for a right hand.
//! Branch A (Overclock) lights its vents and heats the teeth, then mounts
//! a second saw and exhausts; branch B (Plating) bolts on plates, then a
//! horned juggernaut hull and a great saw.

use bevy::prelude::*;

use crate::art::{
    figure::{part, FigureSpec, PartSpec},
    kits::common::{body, Form},
    mesh::Shape::*,
    palette::*,
    rig::{Joint, Joint::*, Node::Body, Node::Secondary, Proportions, Spring},
};

const HOT: Color = Color::srgb(1.0, 0.55, 0.18);

pub fn plan(tier: u8) -> Proportions {
    let mut p = body::plan(false, 0.98, tier);
    p.torso.x *= 0.95;
    p.arm *= 1.1;
    p
}

fn radius(f: Form) -> f32 {
    if f.b && f.t2 {
        0.22
    } else {
        0.16
    }
}

/// The saw's leading edge (right hand space).
pub fn tip(tier: u8, branch: u8) -> Vec3 {
    let f = Form::of(tier, branch);
    Vec3::new(0.0, -0.13, radius(f))
}

pub fn spec(tier: u8, branch: u8) -> FigureSpec {
    let f = Form::of(tier, branch);
    let p = plan(tier);
    let mut v = Vec::with_capacity(140);
    let look = body::Look {
        skin: IRON,
        cloth: GUNMETAL,
        legs: GUNMETAL,
        boots: IRON,
    };
    body::humanoid(&mut v, &p, &look);
    head(&mut v, &p, f);
    chest(&mut v, &p, f);
    saw(&mut v, HandR, f, radius(f));
    if f.a && f.t2 {
        saw(&mut v, HandL, f, 0.13);
    } else {
        // A clamp hand.
        for x in [-0.04, 0.04] {
            v.push(part(
                Body(HandL),
                Box(0.03, 0.12, 0.05),
                BRASS,
                [x, -0.08, 0.0],
            ));
        }
    }
    FigureSpec { plan: p, parts: v }
}

fn head(v: &mut Vec<PartSpec>, p: &Proportions, f: Form) {
    let h = p.head;
    v.extend([
        part(
            Body(Head),
            Box(h * 1.8, h * 1.5, h * 1.6),
            GUNMETAL,
            [0.0, 0.0, 0.0],
        ),
        part(Body(Head), Cyl(h * 0.5, 0.04), BRASS, [0.0, 0.0, h * 0.8]).rot([90.0, 0.0, 0.0]),
        part(Body(Head), Ball(h * 0.38), HOT, [0.0, 0.0, h * 0.8]).glow(),
        part(
            Body(Head),
            Cyl(0.008, h * 0.9),
            BRASS,
            [h * 0.5, h * 1.1, 0.0],
        ),
        part(Body(Head), Ball(0.02), ORANGE, [h * 0.5, h * 1.55, 0.0]),
    ]);
    if f.b && f.t2 {
        for s in [-1.0f32, 1.0] {
            v.push(
                part(
                    Body(Head),
                    Cone(h * 0.25, h * 0.9),
                    IRON,
                    [s * h * 0.95, h * 0.8, 0.0],
                )
                .rot([0.0, 0.0, -35.0 * s]),
            );
        }
    }
}

fn chest(v: &mut Vec<PartSpec>, p: &Proportions, f: Form) {
    let t = p.torso;
    v.extend([
        part(
            Body(Chest),
            Box(t.x * 0.6, t.y * 0.45, 0.03),
            DARK,
            [0.0, t.y * 0.55, t.z * 0.42],
        ),
        part(
            Body(Chest),
            Box(t.x * 0.25, 0.04, 0.03),
            ALLY,
            [0.0, t.y * 0.88, t.z * 0.42],
        ),
        part(
            Secondary(Spring::Back),
            Cyl(0.04, 0.3),
            IRON,
            [0.1, 0.12, -0.04],
        ),
    ]);
    let vent = if f.a { HOT } else { RUST };
    for i in 0..3 {
        let slit = part(
            Body(Chest),
            Box(t.x * 0.5, 0.02, 0.035),
            vent,
            [0.0, t.y * (0.42 + 0.1 * i as f32), t.z * 0.43],
        );
        v.push(if f.a { slit.glow() } else { slit });
    }
    if f.a && f.t2 {
        for x in [-0.07, 0.07] {
            v.extend([
                part(
                    Secondary(Spring::Back),
                    Cyl(0.03, 0.22),
                    IRON,
                    [x, 0.0, -0.08],
                )
                .rot([-30.0, 0.0, 0.0]),
                part(
                    Secondary(Spring::Back),
                    Cone(0.028, 0.1),
                    HOT,
                    [x, -0.14, -0.15],
                )
                .rot([150.0, 0.0, 0.0])
                .glow(),
            ]);
        }
    }
    if f.b {
        body::pauldrons(v, IRON, if f.t2 { 0.13 } else { 0.1 });
        v.push(part(
            Body(Chest),
            Box(t.x * 1.05, t.y * 0.5, t.z * 1.15),
            IRON,
            [0.0, t.y * 0.65, 0.0],
        ));
        body::greaves(v, IRON);
    }
}

/// A saw disk across the hand's axis (so turning the hand spins it), its
/// teeth round the rim.
fn saw(v: &mut Vec<PartSpec>, hand: Joint, f: Form, r: f32) {
    let at = Body(hand);
    let teeth = if f.a { HOT } else { STEEL };
    v.extend([
        part(at, Cyl(0.035, 0.08), BRASS, [0.0, -0.06, 0.0]),
        part(at, Cyl(r, 0.018), STEEL, [0.0, -0.13, 0.0]),
        part(at, Cyl(r * 0.35, 0.03), GUNMETAL, [0.0, -0.13, 0.0]),
    ]);
    for i in 0..10 {
        let a = i as f32 / 10.0 * std::f32::consts::TAU;
        let tooth = part(
            at,
            Cone(0.02, 0.05),
            teeth,
            [a.sin() * r, -0.13, a.cos() * r],
        )
        .rot([90.0, a.to_degrees(), 0.0]);
        v.push(if f.a { tooth.glow() } else { tooth });
    }
}
