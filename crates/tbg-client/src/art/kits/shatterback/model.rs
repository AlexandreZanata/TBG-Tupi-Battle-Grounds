//! Shatterback figure: a hulking stone-bark grove beast carrying four
//! great mossy plates on its back, bark pauldrons and amber eyes under a
//! heavy brow. Branch A (Thornplates) thickens the plates and studs them
//! with thorns, then lights the thorn tips; branch B (Unleashed) cracks
//! them open on a glowing core, then sheds them for a bare blazing back
//! and spiked fists.

use bevy::prelude::*;

use crate::art::{
    figure::{part, FigureSpec, PartSpec},
    kits::common::{body, Form},
    mesh::Shape::*,
    palette::*,
    rig::{Joint::*, Node::Body, Node::Secondary, Proportions, Spring},
};

const HIDE: Color = Color::srgb(0.45, 0.36, 0.26);
const SLATE: Color = Color::srgb(0.5, 0.5, 0.46);
const MOSS: Color = Color::srgb(0.36, 0.5, 0.2);
const THORN: Color = Color::srgb(0.24, 0.16, 0.1);
const CORE: Color = Color::srgb(1.0, 0.5, 0.15);
const AMBER: Color = Color::srgb(1.0, 0.72, 0.25);

pub fn plan(tier: u8) -> Proportions {
    let mut p = body::plan(true, 0.88, tier);
    p.torso.z *= 1.3;
    p.arm *= 1.2;
    p.head *= 0.85;
    p
}

pub fn tip() -> Vec3 {
    Vec3::new(0.0, -0.12, 0.0)
}

pub fn spec(tier: u8, branch: u8) -> FigureSpec {
    let f = Form::of(tier, branch);
    let p = plan(tier);
    let mut v = Vec::with_capacity(130);
    let look = body::Look {
        skin: HIDE,
        cloth: BARK,
        legs: BARK,
        boots: SLATE,
    };
    body::humanoid(&mut v, &p, &look);
    head(&mut v, &p, f);
    plates(&mut v, &p, f);
    fists(&mut v, f);
    body::pauldrons(&mut v, if f.b && f.t2 { BARK } else { SLATE }, 0.13);
    let t = p.torso;
    // Moss over the shoulders and a team sash across the chest.
    v.extend([
        part(
            Body(Chest),
            Box(t.x * 0.9, 0.06, t.z * 0.9),
            MOSS,
            [0.0, t.y * 0.95, 0.0],
        ),
        part(
            Body(Chest),
            Box(t.x * 1.0, 0.07, t.z * 0.95),
            ALLY,
            [0.0, t.y * 0.45, 0.02],
        )
        .rot([0.0, 0.0, -18.0]),
    ]);
    FigureSpec { plan: p, parts: v }
}

fn head(v: &mut Vec<PartSpec>, p: &Proportions, f: Form) {
    let h = p.head;
    v.extend([
        part(
            Body(Head),
            Box(h * 2.0, h * 0.4, h * 0.6),
            SLATE,
            [0.0, h * 0.45, h * 0.55],
        ),
        part(
            Body(Head),
            Box(h * 1.4, h * 0.5, h * 0.6),
            HIDE,
            [0.0, -h * 0.55, h * 0.6],
        ),
        part(Body(Head), Ball(h * 0.5), MOSS, [0.0, h * 0.85, -h * 0.1]).scale([1.6, 0.5, 1.3]),
    ]);
    body::glowing_eyes(v, p, if f.b { CORE } else { AMBER });
}

/// Four plates down the back spring, overlapping like a pangolin's.
fn plates(v: &mut Vec<PartSpec>, p: &Proportions, f: Form) {
    let b = Secondary(Spring::Back);
    let t = p.torso;
    let keep = if f.b && f.t2 { 2 } else { 4 };
    let thick = if f.a { 0.09 } else { 0.06 };
    for i in 0..keep {
        let k = i as f32;
        let (y, w) = (0.26 - 0.17 * k, t.x * (1.15 - 0.14 * k));
        let at = [0.0, y, -0.12 - 0.03 * k];
        // Rounded overlapping shell plates, mossy along the top.
        v.extend([
            part(b, Ball(w * 0.5), SLATE, at)
                .scale([1.0, 0.5, thick * 4.0])
                .rot([-35.0, 0.0, 0.0]),
            part(b, Ball(w * 0.42), MOSS, [0.0, y + 0.07, -0.08 - 0.03 * k])
                .scale([1.0, 0.18, thick * 4.5])
                .rot([-35.0, 0.0, 0.0]),
        ]);
        if f.a {
            for x in [-0.35, 0.0, 0.35] {
                let thorn = part(
                    b,
                    Cone(0.03, 0.13),
                    THORN,
                    [x * w, y + 0.02, -0.16 - 0.02 * k],
                )
                .rot([-115.0, 0.0, 0.0]);
                v.push(thorn);
                if f.t2 {
                    v.push(part(b, Ball(0.025), AMBER, [x * w, y - 0.02, -0.33 - 0.03 * k]).glow());
                }
            }
        }
        if f.b && !f.t2 {
            // Unleashed: a crack through each plate shows the core.
            v.push(
                part(
                    b,
                    Box(0.025, 0.18, thick * 1.2),
                    CORE,
                    [w * 0.12, y, -0.1 - 0.02 * k],
                )
                .rot([-25.0, 0.0, 20.0])
                .glow(),
            );
        }
    }
    if f.b && f.t2 {
        // Rampage: the shed plates bare a blazing core between the
        // shoulder blades.
        v.extend([
            part(b, Ball(0.13), CORE, [0.0, -0.2, -0.06]).glow(),
            part(b, Ring(0.025, 0.15, 360.0), BARK, [0.0, -0.2, -0.06]).rot([90.0, 0.0, 0.0]),
        ]);
        for x in [-0.12, 0.12] {
            v.push(
                part(b, Box(0.1, 0.12, 0.05), SLATE, [x, -0.36, -0.08]).rot([
                    -30.0,
                    0.0,
                    30.0 * x.signum(),
                ]),
            );
        }
    }
}

fn fists(v: &mut Vec<PartSpec>, f: Form) {
    for hand in [HandL, HandR] {
        v.push(part(
            Body(hand),
            Box(0.19, 0.17, 0.19),
            SLATE,
            [0.0, -0.05, 0.0],
        ));
        if f.b && f.t2 {
            for x in [-0.06, 0.06] {
                v.push(
                    part(Body(hand), Cone(0.03, 0.1), THORN, [x, -0.03, 0.12])
                        .rot([90.0, 0.0, 0.0]),
                );
            }
        }
    }
}
