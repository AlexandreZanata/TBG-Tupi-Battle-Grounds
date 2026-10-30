//! Bone Brute figure: a hunched giant skeleton with a ribcage around a
//! glowing soul, rags and a rusty cleaver. Branch A armors it in bone
//! (buckler, then a tower shield of skulls); branch B arms it (twin
//! cleavers, then a great reaping scythe and a hooded cloak).

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

const OLD_BONE: Color = Color::srgb(0.75, 0.71, 0.6);
const BLOOD: Color = Color::srgb(0.45, 0.12, 0.12);
const SOUL: Color = Color::srgb(0.55, 1.0, 0.68);

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

/// Long arms and a deep chest; the hunch is posed, not modeled.
pub fn plan(tier: u8) -> Proportions {
    Proportions {
        scale: 1.45 * 1.2 * (1.0 + 0.08 * f32::from(tier.min(2))),
        leg: 0.36,
        torso: Vec3::new(0.5, 0.38, 0.28),
        head: 0.14,
        arm: 0.25,
    }
}

/// The blade point in the weapon socket's space (impact anchor).
pub fn tip(f: Form) -> Vec3 {
    if f.b && f.t2 {
        Vec3::new(0.0, 0.42, 0.44)
    } else {
        Vec3::new(0.0, -0.05, 0.5)
    }
}

pub fn spec(tier: u8, branch: u8) -> FigureSpec {
    let f = Form::of(tier, branch);
    let p = plan(tier);
    let mut v = Vec::with_capacity(140);
    legs(&mut v, &p, f);
    pelvis(&mut v, f);
    ribcage(&mut v, &p, f);
    skull(&mut v, f);
    arms(&mut v, &p, f);
    weapons(&mut v, f);
    colors(&mut v);
    FigureSpec { plan: p, parts: v }
}

/// Two parallel bones between joint balls.
fn bones(v: &mut Vec<PartSpec>, j: crate::art::rig::Joint, len: f32, r: f32) {
    v.push(part(Body(j), Ball(r * 1.6), BONE, [0.0, 0.0, 0.0]));
    v.push(part(
        Body(j),
        Pill(r, len * 0.75),
        BONE,
        [r * 0.6, -len * 0.5, 0.0],
    ));
    v.push(part(
        Body(j),
        Pill(r * 0.7, len * 0.75),
        OLD_BONE,
        [-r * 0.9, -len * 0.5, 0.0],
    ));
}

fn legs(v: &mut Vec<PartSpec>, p: &Proportions, f: Form) {
    let l = p.leg;
    for (thigh, shin, foot) in [(ThighL, ShinL, FootL), (ThighR, ShinR, FootR)] {
        v.push(part(Body(thigh), Ball(0.06), BONE, [0.0, 0.0, 0.0]));
        v.push(part(
            Body(thigh),
            Pill(0.048, l * 0.36),
            BONE,
            [0.0, -l * 0.25, 0.0],
        ));
        bones(v, shin, l * 0.5, 0.034);
        v.extend([
            part(Body(foot), Box(0.13, 0.06, 0.12), BONE, [0.0, -0.03, 0.0]),
            part(Body(foot), Ball(0.045), OLD_BONE, [0.0, -0.01, -0.05]),
        ]);
        for x in [-0.04, 0.0, 0.04] {
            v.push(
                part(Body(foot), Cone(0.022, 0.12), BONE, [x, -0.04, 0.1]).rot([90.0, 0.0, 0.0]),
            );
        }
        if f.a {
            v.push(part(
                Body(shin),
                Box(0.12, 0.16, 0.06),
                BONE,
                [0.0, -l * 0.18, 0.06],
            ));
        }
    }
}

/// Team rag tied round the left arm and a team hem on the cape: the side
/// reads from the match camera.
fn colors(v: &mut Vec<PartSpec>) {
    v.extend([
        part(Body(UpperArmL), Box(0.1, 0.06, 0.1), ALLY, [0.0, -0.1, 0.0]),
        part(
            Body(UpperArmL),
            Box(0.03, 0.12, 0.02),
            ALLY,
            [0.04, -0.17, -0.04],
        )
        .rot([0.0, 0.0, 12.0]),
        part(
            Secondary(Spring::Back),
            Box(0.43, 0.06, 0.025),
            ALLY,
            [0.0, -0.32, -0.04],
        ),
    ]);
}

fn pelvis(v: &mut Vec<PartSpec>, f: Form) {
    let rag = match (f.a, f.b) {
        (true, _) => DARK,
        (_, true) => BLOOD,
        _ => VIOLET,
    };
    v.extend([
        part(Body(Hips), Box(0.34, 0.1, 0.18), BONE, [0.0, 0.0, 0.0]),
        part(Body(Hips), Ball(0.1), OLD_BONE, [0.0, 0.0, -0.06]).scale([1.6, 0.7, 0.8]),
        part(
            Body(Hips),
            Box(0.4, 0.05, 0.24),
            DARK_LEATHER,
            [0.0, 0.06, 0.0],
        ),
        part(
            Secondary(Spring::Tabard),
            Box(0.26, 0.3, 0.02),
            rag,
            [0.0, -0.1, 0.0],
        ),
        part(
            Secondary(Spring::Tabard),
            Box(0.08, 0.1, 0.02),
            rag,
            [0.08, -0.27, 0.0],
        )
        .rot([0.0, 0.0, 12.0]),
        part(
            Secondary(Spring::Tabard),
            Box(0.07, 0.08, 0.02),
            rag,
            [-0.09, -0.26, 0.0],
        )
        .rot([0.0, 0.0, -9.0]),
    ]);
    // Spine column from the pelvis up into the ribcage.
    for i in 0..3 {
        v.push(part(
            Body(Spine),
            Box(0.07, 0.05, 0.07),
            OLD_BONE,
            [0.0, 0.03 + i as f32 * 0.065, -0.06],
        ));
    }
    if f.b {
        // Butcher: meat hooks hanging from the belt.
        for x in [-0.17, 0.17] {
            v.push(
                part(
                    Body(Hips),
                    Ring(0.012, 0.045, 220.0),
                    IRON,
                    [x, -0.02, 0.13],
                )
                .rot([0.0, 90.0, 0.0]),
            );
        }
    }
}

fn ribcage(v: &mut Vec<PartSpec>, p: &Proportions, f: Form) {
    let h = p.torso.y;
    // Ribs: open half rings around the soul, widest in the middle.
    for (i, w) in [0.17, 0.2, 0.21, 0.19, 0.15].into_iter().enumerate() {
        let y = 0.07 + i as f32 * 0.062;
        // The 110° gap faces the spine (-Z).
        v.push(part(Body(Chest), Ring(0.018, w, 250.0), BONE, [0.0, y, 0.0]).rot([0.0, 35.0, 0.0]));
    }
    v.extend([
        part(Body(Chest), Box(0.05, 0.28, 0.04), BONE, [0.0, 0.2, 0.15]),
        part(
            Body(Chest),
            Box(0.08, 0.36, 0.08),
            OLD_BONE,
            [0.0, 0.2, -0.13],
        ),
        part(
            Body(Chest),
            Box(0.56, 0.06, 0.07),
            BONE,
            [0.0, h * 0.95, 0.04],
        ),
        part(
            Body(Chest),
            Box(0.18, 0.2, 0.04),
            OLD_BONE,
            [0.14, h * 0.75, -0.13],
        )
        .rot([0.0, 0.0, 10.0]),
        part(
            Body(Chest),
            Box(0.18, 0.2, 0.04),
            OLD_BONE,
            [-0.14, h * 0.75, -0.13],
        )
        .rot([0.0, 0.0, -10.0]),
        // The spectral soul burning inside the ribs.
        part(Body(Chest), Ball(0.085), SOUL, [0.0, 0.2, 0.0]).glow(),
        part(Body(Chest), Ball(0.05), GLOW, [0.0, 0.2, 0.0])
            .scale([1.0, 1.6, 1.0])
            .glow(),
    ]);
    // Tattered back cape on the back spring.
    let cape = if f.b && f.t2 { VIOLET } else { DARK };
    v.push(part(
        Secondary(Spring::Back),
        Box(0.42, 0.42, 0.02),
        cape,
        [0.0, -0.12, -0.04],
    ));
    v.push(
        part(
            Secondary(Spring::Back),
            Box(0.14, 0.12, 0.02),
            cape,
            [0.12, -0.38, -0.04],
        )
        .rot([0.0, 0.0, 14.0]),
    );
    if f.a {
        // Ossified: bone plates bolted over the ribs (large surface).
        let plate = if f.t2 { BONE } else { OLD_BONE };
        v.push(
            part(
                Body(Chest),
                Box(0.44, 0.2, 0.06),
                plate,
                [0.0, h * 0.62, 0.17],
            )
            .rot([-6.0, 0.0, 0.0]),
        );
        v.push(part(
            Body(Chest),
            Box(0.36, 0.12, 0.06),
            plate,
            [0.0, h * 0.3, 0.16],
        ));
        v.push(
            part(
                Body(Chest),
                Box(0.1, 0.1, 0.03),
                SOUL,
                [0.0, h * 0.62, 0.205],
            )
            .rot([0.0, 0.0, 45.0])
            .glow(),
        );
    }
    if f.b && !f.t2 {
        // Butcher: a stained apron over the front.
        v.push(
            part(
                Body(Chest),
                Box(0.38, 0.34, 0.025),
                BLOOD,
                [0.0, 0.16, 0.19],
            )
            .rot([-5.0, 0.0, 0.0]),
        );
        v.push(
            part(
                Body(Chest),
                Box(0.04, 0.2, 0.03),
                LEATHER,
                [0.13, h * 0.75, 0.16],
            )
            .rot([0.0, 0.0, -20.0]),
        );
    }
    if f.b && f.t2 {
        // Reaper: a hooded cloak around the shoulders.
        v.push(
            part(
                Body(Chest),
                Frustum(0.17, 0.3, 0.16),
                VIOLET,
                [0.0, h * 0.9, -0.03],
            )
            .scale([1.0, 1.0, 0.8]),
        );
    }
}

fn skull(v: &mut Vec<PartSpec>, f: Form) {
    let eye = if f.t2 { SOUL } else { GLOW };
    v.extend([
        part(Body(Head), Ball(0.14), BONE, [0.0, 0.02, 0.0]).scale([1.0, 0.95, 1.1]),
        part(Body(Head), Box(0.2, 0.06, 0.1), BONE, [0.0, 0.05, 0.1]),
        part(Body(Head), Box(0.05, 0.05, 0.02), DARK, [0.05, 0.0, 0.15]),
        part(Body(Head), Box(0.05, 0.05, 0.02), DARK, [-0.05, 0.0, 0.15]),
        part(
            Body(Head),
            Box(0.03, 0.04, 0.02),
            DARK,
            [0.0, -0.045, 0.155],
        ),
        part(Body(Head), Box(0.17, 0.06, 0.13), BONE, [0.0, -0.1, 0.06]),
        part(
            Body(Head),
            Box(0.15, 0.02, 0.02),
            OLD_BONE,
            [0.0, -0.07, 0.125],
        ),
        part(Body(Head), Ball(0.022), eye, [0.05, 0.0, 0.15]).glow(),
        part(Body(Head), Ball(0.022), eye, [-0.05, 0.0, 0.15]).glow(),
    ]);
    if f.a && f.t2 {
        // Bone Wall: a horned skull helm.
        for s in [-1.0f32, 1.0] {
            v.push(
                part(
                    Body(Head),
                    Cone(0.045, 0.2),
                    OLD_BONE,
                    [0.12 * s, 0.12, -0.02],
                )
                .rot([-20.0, 0.0, -40.0 * s]),
            );
        }
        v.push(part(Body(Head), Ball(0.15), BONE, [0.0, 0.06, -0.01]).scale([1.05, 0.55, 1.1]));
    }
    if f.b && f.t2 {
        // Reaper: a hood and a crown of bone spikes.
        v.push(
            part(
                Secondary(Spring::HeadGear),
                Ball(0.165),
                VIOLET,
                [0.0, 0.07, -0.07],
            )
            .scale([1.1, 1.05, 0.95]),
        );
        for i in 0..5 {
            let x = (i as f32 - 2.0) * 0.045;
            v.push(
                part(Body(Head), Cone(0.018, 0.09), GOLD, [x, 0.15, 0.06]).rot([
                    -15.0,
                    0.0,
                    -x * 120.0,
                ]),
            );
        }
    }
}

fn arms(v: &mut Vec<PartSpec>, p: &Proportions, f: Form) {
    let a = p.arm;
    for (upper, fore, hand) in [(UpperArmL, ForearmL, HandL), (UpperArmR, ForearmR, HandR)] {
        v.push(part(Body(upper), Ball(0.07), BONE, [0.0, 0.0, 0.0]));
        v.push(part(
            Body(upper),
            Pill(0.045, a * 0.72),
            BONE,
            [0.0, -a * 0.5, 0.0],
        ));
        bones(v, fore, a * 0.9, 0.03);
        v.push(part(
            Body(hand),
            Box(0.1, 0.07, 0.1),
            BONE,
            [0.0, -0.035, 0.0],
        ));
        for x in [-0.035, 0.0, 0.035] {
            v.push(
                part(Body(hand), Cone(0.017, 0.1), OLD_BONE, [x, -0.1, 0.02])
                    .rot([180.0, 0.0, 0.0]),
            );
        }
        // Shoulder spikes.
        v.push(part(
            Body(upper),
            Cone(0.04, 0.14),
            OLD_BONE,
            [0.0, 0.08, 0.0],
        ));
    }
    if f.a {
        // Ossified: bone pauldrons.
        for (upper, s) in [(UpperArmL, 1.0f32), (UpperArmR, -1.0)] {
            v.push(
                part(
                    Body(upper),
                    Box(0.17, 0.08, 0.2),
                    BONE,
                    [0.03 * s, 0.02, 0.0],
                )
                .rot([0.0, 0.0, -18.0 * s]),
            );
        }
    }
}

fn weapons(v: &mut Vec<PartSpec>, f: Form) {
    let r = Prop(Socket::WeaponR);
    let l = Prop(Socket::OffhandL);
    if f.b && f.t2 {
        scythe(v, r);
    } else {
        cleaver(v, r, f.b);
    }
    match (f.a, f.b, f.t2) {
        (true, _, false) => buckler(v, l),
        (true, _, true) => tower_shield(v, l),
        (_, true, false) => cleaver(v, l, true),
        _ => {}
    }
}

/// A heavy chopping blade along +Z from the fist, edge down: dark iron,
/// a bright edge, rust bitten into the flat.
fn cleaver(v: &mut Vec<PartSpec>, at: crate::art::rig::Node, bloody: bool) {
    v.extend([
        part(at, Cyl(0.026, 0.22), DARK_LEATHER, [0.0, 0.0, 0.03]).rot([90.0, 0.0, 0.0]),
        part(at, Ball(0.038), BONE, [0.0, 0.0, -0.09]),
        part(at, Box(0.06, 0.07, 0.04), IRON, [0.0, -0.01, 0.15]),
        part(at, Box(0.032, 0.21, 0.36), GUNMETAL, [0.0, -0.06, 0.33]),
        part(at, Box(0.038, 0.03, 0.37), DARK, [0.0, 0.05, 0.33]),
        part(at, Box(0.036, 0.03, 0.36), STEEL, [0.0, -0.165, 0.33]),
        part(at, Box(0.034, 0.08, 0.1), RUST, [0.0, -0.04, 0.42]),
        part(at, Cyl(0.025, 0.04), DARK, [0.0, -0.02, 0.24]).rot([0.0, 0.0, 90.0]),
    ]);
    if bloody {
        v.push(part(at, Box(0.037, 0.06, 0.14), BLOOD, [0.0, -0.13, 0.44]));
    }
}

/// Reaper: a long haft along the hand's +Y with a blade curving from its
/// top forward and down (a quarter circle in the YZ plane).
fn scythe(v: &mut Vec<PartSpec>, at: crate::art::rig::Node) {
    let center = [0.0, 0.5, 0.0];
    v.extend([
        part(at, Cyl(0.024, 1.3), DARK, [0.0, 0.3, 0.0]),
        part(at, Cyl(0.032, 0.05), GOLD, [0.0, 0.92, 0.0]),
        part(at, Ball(0.045), BONE, [0.0, -0.36, 0.0]),
        part(at, Ring(0.032, 0.45, 100.0), IRON, center).rot([0.0, 0.0, 90.0]),
        part(at, Ring(0.012, 0.418, 96.0), SOUL, center)
            .rot([0.0, 0.0, 90.0])
            .glow(),
        part(at, Box(0.05, 0.08, 0.08), BONE, [0.0, 0.95, 0.02]),
    ]);
}

fn buckler(v: &mut Vec<PartSpec>, at: crate::art::rig::Node) {
    v.extend([
        part(at, Cyl(0.2, 0.04), OLD_BONE, [0.07, -0.02, 0.0]).rot([0.0, 0.0, 90.0]),
        part(at, Ball(0.06), BONE, [0.1, -0.02, 0.0]),
        part(at, Ring(0.015, 0.19, 360.0), BONE, [0.09, -0.02, 0.0]).rot([0.0, 0.0, 90.0]),
    ]);
}

/// Bone Wall: a tower shield of bones with three skulls and soul runes.
fn tower_shield(v: &mut Vec<PartSpec>, at: crate::art::rig::Node) {
    v.extend([
        part(at, Box(0.05, 0.66, 0.42), OLD_BONE, [0.08, 0.05, 0.02]),
        part(at, Box(0.06, 0.05, 0.46), BONE, [0.085, 0.38, 0.02]),
        part(at, Box(0.06, 0.05, 0.46), BONE, [0.085, -0.28, 0.02]),
    ]);
    for (y, z) in [(0.2, -0.1), (0.2, 0.12), (-0.08, 0.01)] {
        v.push(part(at, Ball(0.07), BONE, [0.12, y, z]));
        v.push(part(at, Box(0.02, 0.025, 0.04), SOUL, [0.18, y + 0.01, z]).glow());
    }
    v.push(part(at, Box(0.02, 0.22, 0.03), SOUL, [0.115, 0.06, -0.17]).glow());
    v.push(part(at, Box(0.02, 0.22, 0.03), SOUL, [0.115, 0.06, 0.21]).glow());
}
