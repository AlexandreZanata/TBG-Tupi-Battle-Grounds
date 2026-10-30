//! Shieldbearer figure: a man-at-arms in mail and a team tabard behind a
//! kite shield, an arming sword in hand. Branch A (Bulwark) grows the
//! shield and the plate, then a castle-crested tower shield and a great
//! helm; branch B (Vanguard) lightens him with a longer blade, plume and
//! cape, then a winged helm and a glowing champion's sword.

use bevy::prelude::*;

use crate::art::{
    figure::{part, FigureSpec, PartSpec},
    kits::common::{body, gear, Form},
    mesh::Shape::*,
    palette::*,
    rig::{Joint::*, Node::Body, Node::Prop, Node::Secondary, Proportions, Socket, Spring},
};

const MAIL: Color = Color::srgb(0.62, 0.65, 0.7);

pub fn plan(tier: u8) -> Proportions {
    let mut p = body::plan(false, 1.05, tier);
    p.torso.x *= 1.08;
    p
}

fn blade(f: Form) -> f32 {
    match (f.b, f.t2) {
        (true, true) => 0.62,
        (true, false) => 0.52,
        _ => 0.42,
    }
}

/// The sword's point (weapon space).
pub fn tip(f: Form) -> Vec3 {
    Vec3::new(0.0, 0.0, 0.2 + blade(f))
}

pub fn spec(tier: u8, branch: u8) -> FigureSpec {
    let f = Form::of(tier, branch);
    let p = plan(tier);
    let mut v = Vec::with_capacity(130);
    let look = body::Look {
        skin: SKIN,
        cloth: MAIL,
        legs: DARK_LEATHER,
        boots: DARK_LEATHER,
    };
    body::humanoid(&mut v, &p, &look);
    body::tabard(&mut v, p.torso.x * 0.8, 0.42);
    body::greaves(&mut v, STEEL);
    body::bracers(&mut v, &p, LEATHER);
    body::pauldrons(
        &mut v,
        if f.a { STEEL } else { MAIL },
        if f.a { 0.12 } else { 0.095 },
    );
    head(&mut v, &p, f);
    arms(&mut v, f);
    if f.b {
        body::cape(&mut v, RED_CLOTH, 0.3, if f.t2 { 0.6 } else { 0.48 });
    }
    if f.a && f.t2 {
        let t = p.torso;
        v.push(part(
            Body(Chest),
            Box(t.x * 1.0, t.y * 0.6, t.z * 1.15),
            STEEL,
            [0.0, t.y * 0.6, 0.0],
        ));
        v.push(part(
            Body(Chest),
            Box(t.x * 0.4, 0.04, 0.03),
            GOLD,
            [0.0, t.y * 0.88, t.z * 0.6],
        ));
    }
    FigureSpec { plan: p, parts: v }
}

fn head(v: &mut Vec<PartSpec>, p: &Proportions, f: Form) {
    let h = p.head;
    match (f.a, f.b, f.t2) {
        (true, _, true) => {
            // A great helm with a gold cross and eye slit.
            v.extend([
                part(
                    Body(Head),
                    Cyl(h * 1.08, h * 1.7),
                    STEEL,
                    [0.0, h * 0.1, 0.0],
                ),
                part(
                    Body(Head),
                    Box(h * 1.5, 0.025, 0.03),
                    DARK,
                    [0.0, h * 0.15, h * 1.06],
                ),
                part(
                    Body(Head),
                    Box(0.03, h * 1.0, 0.03),
                    GOLD,
                    [0.0, -h * 0.25, h * 1.07],
                ),
                part(
                    Body(Head),
                    Frustum(h * 0.5, h * 1.08, h * 0.35),
                    STEEL,
                    [0.0, h * 1.12, 0.0],
                ),
            ]);
        }
        (_, true, true) => {
            body::helmet(v, p, body::Helm::Cap(STEEL));
            for s in [-1.0f32, 1.0] {
                v.push(
                    part(
                        Secondary(Spring::HeadGear),
                        Box(0.02, h * 1.1, h * 0.9),
                        LINEN,
                        [s * h * 0.95, h * 0.85, -h * 0.2],
                    )
                    .rot([-20.0, 0.0, -25.0 * s]),
                );
            }
        }
        (_, true, false) => body::helmet(v, p, body::Helm::Plumed(STEEL, RED_CLOTH)),
        (true, _, false) => body::helmet(v, p, body::Helm::Kettle(STEEL)),
        _ => {
            body::helmet(v, p, body::Helm::Cap(STEEL));
            v.push(part(
                Body(Head),
                Box(0.025, h * 0.7, 0.03),
                STEEL,
                [0.0, h * 0.0, h * 1.02],
            ));
        }
    }
}

fn arms(v: &mut Vec<PartSpec>, f: Form) {
    let r = Prop(Socket::WeaponR);
    let l = Prop(Socket::OffhandL);
    v.extend(gear::sword(
        r,
        blade(f),
        STEEL,
        if f.b { GOLD } else { STEEL },
    ));
    if f.b && f.t2 {
        v.push(
            part(
                r,
                Box(0.02, 0.022, blade(f)),
                ALLY,
                [0.0, 0.0, 0.1 + blade(f) * 0.5],
            )
            .glow(),
        );
    }
    if f.a && f.t2 {
        // A tower shield crowned with a castle's merlons.
        v.extend([
            part(l, Box(0.05, 0.78, 0.5), ALLY, [0.09, -0.08, 0.0]),
            part(l, Box(0.055, 0.8, 0.06), GOLD, [0.09, -0.08, 0.22]),
            part(l, Box(0.055, 0.8, 0.06), GOLD, [0.09, -0.08, -0.22]),
            part(l, Box(0.06, 0.2, 0.2), GOLD, [0.1, 0.0, 0.0]),
        ]);
        for z in [-0.18, 0.0, 0.18] {
            v.push(part(l, Box(0.05, 0.1, 0.1), STEEL, [0.09, 0.36, z]));
        }
        return;
    }
    v.extend(gear::kite_shield(
        l,
        if f.a { 0.66 } else { 0.54 },
        if f.a { GOLD } else { STEEL },
    ));
}
