//! Nester figure: a plump moss mother with a round leafy belly, a leaf cap
//! and a woven nest of eggs on her back, a seed ready in hand. Branch A
//! (Brood) grows the clutch, then carries a glowing amber hive; branch B
//! (Venom) lays green glowing eggs, then dark spiked blight eggs.

use bevy::prelude::*;

use crate::art::{
    figure::{part, FigureSpec, PartSpec},
    kits::common::{body, Form},
    mesh::Shape::*,
    palette::*,
    rig::{Joint::*, Node::Body, Node::Prop, Node::Secondary, Proportions, Socket, Spring},
};

const MOSS: Color = Color::srgb(0.5, 0.62, 0.3);
const WICKER: Color = Color::srgb(0.66, 0.5, 0.28);
const SEED: Color = Color::srgb(0.55, 0.4, 0.22);
const VENOM: Color = Color::srgb(0.55, 0.95, 0.2);
const BLIGHT: Color = Color::srgb(0.36, 0.2, 0.42);
const HONEY: Color = Color::srgb(0.95, 0.7, 0.25);
const THORN: Color = Color::srgb(0.24, 0.16, 0.1);

pub fn plan(tier: u8) -> Proportions {
    let mut p = body::plan(false, 1.0, tier);
    p.torso.x *= 1.25;
    p.torso.z *= 1.3;
    p.leg *= 0.85;
    p
}

/// The seed leaves the right hand (weapon space).
pub fn muzzle() -> Vec3 {
    Vec3::new(0.0, 0.03, 0.06)
}

pub fn spec(tier: u8, branch: u8) -> FigureSpec {
    let f = Form::of(tier, branch);
    let p = plan(tier);
    let mut v = Vec::with_capacity(130);
    let look = body::Look {
        skin: MOSS,
        cloth: LEAF_DARK,
        legs: BARK,
        boots: BARK,
    };
    body::humanoid(&mut v, &p, &look);
    let (t, h) = (p.torso, p.head);
    v.extend([
        part(
            Body(Chest),
            Ball(t.x * 0.55),
            LEAF,
            [0.0, t.y * 0.32, t.z * 0.18],
        )
        .scale([1.0, 0.9, 0.85]),
        part(
            Body(Chest),
            Box(t.x * 0.9, 0.05, t.z * 1.1),
            ALLY,
            [0.0, t.y * 0.75, 0.0],
        ),
        part(Body(Head), Ball(h * 1.15), LEAF_DARK, [0.0, h * 0.45, 0.0]).scale([1.2, 0.45, 1.2]),
        part(
            Body(Head),
            Box(h * 0.9, h * 0.15, 0.02),
            DARK,
            [0.0, -h * 0.4, h * 0.96],
        ),
        part(
            Secondary(Spring::HeadGear),
            Ball(h * 0.35),
            LEAF,
            [0.0, h * 0.95, 0.0],
        )
        .scale([0.5, 1.4, 0.25]),
        part(
            Prop(Socket::WeaponR),
            Ball(0.045),
            SEED,
            muzzle().to_array(),
        )
        .scale([0.8, 1.2, 0.8]),
    ]);
    nest(&mut v, f);
    if f.b && !f.t2 {
        for (x, y) in [(-0.1, 0.5), (0.12, 0.3), (0.0, 0.15)] {
            v.push(
                part(
                    Body(Chest),
                    Ball(0.025),
                    VENOM,
                    [t.x * x, t.y * y, t.z * 0.6],
                )
                .glow(),
            );
        }
    }
    if f.b && f.t2 {
        for arm in [UpperArmL, UpperArmR] {
            v.push(part(Body(arm), Cone(0.025, 0.1), THORN, [0.0, 0.08, 0.0]));
        }
    }
    FigureSpec { plan: p, parts: v }
}

/// The nest (or hive) on the back spring and its eggs.
fn nest(v: &mut Vec<PartSpec>, f: Form) {
    let b = Secondary(Spring::Back);
    if f.a && f.t2 {
        // Hive: a straw skep of stacked rings with glowing amber eggs.
        for (i, r) in [0.2f32, 0.18, 0.15, 0.11].into_iter().enumerate() {
            v.push(part(
                b,
                Cyl(r, 0.07),
                HONEY,
                [0.0, -0.05 + 0.07 * i as f32, -0.14],
            ));
        }
        v.push(part(b, Ball(0.05), DARK, [0.0, -0.04, 0.03]).scale([1.0, 0.7, 0.4]));
        for x in [-0.08, 0.08] {
            v.push(part(b, Ball(0.045), SAP, [x, -0.12, -0.02]).glow());
        }
        return;
    }
    let big = if f.a || f.t2 { 1.2 } else { 1.0 };
    v.extend([
        part(
            b,
            Frustum(0.2 * big, 0.13 * big, 0.12),
            WICKER,
            [0.0, -0.02, -0.14],
        ),
        part(b, Ring(0.025, 0.2 * big, 360.0), BARK, [0.0, 0.04, -0.14]),
    ]);
    let eggs: &[(f32, f32)] = if f.a || f.t2 {
        &[(-0.08, -0.16), (0.08, -0.12), (0.0, -0.06)]
    } else {
        &[(-0.06, -0.14), (0.06, -0.12)]
    };
    let (shell, lit) = match (f.b, f.t2) {
        (true, true) => (BLIGHT, true),
        (true, false) => (VENOM, true),
        _ => (LINEN, false),
    };
    for &(x, z) in eggs {
        let egg = part(b, Ball(0.065), shell, [x, 0.1, z]).scale([0.85, 1.15, 0.85]);
        v.push(if lit { egg.glow() } else { egg });
        if f.b && f.t2 {
            v.push(part(b, Cone(0.015, 0.06), THORN, [x, 0.19, z]));
        }
    }
}
