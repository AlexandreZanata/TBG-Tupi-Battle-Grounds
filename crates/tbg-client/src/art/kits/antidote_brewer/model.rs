//! Antidote Brewer figure: a sprout-folk herbalist in a leaf hood and a
//! linen apron, a glowing cure flask in hand and an herb basket on the
//! back. Branch A (Master Antidote) gilds the flask and crowns the hood,
//! then carries a glowing still; branch B (Rapid Brewer) straps on a
//! bandolier of flasks, then brews in both hands from a back cauldron.

use bevy::prelude::*;

use crate::art::{
    figure::{part, FigureSpec, PartSpec},
    kits::common::{body, gear, Form},
    mesh::Shape::*,
    palette::*,
    rig::{Joint::*, Node::Body, Node::Prop, Node::Secondary, Proportions, Socket, Spring},
};

const SPROUT: Color = Color::srgb(0.74, 0.88, 0.56);
pub const CURE: Color = Color::srgb(0.4, 0.95, 0.85);
const WICKER: Color = Color::srgb(0.66, 0.5, 0.28);
const GLASS: Color = Color::srgb(0.7, 0.9, 0.95);

/// The flask in the right hand (weapon space): the cure leaves from here.
pub const FLASK: Vec3 = Vec3::new(0.0, 0.07, 0.03);

pub fn plan(tier: u8) -> Proportions {
    body::plan(false, 0.92, tier)
}

pub fn spec(tier: u8, branch: u8) -> FigureSpec {
    let f = Form::of(tier, branch);
    let p = plan(tier);
    let mut v = Vec::with_capacity(130);
    let look = body::Look {
        skin: SPROUT,
        cloth: HOOD_GREEN,
        legs: BARK,
        boots: DARK_LEATHER,
    };
    body::humanoid(&mut v, &p, &look);
    body::tunic(&mut v, &p, HOOD_GREEN, LEAF_DARK, LEATHER);
    let t = p.torso;
    v.extend([
        // Apron with the team's leaf on it.
        part(
            Body(Chest),
            Box(t.x * 0.7, t.y * 0.7, 0.02),
            LINEN,
            [0.0, t.y * 0.4, t.z * 0.42],
        ),
        part(
            Body(Hips),
            Box(t.x * 0.7, 0.24, 0.02),
            LINEN,
            [0.0, -0.1, t.z * 0.55],
        ),
        part(Body(Chest), Ball(0.05), ALLY, [0.0, t.y * 0.55, t.z * 0.44]).scale([0.7, 1.3, 0.3]),
    ]);
    head(&mut v, &p, f);
    hands(&mut v, f);
    back(&mut v, f);
    if f.b {
        // Quick Mixing: a bandolier of flasks across the chest.
        v.push(
            part(
                Body(Chest),
                Box(0.05, t.y * 1.2, 0.03),
                LEATHER,
                [0.0, t.y * 0.5, t.z * 0.46],
            )
            .rot([0.0, 0.0, 40.0]),
        );
        for k in [-0.3f32, -0.1, 0.1, 0.3] {
            v.extend(gear::flask(
                Body(Chest),
                [k * t.x * 1.2, t.y * (0.5 - k * 0.9), t.z * 0.52],
                CURE,
                0.6,
            ));
        }
    }
    FigureSpec { plan: p, parts: v }
}

/// A hood of two broad leaves and a sprout on top.
fn head(v: &mut Vec<PartSpec>, p: &Proportions, f: Form) {
    let h = p.head;
    body::hood(v, p, LEAF_DARK, LEAF);
    let g = Secondary(Spring::HeadGear);
    v.extend([
        part(g, Cyl(0.012, h * 0.8), LEAF_DARK, [0.0, h * 1.3, -h * 0.2]),
        part(g, Ball(h * 0.32), LEAF, [h * 0.25, h * 1.7, -h * 0.2])
            .scale([1.4, 0.35, 0.8])
            .rot([0.0, 0.0, 25.0]),
        part(g, Ball(h * 0.28), LEAF, [-h * 0.22, h * 1.6, -h * 0.2])
            .scale([1.4, 0.35, 0.8])
            .rot([0.0, 0.0, -25.0]),
    ]);
    if f.a {
        // A crown of herbs: little blossoms round the hood.
        for a in [-50.0f32, -20.0, 20.0, 50.0] {
            let r = a.to_radians();
            v.push(part(
                Body(Head),
                Ball(0.03),
                if f.t2 { GOLD } else { BLOSSOM },
                [r.sin() * h * 1.05, h * 0.75, r.cos() * h * 0.7],
            ));
        }
    }
}

fn hands(v: &mut Vec<PartSpec>, f: Form) {
    let r = Prop(Socket::WeaponR);
    let big = if f.a { 1.5 } else { 1.25 };
    v.extend(gear::flask(r, FLASK.to_array(), CURE, big));
    if f.a {
        v.push(part(
            r,
            Ring(0.01, 0.075 * big * 0.85, 360.0),
            GOLD,
            [0.0, 0.07, 0.03],
        ));
    }
    if f.b && f.t2 {
        v.extend(gear::flask(
            Prop(Socket::OffhandL),
            [0.0, 0.07, 0.03],
            CURE,
            1.25,
        ));
    }
}

/// The herb basket, the glowing still or the cauldron.
fn back(v: &mut Vec<PartSpec>, f: Form) {
    let b = Secondary(Spring::Back);
    match (f.a, f.b, f.t2) {
        (true, _, true) => v.extend([
            part(b, Ball(0.13), GLASS, [0.0, 0.0, -0.1]),
            part(b, Ball(0.09), CURE, [0.0, -0.01, -0.1]).glow(),
            part(b, Cyl(0.03, 0.2), BRASS, [0.0, 0.18, -0.1]),
            gear::strut(b, [0.0, 0.27, -0.1], [0.16, 0.12, -0.02], 0.015, BRASS),
            part(b, Box(0.3, 0.05, 0.16), WOOD, [0.0, -0.14, -0.1]),
        ]),
        (_, true, true) => v.extend([
            part(b, Frustum(0.15, 0.11, 0.18), IRON, [0.0, -0.02, -0.12]),
            part(b, Cyl(0.13, 0.02), CURE, [0.0, 0.07, -0.12]).glow(),
            part(b, Ring(0.015, 0.15, 360.0), DARK, [0.0, 0.07, -0.12]),
        ]),
        _ => {
            v.extend([
                part(b, Cyl(0.12, 0.2), WICKER, [0.0, 0.0, -0.1]),
                part(b, Ring(0.015, 0.12, 360.0), BARK, [0.0, 0.1, -0.1]),
            ]);
            for (x, z) in [(-0.05, -0.08), (0.05, -0.12), (0.0, -0.06)] {
                v.push(part(b, Ball(0.05), LEAF, [x, 0.13, z]).scale([0.6, 1.4, 0.6]));
            }
        }
    }
}
