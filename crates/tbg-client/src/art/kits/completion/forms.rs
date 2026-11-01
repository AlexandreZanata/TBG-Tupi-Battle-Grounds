//! Shared construction tools, not character silhouettes: bowls, blades, insect anatomy and team marks.
pub(super) use crate::art::{
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
pub(super) use bevy::prelude::*;
pub fn humanoid(
    p: &Proportions,
    skin: Color,
    cloth: Color,
    legs: Color,
    boots: Color,
) -> Vec<PartSpec> {
    let mut v = Vec::new();
    body::humanoid(
        &mut v,
        p,
        &body::Look {
            skin,
            cloth,
            legs,
            boots,
        },
    );
    v
}
pub fn team(v: &mut Vec<PartSpec>, n: Node, width: f32) {
    v.push(part(n, Box(width, 0.14, 0.02), ALLY, [0.0, -0.03, 0.145]));
    v.push(part(
        n,
        Box(width * 0.8, 0.018, 0.022),
        LINEN,
        [0.0, -0.09, 0.146],
    ));
}
pub fn blade(v: &mut Vec<PartSpec>, n: Node, len: f32, c: Color, curved: bool) {
    v.push(part(n, Cyl(0.022, 0.14), OAK, [0.0, 0.0, 0.0]).rot([90.0, 0.0, 0.0]));
    v.push(part(n, Box(0.17, 0.035, 0.04), GOLD, [0.0, 0.0, 0.06]));
    if curved {
        v.push(
            part(n, Ring(0.026, len * 0.58, 145.0), c, [0.0, 0.0, len * 0.43])
                .rot([90.0, 0.0, 15.0])
                .scale([0.7, 1.0, 1.0]),
        );
    } else {
        v.push(
            part(
                n,
                Bevel(0.095, 0.035, len, 0.012),
                c,
                [0.0, 0.0, 0.08 + len * 0.46],
            )
            .rot([0.0, -8.0, 0.0]),
        );
        v.push(
            part(n, Cone(0.06, 0.13), c, [-0.025, 0.0, 0.08 + len])
                .rot([90.0, 0.0, 0.0])
                .scale([1.0, 1.0, 0.3]),
        );
    }
}
pub fn bell(v: &mut Vec<PartSpec>, n: Node, r: f32, c: Color) {
    v.push(part(n, Cyl(0.025, 0.12), RUST, [0.0, 0.045, 0.0]));
    v.push(part(
        n,
        Frustum(r * 0.45, r, r * 1.25),
        c,
        [0.0, -0.09, 0.0],
    ));
    v.push(part(n, Ring(0.014, r, 360.0), BONE, [0.0, -0.16, 0.0]));
    v.push(part(n, Ball(0.023), GLOW, [0.0, -0.17, 0.0]).glow());
}
pub fn bowl(v: &mut Vec<PartSpec>, n: Node, r: f32, c: Color, liquid: Color) {
    v.push(part(n, Frustum(r, r * 0.55, 0.10), c, [0.0, -0.035, 0.06]));
    v.push(part(n, Cyl(r * 0.88, 0.012), liquid, [0.0, 0.018, 0.06]).glow());
    v.push(part(n, Ring(0.012, r, 360.0), c, [0.0, 0.022, 0.06]));
}
pub fn spoon(v: &mut Vec<PartSpec>, n: Node, len: f32, c: Color) {
    v.push(part(n, Cyl(0.018, len), c, [0.0, 0.0, len * 0.3]).rot([90.0, 0.0, 0.0]));
    v.push(part(n, Ball(0.07), c, [0.0, 0.0, len * 0.85]).scale([0.65, 0.22, 1.0]));
}
pub fn needle(v: &mut Vec<PartSpec>, n: Node, len: f32, c: Color) {
    v.push(part(n, Cyl(0.012, len), c, [0.0, 0.0, len * 0.45]).rot([90.0, 0.0, 0.0]));
    v.push(part(n, Cone(0.035, 0.14), c, [0.0, 0.0, len]).rot([90.0, 0.0, 0.0]));
}
pub fn antennae(v: &mut Vec<PartSpec>, len: f32, bent: bool) {
    for s in [-1.0, 1.0] {
        v.push(
            part(
                Body(Head),
                Cyl(0.012, len),
                CHITIN,
                [s * 0.07, 0.13 + len * 0.35, 0.0],
            )
            .rot([-15.0, 0.0, s * -23.0]),
        );
        v.push(part(
            Body(Head),
            if bent {
                Box(0.018, 0.018, 0.09)
            } else {
                Ball(0.025)
            },
            AMBER,
            [s * (0.07 + len * 0.18), 0.13 + len * 0.8, 0.035],
        ));
    }
}
pub fn insect(p: &Proportions, f: Form, waist: f32) -> Vec<PartSpec> {
    let mut v = humanoid(p, CHITIN, MOSS, CHITIN, CHITIN);
    v.push(part(Body(Hips), Ball(waist), CHITIN, [0.0, 0.0, -0.02]));
    for (u, l, foot) in [
        (
            Secondary(Spring::Back),
            Prop(Socket::StringTop),
            Prop(Socket::Back),
        ),
        (
            Secondary(Spring::Tabard),
            Prop(Socket::StringBottom),
            Prop(Socket::Nock),
        ),
    ] {
        v.push(part(
            u,
            Frustum(0.035, 0.023, p.leg * 0.55),
            if f.a { IVORY } else { MOSS },
            [0.0, -p.leg * 0.275, 0.0],
        ));
        v.push(part(
            l,
            Cyl(0.025, p.leg * 0.55),
            CHITIN,
            [0.0, -p.leg * 0.275, 0.0],
        ));
        v.push(part(
            foot,
            Bevel(0.09, 0.05, 0.13, 0.01),
            CHITIN,
            [0.0, -0.025, 0.025],
        ));
    }
    for s in [-1.0, 1.0] {
        v.push(part(
            Body(Head),
            Ball(0.04),
            AMBER,
            [s * 0.105, 0.02, 0.105],
        ));
    }
    team(&mut v, Body(Hips), 0.08);
    v
}
pub fn insect_parent(n: Node) -> Option<Node> {
    match n {
        Secondary(Spring::Back | Spring::Tabard) => Some(Body(Hips)),
        Prop(Socket::StringTop) => Some(Secondary(Spring::Back)),
        Prop(Socket::StringBottom) => Some(Secondary(Spring::Tabard)),
        Prop(Socket::Back) => Some(Prop(Socket::StringTop)),
        Prop(Socket::Nock) => Some(Prop(Socket::StringBottom)),
        Secondary(Spring::HeadGear) => Some(Body(Chest)),
        _ => n.parent(),
    }
}
pub fn insect_rest(n: Node, p: &Proportions) -> Vec3 {
    match n {
        Secondary(Spring::Back) => Vec3::new(p.torso.x * 0.45, -0.02, -0.08),
        Secondary(Spring::Tabard) => Vec3::new(-p.torso.x * 0.45, -0.02, -0.08),
        Prop(Socket::StringTop | Socket::StringBottom | Socket::Back | Socket::Nock) => {
            Vec3::NEG_Y * p.leg * 0.55
        }
        Secondary(Spring::HeadGear) => Vec3::ZERO,
        _ => rig::rest(n, p),
    }
}
pub fn needle_parent(n: Node) -> Option<Node> {
    if matches!(n, Secondary(Spring::HeadGear)) {
        Some(Prop(Socket::WeaponR))
    } else {
        n.parent()
    }
}
pub fn needle_rest(n: Node, p: &Proportions) -> Vec3 {
    if matches!(n, Secondary(Spring::HeadGear)) {
        Vec3::ZERO
    } else {
        rig::rest(n, p)
    }
}

pub fn dragonfly_parent(n: Node) -> Option<Node> {
    match n {
        Prop(Socket::WeaponR) => Some(Body(Chest)),
        Secondary(Spring::HeadGear) => Some(Body(HandR)),
        _ => insect_parent(n),
    }
}
pub fn dragonfly_rest(n: Node, p: &Proportions) -> Vec3 {
    match n {
        Prop(Socket::WeaponR) => Vec3::ZERO,
        Secondary(Spring::HeadGear) => Vec3::new(0.0, 0.0, 0.13),
        _ => insect_rest(n, p),
    }
}
