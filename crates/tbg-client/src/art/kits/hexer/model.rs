//! Hexer figure: a hunched skeleton witch in a ragged violet robe and a
//! crooked pointed hat, a gnarled staff topped with a lantern skull.
//! Branch A (Withering) rings the skull with hex runes, then crowns the
//! hat and swells the skull with doom-lit eyes; branch B (Plague) swings
//! a plague censer, then wears a plague beak and a greater censer.

use bevy::prelude::*;

use crate::art::{
    figure::{part, FigureSpec, PartSpec},
    kits::common::{body, gear, Form},
    mesh::Shape::*,
    palette::*,
    rig::{Joint::*, Node::Body, Node::Prop, Node::Secondary, Proportions, Socket, Spring},
};

const ROBE: Color = Color::srgb(0.3, 0.22, 0.36);
const RAG: Color = Color::srgb(0.2, 0.15, 0.24);
pub const HEX: Color = Color::srgb(0.78, 0.48, 1.0);
const PLAGUE: Color = Color::srgb(0.55, 0.85, 0.25);
/// Staff length (socket space, +Y).
const STAFF: f32 = 1.0;

pub fn plan(tier: u8) -> Proportions {
    body::plan(false, 0.98, tier)
}

/// The hex leaves the skull's mouth on the staff (weapon space).
pub fn muzzle() -> Vec3 {
    Vec3::from(gear::staff_top(STAFF)) + Vec3::new(0.0, 0.08, 0.06)
}

pub fn spec(tier: u8, branch: u8) -> FigureSpec {
    let f = Form::of(tier, branch);
    let p = plan(tier);
    let mut v = Vec::with_capacity(140);
    body::skeleton(&mut v, &p, BONE);
    robe(&mut v, &p, f);
    hat(&mut v, &p, f);
    body::glowing_eyes(&mut v, &p, if f.b { PLAGUE } else { HEX });
    staff(&mut v, f);
    if f.b {
        censer(&mut v, if f.t2 { 1.4 } else { 1.0 });
    }
    FigureSpec { plan: p, parts: v }
}

fn robe(v: &mut Vec<PartSpec>, p: &Proportions, f: Form) {
    let t = p.torso;
    v.extend([
        part(
            Body(Chest),
            Frustum(t.x * 0.55, t.x * 0.4, t.y * 0.95),
            ROBE,
            [0.0, t.y * 0.5, 0.0],
        )
        .scale([1.0, 1.0, 0.75]),
        part(
            Body(Hips),
            Frustum(t.x * 0.45, t.x * 0.75, p.leg * 0.85),
            ROBE,
            [0.0, -p.leg * 0.4, 0.0],
        )
        .scale([1.0, 1.0, 0.85]),
        part(
            Body(Chest),
            Box(t.x * 1.1, 0.12, t.z * 0.9),
            RAG,
            [0.0, t.y * 0.95, -0.02],
        ),
        part(
            Body(Hips),
            Box(0.06, 0.2, 0.02),
            ALLY,
            [t.x * 0.2, -0.08, t.z * 0.5],
        ),
    ]);
    // Ragged hem: tatters round the skirt.
    for i in 0..6 {
        let a = i as f32 / 6.0 * std::f32::consts::TAU;
        v.push(
            part(
                Body(Hips),
                Box(0.08, 0.12, 0.02),
                RAG,
                [a.sin() * t.x * 0.72, -p.leg * 0.85, a.cos() * t.x * 0.62],
            )
            .rot([0.0, a.to_degrees(), 0.0]),
        );
    }
    for arm in [UpperArmL, UpperArmR] {
        v.push(part(
            Body(arm),
            Frustum(0.06, 0.08, p.arm * 0.9),
            ROBE,
            [0.0, -p.arm * 0.45, 0.0],
        ));
    }
    if f.b && f.t2 {
        // Pestilence: a plague doctor's beak over the skull.
        let h = p.head;
        v.push(
            part(
                Body(Head),
                Cone(h * 0.35, h * 1.2),
                LINEN,
                [0.0, -h * 0.25, h * 1.3],
            )
            .rot([100.0, 0.0, 0.0]),
        );
    }
}

/// A crooked pointed hat; Doom crowns it.
fn hat(v: &mut Vec<PartSpec>, p: &Proportions, f: Form) {
    let h = p.head;
    let g = Secondary(Spring::HeadGear);
    v.extend([
        part(Body(Head), Cyl(h * 1.6, 0.02), RAG, [0.0, h * 0.6, 0.0]),
        part(
            Body(Head),
            Cone(h * 0.85, h * 1.2),
            ROBE,
            [0.0, h * 1.2, -h * 0.05],
        ),
        part(g, Cone(h * 0.4, h * 0.9), ROBE, [0.0, h * 2.1, -h * 0.35]).rot([-35.0, 0.0, 0.0]),
        part(
            Body(Head),
            Cyl(h * 0.87, h * 0.15),
            if f.a { HEX } else { RAG },
            [0.0, h * 0.72, 0.0],
        ),
    ]);
    if f.a && f.t2 {
        for i in 0..5 {
            let a = (i as f32 - 2.0) * 0.5;
            v.push(part(
                Body(Head),
                Cone(h * 0.12, h * 0.4),
                GOLD,
                [a.sin() * h * 0.9, h * 0.95, a.cos() * h * 0.9],
            ));
        }
    }
}

/// A gnarled staff with a lantern skull; Withering rings it with runes.
fn staff(v: &mut Vec<PartSpec>, f: Form) {
    let w = Prop(Socket::WeaponR);
    v.extend(gear::staff(w, STAFF, BARK));
    let [x, y, z] = gear::staff_top(STAFF);
    let s = if f.a && f.t2 { 1.35 } else { 1.0 };
    v.extend([
        gear::strut(
            w,
            [x, y - 0.1, z],
            [x + 0.05, y + 0.02, z + 0.04],
            0.018,
            BARK,
        ),
        part(w, Ball(0.07 * s), BONE, [x, y + 0.08, z + 0.03]),
        part(
            w,
            Box(0.11 * s, 0.04 * s, 0.07 * s),
            BONE,
            [x, y + 0.03, z + 0.06],
        ),
        part(
            w,
            Ball(0.025 * s),
            HEX,
            [x - 0.03 * s, y + 0.09, z + 0.09 * s],
        )
        .glow(),
        part(
            w,
            Ball(0.025 * s),
            HEX,
            [x + 0.03 * s, y + 0.09, z + 0.09 * s],
        )
        .glow(),
    ]);
    if f.a {
        v.push(
            part(w, Ring(0.01, 0.13 * s, 360.0), HEX, [x, y + 0.08, z + 0.03])
                .rot([20.0, 0.0, 0.0])
                .glow(),
        );
    }
}

/// A smoking plague censer hanging from the left hand.
fn censer(v: &mut Vec<PartSpec>, s: f32) {
    let l = Prop(Socket::OffhandL);
    v.extend([
        part(l, Cyl(0.006, 0.18), IRON, [0.0, -0.09, 0.04]),
        part(l, Ball(0.06 * s), IRON, [0.0, -0.2 * s, 0.04]),
        part(l, Ball(0.045 * s), PLAGUE, [0.0, -0.2 * s, 0.04]).glow(),
        part(
            l,
            Ring(0.012, 0.062 * s, 360.0),
            BRASS,
            [0.0, -0.2 * s, 0.04],
        ),
    ]);
}
