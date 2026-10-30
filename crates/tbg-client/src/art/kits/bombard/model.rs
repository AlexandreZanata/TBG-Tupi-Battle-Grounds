//! Bombard figure: a stout Ironworks gunner in riveted plate with a short
//! mortar at the hip. Branch A (Destruction) flares the bell and racks
//! bombs on the back, then a great mortar with a glowing band and a
//! furnace pack; branch B (Iron Fortress) adds heavy plate and a hull
//! shield, then a twin-barrel mortar fed from an ammunition drum.

use bevy::prelude::*;

use crate::art::{
    figure::{part, FigureSpec, PartSpec},
    kits::common::{body, gear, Form},
    mesh::Shape::*,
    palette::*,
    rig::{Joint::*, Node::Body, Node::Prop, Node::Secondary, Proportions, Socket, Spring},
};

/// Where the left hand holds the barrel (weapon space).
pub const FORE: Vec3 = Vec3::new(0.0, -0.03, 0.24);
/// Barrel axis height above the grip.
const BORE: f32 = 0.1;
const X90: [f32; 3] = [90.0, 0.0, 0.0];
const HOT: Color = Color::srgb(1.0, 0.55, 0.18);

pub fn plan(tier: u8) -> Proportions {
    let mut p = body::plan(false, 1.1, tier);
    p.leg *= 0.86;
    p.torso.x *= 1.2;
    p.torso.z *= 1.2;
    p
}

/// Barrel radius and length per form.
fn barrel(f: Form) -> (f32, f32) {
    match (f.a, f.b, f.t2) {
        (true, _, false) => (0.1, 0.4),
        (true, _, true) => (0.112, 0.46),
        (_, true, true) => (0.07, 0.42),
        _ => (0.088, 0.38),
    }
}

/// The bomb leaves here (weapon space).
pub fn muzzle(f: Form) -> Vec3 {
    let (_, len) = barrel(f);
    Vec3::new(0.0, BORE, len + 0.05)
}

pub fn spec(tier: u8, branch: u8) -> FigureSpec {
    let f = Form::of(tier, branch);
    let p = plan(tier);
    let mut v = Vec::with_capacity(140);
    let look = body::Look {
        skin: OGRE_SKIN,
        cloth: DARK_LEATHER,
        legs: GUNMETAL,
        boots: DARK,
    };
    body::humanoid(&mut v, &p, &look);
    armor(&mut v, &p, f);
    head(&mut v, &p, f);
    back(&mut v, f);
    mortar(&mut v, f);
    FigureSpec { plan: p, parts: v }
}

fn armor(v: &mut Vec<PartSpec>, p: &Proportions, f: Form) {
    let t = p.torso;
    let plate = if f.b { IRON } else { GUNMETAL };
    v.extend([
        part(
            Body(Chest),
            Box(t.x * 1.1, t.y * 0.72, t.z * 1.18),
            plate,
            [0.0, t.y * 0.55, 0.0],
        ),
        part(
            Body(Chest),
            Box(t.x * 0.5, t.y * 0.3, 0.03),
            BRASS,
            [0.0, t.y * 0.62, t.z * 0.6],
        ),
        part(
            Body(Hips),
            Box(t.x * 1.05, 0.06, t.z * 1.15),
            DARK_LEATHER,
            [0.0, 0.02, 0.0],
        ),
        part(
            Body(Hips),
            Box(0.06, 0.06, 0.02),
            BRASS,
            [0.0, 0.02, t.z * 0.6],
        ),
    ]);
    for (x, y) in [(-0.3, 0.4), (0.3, 0.4), (-0.3, 0.8), (0.3, 0.8)] {
        v.push(part(
            Body(Chest),
            Ball(0.014),
            BRASS,
            [t.x * x, t.y * y, t.z * 0.6],
        ));
    }
    // Two bombs ready on the belt.
    for x in [-0.15, 0.15] {
        v.extend(gear::bomb(Body(Hips), [x, -0.04, t.z * 0.62], 0.8));
    }
    body::pauldrons(v, plate, if f.b { 0.13 } else { 0.095 });
    body::greaves(v, IRON);
    body::bracers(v, p, BRASS);
    if f.b {
        // Iron Fortress: tassets over the thighs and plated forearms.
        for thigh in [ThighL, ThighR] {
            v.push(part(
                Body(thigh),
                Box(0.15, 0.14, 0.15),
                IRON,
                [0.0, -0.06, 0.02],
            ));
        }
    }
    if f.a {
        v.push(
            part(
                Body(Chest),
                Box(0.06, t.y * 1.1, 0.03),
                ORANGE,
                [0.0, t.y * 0.5, t.z * 0.62],
            )
            .rot([0.0, 0.0, 35.0]),
        );
    }
}

fn head(v: &mut Vec<PartSpec>, p: &Proportions, f: Form) {
    let h = p.head;
    body::helmet(v, p, body::Helm::Cap(if f.b { IRON } else { GUNMETAL }));
    body::beard(v, p, DARK_LEATHER, 0.1);
    if f.b {
        // A visor plate with a vision slit.
        v.push(part(
            Body(Head),
            Box(h * 1.5, h * 0.55, 0.03),
            IRON,
            [0.0, h * 0.1, h * 0.98],
        ));
        v.push(part(
            Body(Head),
            Box(h * 1.1, 0.02, 0.035),
            DARK,
            [0.0, h * 0.18, h * 0.99],
        ));
        if f.t2 {
            v.push(
                part(
                    Body(Head),
                    Box(h * 0.9, 0.012, 0.04),
                    HOT,
                    [0.0, h * 0.18, h * 1.0],
                )
                .glow(),
            );
        }
        return;
    }
    // Brass goggles; Heavy Ordnance lights them.
    for s in [-1.0f32, 1.0] {
        let at = [s * h * 0.36, h * 0.3, h * 0.92];
        v.push(part(Body(Head), Cyl(0.04, 0.04), BRASS, at).rot(X90));
        let lens = part(
            Body(Head),
            Ball(0.028),
            ORANGE,
            [at[0], at[1], at[2] + 0.02],
        );
        v.push(if f.a && f.t2 { lens.glow() } else { lens });
    }
    v.push(part(
        Body(Head),
        Box(h * 1.9, 0.025, h * 0.6),
        DARK_LEATHER,
        [0.0, h * 0.3, h * 0.4],
    ));
}

/// The back: a powder keg, a bomb rack, a furnace pack, a hull shield or
/// an ammunition drum.
fn back(v: &mut Vec<PartSpec>, f: Form) {
    let b = Secondary(Spring::Back);
    match (f.a, f.b, f.t2) {
        (true, _, false) => {
            v.push(part(b, Box(0.3, 0.05, 0.05), WOOD, [0.0, -0.05, -0.06]));
            for x in [-0.1, 0.0, 0.1] {
                v.extend(gear::bomb(b, [x, 0.02, -0.08], 1.1));
            }
        }
        (true, _, true) => {
            v.extend([
                part(b, Box(0.32, 0.36, 0.18), GUNMETAL, [0.0, 0.0, -0.1]),
                part(b, Box(0.2, 0.12, 0.02), HOT, [0.0, -0.04, -0.2]).glow(),
                part(b, Cyl(0.045, 0.3), IRON, [0.1, 0.28, -0.1]),
                part(b, Cyl(0.06, 0.04), BRASS, [0.1, 0.44, -0.1]),
            ]);
            for x in [-0.1, 0.0, 0.1] {
                v.extend(gear::bomb(b, [x, 0.24, -0.06], 1.0));
            }
        }
        (_, true, false) => hull_shield(v, 0.42),
        (_, true, true) => {
            hull_shield(v, 0.36);
            v.extend([
                part(b, Cyl(0.13, 0.14), BRASS, [0.0, 0.1, -0.2]).rot([0.0, 0.0, 90.0]),
                part(b, Cyl(0.06, 0.15), GUNMETAL, [0.0, 0.1, -0.2]).rot([0.0, 0.0, 90.0]),
            ]);
            // The feed belt runs over the right shoulder.
            for i in 0..5 {
                let k = i as f32 / 4.0;
                v.push(part(
                    b,
                    Box(0.05, 0.03, 0.05),
                    BRASS,
                    [-0.12 - 0.06 * k, 0.2 + 0.08 * k, -0.08 + 0.2 * k],
                ));
            }
        }
        _ => v.extend([
            part(b, Cyl(0.09, 0.18), OAK, [0.0, -0.02, -0.08]),
            part(b, Cyl(0.095, 0.025), IRON, [0.0, 0.04, -0.08]),
            part(b, Cyl(0.095, 0.025), IRON, [0.0, -0.08, -0.08]),
        ]),
    }
}

fn hull_shield(v: &mut Vec<PartSpec>, size: f32) {
    let b = Secondary(Spring::Back);
    v.extend([
        part(b, Box(size, size * 1.15, 0.04), IRON, [0.0, 0.0, -0.06]),
        part(
            b,
            Box(size * 0.8, size * 0.95, 0.02),
            ALLY,
            [0.0, 0.0, -0.085],
        ),
        part(
            b,
            Box(size * 0.15, size * 0.95, 0.025),
            BRASS,
            [0.0, 0.0, -0.09],
        ),
    ]);
}

/// The mortar in the right hand's socket: grip at the origin, barrel
/// along +Z above it.
fn mortar(v: &mut Vec<PartSpec>, f: Form) {
    let w = Prop(Socket::WeaponR);
    let (r, len) = barrel(f);
    v.extend([
        part(w, Box(0.05, 0.1, 0.08), DARK_LEATHER, [0.0, 0.02, 0.0]),
        part(w, Ball(r * 1.1), GUNMETAL, [0.0, BORE, 0.0]),
        part(
            w,
            Box(0.03, 0.08, 0.04),
            DARK_LEATHER,
            [0.0, BORE - r - 0.03, 0.24],
        ),
    ]);
    let bores: &[f32] = if f.b && f.t2 {
        &[-0.075, 0.075]
    } else {
        &[0.0]
    };
    let bell = if f.a { 1.38 } else { 1.2 };
    for &x in bores {
        v.extend([
            part(w, Cyl(r, len), GUNMETAL, [x, BORE, len * 0.5]).rot(X90),
            part(
                w,
                Frustum(r * bell, r * 1.02, 0.09),
                GUNMETAL,
                [x, BORE, len],
            )
            .rot(X90),
            part(w, Cyl(r * bell * 0.72, 0.01), DARK, [x, BORE, len + 0.046]).rot(X90),
        ]);
        for z in [0.25, 0.62] {
            v.push(part(w, Cyl(r * 1.1, 0.03), BRASS, [x, BORE, len * z]).rot(X90));
        }
    }
    if f.a && f.t2 {
        v.push(
            part(w, Cyl(r * 1.12, 0.045), HOT, [0.0, BORE, len * 0.44])
                .rot(X90)
                .glow(),
        );
    }
    if f.b {
        // Reinforced trunnions and a sighting post.
        v.push(part(w, Cyl(0.03, r * 2.6 + 0.15), IRON, [0.0, BORE, 0.08]).rot([0.0, 0.0, 90.0]));
        v.push(part(
            w,
            Box(0.02, 0.06, 0.02),
            BRASS,
            [0.0, BORE + r + 0.03, len * 0.8],
        ));
    }
}
