//! Solar Sentinel's sun-disk shield, ivory spear armor and five light/coverage forms.
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
pub fn plan(t: u8) -> Proportions {
    body::plan(false, 1.04, t)
}
pub fn parent(n: Node) -> Option<Node> {
    match n {
        Prop(Socket::StringTop) | Prop(Socket::StringBottom) => Some(Prop(Socket::OffhandL)),
        _ => n.parent(),
    }
}
pub fn rest(n: Node, p: &Proportions) -> Vec3 {
    match n {
        Secondary(Spring::HeadGear) => Vec3::new(0.0, 0.15, -0.06),
        Prop(Socket::StringTop) => Vec3::new(-0.15, 0.0, 0.0),
        Prop(Socket::StringBottom) => Vec3::new(0.15, 0.0, 0.0),
        _ => rig::rest(n, p),
    }
}
pub fn spec(t: u8, b: u8) -> FigureSpec {
    let p = plan(t);
    let f = Form::of(t, b);
    let mut v = Vec::new();
    body::humanoid(
        &mut v,
        &p,
        &body::Look {
            skin: IVORY,
            cloth: NAVY,
            legs: NAVY,
            boots: GOLD,
        },
    );
    body::tabard(&mut v, 0.16, 0.31);
    v.extend([
        part(
            Body(Chest),
            Box(if f.a { 0.37 } else { 0.30 }, 0.26, 0.10),
            IVORY,
            [0.0, 0.16, 0.12],
        ),
        part(Body(Chest), Cyl(0.07, 0.025), GOLD, [0.0, 0.22, 0.18]).rot([90.0, 0.0, 0.0]),
        part(Body(Head), Ball(0.18), IVORY, [0.0, 0.02, 0.0]),
        part(Body(Head), Box(0.15, 0.05, 0.04), NAVY, [0.0, 0.0, 0.16]),
        part(Body(Head), Box(0.05, 0.22, 0.035), GOLD, [0.0, 0.07, 0.18]),
        part(
            Secondary(Spring::HeadGear),
            Ring(0.016, if f.b && f.t2 { 0.29 } else { 0.22 }, 360.0),
            GOLD,
            [0.0, 0.02, -0.08],
        )
        .rot([90.0, 0.0, 0.0]),
        part(
            Prop(Socket::WeaponR),
            Cyl(0.022, 0.90),
            OAK,
            [0.0, 0.0, 0.12],
        )
        .rot([90.0, 0.0, 0.0]),
        part(
            Prop(Socket::WeaponR),
            Cone(0.055, 0.20),
            GOLD,
            [0.0, 0.0, 0.65],
        )
        .rot([90.0, 0.0, 0.0]),
        part(
            Secondary(Spring::Back),
            Box(0.28, 0.40, 0.025),
            NAVY,
            [0.0, 0.04, -0.16],
        ),
    ]);
    shield(&mut v, f);
    for (j, s) in [(UpperArmL, 1.0), (UpperArmR, -1.0)] {
        v.push(
            part(
                Body(j),
                Ball(if f.b { 0.11 } else { 0.09 }),
                IVORY,
                [0.0, -0.01, 0.0],
            )
            .scale([1.2, 0.7, 1.0]),
        );
        if f.b {
            v.push(
                part(Body(j), Cyl(0.07, 0.025), GOLD, [s * 0.09, 0.02, 0.01]).rot([0.0, 0.0, 90.0]),
            );
        }
    }
    if f.b && f.t2 {
        for s in [-1.0, 1.0] {
            v.push(
                part(
                    Secondary(Spring::HeadGear),
                    Box(0.09, 0.18, 0.025),
                    IVORY,
                    [s * 0.29, 0.02, -0.08],
                )
                .rot([0.0, 0.0, -s * 15.0]),
            );
        }
    }
    FigureSpec { plan: p, parts: v }
}
fn shield(v: &mut Vec<PartSpec>, f: Form) {
    let w = Prop(Socket::OffhandL);
    v.extend([
        part(w, Cyl(0.25, 0.045), GOLD, [0.0, 0.0, 0.05]).rot([90.0, 0.0, 0.0]),
        part(w, Cyl(0.21, 0.05), IVORY, [0.0, 0.0, 0.07]).rot([90.0, 0.0, 0.0]),
        part(w, Ball(0.075), GOLD, [0.0, 0.0, 0.12]).scale([1.0, 1.0, 0.5]),
    ]);
    for i in 0..8 {
        let a = i as f32 * std::f32::consts::FRAC_PI_4;
        v.push(
            part(
                w,
                Cone(0.045, 0.12),
                GOLD,
                [a.sin() * 0.25, a.cos() * 0.25, 0.05],
            )
            .rot([0.0, 0.0, -a.to_degrees()]),
        );
    }
    for (s, n) in [(-1.0, Socket::StringTop), (1.0, Socket::StringBottom)] {
        v.push(
            part(Prop(n), Box(0.08, 0.30, 0.025), GOLD, [0.0, 0.0, 0.12]).rot([0.0, 0.0, s * 15.0]),
        );
    }
    if f.a && f.t2 {
        v.push(part(w, Box(0.07, 0.23, 0.04), IVORY, [0.0, 0.32, 0.08]));
        v.push(part(w, Ball(0.04), GOLD, [0.0, 0.45, 0.08]));
    }
}
