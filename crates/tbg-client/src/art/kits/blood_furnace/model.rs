//! Blood Furnace figure: a skeleton knight with a furnace burning in its
//! ribcage behind an iron grate, a flame crest on the skull and a war
//! hammer with a heated head. Branch A (Infernal Core) feeds the fire
//! (bigger core and flame eyes, then a blazing sun with rays and a crown
//! of flame); branch B (Controlled Burn) banks it (a furnace door and
//! bellows, then ember lanterns on chains and smoldering shoulders).

use bevy::prelude::*;

use crate::art::{
    figure::{part, FigureSpec, PartSpec},
    kits::common::{body, gear, Form},
    mesh::Shape::*,
    palette::*,
    rig::{Joint::*, Node::Body, Node::Prop, Node::Secondary, Proportions, Socket, Spring},
};

const EMBER: Color = Color::srgb(1.0, 0.45, 0.12);
const SUN: Color = Color::srgb(1.0, 0.78, 0.3);
const SOOT: Color = Color::srgb(0.22, 0.2, 0.2);
const OLD_BONE: Color = Color::srgb(0.75, 0.71, 0.6);
/// Hammer length (socket space, +Z).
const HAFT: f32 = 0.6;

pub fn plan(tier: u8) -> Proportions {
    let mut p = body::plan(false, 1.06, tier);
    p.torso.x *= 1.1;
    p
}

/// The hammer head (weapon space).
pub fn tip() -> Vec3 {
    Vec3::new(0.0, 0.0, HAFT * 0.85)
}

pub fn spec(tier: u8, branch: u8) -> FigureSpec {
    let f = Form::of(tier, branch);
    let p = plan(tier);
    let mut v = Vec::with_capacity(150);
    body::skeleton(&mut v, &p, OLD_BONE);
    furnace(&mut v, &p, f);
    head(&mut v, &p, f);
    armor(&mut v, &p, f);
    hammer(&mut v, f);
    FigureSpec { plan: p, parts: v }
}

/// The fire in the ribcage and its grate.
fn furnace(v: &mut Vec<PartSpec>, p: &Proportions, f: Form) {
    let t = p.torso;
    let core = if f.a { t.x * 0.3 } else { t.x * 0.24 };
    let fire = if f.a && f.t2 { SUN } else { EMBER };
    v.push(part(Body(Chest), Ball(core), fire, [0.0, t.y * 0.6, 0.0]).glow());
    if f.a && f.t2 {
        // Dying Sun: rays burst out between the ribs.
        for a in [0.0f32, 60.0, 120.0, 180.0, 240.0, 300.0] {
            let r = a.to_radians();
            v.push(
                part(
                    Body(Chest),
                    Cone(0.025, 0.12),
                    SUN,
                    [
                        r.cos() * core * 1.2,
                        t.y * 0.6 + r.sin() * core * 1.2,
                        t.x * 0.15,
                    ],
                )
                .rot([0.0, 0.0, a - 90.0])
                .glow(),
            );
        }
        return;
    }
    // An iron grate across the front of the ribs.
    for x in [-0.06, 0.0, 0.06] {
        v.push(part(
            Body(Chest),
            Box(0.018, t.y * 0.45, 0.02),
            IRON,
            [x, t.y * 0.6, t.x * 0.31],
        ));
    }
    if f.b {
        // Controlled Burn: a furnace door with a vent slit.
        v.extend([
            part(
                Body(Chest),
                Box(t.x * 0.42, t.y * 0.36, 0.02),
                SOOT,
                [0.0, t.y * 0.58, t.x * 0.33],
            ),
            part(
                Body(Chest),
                Box(t.x * 0.3, 0.025, 0.025),
                EMBER,
                [0.0, t.y * 0.62, t.x * 0.34],
            )
            .glow(),
        ]);
    }
}

fn head(v: &mut Vec<PartSpec>, p: &Proportions, f: Form) {
    let h = p.head;
    let fire = if f.a && f.t2 { SUN } else { EMBER };
    body::glowing_eyes(v, p, fire);
    let flames: &[(f32, f32, f32)] = if f.a && f.t2 {
        &[
            (-0.6, 0.8, 0.8),
            (0.0, 1.3, 1.1),
            (0.6, 0.8, 0.8),
            (-0.3, 1.0, 0.9),
            (0.3, 1.0, 0.9),
        ]
    } else if f.a {
        &[(0.0, 1.1, 1.0), (-0.4, 0.8, 0.7), (0.4, 0.8, 0.7)]
    } else {
        &[(0.0, 0.9, 0.8)]
    };
    for &(x, y, s) in flames {
        v.push(
            part(
                Body(Head),
                Cone(h * 0.28 * s, h * 0.9 * s),
                fire,
                [x * h, y * h, -h * 0.1],
            )
            .glow(),
        );
    }
    if f.b {
        body::helmet(v, p, body::Helm::Cap(SOOT));
    }
}

fn armor(v: &mut Vec<PartSpec>, p: &Proportions, f: Form) {
    let t = p.torso;
    body::pauldrons(v, if f.b && f.t2 { SOOT } else { IRON }, 0.1);
    body::greaves(v, IRON);
    v.extend([
        part(
            Body(Hips),
            Box(t.x * 0.9, 0.16, t.z * 0.9),
            SOOT,
            [0.0, -0.06, 0.0],
        ),
        part(
            Body(Hips),
            Box(t.x * 0.35, 0.18, 0.02),
            ALLY,
            [0.0, -0.08, t.z * 0.47],
        ),
    ]);
    if f.b && f.t2 {
        // Eternal Embers: lanterns of ember swinging from chains, and
        // smoldering coals on the shoulders.
        for x in [-0.15, 0.15] {
            v.push(part(Body(Hips), Cyl(0.008, 0.12), IRON, [x, -0.12, 0.08]));
            v.extend(gear::lantern(Body(Hips), [x, -0.22, 0.08], 0.9));
        }
        for arm in [UpperArmL, UpperArmR] {
            v.push(part(Body(arm), Ball(0.035), EMBER, [0.0, 0.08, 0.0]).glow());
        }
    } else if f.b {
        // Bellows on the back.
        let b = Secondary(Spring::Back);
        v.extend([
            part(b, Box(0.2, 0.26, 0.05), WOOD, [0.0, 0.0, -0.06]),
            part(b, Box(0.18, 0.2, 0.06), DARK_LEATHER, [0.0, 0.0, -0.11]),
            part(b, Cone(0.03, 0.12), BRASS, [0.0, 0.18, -0.08]),
        ]);
    }
}

/// A war hammer in the right hand with a heated head.
fn hammer(v: &mut Vec<PartSpec>, f: Form) {
    let w = Prop(Socket::WeaponR);
    let head = if f.a && f.t2 { SUN } else { IRON };
    v.extend(gear::hammer(w, HAFT, head));
    let band = part(w, Cyl(0.075, 0.05), EMBER, [0.0, 0.0, HAFT * 0.85]).rot([90.0, 0.0, 0.0]);
    v.push(if f.a || f.t2 { band.glow() } else { band });
}
