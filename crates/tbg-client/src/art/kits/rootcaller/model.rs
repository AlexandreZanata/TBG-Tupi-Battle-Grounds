//! Rootcaller figure: a knobbly treant shaman with a bark trunk, a moss
//! collar and beard, amber eyes and long root claws. Branch A grows roots
//! and thorns (vine coils, then a thorn crown and an amber heart); branch B
//! grows fungus (a mushroom cap, then a great glowing cap and back caps).

use bevy::prelude::*;

use crate::art::{
    figure::{part, FigureSpec, PartSpec},
    mesh::Shape::*,
    palette::*,
    rig::{
        Joint::*,
        Node::{Body, Prop, Secondary},
        Proportions, Socket, Spring,
    },
};

const MOSS: Color = Color::srgb(0.36, 0.5, 0.2);
const DEEP_MOSS: Color = Color::srgb(0.18, 0.36, 0.16);
const BARK_DARK: Color = Color::srgb(0.3, 0.21, 0.13);
const THORN: Color = Color::srgb(0.24, 0.16, 0.1);
const AMBER: Color = Color::srgb(1.0, 0.72, 0.25);
const SPORE: Color = Color::srgb(0.85, 0.95, 0.4);
const CAP_SPOT: Color = Color::srgb(0.95, 0.9, 0.78);

#[derive(Clone, Copy)]
pub struct Form {
    pub a: bool,
    pub b: bool,
    pub t2: bool,
}

impl Form {
    pub fn of(tier: u8, branch: u8) -> Form {
        Form {
            a: tier >= 1 && branch == 0,
            b: tier >= 1 && branch == 1,
            t2: tier >= 2,
        }
    }
}

pub fn plan(tier: u8) -> Proportions {
    Proportions {
        scale: 1.05 * 1.2 * (1.0 + 0.08 * f32::from(tier.min(2))),
        leg: 0.3,
        torso: Vec3::new(0.36, 0.36, 0.26),
        head: 0.15,
        arm: 0.21,
    }
}

/// Claw length per form: longer on the root branch.
fn claw(f: Form) -> f32 {
    match (f.a, f.t2) {
        (true, true) => 0.3,
        (true, false) => 0.26,
        _ => 0.19,
    }
}

/// The claw tips in the right hand's socket space (cast origin).
pub fn tips(f: Form) -> Vec3 {
    Vec3::new(0.0, -0.06 - claw(f), 0.04)
}

pub fn spec(tier: u8, branch: u8) -> FigureSpec {
    let f = Form::of(tier, branch);
    let p = plan(tier);
    let mut v = Vec::with_capacity(130);
    legs(&mut v, &p, f);
    trunk(&mut v, &p, f);
    head(&mut v, f);
    arms(&mut v, &p, f);
    growth(&mut v, f);
    FigureSpec { plan: p, parts: v }
}

fn legs(v: &mut Vec<PartSpec>, p: &Proportions, f: Form) {
    let l = p.leg;
    for (thigh, shin, foot) in [(ThighL, ShinL, FootL), (ThighR, ShinR, FootR)] {
        v.extend([
            part(
                Body(thigh),
                Frustum(0.075, 0.06, l * 0.5),
                BARK,
                [0.0, -l * 0.25, 0.0],
            ),
            part(
                Body(shin),
                Frustum(0.06, 0.07, l * 0.5),
                BARK_DARK,
                [0.0, -l * 0.25, 0.0],
            ),
            part(Body(foot), Box(0.13, 0.07, 0.17), BARK, [0.0, -0.03, 0.03]),
        ]);
        // Root toes; the root branch trails extra roots behind the heels.
        for x in [-0.04, 0.0, 0.04] {
            v.push(
                part(Body(foot), Cone(0.025, 0.11), BARK_DARK, [x, -0.05, 0.13])
                    .rot([84.0, 0.0, 0.0]),
            );
        }
        if f.a {
            v.push(
                part(Body(foot), Cone(0.03, 0.16), BARK_DARK, [0.0, -0.05, -0.1])
                    .rot([-80.0, 0.0, 0.0]),
            );
        }
    }
}

fn trunk(v: &mut Vec<PartSpec>, p: &Proportions, f: Form) {
    let h = p.torso.y;
    let moss = if f.a || (f.b && f.t2) {
        DEEP_MOSS
    } else {
        MOSS
    };
    v.extend([
        // Root skirt flaring into the ground.
        part(
            Body(Hips),
            Frustum(0.17, 0.22, 0.16),
            BARK,
            [0.0, -0.02, 0.0],
        )
        .scale([1.0, 1.0, 0.85]),
        part(
            Body(Chest),
            Frustum(0.16, 0.2, h),
            BARK,
            [0.0, h * 0.5, 0.0],
        )
        .scale([1.0, 1.0, 0.82]),
        // Bark ridges down the front.
        part(
            Body(Chest),
            Box(0.03, h * 0.8, 0.03),
            BARK_DARK,
            [0.07, h * 0.45, 0.16],
        ),
        part(
            Body(Chest),
            Box(0.03, h * 0.6, 0.03),
            BARK_DARK,
            [-0.06, h * 0.4, 0.165],
        ),
        // Moss collar over the shoulders.
        part(
            Body(Chest),
            Frustum(0.15, 0.22, 0.09),
            moss,
            [0.0, h * 0.93, 0.0],
        )
        .scale([1.0, 1.0, 0.85]),
    ]);
    for (i, a) in [0.0f32, 1.3, 2.6, 3.9, 5.1].into_iter().enumerate() {
        let (x, z) = (a.cos() * 0.19, a.sin() * 0.16);
        let lean = [z * -200.0, 0.0, x * 200.0];
        v.push(
            part(
                Body(Hips),
                Cone(0.035, 0.16 + 0.02 * (i % 2) as f32),
                BARK_DARK,
                [x, -0.1, z],
            )
            .rot([180.0 + lean[0], 0.0, lean[2]]),
        );
    }
    // Moss cape on the back spring.
    let cape = if f.b && f.t2 { DEEP_MOSS } else { moss };
    v.push(part(
        Secondary(Spring::Back),
        Box(0.3, 0.34, 0.025),
        cape,
        [0.0, -0.12, -0.03],
    ));
    if f.a {
        // Deep Roots: moss mantle over the chest (large surface) and vines.
        v.push(
            part(
                Body(Chest),
                Box(0.34, 0.16, 0.05),
                DEEP_MOSS,
                [0.0, h * 0.78, 0.14],
            )
            .rot([-10.0, 0.0, 0.0]),
        );
        for (y, s) in [(0.12, 1.0f32), (0.24, -1.0)] {
            v.push(
                part(Body(Chest), Ring(0.014, 0.19, 300.0), LEAF, [0.0, y, 0.0]).rot([
                    0.0,
                    0.0,
                    9.0 * s,
                ]),
            );
        }
    }
    if f.a && f.t2 {
        // Strangle: a glowing amber heart in a split in the trunk.
        v.push(part(
            Body(Chest),
            Box(0.12, 0.16, 0.04),
            DARK,
            [0.0, h * 0.5, 0.155],
        ));
        v.push(part(Body(Chest), Ball(0.06), AMBER, [0.0, h * 0.5, 0.15]).glow());
    }
}

fn head(v: &mut Vec<PartSpec>, f: Form) {
    let eye = if f.t2 { AMBER } else { SAP };
    v.extend([
        part(Body(Head), Ball(0.15), BARK, [0.0, 0.02, 0.0]).scale([0.92, 1.12, 0.9]),
        part(
            Body(Head),
            Box(0.22, 0.05, 0.07),
            BARK_DARK,
            [0.0, 0.05, 0.11],
        )
        .rot([-14.0, 0.0, 0.0]),
        part(Body(Head), Cone(0.03, 0.1), BARK, [0.0, 0.0, 0.15]).rot([95.0, 0.0, 0.0]),
        part(Body(Head), Cone(0.1, 0.2), MOSS, [0.0, -0.17, 0.07]).rot([168.0, 0.0, 0.0]),
        part(Body(Head), Box(0.045, 0.03, 0.02), eye, [0.05, 0.02, 0.13]).glow(),
        part(Body(Head), Box(0.045, 0.03, 0.02), eye, [-0.05, 0.02, 0.13]).glow(),
    ]);
    match (f.a, f.b, f.t2) {
        // Strangle: a crown of thorns with amber tips.
        (true, _, true) => {
            for i in 0..7 {
                let a = i as f32 / 7.0 * std::f32::consts::TAU;
                let (x, z) = (a.cos() * 0.12, a.sin() * 0.11);
                v.push(
                    part(Body(Head), Cone(0.025, 0.17), THORN, [x, 0.14, z]).rot([
                        z * 160.0,
                        0.0,
                        -x * 160.0,
                    ]),
                );
                v.push(part(Body(Head), Ball(0.014), AMBER, [x * 1.5, 0.22, z * 1.5]).glow());
            }
        }
        // Spores / Overgrowth: a mushroom cap; the great cap glows.
        (_, true, t2) => {
            let r = if t2 { 0.3 } else { 0.24 };
            v.push(part(Body(Head), Ball(r), MUSHROOM, [0.0, 0.16, -0.01]).scale([1.0, 0.45, 1.0]));
            v.push(part(
                Body(Head),
                Cyl(r * 0.85, 0.02),
                CAP_SPOT,
                [0.0, 0.14, -0.01],
            ));
            for (x, z) in [
                (0.1, 0.08),
                (-0.12, 0.03),
                (0.02, -0.13),
                (0.15, -0.06),
                (-0.06, 0.15),
            ] {
                let spot = part(
                    Body(Head),
                    Ball(0.03),
                    if t2 { SPORE } else { CAP_SPOT },
                    [x * r / 0.24, 0.24, z * r / 0.24],
                );
                v.push(if t2 { spot.glow() } else { spot });
            }
        }
        // Base / Deep Roots: a crown of leaves and a twig.
        _ => {
            let n = if f.a { 7 } else { 5 };
            for i in 0..n {
                let a = i as f32 / n as f32 * std::f32::consts::TAU;
                let (x, z) = (a.cos() * 0.1, a.sin() * 0.09);
                v.push(
                    part(Body(Head), Cone(0.05, 0.17), LEAF, [x, 0.15, z])
                        .rot([z * 220.0, 0.0, -x * 220.0])
                        .scale([1.0, 1.0, 0.4]),
                );
            }
            v.push(
                part(Body(Head), Cyl(0.015, 0.16), BARK_DARK, [0.03, 0.22, 0.0])
                    .rot([0.0, 0.0, -15.0]),
            );
        }
    }
}

fn arms(v: &mut Vec<PartSpec>, p: &Proportions, f: Form) {
    let a = p.arm;
    let len = claw(f);
    let nail = if f.a && f.t2 { THORN } else { BARK_DARK };
    for (upper, fore, hand, sock) in [
        (UpperArmL, ForearmL, HandL, Socket::OffhandL),
        (UpperArmR, ForearmR, HandR, Socket::WeaponR),
    ] {
        v.extend([
            part(Body(upper), Ball(0.065), BARK, [0.0, -0.01, 0.0]),
            part(
                Body(upper),
                Frustum(0.055, 0.045, a),
                BARK,
                [0.0, -a * 0.5, 0.0],
            ),
            part(
                Body(fore),
                Frustum(0.05, 0.06, a * 0.9),
                BARK_DARK,
                [0.0, -a * 0.45, 0.0],
            ),
            part(Body(hand), Ball(0.06), BARK, [0.0, -0.04, 0.0]).scale([1.1, 0.9, 1.0]),
        ]);
        // Three root claws, splayed and curling forward.
        for (x, splay) in [(-0.035f32, -12.0f32), (0.0, 0.0), (0.035, 12.0)] {
            v.push(
                part(
                    Prop(sock),
                    Cone(0.022, len),
                    nail,
                    [x, -0.06 - len * 0.5, 0.03],
                )
                .rot([168.0, 0.0, splay]),
            );
        }
        if f.a {
            v.push(
                part(
                    Body(fore),
                    Ring(0.013, 0.065, 360.0),
                    LEAF,
                    [0.0, -a * 0.3, 0.0],
                )
                .rot([12.0, 0.0, 0.0]),
            );
            v.push(
                part(
                    Body(upper),
                    Ring(0.013, 0.07, 360.0),
                    LEAF,
                    [0.0, -a * 0.6, 0.0],
                )
                .rot([-10.0, 0.0, 0.0]),
            );
        }
        if f.a && f.t2 {
            for y in [-0.05, -0.12] {
                v.push(
                    part(Body(fore), Cone(0.018, 0.07), THORN, [0.05, y, 0.0])
                        .rot([0.0, 0.0, -80.0]),
                );
            }
        }
    }
}

/// Fungus on the spore branch: shoulder sacs, then a cluster on the back.
fn growth(v: &mut Vec<PartSpec>, f: Form) {
    if !f.b {
        return;
    }
    for (upper, x) in [(UpperArmL, 0.03), (UpperArmR, -0.03)] {
        v.push(part(Body(upper), Ball(0.06), SPORE, [x, 0.05, 0.0]).glow());
        v.push(part(
            Body(upper),
            Ball(0.045),
            MUSHROOM,
            [x * 2.0, 0.02, 0.05],
        ));
    }
    if f.t2 {
        let b = Secondary(Spring::Back);
        for (x, y, r) in [(0.08, 0.05, 0.08), (-0.09, -0.02, 0.1), (0.02, -0.16, 0.07)] {
            v.push(part(b, Cyl(0.02, 0.1), CAP_SPOT, [x, y, -0.06]));
            v.push(part(b, Ball(r), MUSHROOM, [x, y + 0.05, -0.07]).scale([1.0, 0.5, 1.0]));
            v.push(part(b, Ball(0.018), SPORE, [x + 0.03, y + 0.08, -0.04]).glow());
        }
    }
}
