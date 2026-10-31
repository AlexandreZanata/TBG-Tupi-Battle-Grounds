//! Beetling figure: a small beetle hatched by a Nester, a domed shell
//! split down the middle, a dark head with mandibles and antennae, six
//! legs on the hip joints so they scuttle with the gait. It wears its
//! Nester's form: amber-banded (Brood), a honey shell with glowing spots
//! (Hive), green venom spots and dripping jaws (Venom), or a dark spiked
//! blight shell (Blight).

use bevy::prelude::*;

use crate::art::{
    figure::{part, FigureSpec, PartSpec},
    kits::common::{body, gear, Form},
    mesh::Shape::*,
    palette::*,
    rig::{Joint::*, Node::Body, Proportions},
};

const VENOM: Color = Color::srgb(0.55, 0.95, 0.2);
const BLIGHT: Color = Color::srgb(0.36, 0.2, 0.42);
const HONEY: Color = Color::srgb(0.95, 0.7, 0.25);
const AMBER: Color = Color::srgb(1.0, 0.72, 0.25);
const THORN: Color = Color::srgb(0.24, 0.16, 0.1);
const VIOLET_GLOW: Color = Color::srgb(0.78, 0.48, 1.0);

pub fn plan(tier: u8) -> Proportions {
    let mut p = body::plan(false, 0.6, tier);
    p.leg = 0.16;
    p.torso = Vec3::new(0.3, 0.12, 0.34);
    p.head = 0.06;
    p.arm = 0.08;
    p
}

/// Where the mandibles close (chest space).
pub fn tip() -> Vec3 {
    Vec3::new(0.0, 0.0, 0.38)
}

pub fn spec(tier: u8, branch: u8) -> FigureSpec {
    let f = Form::of(tier, branch);
    let p = plan(tier);
    let mut v = Vec::with_capacity(90);
    shell(&mut v, f);
    head(&mut v, f);
    for s in [-1.0f32, 1.0] {
        legs(&mut v, &p, s);
    }
    FigureSpec { plan: p, parts: v }
}

fn shell(v: &mut Vec<PartSpec>, f: Form) {
    let h = Body(Hips);
    let color = match (f.a, f.b, f.t2) {
        (true, _, true) => HONEY,
        (_, true, true) => BLIGHT,
        _ => LEAF_DARK,
    };
    v.extend([
        part(h, Ball(0.22), color, [0.0, 0.06, -0.02]).scale([1.0, 0.62, 1.3]),
        part(h, Ball(0.18), DARK, [0.0, 0.0, -0.02]).scale([1.0, 0.3, 1.25]),
        part(h, Box(0.012, 0.02, 0.48), DARK, [0.0, 0.185, -0.02]),
        part(h, Box(0.07, 0.015, 0.12), ALLY, [0.0, 0.19, -0.12]),
    ]);
    if f.a && !f.t2 {
        for z in [0.06, -0.08] {
            v.push(part(h, Box(0.4, 0.02, 0.04), AMBER, [0.0, 0.15, z]));
        }
    }
    let spots: &[(f32, f32)] = &[(-0.09, 0.05), (0.09, 0.05), (-0.1, -0.12), (0.1, -0.12)];
    if (f.a && f.t2) || (f.b && !f.t2) {
        let c = if f.a { AMBER } else { VENOM };
        for &(x, z) in spots {
            v.push(
                part(h, Ball(0.03), c, [x, 0.16, z])
                    .scale([1.0, 0.4, 1.0])
                    .glow(),
            );
        }
    }
    if f.b && f.t2 {
        for &(x, z) in spots {
            v.push(part(h, Cone(0.025, 0.1), THORN, [x, 0.17, z]));
        }
    }
}

fn head(v: &mut Vec<PartSpec>, f: Form) {
    let c = Body(Chest);
    let eye = match (f.b, f.t2) {
        (true, true) => VIOLET_GLOW,
        (true, false) => VENOM,
        _ => AMBER,
    };
    v.extend([
        part(c, Ball(0.09), DARK, [0.0, 0.03, 0.24]).scale([1.0, 0.8, 1.0]),
        part(c, Ball(0.018), eye, [0.05, 0.07, 0.3]).glow(),
        part(c, Ball(0.018), eye, [-0.05, 0.07, 0.3]).glow(),
    ]);
    for s in [-1.0f32, 1.0] {
        v.extend([
            part(c, Cone(0.022, 0.12), BONE, [s * 0.04, 0.0, 0.34]).rot([90.0, 0.0, 25.0 * s]),
            gear::strut(c, [s * 0.03, 0.08, 0.3], [s * 0.09, 0.2, 0.42], 0.006, DARK),
        ]);
        if f.b && !f.t2 {
            v.push(
                part(c, Ball(0.015), VENOM, [s * 0.05, -0.04, 0.4])
                    .scale([1.0, 1.8, 1.0])
                    .glow(),
            );
        }
    }
}

/// Three legs per side on the thigh, so they swing with the stride.
fn legs(v: &mut Vec<PartSpec>, p: &Proportions, s: f32) {
    let thigh = Body(if s > 0.0 { ThighL } else { ThighR });
    let ground = -(p.leg + 0.04);
    for z in [-0.12f32, 0.0, 0.12] {
        let knee = [s * 0.15, 0.05, z * 1.05];
        let foot = [s * 0.2, ground, z * 1.25];
        v.extend([
            gear::strut(thigh, [s * 0.04, 0.0, z * 0.5], knee, 0.02, DARK),
            gear::strut(thigh, knee, foot, 0.015, DARK),
        ]);
    }
}
