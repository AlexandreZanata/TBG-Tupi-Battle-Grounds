//! Puppetmaster figure: a tall gaunt skeleton showman in a violet tail
//! coat and top hat, working a marionette from a control cross. Branch A
//! (Cruel Strings) hooks the strings and arms the puppet, then lights the
//! cross and strings and masks the face; branch B (Twisted Chorus) works a
//! second puppet from the left hand, then carries a chorus on the coat.

use bevy::prelude::*;

use crate::art::{
    figure::{part, FigureSpec, PartSpec},
    kits::common::{body, gear, Form},
    mesh::Shape::*,
    palette::*,
    rig::{Joint::*, Node, Node::Body, Node::Prop, Node::Secondary, Proportions, Socket, Spring},
};

const COAT: Color = Color::srgb(0.36, 0.24, 0.46);
pub const MAGIC: Color = Color::srgb(0.78, 0.48, 1.0);
const STRING: Color = Color::srgb(0.92, 0.9, 0.85);
/// How far below the control the puppet hangs (socket space).
const DROP: f32 = 0.4;
/// The control cross's center (weapon space): the hex leaves from here.
pub const CROSS: Vec3 = Vec3::new(0.0, -0.05, 0.12);

pub fn plan(tier: u8) -> Proportions {
    let mut p = body::plan(false, 1.04, tier);
    p.leg *= 1.12;
    p.arm *= 1.15;
    p.torso.x *= 0.9;
    p
}

pub fn spec(tier: u8, branch: u8) -> FigureSpec {
    let f = Form::of(tier, branch);
    let p = plan(tier);
    let mut v = Vec::with_capacity(160);
    body::skeleton(&mut v, &p, BONE);
    coat(&mut v, &p, f);
    hat(&mut v, &p, f);
    body::glowing_eyes(&mut v, &p, MAGIC);
    marionette(&mut v, Prop(Socket::WeaponR), f, RED_CLOTH);
    if f.b {
        marionette(&mut v, Prop(Socket::OffhandL), f, ALLY);
    }
    if f.b && f.t2 {
        let b = Secondary(Spring::Back);
        for x in [-0.14, 0.14] {
            puppet(&mut v, b, [x, -0.3, -0.1], LINEN, false);
            v.push(gear::strut(
                b,
                [x, 0.0, -0.08],
                [x, -0.24, -0.1],
                0.004,
                MAGIC,
            ));
        }
    }
    FigureSpec { plan: p, parts: v }
}

fn coat(v: &mut Vec<PartSpec>, p: &Proportions, f: Form) {
    let t = p.torso;
    v.extend([
        part(
            Body(Chest),
            Frustum(t.x * 0.58, t.x * 0.44, t.y * 0.95),
            COAT,
            [0.0, t.y * 0.5, 0.0],
        )
        .scale([1.0, 1.0, 0.72]),
        part(
            Body(Chest),
            Box(t.x * 0.3, t.y * 0.9, 0.02),
            LINEN,
            [0.0, t.y * 0.48, t.x * 0.33],
        ),
        part(
            Body(Chest),
            Box(0.06, 0.05, 0.02),
            ALLY,
            [0.0, t.y * 0.86, t.x * 0.35],
        ),
        part(
            Body(Chest),
            Box(t.x * 1.05, 0.14, 0.04),
            COAT,
            [0.0, t.y * 1.0, -0.06],
        )
        .rot([-25.0, 0.0, 0.0]),
        part(
            Body(Hips),
            Box(t.x * 0.9, 0.1, t.z * 0.9),
            COAT,
            [0.0, 0.0, 0.0],
        ),
    ]);
    for arm in [UpperArmL, UpperArmR] {
        v.push(part(
            Body(arm),
            Frustum(0.055, 0.065, p.arm * 0.9),
            COAT,
            [0.0, -p.arm * 0.45, 0.0],
        ));
    }
    // Tails of the coat on the back spring.
    let b = Secondary(Spring::Back);
    for x in [-0.07, 0.07] {
        v.push(part(b, Box(0.12, 0.5, 0.02), COAT, [x, -0.42, -0.02]).rot([
            8.0,
            0.0,
            6.0 * x.signum(),
        ]));
    }
    if f.a && f.t2 {
        // A porcelain half mask over the skull.
        let h = p.head;
        v.push(part(
            Body(Head),
            Box(h * 1.2, h * 0.6, 0.03),
            LINEN,
            [h * 0.2, h * 0.15, h * 0.99],
        ));
    }
}

fn hat(v: &mut Vec<PartSpec>, p: &Proportions, f: Form) {
    let h = p.head;
    v.extend([
        part(Body(Head), Cyl(h * 1.35, 0.025), DARK, [0.0, h * 0.75, 0.0]),
        part(
            Body(Head),
            Cyl(h * 0.82, h * 1.3),
            DARK,
            [0.0, h * 1.45, 0.0],
        )
        .rot([0.0, 0.0, -6.0]),
        part(
            Body(Head),
            Cyl(h * 0.84, h * 0.22),
            if f.a { RED_CLOTH } else { COAT },
            [0.0, h * 0.95, 0.0],
        ),
    ]);
    if f.t2 {
        v.push(
            part(
                Body(Head),
                Ball(h * 0.18),
                MAGIC,
                [h * 0.6, h * 1.0, h * 0.5],
            )
            .glow(),
        );
    }
}

/// A control cross held below the hand and its puppet on strings.
fn marionette(v: &mut Vec<PartSpec>, at: Node, f: Form, dress: Color) {
    let lit = f.a && f.t2;
    let string = if f.a { RED_CLOTH } else { STRING };
    v.extend([
        part(at, Box(0.035, 0.03, 0.34), WOOD, [0.0, -0.05, 0.06]),
        part(at, Box(0.3, 0.03, 0.035), WOOD, [0.0, -0.05, 0.12]),
    ]);
    if lit {
        v.push(part(at, Box(0.31, 0.012, 0.04), MAGIC, [0.0, -0.03, 0.12]).glow());
    }
    puppet(v, at, [0.0, -DROP, 0.08], dress, f.a);
    for (a, b) in [
        ([-0.15, -0.05, 0.12], [-0.07, -DROP + 0.02, 0.08]),
        ([0.15, -0.05, 0.12], [0.07, -DROP + 0.02, 0.08]),
        ([0.0, -0.05, 0.22], [0.0, -DROP + 0.11, 0.08]),
        ([0.0, -0.05, -0.1], [0.0, -DROP - 0.06, 0.08]),
    ] {
        let s = gear::strut(at, a, b, 0.004, if lit { MAGIC } else { string });
        v.push(if lit { s.glow() } else { s });
    }
}

/// A little wooden doll: head, dress, limbs; Painful Betrayal arms it.
fn puppet(v: &mut Vec<PartSpec>, at: Node, [x, y, z]: [f32; 3], dress: Color, armed: bool) {
    v.extend([
        part(at, Ball(0.045), LINEN, [x, y + 0.11, z]),
        part(at, Frustum(0.035, 0.06, 0.1), dress, [x, y + 0.03, z]),
        gear::strut(
            at,
            [x - 0.03, y + 0.06, z],
            [x - 0.07, y + 0.0, z],
            0.01,
            OAK,
        ),
        gear::strut(
            at,
            [x + 0.03, y + 0.06, z],
            [x + 0.07, y + 0.0, z],
            0.01,
            OAK,
        ),
        gear::strut(
            at,
            [x - 0.02, y - 0.02, z],
            [x - 0.03, y - 0.07, z],
            0.012,
            OAK,
        ),
        gear::strut(
            at,
            [x + 0.02, y - 0.02, z],
            [x + 0.03, y - 0.07, z],
            0.012,
            OAK,
        ),
    ]);
    if armed {
        v.push(part(
            at,
            Box(0.01, 0.07, 0.01),
            STEEL,
            [x + 0.075, y + 0.02, z],
        ));
    }
}
