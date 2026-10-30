//! Farwatch figure: a hooded watchman with a heavy crossbow (arbalest)
//! that outranges the lane. Branch A steadies the shot (a brass scope,
//! then a gilded great arbalest, feathered hat and team cape); branch B
//! cruels it (barbed red-fletched bolts and a masked hood, then a glowing
//! wounding bolt and a dark red cloak).

use bevy::prelude::*;

use crate::art::{
    figure::{part, FigureSpec, PartSpec},
    kits::common::{body, gear, Form},
    mesh::Shape::*,
    palette::*,
    rig::{Joint::*, Node::Body, Node::Prop, Proportions, Socket},
};

const COAT: Color = Color::srgb(0.27, 0.34, 0.4);
const WOUND: Color = Color::srgb(0.9, 0.18, 0.2);

pub fn plan(tier: u8) -> Proportions {
    let mut p = body::plan(false, 1.0, tier);
    p.arm = 0.2;
    p
}

/// The bolt's tip on the crossbow (weapon socket space).
pub const MUZZLE: Vec3 = Vec3::new(0.0, 0.05, 0.58);
/// Where the left hand holds the fore-stock.
pub const FORE: Vec3 = Vec3::new(0.02, 0.0, 0.3);

pub fn spec(tier: u8, branch: u8) -> FigureSpec {
    let f = Form::of(tier, branch);
    let p = plan(tier);
    let mut v = Vec::with_capacity(110);
    let look = body::Look {
        skin: SKIN,
        cloth: COAT,
        legs: DARK_LEATHER,
        boots: DARK_LEATHER,
    };
    body::humanoid(&mut v, &p, &look);
    body::tunic(&mut v, &p, COAT, DARK_LEATHER, LEATHER);
    body::bracers(&mut v, &p, DARK_LEATHER);
    let fletch = if f.b { WOUND } else { LINEN };
    v.extend(gear::quiver(DARK_LEATHER, fletch));
    head(&mut v, &p, f);
    dress(&mut v, &p, f);
    crossbow(&mut v, f);
    FigureSpec { plan: p, parts: v }
}

fn head(v: &mut Vec<PartSpec>, p: &Proportions, f: Form) {
    let h = p.head;
    let hood = if f.b { DARK } else { COAT };
    if f.a && f.t2 {
        // Finishing Shot: a wide feathered hat.
        v.push(part(
            Body(Head),
            Cyl(h * 1.7, 0.025),
            DARK_LEATHER,
            [0.0, h * 0.5, 0.0],
        ));
        v.push(
            part(
                Body(Head),
                Ball(h * 0.95),
                DARK_LEATHER,
                [0.0, h * 0.62, 0.0],
            )
            .scale([1.0, 0.6, 1.0]),
        );
        v.push(
            part(
                Body(Head),
                Box(0.02, 0.05, h * 1.6),
                RED_CLOTH,
                [h * 0.6, h * 0.8, -h * 0.2],
            )
            .rot([20.0, 0.0, -15.0]),
        );
    } else {
        body::hood(v, p, hood, DARK_LEATHER);
    }
    if f.b {
        // A scarf over the mouth.
        v.push(part(
            Body(Head),
            Box(h * 1.6, h * 0.5, h * 0.5),
            DARK,
            [0.0, -h * 0.45, h * 0.62],
        ));
    }
    if f.b && f.t2 {
        body::glowing_eyes(v, p, WOUND);
    }
}

fn dress(v: &mut Vec<PartSpec>, p: &Proportions, f: Form) {
    if f.a {
        // A longer field coat over the shoulders.
        v.push(
            part(
                Body(Chest),
                Frustum(p.torso.x * 0.5, p.torso.x * 0.62, 0.16),
                COAT,
                [0.0, p.torso.y * 0.86, 0.0],
            )
            .scale([1.0, 1.0, 0.8]),
        );
    }
    if f.t2 {
        let cape = if f.a {
            ALLY
        } else {
            Color::srgb(0.35, 0.08, 0.1)
        };
        body::cape(v, cape, 0.28, 0.48);
    }
    if f.b {
        v.push(
            part(
                Body(Chest),
                Box(0.06, p.torso.y * 1.05, 0.03),
                WOUND,
                [0.03, p.torso.y * 0.5, p.torso.z * 0.45],
            )
            .rot([0.0, 0.0, 30.0]),
        );
    }
}

/// The arbalest in the right hand's socket, along +Z.
fn crossbow(v: &mut Vec<PartSpec>, f: Form) {
    let w = Prop(Socket::WeaponR);
    let (wood, trim) = if f.a && f.t2 {
        (OAK, GOLD)
    } else {
        (WOOD, IRON)
    };
    let span = if f.a && f.t2 { 0.62 } else { 0.5 };
    let bolt = if f.b { WOUND } else { TIMBER };
    v.extend([
        part(w, Box(0.06, 0.08, 0.6), wood, [0.0, 0.02, 0.22]),
        part(w, Box(0.05, 0.12, 0.08), wood, [0.0, -0.05, -0.04]),
        part(w, Box(span, 0.035, 0.04), STEEL, [0.0, 0.05, 0.48]),
        part(w, Box(span * 0.95, 0.008, 0.008), LINEN, [0.0, 0.06, 0.34]),
        part(w, Box(0.012, 0.012, 0.3), bolt, [0.0, 0.08, 0.42]),
        part(w, Ring(0.008, 0.05, 360.0), trim, [0.0, 0.0, 0.6]).rot([90.0, 0.0, 0.0]),
        part(w, Cyl(0.04, 0.04), trim, [0.04, 0.02, 0.02]).rot([0.0, 0.0, 90.0]),
    ]);
    if f.a {
        let top = if f.t2 { GOLD } else { BRASS };
        v.push(part(w, Cyl(0.025, 0.3), top, [0.0, 0.13, 0.25]).rot([90.0, 0.0, 0.0]));
        let lens = part(w, Cyl(0.028, 0.02), WATER, [0.0, 0.13, 0.41]).rot([90.0, 0.0, 0.0]);
        v.push(if f.t2 { lens.glow() } else { lens });
    }
    if f.b && f.t2 {
        v.push(
            part(w, Cone(0.02, 0.06), WOUND, [0.0, 0.08, 0.6])
                .rot([90.0, 0.0, 0.0])
                .glow(),
        );
    }
    if f.t2 {
        v.push(part(w, Box(0.065, 0.02, 0.4), trim, [0.0, 0.065, 0.2]));
    }
}
