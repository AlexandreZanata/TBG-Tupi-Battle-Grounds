//! P15 Crown Archer base: linen, oak hood, leather bracer, longbow, spring quiver.

use super::super::body;
use crate::art::{
    figure::{part, FigureSpec, PartSpec},
    mesh::Shape::*,
    palette::*,
    rig::{
        BodyPlan,
        Joint::*,
        Node::{Body, Prop, Secondary},
        Socket, Spring,
    },
};

pub fn spec() -> FigureSpec {
    let mut plan = BodyPlan::Humanoid.proportions();
    // Preserve the existing client's 20% lane-readability scale.
    plan.scale *= 1.2;
    let mut parts = body::parts(&plan, LINEN, SKIN, DARK_LEATHER);
    costume(&mut parts);
    bow(&mut parts);
    quiver(&mut parts);
    FigureSpec { plan, parts }
}
fn costume(parts: &mut Vec<PartSpec>) {
    parts.extend([
        part(Body(Head), Ball(0.163), OAK, [0.0, 0.035, -0.055]).scale([1.04, 1.05, 1.0]),
        part(Body(Head), Box(0.25, 0.035, 0.065), OAK, [0.0, 0.10, 0.095]).rot([-12.0, 0.0, 0.0]),
        part(
            Body(Head),
            Box(0.027, 0.045, 0.04),
            SKIN,
            [0.0, -0.005, 0.139],
        ),
        part(
            Body(Chest),
            Frustum(0.16, 0.21, 0.10),
            OAK,
            [0.0, 0.32, -0.015],
        )
        .scale([1.0, 1.0, 0.8]),
        part(
            Body(Hips),
            Frustum(0.16, 0.21, 0.20),
            LINEN,
            [0.0, -0.08, 0.0],
        )
        .scale([1.0, 1.0, 0.75]),
        part(
            Body(Hips),
            Box(0.34, 0.045, 0.255),
            DARK_LEATHER,
            [0.0, 0.04, 0.0],
        ),
        part(
            Body(Hips),
            Box(0.055, 0.055, 0.015),
            GOLD,
            [0.0, 0.04, 0.14],
        ),
        part(
            Body(ForearmL),
            Box(0.125, 0.13, 0.125),
            DARK_LEATHER,
            [0.0, -0.08, 0.0],
        ),
        part(
            Body(ForearmR),
            Box(0.115, 0.055, 0.12),
            DARK_LEATHER,
            [0.0, -0.10, 0.0],
        ),
        part(
            Secondary(Spring::Tabard),
            Box(0.145, 0.34, 0.018),
            ALLY,
            [0.0, 0.09, 0.025],
        ),
        part(
            Secondary(Spring::Tabard),
            Box(0.12, 0.02, 0.022),
            GOLD,
            [0.0, -0.075, 0.025],
        ),
        part(
            Secondary(Spring::HeadGear),
            Cone(0.10, 0.18),
            OAK,
            [0.0, 0.13, -0.14],
        )
        .rot([-45.0, 0.0, 0.0]),
    ]);
    face(parts);
}
fn face(parts: &mut Vec<PartSpec>) {
    for x in [-1.0, 1.0] {
        parts.push(part(
            Body(Head),
            Box(0.035, 0.15, 0.10),
            OAK,
            [x * 0.125, -0.015, 0.055],
        ));
        parts.push(part(
            Body(Head),
            Box(0.018, 0.026, 0.012),
            INK,
            [x * 0.049, 0.025, 0.125],
        ));
    }
}

fn bow(parts: &mut Vec<PartSpec>) {
    let node = Prop(Socket::OffhandL);
    parts.extend([
        part(node, Ring(0.019, 0.64, 180.0), OAK, [0.0, 0.08, -0.23])
            .scale([1.0, 1.0, 0.45])
            .rot([0.0, 0.0, 90.0]),
        part(
            node,
            Box(0.045, 0.15, 0.06),
            DARK_LEATHER,
            [0.0, 0.08, 0.06],
        ),
    ]);
    for y in [-0.56, 0.72] {
        parts.push(part(node, Box(0.026, 0.05, 0.026), LINEN, [0.0, y, -0.23]));
    }
}
fn quiver(parts: &mut Vec<PartSpec>) {
    let node = Prop(Socket::Back);
    parts.extend([
        part(node, Cyl(0.065, 0.34), DARK_LEATHER, [0.04, 0.02, -0.03]).rot([-18.0, 0.0, -18.0]),
        part(node, Ring(0.008, 0.065, 360.0), GOLD, [0.08, 0.18, -0.07]).rot([-18.0, 0.0, -18.0]),
        part(
            Body(Chest),
            Box(0.043, 0.37, 0.018),
            DARK_LEATHER,
            [0.015, 0.18, 0.13],
        )
        .rot([0.0, 0.0, -32.0]),
    ]);
    for x in [-0.025, 0.02, 0.065] {
        parts.push(
            part(node, Box(0.009, 0.29, 0.009), TIMBER, [x, 0.21, -0.07]).rot([-12.0, 0.0, -10.0]),
        );
        parts.push(
            part(
                node,
                Box(0.015, 0.07, 0.055),
                ALLY,
                [x + 0.012, 0.34, -0.10],
            )
            .rot([-12.0, 0.0, -10.0]),
        );
    }
}
