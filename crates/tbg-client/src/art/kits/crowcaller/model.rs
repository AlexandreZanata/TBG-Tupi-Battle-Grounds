//! Crowcaller's plague mask, torn coat, shoulder raven and ritual crossbow in five forms.

use crate::art::{
    figure::{part, FigureSpec, PartSpec},
    kits::common::{body, Form},
    mesh::Shape::*,
    palette::*,
    rig::{
        self,
        Joint::*,
        Node,
        Node::{Body, Prop, Secondary},
        Proportions, Socket, Spring,
    },
};
use bevy::prelude::*;
pub const MUZZLE: Vec3 = Vec3::new(0.0, 0.035, 0.34);
pub fn plan(tier: u8) -> Proportions {
    let mut p = body::plan(false, 0.98, tier);
    p.torso.x *= 0.82;
    p.arm = 0.20;
    p
}
pub fn parent(n: Node) -> Option<Node> {
    match n {
        Secondary(Spring::HeadGear) => Some(Secondary(Spring::Back)),
        Prop(Socket::Nock) => Some(Prop(Socket::WeaponR)),
        _ => n.parent(),
    }
}
pub fn rest(n: Node, p: &Proportions) -> Vec3 {
    match n {
        Secondary(Spring::Back) => Vec3::new(0.22, 0.35, -0.06),
        Secondary(Spring::HeadGear) => Vec3::new(0.0, 0.075, 0.055),
        Secondary(Spring::Tabard) => Vec3::new(0.0, 0.30, -0.15),
        Prop(Socket::Nock) => Vec3::new(0.0, 0.035, 0.0),
        _ => rig::rest(n, p),
    }
}
pub fn spec(tier: u8, branch: u8) -> FigureSpec {
    let p = plan(tier);
    let f = Form::of(tier, branch);
    let mut v = Vec::new();
    body::humanoid(
        &mut v,
        &p,
        &body::Look {
            skin: BONE,
            cloth: CHARCOAL,
            legs: DARK,
            boots: DARK_LEATHER,
        },
    );
    v.extend([
        part(Body(Head), Ball(0.17), CHARCOAL, [0.0, 0.01, -0.03]).scale([1.0, 1.05, 1.0]),
        part(Body(Head), Box(0.15, 0.17, 0.06), BONE, [0.0, -0.005, 0.12]),
        part(
            Body(Head),
            Cone(0.05, if f.a && f.t2 { 0.30 } else { 0.20 }),
            BONE,
            [0.0, -0.05, if f.a && f.t2 { 0.30 } else { 0.25 }],
        )
        .rot([90.0, 0.0, 0.0]),
        part(
            Body(Hips),
            Frustum(0.16, 0.21, 0.28),
            CHARCOAL,
            [0.0, -0.08, 0.0],
        ),
        part(
            Secondary(Spring::Tabard),
            Box(0.27, 0.37, 0.025),
            CHARCOAL,
            [0.0, -0.20, 0.0],
        ),
        part(
            Secondary(Spring::Tabard),
            Box(0.20, 0.31, 0.018),
            ALLY,
            [0.0, -0.18, 0.017],
        ),
        part(
            Body(Hips),
            Box(0.10, 0.12, 0.09),
            DARK_LEATHER,
            [-0.18, -0.03, 0.13],
        ),
        part(
            Body(Hips),
            Box(0.11, 0.02, 0.10),
            BRASS,
            [-0.18, 0.025, 0.13],
        ),
    ]);
    for s in [-1.0, 1.0] {
        v.push(part(Body(Head), Ball(0.026), GLOW, [s * 0.05, 0.025, 0.158]).glow());
        v.push(
            part(
                Body(Chest),
                Box(0.035, 0.28, 0.03),
                BRASS,
                [s * 0.11, 0.17, 0.13],
            )
            .rot([0.0, 0.0, s * 10.0]),
        );
        v.push(
            part(
                Secondary(Spring::Tabard),
                Cone(0.052, 0.14),
                CHARCOAL,
                [s * 0.09, -0.39, 0.0],
            )
            .rot([180.0, 0.0, 0.0]),
        );
    }
    raven(&mut v);
    crossbow(&mut v, f);
    upgrades(&mut v, f);
    FigureSpec { plan: p, parts: v }
}
fn raven(v: &mut Vec<PartSpec>) {
    let body = Secondary(Spring::Back);
    let head = Secondary(Spring::HeadGear);
    v.extend([
        part(body, Ball(0.07), CHARCOAL, [0.0, 0.0, 0.0]).scale([1.0, 0.9, 1.4]),
        part(body, Cone(0.045, 0.15), CHARCOAL, [0.0, -0.01, -0.13]).rot([-105.0, 0.0, 0.0]),
        part(head, Ball(0.045), CHARCOAL, [0.0, 0.0, 0.0]),
        part(head, Cone(0.018, 0.065), BONE, [0.0, -0.01, 0.065]).rot([90.0, 0.0, 0.0]),
        part(
            Prop(Socket::Back),
            Box(0.02, 0.12, 0.11),
            DARK,
            [-0.07, -0.015, -0.02],
        )
        .rot([0.0, 0.0, -18.0]),
    ]);
    for s in [-1.0, 1.0] {
        v.push(part(
            body,
            Cyl(0.009, 0.06),
            BRASS,
            [s * 0.025, -0.06, 0.01],
        ));
        v.push(part(head, Ball(0.012), GLOW, [s * 0.033, 0.005, 0.025]).glow());
    }
}
fn crossbow(v: &mut Vec<PartSpec>, f: Form) {
    let w = Prop(Socket::WeaponR);
    v.extend([
        part(w, Box(0.055, 0.055, 0.37), OAK, [0.0, 0.0, 0.08]),
        part(w, Box(0.10, 0.08, 0.10), BRASS, [0.0, -0.025, -0.055]),
        part(w, Cyl(0.018, 0.23), BONE, [0.0, 0.04, 0.18]).rot([90.0, 0.0, 0.0]),
        part(Prop(Socket::Nock), Cyl(0.008, 0.29), DARK, [0.0, 0.0, 0.17]).rot([90.0, 0.0, 0.0]),
        part(
            Prop(Socket::Nock),
            Cone(0.022, 0.07),
            BONE,
            [0.0, 0.0, 0.34],
        )
        .rot([90.0, 0.0, 0.0]),
    ]);
    for s in [-1.0, 1.0] {
        v.extend([
            part(
                w,
                Box(if f.b { 0.26 } else { 0.20 }, 0.035, 0.045),
                if f.b { IRON } else { OAK },
                [s * 0.13, 0.0, 0.19],
            )
            .rot([0.0, s * -20.0, 0.0]),
            part(w, Cyl(0.006, 0.23), LINEN, [s * 0.11, 0.0, 0.06]).rot([65.0, 0.0, s * -62.0]),
            part(w, Ball(0.025), BRASS, [s * 0.23, 0.0, 0.15]),
        ]);
    }
    if f.b && f.t2 {
        v.push(part(w, Box(0.07, 0.17, 0.055), BONE, [0.0, 0.09, 0.06]));
        v.push(part(w, Ring(0.012, 0.05, 310.0), IRON, [0.0, 0.17, 0.08]).rot([90.0, 0.0, 0.0]));
    }
}
fn upgrades(v: &mut Vec<PartSpec>, f: Form) {
    if f.a {
        for s in [-1.0, 1.0] {
            for z in [-0.07, 0.02, 0.11] {
                v.push(
                    part(
                        Body(Chest),
                        Cone(0.052, 0.19),
                        CHARCOAL,
                        [s * 0.18, 0.29, z],
                    )
                    .rot([-20.0, 0.0, s * -70.0]),
                );
            }
            v.push(part(
                Body(Hips),
                Cyl(0.025, 0.09),
                LEAF_DARK,
                [s * 0.12, -0.01, 0.18],
            ));
            v.push(part(Body(Hips), Ball(0.022), GLOW, [s * 0.12, -0.035, 0.20]).glow());
        }
        if f.t2 {
            v.extend([
                part(Body(Hips), Cyl(0.008, 0.21), BRASS, [0.20, -0.12, 0.13]),
                part(Body(Hips), Ball(0.06), BRASS, [0.20, -0.27, 0.13]),
                part(Body(Hips), Cone(0.05, 0.05), GLOW, [0.20, -0.22, 0.13]).glow(),
            ]);
        }
    }
    if f.b {
        for j in [UpperArmL, UpperArmR] {
            v.push(part(Body(j), Pill(0.062, 0.09), BONE, [0.0, -0.08, 0.0]));
        }
    }
}
