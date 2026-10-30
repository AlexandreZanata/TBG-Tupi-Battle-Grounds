//! Bloodweaver figure: a hooded skeleton priestess in a wine robe with a
//! spindle staff wound in blood thread and drops of blood hovering at her
//! hand. Branch A (Blood Sacrifice) carries a chalice, then wears a
//! glowing heart reliquary; branch B (Blood Circulation) winds threads
//! round her arms, then weaves with two spindles amid orbiting drops.

use bevy::prelude::*;

use crate::art::{
    figure::{part, FigureSpec, PartSpec},
    kits::common::{body, gear, Form},
    mesh::Shape::*,
    palette::*,
    rig::{Joint::*, Node, Node::Body, Node::Prop, Node::Secondary, Proportions, Socket, Spring},
};

const WINE: Color = Color::srgb(0.38, 0.08, 0.13);
const WINE_DARK: Color = Color::srgb(0.22, 0.05, 0.08);
pub const BLOOD: Color = Color::srgb(0.88, 0.1, 0.14);
/// Staff length (socket space, +Y).
pub const STAFF: f32 = 1.05;

pub fn plan(tier: u8) -> Proportions {
    body::plan(false, 1.0, tier)
}

pub fn spec(tier: u8, branch: u8) -> FigureSpec {
    let f = Form::of(tier, branch);
    let p = plan(tier);
    let mut v = Vec::with_capacity(140);
    body::skeleton(&mut v, &p, BONE);
    robe(&mut v, &p, f);
    body::hood(&mut v, &p, WINE, WINE_DARK);
    body::glowing_eyes(&mut v, &p, BLOOD);
    spindle(&mut v, Prop(Socket::WeaponR), f.t2);
    let l = Prop(Socket::OffhandL);
    if f.b && f.t2 {
        spindle(&mut v, l, true);
    } else if f.a {
        // Deep Transfer: a gold chalice brimming with blood.
        v.extend([
            part(l, Frustum(0.07, 0.035, 0.08), GOLD, [0.0, 0.1, 0.05]),
            part(l, Cyl(0.06, 0.01), BLOOD, [0.0, 0.14, 0.05]).glow(),
            part(l, Cyl(0.015, 0.06), GOLD, [0.0, 0.04, 0.05]),
        ]);
    } else {
        for (y, r) in [(0.1, 0.03), (0.17, 0.02)] {
            v.push(part(l, Ball(r), BLOOD, [0.0, y, 0.07]).glow());
        }
    }
    FigureSpec { plan: p, parts: v }
}

fn robe(v: &mut Vec<PartSpec>, p: &Proportions, f: Form) {
    let t = p.torso;
    v.extend([
        part(
            Body(Chest),
            Frustum(t.x * 0.55, t.x * 0.42, t.y * 0.95),
            WINE,
            [0.0, t.y * 0.5, 0.0],
        )
        .scale([1.0, 1.0, 0.75]),
        part(
            Body(Hips),
            Frustum(t.x * 0.48, t.x * 0.8, p.leg * 0.95),
            WINE,
            [0.0, -p.leg * 0.45, 0.0],
        )
        .scale([1.0, 1.0, 0.85]),
        part(
            Body(Hips),
            Ring(0.02, t.x * 0.8, 360.0),
            WINE_DARK,
            [0.0, -p.leg * 0.9, 0.0],
        )
        .scale([1.0, 1.0, 0.85]),
        part(
            Body(Hips),
            Box(t.x * 0.9, 0.05, t.z * 0.95),
            DARK_LEATHER,
            [0.0, 0.02, 0.0],
        ),
        part(
            Body(Chest),
            Box(0.08, t.y * 0.9, 0.02),
            ALLY,
            [0.0, t.y * 0.45, t.x * 0.33],
        ),
    ]);
    for arm in [UpperArmL, UpperArmR] {
        v.push(part(
            Body(arm),
            Frustum(0.065, 0.08, p.arm * 0.9),
            WINE,
            [0.0, -p.arm * 0.45, 0.0],
        ));
    }
    if f.a && f.t2 {
        // Last Offering: a heart in a gold cage on the chest.
        v.extend([
            part(
                Body(Chest),
                Ball(0.065),
                BLOOD,
                [0.0, t.y * 0.65, t.x * 0.38],
            )
            .scale([1.0, 1.1, 0.8])
            .glow(),
            part(
                Body(Chest),
                Ring(0.012, 0.085, 360.0),
                GOLD,
                [0.0, t.y * 0.65, t.x * 0.38],
            )
            .rot([90.0, 0.0, 0.0]),
            part(
                Body(Chest),
                Ring(0.012, 0.085, 360.0),
                GOLD,
                [0.0, t.y * 0.65, t.x * 0.38],
            )
            .rot([90.0, 90.0, 0.0]),
        ]);
    }
    if f.b {
        for fore in [ForearmL, ForearmR] {
            for y in [0.3, 0.6] {
                v.push(
                    part(
                        Body(fore),
                        Ring(0.01, 0.06, 360.0),
                        BLOOD,
                        [0.0, -p.arm * y, 0.0],
                    )
                    .glow(),
                );
            }
        }
    }
    if f.b && f.t2 {
        let b = Secondary(Spring::Back);
        for (x, y) in [(-0.22, 0.25), (0.24, 0.15), (0.0, 0.42)] {
            v.push(part(b, Ball(0.04), BLOOD, [x, y, 0.05]).glow());
        }
    }
}

/// A spindle staff along the socket's +Y, its head wound in blood thread.
fn spindle(v: &mut Vec<PartSpec>, at: Node, lit: bool) {
    v.extend(gear::staff(at, STAFF, WINE_DARK));
    let [x, y, z] = gear::staff_top(STAFF);
    let thread = part(at, Ball(0.07), BLOOD, [x, y, z]).scale([0.8, 1.2, 0.8]);
    v.extend([
        if lit { thread.glow() } else { thread },
        part(at, Cone(0.03, 0.12), BONE, [x, y + 0.13, z]),
        part(at, Ring(0.01, 0.065, 360.0), BONE, [x, y, z]),
    ]);
}
