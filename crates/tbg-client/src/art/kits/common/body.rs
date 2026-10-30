//! The shared rig-v2 anatomy (the Archer v2 family) and dress pieces:
//! tunics, belts, team tabards, capes, hoods, helmets, pauldrons, beards.
//! Coordinates are in each joint's space; figures face +Z, left is +X.

use bevy::prelude::*;

use crate::art::{
    figure::{part, PartSpec},
    mesh::Shape::*,
    palette::*,
    rig::{BodyPlan, Joint::*, Node::Body, Node::Secondary, Proportions, Spring},
    roster::body,
};

/// The colors of a body: skin, upper cloth, legs, boots.
#[derive(Clone, Copy)]
pub struct Look {
    pub skin: Color,
    pub cloth: Color,
    pub legs: Color,
    pub boots: Color,
}

/// Humanoid or Large proportions with the lane-readability scale (×1.2)
/// and the Evolution growth (+8 % per tier, ART-DIRECTION §5).
pub fn plan(large: bool, size: f32, tier: u8) -> Proportions {
    let mut p = if large {
        BodyPlan::Large.proportions()
    } else {
        BodyPlan::Humanoid.proportions()
    };
    p.scale *= size * 1.2 * (1.0 + 0.08 * f32::from(tier.min(2)));
    p
}

/// The shared anatomy: hips, tapered chest, ball head with eyes, tapered
/// limbs, block hands and boots.
pub fn humanoid(v: &mut Vec<PartSpec>, p: &Proportions, l: &Look) {
    let mut parts = body::parts(p, l.cloth, l.skin, l.boots);
    for x in &mut parts {
        if matches!(x.node, Body(ThighL | ThighR)) {
            x.color = l.legs;
        }
    }
    v.extend(parts);
    let h = p.head;
    for s in [-1.0f32, 1.0] {
        v.push(part(
            Body(Head),
            Box(0.03, 0.04, 0.02),
            DARK,
            [s * h * 0.36, h * 0.1, h * 0.92],
        ));
    }
}

/// A skirt with a hem, a belt with a buckle.
pub fn tunic(v: &mut Vec<PartSpec>, p: &Proportions, cloth: Color, hem: Color, belt: Color) {
    let w = p.torso.x;
    v.extend([
        part(
            Body(Hips),
            Frustum(w * 0.45, w * 0.58, 0.2),
            cloth,
            [0.0, -0.08, 0.0],
        )
        .scale([1.0, 1.0, 0.75]),
        part(
            Body(Hips),
            Frustum(w * 0.59, w * 0.6, 0.03),
            hem,
            [0.0, -0.17, 0.0],
        )
        .scale([1.0, 1.0, 0.77]),
        part(
            Body(Hips),
            Box(w * 0.95, 0.045, p.torso.z * 1.05),
            belt,
            [0.0, 0.04, 0.0],
        ),
        part(
            Body(Hips),
            Box(0.055, 0.055, 0.015),
            GOLD,
            [0.0, 0.04, p.torso.z * 0.55],
        ),
    ]);
}

/// A team tabard on the tabard spring, gold hem and lozenge.
pub fn tabard(v: &mut Vec<PartSpec>, width: f32, length: f32) {
    let t = Secondary(Spring::Tabard);
    v.extend([
        part(
            t,
            Box(width, length, 0.018),
            ALLY,
            [0.0, 0.09 - (length - 0.34) * 0.5, 0.025],
        ),
        part(
            t,
            Box(width * 0.83, 0.02, 0.022),
            GOLD,
            [0.0, 0.09 - length + 0.17 + 0.005, 0.025],
        ),
        part(t, Box(0.05, 0.05, 0.022), GOLD, [0.0, 0.16, 0.026]).rot([0.0, 0.0, 45.0]),
    ]);
}

/// A cape hanging from the back spring.
pub fn cape(v: &mut Vec<PartSpec>, color: Color, width: f32, length: f32) {
    let b = Secondary(Spring::Back);
    v.push(part(
        b,
        Box(width, length, 0.02),
        color,
        [0.0, -length * 0.4, -0.03],
    ));
}

/// A rounded hood over the head with a brim, and a capelet on the chest.
pub fn hood(v: &mut Vec<PartSpec>, p: &Proportions, color: Color, brim: Color) {
    let h = p.head;
    v.extend([
        part(Body(Head), Ball(h * 1.2), color, [0.0, h * 0.25, -h * 0.4]).scale([1.04, 1.05, 1.0]),
        part(
            Body(Head),
            Box(h * 1.85, h * 0.28, h * 0.5),
            brim,
            [0.0, h * 0.73, h * 0.7],
        )
        .rot([-12.0, 0.0, 0.0]),
        part(
            Body(Chest),
            Frustum(p.torso.x * 0.45, p.torso.x * 0.58, 0.1),
            color,
            [0.0, p.torso.y * 0.94, -0.015],
        )
        .scale([1.0, 1.0, 0.8]),
    ]);
}

#[derive(Clone, Copy)]
pub enum Helm {
    /// Round iron cap.
    Cap(Color),
    /// Brimmed kettle hat.
    Kettle(Color),
    /// Cap with a plume on the head-gear spring.
    Plumed(Color, Color),
    /// Cap with two horns.
    Horned(Color, Color),
    /// A circlet of spikes.
    Crown(Color),
}

pub fn helmet(v: &mut Vec<PartSpec>, p: &Proportions, helm: Helm) {
    let h = p.head;
    let cap = |c| {
        part(Body(Head), Ball(h * 1.07), c, [0.0, h * 0.25, -h * 0.05]).scale([1.05, 0.62, 1.05])
    };
    match helm {
        Helm::Cap(c) => v.push(cap(c)),
        Helm::Kettle(c) => {
            v.push(cap(c));
            v.push(part(
                Body(Head),
                Cyl(h * 1.55, 0.02),
                c,
                [0.0, h * 0.28, 0.0],
            ));
        }
        Helm::Plumed(c, plume) => {
            v.push(cap(c));
            v.push(part(
                Secondary(Spring::HeadGear),
                Box(0.035, h * 0.9, h * 1.6),
                plume,
                [0.0, h * 1.15, -h * 0.15],
            ));
        }
        Helm::Horned(c, horn) => {
            v.push(cap(c));
            for s in [-1.0f32, 1.0] {
                v.push(
                    part(
                        Body(Head),
                        Cone(h * 0.28, h * 1.2),
                        horn,
                        [s * h * 0.95, h * 0.75, 0.0],
                    )
                    .rot([0.0, 0.0, -40.0 * s]),
                );
            }
        }
        Helm::Crown(c) => {
            v.push(part(
                Body(Head),
                Cyl(h * 0.95, 0.04),
                c,
                [0.0, h * 0.62, 0.0],
            ));
            for i in 0..5 {
                let a = (i as f32 - 2.0) * 0.45;
                v.push(part(
                    Body(Head),
                    Cone(h * 0.16, h * 0.55),
                    c,
                    [a.sin() * h * 0.9, h * 0.88, a.cos() * h * 0.9],
                ));
            }
        }
    }
}

/// Rounded shoulder plates on both upper arms.
pub fn pauldrons(v: &mut Vec<PartSpec>, color: Color, size: f32) {
    for (upper, s) in [(UpperArmL, 1.0f32), (UpperArmR, -1.0)] {
        v.push(
            part(Body(upper), Ball(size), color, [0.02 * s, 0.02, 0.0]).scale([1.25, 0.7, 1.15]),
        );
    }
}

/// A beard hanging from the chin.
pub fn beard(v: &mut Vec<PartSpec>, p: &Proportions, color: Color, length: f32) {
    let h = p.head;
    v.push(
        part(
            Body(Head),
            Cone(h * 0.7, length),
            color,
            [0.0, -h * 0.55 - length * 0.3, h * 0.55],
        )
        .rot([168.0, 0.0, 0.0]),
    );
}

/// Glowing eyes over the painted ones.
pub fn glowing_eyes(v: &mut Vec<PartSpec>, p: &Proportions, color: Color) {
    let h = p.head;
    for s in [-1.0f32, 1.0] {
        v.push(
            part(
                Body(Head),
                Box(0.04, 0.03, 0.02),
                color,
                [s * h * 0.36, h * 0.1, h * 0.95],
            )
            .glow(),
        );
    }
}

/// Bracers on both forearms.
pub fn bracers(v: &mut Vec<PartSpec>, p: &Proportions, color: Color) {
    for fore in [ForearmL, ForearmR] {
        v.push(part(
            Body(fore),
            Box(0.12, 0.12, 0.12),
            color,
            [0.0, -p.arm * 0.5, 0.0],
        ));
    }
}

/// Knee guards over both shins.
pub fn greaves(v: &mut Vec<PartSpec>, color: Color) {
    for shin in [ShinL, ShinR] {
        v.push(part(
            Body(shin),
            Box(0.12, 0.1, 0.05),
            color,
            [0.0, -0.02, 0.07],
        ));
    }
}
