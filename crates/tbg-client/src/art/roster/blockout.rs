//! Gray v2 silhouettes only: the seven other units receive final art in P22–P29.

use super::body;
use crate::art::{
    figure::{part, FigureSpec, PartSpec},
    mesh::Shape::*,
    palette::*,
    rig::{
        BodyPlan,
        Joint::*,
        Node::{Body, Prop},
        Socket,
    },
};
use tbg_core::UnitKind;

#[rustfmt::skip]
pub fn spec(kind: UnitKind) -> FigureSpec {
    let plan = if super::large(kind) {
        BodyPlan::Large
    } else {
        BodyPlan::Humanoid
    }
    .proportions();
    let mut parts = body::parts(&plan, STEEL, STEEL, FRAME);
    match kind {
        UnitKind::Shieldbearer => parts.extend([
            part(
                Prop(Socket::OffhandL),
                Box(0.40, 0.6, 0.08),
                FRAME,
                [0.05, 0.12, 0.14],
            ),
            part(
                Prop(Socket::WeaponR),
                Cyl(0.03, 0.5),
                FRAME,
                [0.0, 0.18, 0.0],
            ),
            part(Prop(Socket::WeaponR), Ball(0.11), STEEL, [0.0, 0.45, 0.0]),
        ]),
        UnitKind::Archer => {
            parts.push(part(
                Body(Head),
                Cone(0.17, 0.3),
                FRAME,
                [0.0, 0.09, -0.025],
            ));
            bow(&mut parts, 0.68, false);
        }
        UnitKind::BoneBrute => parts.extend([
            part(Body(Chest), Box(0.52, 0.12, 0.3), FRAME, [0.0, 0.35, -0.04])
                .rot([15.0, 0.0, 0.0]),
            part(
                Prop(Socket::WeaponR),
                Box(0.12, 0.6, 0.28),
                FRAME,
                [0.0, 0.3, 0.06],
            ),
        ]),
        UnitKind::Hexer => {
            parts.push(part(
                Body(Head),
                Cone(0.18, 0.36),
                FRAME,
                [0.0, 0.14, -0.04],
            ));
            staff(&mut parts, false);
        }
        UnitKind::Sawbot => saw(&mut parts),
        UnitKind::SiegeWalker => bow(&mut parts, 0.78, true),
        UnitKind::Bloomkeeper => staff(&mut parts, true),
        UnitKind::Rootcaller => roots(&mut parts),
        UnitKind::Nester => staff(&mut parts, false),
        UnitKind::Rampart => parts.push(part(Prop(Socket::OffhandL), Box(0.55, 0.8, 0.1), FRAME, [0.05, 0.12, 0.14])),
        UnitKind::Beetling => roots(&mut parts),
    }
    FigureSpec { plan, parts }
}
pub fn bow(parts: &mut Vec<PartSpec>, radius: f32, great: bool) {
    let node = Prop(Socket::OffhandL);
    parts.push(
        part(node, Ring(0.025, radius, 180.0), FRAME, [0.0, 0.12, 0.05]).rot([0.0, 90.0, 90.0]),
    );
    parts.push(part(
        node,
        Box(0.012, radius * 2.0, 0.012),
        STEEL,
        [0.0, 0.12, 0.05],
    ));
    if great {
        parts.push(part(node, Box(0.55, 0.15, 0.22), FRAME, [0.0, 0.12, 0.05]));
        parts.push(part(node, Cyl(0.14, 0.08), STEEL, [0.0, 0.12, 0.19]).rot([90.0, 0.0, 0.0]));
    }
}
fn staff(parts: &mut Vec<PartSpec>, flower: bool) {
    let node = Prop(Socket::WeaponR);
    parts.push(part(node, Cyl(0.025, 1.15), FRAME, [0.0, 0.25, 0.0]));
    if flower {
        for i in 0..5 {
            let a = i as f32 * std::f32::consts::TAU / 5.0;
            parts.push(part(
                node,
                Ball(0.09),
                STEEL,
                [a.cos() * 0.14, 0.85 + a.sin() * 0.14, 0.0],
            ));
        }
    } else {
        parts.push(part(node, Ball(0.14), STEEL, [0.0, 0.91, 0.0]).scale([0.85, 1.1, 0.8]));
        parts.push(part(node, Box(0.18, 0.08, 0.12), FRAME, [0.0, 0.79, 0.05]));
    }
}

fn saw(parts: &mut Vec<PartSpec>) {
    parts.push(part(
        Body(Head),
        Box(0.24, 0.24, 0.22),
        FRAME,
        [0.0, 0.0, 0.0],
    ));
    parts.push(
        part(
            Prop(Socket::WeaponR),
            Cyl(0.26, 0.07),
            FRAME,
            [0.0, 0.0, 0.12],
        )
        .rot([90.0, 0.0, 0.0]),
    );
    for i in 0..8 {
        let a = i as f32 * 45.0;
        parts.push(
            part(
                Prop(Socket::WeaponR),
                Cone(0.065, 0.13),
                STEEL,
                [
                    0.0,
                    a.to_radians().cos() * 0.27,
                    0.12 + a.to_radians().sin() * 0.27,
                ],
            )
            .rot([0.0, 0.0, -a]),
        );
    }
}

fn roots(parts: &mut Vec<PartSpec>) {
    for hand in [HandL, HandR] {
        for i in [-1.0, 0.0, 1.0] {
            parts.push(
                part(
                    Body(hand),
                    Cone(0.045, 0.48),
                    FRAME,
                    [i * 0.08, -0.01, 0.22],
                )
                .rot([65.0, i * 22.0, 0.0]),
            );
        }
    }
    parts.push(part(Body(Head), Cone(0.1, 0.25), FRAME, [-0.10, 0.15, 0.0]).rot([0.0, 0.0, 30.0]));
}
