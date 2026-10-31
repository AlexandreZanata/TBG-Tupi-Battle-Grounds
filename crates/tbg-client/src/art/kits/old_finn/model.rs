//! Old Finn figure: a stooped old fisherman in a yellow oilskin and
//! sou'wester, white beard, pipe, a long rod and a wicker creel. Branch A
//! lands bigger fish (a fuller creel, then a giant catfish over the back
//! and a gold hook); branch B lands more (a net and a second rod, then a
//! net heaving with fish and a lantern).

use bevy::prelude::*;

use crate::art::{
    figure::{part, FigureSpec, PartSpec},
    kits::common::{body, gear, Form},
    mesh::Shape::*,
    palette::*,
    rig::{Joint::*, Node::Body, Node::Prop, Node::Secondary, Proportions, Socket, Spring},
};

const OILSKIN: Color = Color::srgb(0.88, 0.72, 0.22);
const FISH: Color = Color::srgb(0.55, 0.65, 0.72);
const WICKER: Color = Color::srgb(0.66, 0.5, 0.28);

/// The rod's length along the hand socket's +Y, and its tip.
pub const ROD: f32 = 1.1;
pub const TIP: Vec3 = Vec3::new(0.0, ROD * 0.82, 0.0);

pub fn plan(tier: u8) -> Proportions {
    let mut p = body::plan(false, 0.95, tier);
    p.arm = 0.18;
    p
}

pub fn spec(tier: u8, branch: u8) -> FigureSpec {
    let f = Form::of(tier, branch);
    let p = plan(tier);
    let mut v = Vec::with_capacity(110);
    let look = body::Look {
        skin: SKIN,
        cloth: OILSKIN,
        legs: DARK_LEATHER,
        boots: IRON,
    };
    body::humanoid(&mut v, &p, &look);
    body::tunic(
        &mut v,
        &p,
        OILSKIN,
        Color::srgb(0.7, 0.55, 0.15),
        DARK_LEATHER,
    );
    head(&mut v, &p, f);
    creel(&mut v, f);
    rod(&mut v, f);
    // Buttons down the oilskin and a team neckerchief.
    for y in [0.12, 0.22, 0.32] {
        v.push(part(
            Body(Chest),
            Ball(0.018),
            BRASS,
            [0.0, y, p.torso.z * 0.47],
        ));
    }
    v.push(part(
        Body(Chest),
        Box(0.2, 0.06, 0.06),
        ALLY,
        [0.0, p.torso.y * 0.95, p.torso.z * 0.3],
    ));
    if f.a {
        for hand in [HandL, HandR] {
            v.push(part(
                Body(hand),
                Box(0.11, 0.08, 0.12),
                RED_CLOTH,
                [0.0, -0.01, 0.0],
            ));
        }
    }
    FigureSpec { plan: p, parts: v }
}

fn head(v: &mut Vec<PartSpec>, p: &Proportions, f: Form) {
    let h = p.head;
    body::beard(v, p, LINEN, 0.13);
    v.extend([
        // Bushy brows, a red nose, a clay pipe.
        part(
            Body(Head),
            Box(h * 1.2, h * 0.16, h * 0.3),
            LINEN,
            [0.0, h * 0.32, h * 0.8],
        ),
        part(
            Body(Head),
            Ball(h * 0.22),
            Color::srgb(0.85, 0.5, 0.45),
            [0.0, -h * 0.05, h * 1.0],
        ),
        part(
            Body(Head),
            Cyl(0.012, 0.12),
            LINEN,
            [h * 0.3, -h * 0.45, h * 1.05],
        )
        .rot([80.0, 0.0, 0.0]),
        part(
            Body(Head),
            Cyl(0.03, 0.05),
            WOOD,
            [h * 0.3, -h * 0.38, h * 1.6],
        ),
        // Sou'wester: a dome with a long back brim.
        part(
            Body(Head),
            Ball(h * 1.08),
            OILSKIN,
            [0.0, h * 0.3, -h * 0.05],
        )
        .scale([1.05, 0.62, 1.05]),
        part(
            Body(Head),
            Cyl(h * 1.55, 0.02),
            OILSKIN,
            [0.0, h * 0.28, -h * 0.25],
        )
        .rot([-10.0, 0.0, 0.0]),
    ]);
    if f.b && f.t2 {
        // Fish Shower: a hook pinned to the hat.
        v.push(
            part(
                Body(Head),
                Ring(0.008, 0.04, 270.0),
                GOLD,
                [h * 0.6, h * 0.55, 0.0],
            )
            .rot([0.0, 0.0, 90.0]),
        );
    }
}

fn fish(v: &mut Vec<PartSpec>, at: [f32; 3], size: f32, color: Color, tilt: f32) {
    let node = Secondary(Spring::Back);
    v.push(
        part(node, Ball(0.06 * size), color, at)
            .scale([0.7, 0.9, 1.9])
            .rot([tilt, 0.0, 0.0]),
    );
    v.push(
        part(
            node,
            Cone(0.05 * size, 0.08 * size),
            color,
            [at[0], at[1] - 0.12 * size, at[2]],
        )
        .scale([1.0, 1.0, 0.3]),
    );
}

/// The wicker creel on the back spring, fish tails out of the lid.
fn creel(v: &mut Vec<PartSpec>, f: Form) {
    let b = Secondary(Spring::Back);
    let size = if f.a { 1.2 } else { 1.0 };
    v.extend([
        part(
            b,
            Box(0.24 * size, 0.24 * size, 0.14 * size),
            WICKER,
            [0.0, -0.05, -0.08],
        ),
        part(
            b,
            Box(0.25 * size, 0.03, 0.15 * size),
            DARK_LEATHER,
            [0.0, 0.08 * size, -0.08],
        ),
    ]);
    fish(v, [0.06, 0.14, -0.1], 0.8, FISH, 10.0);
    if f.a {
        fish(
            v,
            [-0.05, 0.16, -0.08],
            0.9,
            Color::srgb(0.75, 0.55, 0.4),
            -12.0,
        );
    }
    if f.a && f.t2 {
        // Giant Catfish: slung over the shoulder, whiskers and all.
        v.push(
            part(
                b,
                Ball(0.13),
                Color::srgb(0.4, 0.42, 0.3),
                [0.1, 0.18, -0.02],
            )
            .scale([0.8, 0.7, 2.6])
            .rot([0.0, 30.0, 70.0]),
        );
        v.push(part(b, Box(0.01, 0.01, 0.16), DARK, [0.2, 0.3, 0.15]).rot([20.0, 0.0, 30.0]));
    }
    if f.b {
        // A fishing net bundled on the creel; Fish Shower adds a heaving
        // net of fish and a lantern.
        v.push(part(b, Ball(0.13), LINEN, [0.0, -0.22, -0.1]).scale([1.3, 0.6, 0.8]));
    }
    if f.b && f.t2 {
        for (x, y) in [(-0.08, -0.2), (0.07, -0.18), (0.0, -0.26)] {
            fish(v, [x, y, -0.18], 0.6, FISH, 70.0);
        }
        v.extend(gear::lantern(b, [-0.15, 0.04, -0.02], 1.2));
    }
}

/// The rod in the right hand's socket along +Y, its line and float.
fn rod(v: &mut Vec<PartSpec>, f: Form) {
    let w = Prop(Socket::WeaponR);
    let wood = if f.a && f.t2 { OAK } else { WOOD };
    v.extend([
        part(w, Cyl(0.02, ROD * 0.55), wood, [0.0, ROD * 0.2, 0.0]),
        part(w, Cyl(0.011, ROD * 0.45), wood, [0.0, ROD * 0.6, 0.0]),
        part(w, Cyl(0.04, 0.04), BRASS, [0.04, 0.05, 0.0]).rot([0.0, 0.0, 90.0]),
        part(w, Box(0.004, 0.25, 0.004), LINEN, [0.0, TIP.y - 0.12, 0.02]),
        part(w, Ball(0.025), RED_CLOTH, [0.0, TIP.y - 0.26, 0.02]),
    ]);
    if f.a && f.t2 {
        v.push(
            part(w, Ring(0.008, 0.035, 270.0), GOLD, [0.0, TIP.y - 0.3, 0.02])
                .rot([0.0, 0.0, 90.0])
                .glow(),
        );
    }
    if f.b {
        // A second rod across the back.
        let b = Secondary(Spring::Back);
        v.push(part(b, Cyl(0.012, 0.8), WOOD, [0.12, 0.1, -0.12]).rot([0.0, 0.0, -30.0]));
    }
}
