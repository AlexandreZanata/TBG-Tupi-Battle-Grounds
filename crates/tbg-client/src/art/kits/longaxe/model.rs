//! Longaxe figure: a halberdier in a padded gambeson and kettle hat with a
//! poleaxe longer than he is tall. Branch A hardens the cut (breastplate
//! and broad head, then a gilded executioner's crescent); branch B widens
//! the swing (twin-bladed bardiche, then a great halberd and horned helm).

use bevy::prelude::*;

use crate::art::{
    figure::{part, FigureSpec, PartSpec},
    kits::common::{body, body::Helm, Form},
    mesh::Shape::*,
    palette::*,
    rig::{Joint::*, Node::Body, Node::Prop, Proportions, Socket},
};

/// The poleaxe's haft length (socket +Y) and where its head sits.
pub const HAFT: f32 = 2.40;
pub const HEAD_Y: f32 = HAFT * 0.72;

pub fn plan(tier: u8) -> Proportions {
    let mut p = body::plan(false, 1.05, tier);
    p.arm = 0.19;
    p
}

/// The blade's working point in the weapon socket (impact anchor).
pub fn tip(_f: Form) -> Vec3 {
    Vec3::new(0.0, HEAD_Y, 0.22)
}

pub fn spec(tier: u8, branch: u8) -> FigureSpec {
    let f = Form::of(tier, branch);
    let p = plan(tier);
    let mut v = Vec::with_capacity(120);
    let look = body::Look {
        skin: SKIN,
        cloth: TRIM,
        legs: RUST,
        boots: DARK_LEATHER,
    };
    body::humanoid(&mut v, &p, &look);
    body::tunic(&mut v, &p, TRIM, DARK_LEATHER, LEATHER);
    body::tabard(&mut v, 0.15, 0.36);
    body::bracers(&mut v, &p, LEATHER);
    armor(&mut v, &p, f);
    head(&mut v, &p, f);
    poleaxe(&mut v, f);
    FigureSpec { plan: p, parts: v }
}

fn armor(v: &mut Vec<PartSpec>, p: &Proportions, f: Form) {
    let h = p.torso.y;
    // Quilted gambeson ribs.
    for y in [0.3, 0.55, 0.8] {
        v.push(part(
            Body(Chest),
            Box(p.torso.x * 0.95, 0.015, 0.02),
            DARK_LEATHER,
            [0.0, h * y, p.torso.z * 0.42],
        ));
    }
    if f.a {
        let plate = if f.t2 { STEEL } else { IRON };
        v.push(
            part(
                Body(Chest),
                Box(p.torso.x * 0.9, h * 0.55, 0.05),
                plate,
                [0.0, h * 0.62, p.torso.z * 0.42],
            )
            .rot([-6.0, 0.0, 0.0]),
        );
        body::pauldrons(v, plate, 0.085);
    }
    if f.b {
        body::pauldrons(v, STEEL, 0.08);
        v.push(
            part(
                Body(Chest),
                Box(0.06, h * 1.1, 0.03),
                ALLY,
                [0.02, h * 0.5, p.torso.z * 0.46],
            )
            .rot([0.0, 0.0, -32.0]),
        );
    }
    if f.t2 {
        body::greaves(v, STEEL);
        let cape = if f.a { RED_CLOTH } else { ALLY };
        body::cape(v, cape, 0.3, 0.46);
    }
}

fn head(v: &mut Vec<PartSpec>, p: &Proportions, f: Form) {
    body::beard(v, p, OAK, 0.07);
    let helm = match (f.a, f.b, f.t2) {
        (true, _, true) => Helm::Plumed(STEEL, RED_CLOTH),
        (_, true, true) => Helm::Horned(IRON, BONE),
        (true, _, false) => Helm::Cap(STEEL),
        _ => Helm::Kettle(IRON),
    };
    body::helmet(v, p, helm);
}

/// The poleaxe in the right hand's socket, haft along +Y.
fn poleaxe(v: &mut Vec<PartSpec>, f: Form) {
    let w = Prop(Socket::WeaponR);
    let top = HEAD_Y;
    v.extend([
        part(w, Cyl(0.024, HAFT), WOOD, [0.0, HAFT * 0.22, 0.0]),
        part(w, Cyl(0.03, 0.05), IRON, [0.0, -HAFT * 0.27, 0.0]),
        part(w, Cone(0.025, 0.16), STEEL, [0.0, top + 0.18, 0.0]),
        part(w, Cyl(0.032, 0.05), BRASS, [0.0, top - 0.24, 0.0]),
    ]);
    match (f.a, f.b, f.t2) {
        // Crushing Edge: a gilded executioner's crescent, its edge aglow.
        (true, _, true) => {
            v.push(part(
                w,
                Box(0.045, 0.55, 0.28),
                STEEL,
                [0.0, top - 0.04, 0.14],
            ));
            v.push(
                part(w, Ring(0.03, 0.24, 150.0), GOLD, [0.0, top - 0.04, 0.12])
                    .rot([0.0, 0.0, 90.0]),
            );
            v.push(part(w, Box(0.032, 0.44, 0.03), SAP, [0.0, top - 0.04, 0.25]).glow());
        }
        // Deep Cut: a broad, heavier head.
        (true, _, false) => {
            v.push(part(
                w,
                Box(0.03, 0.36, 0.26),
                IRON,
                [0.0, top - 0.06, 0.14],
            ));
            v.push(part(
                w,
                Box(0.034, 0.38, 0.03),
                STEEL,
                [0.0, top - 0.06, 0.28],
            ));
        }
        // Sweeping Arc: a twin-bladed bardiche.
        (_, true, false) => {
            for s in [1.0f32, -1.0] {
                v.push(part(
                    w,
                    Box(0.03, 0.3, 0.2),
                    IRON,
                    [0.0, top - 0.06, 0.12 * s],
                ));
                v.push(part(
                    w,
                    Box(0.034, 0.32, 0.03),
                    STEEL,
                    [0.0, top - 0.06, 0.23 * s],
                ));
            }
        }
        // Line Breaker: a great halberd with a hook and a glowing edge.
        (_, true, true) => {
            v.push(part(
                w,
                Box(0.035, 0.46, 0.3),
                IRON,
                [0.0, top - 0.04, 0.17],
            ));
            v.push(part(w, Box(0.036, 0.48, 0.03), ALLY, [0.0, top - 0.04, 0.33]).glow());
            v.push(
                part(w, Cone(0.04, 0.22), IRON, [0.0, top - 0.02, -0.16]).rot([-100.0, 0.0, 0.0]),
            );
            v.push(part(w, Cone(0.035, 0.26), STEEL, [0.0, top + 0.3, 0.0]));
        }
        _ => {
            v.push(part(w, Box(0.03, 0.3, 0.22), IRON, [0.0, top - 0.08, 0.12]));
            v.push(part(
                w,
                Box(0.034, 0.32, 0.03),
                STEEL,
                [0.0, top - 0.08, 0.24],
            ));
            v.push(
                part(w, Cone(0.03, 0.14), IRON, [0.0, top - 0.06, -0.08]).rot([-90.0, 0.0, 0.0]),
            );
        }
    }
}
