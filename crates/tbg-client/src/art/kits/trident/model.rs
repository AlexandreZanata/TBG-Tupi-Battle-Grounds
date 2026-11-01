//! A tall countershaded shark with a fitted jaw, gills, dorsal fin and five armored forms.

use super::weapon;
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

pub fn plan(tier: u8) -> Proportions {
    let mut p = body::plan(true, 0.96, tier);
    p.leg = 0.46;
    p.torso = Vec3::new(0.48, 0.42, 0.26);
    p.arm = 0.35;
    p
}

pub fn parent(node: Node) -> Option<Node> {
    match node {
        // A rigid two-hand prop owns its frame; both wrists solve onto its anchors.
        // This also keeps the shaft out of the torso while joint rotations blend.
        Prop(Socket::WeaponR) => Some(Body(Chest)),
        _ => node.parent(),
    }
}

pub fn rest(node: Node, p: &Proportions) -> Vec3 {
    match node {
        Secondary(Spring::HeadGear) => Vec3::new(0.0, -0.075, 0.13),
        Secondary(Spring::Back) => Vec3::new(0.0, 0.27, -0.18),
        _ => rig::rest(node, p),
    }
}

pub fn spec(tier: u8, branch: u8) -> FigureSpec {
    let p = plan(tier);
    let f = Form::of(tier, branch);
    let mut parts = Vec::new();
    body::humanoid(
        &mut parts,
        &p,
        &body::Look {
            skin: SHARK_SLATE,
            cloth: SHARK_SLATE,
            legs: SHARK_SLATE,
            boots: DARK_LEATHER,
        },
    );
    torso(&mut parts, f);
    head(&mut parts, f);
    fins_and_armor(&mut parts, f);
    weapon::build(&mut parts, f);
    FigureSpec { plan: p, parts }
}

fn torso(v: &mut Vec<PartSpec>, f: Form) {
    let belly = if f.a { BONE } else { SHARK_BELLY };
    v.extend([
        part(Body(Chest), Ball(0.21), belly, [0.0, 0.19, 0.15]).scale([1.0, 1.35, 0.36]),
        part(Body(Hips), Box(0.29, 0.13, 0.04), belly, [0.0, 0.0, 0.15]),
        part(
            Body(Hips),
            Box(0.43, 0.075, 0.29),
            if f.b { RED_CLOTH } else { DARK_LEATHER },
            [0.0, 0.035, 0.0],
        ),
        part(
            Body(Hips),
            Box(0.07, 0.065, 0.025),
            BRASS,
            [0.0, 0.035, 0.16],
        ),
        // The fitted arm band and narrow belt tails remain team identifiers in every form.
        part(
            Body(UpperArmR),
            Box(0.155, 0.075, 0.045),
            ALLY,
            [0.0, -0.09, 0.07],
        ),
    ]);
    for s in [-1.0, 1.0] {
        v.push(part(
            Body(Hips),
            Box(0.04, 0.19, 0.035),
            ALLY,
            [s * 0.13, -0.07, 0.18],
        ));
        v.push(
            part(
                Body(Chest),
                Ring(0.013, 0.049, 300.0),
                RUST,
                [s * 0.18, 0.32, 0.13],
            )
            .rot([90.0, 0.0, s * 25.0]),
        );
    }
    if f.b {
        v.push(
            part(
                Body(Chest),
                Box(0.07, 0.36, 0.035),
                RED_CLOTH,
                [0.10, 0.22, 0.205],
            )
            .rot([0.0, 0.0, -20.0]),
        );
    }
}

fn head(v: &mut Vec<PartSpec>, f: Form) {
    v.extend([
        part(Body(Head), Ball(0.24), SHARK_SLATE, [0.0, 0.02, 0.0]).scale([1.25, 0.85, 1.10]),
        part(
            Body(Head),
            Cone(0.17, 0.28),
            SHARK_SLATE,
            [0.0, 0.015, 0.23],
        )
        .rot([90.0, 0.0, 0.0])
        .scale([1.25, 1.0, 0.65]),
        part(
            Secondary(Spring::HeadGear),
            Ball(0.15),
            if f.a { BONE } else { SHARK_BELLY },
            [0.0, -0.005, 0.085],
        )
        .scale([1.40, 0.40, 1.12]),
        part(
            Body(Head),
            Box(0.30, 0.026, 0.045),
            DARK,
            [0.0, -0.06, 0.29],
        ),
    ]);
    for s in [-1.0, 1.0] {
        v.push(part(
            Body(Head),
            Ball(0.033),
            DARK,
            [s * 0.205, 0.075, 0.21],
        ));
        v.push(part(
            Body(Head),
            Ball(0.015),
            AMBER,
            [s * 0.212, 0.080, 0.229],
        ));
        v.push(
            part(
                Body(Head),
                Box(0.105, 0.025, 0.055),
                SHARK_SLATE,
                [s * 0.205, 0.11, 0.205],
            )
            .rot([0.0, 0.0, -s * 12.0]),
        );
        for x in [0.045, 0.105] {
            v.push(
                part(Body(Head), Cone(0.018, 0.043), BONE, [s * x, -0.065, 0.294])
                    .rot([180.0, 0.0, 0.0]),
            );
        }
        for z in [-0.075, -0.025, 0.025] {
            v.push(
                part(
                    Body(Head),
                    Box(0.018, 0.105, 0.021),
                    DARK,
                    [s * 0.268, -0.015, z],
                )
                .rot([12.0, 0.0, s * 12.0]),
            );
        }
    }
}

fn fins_and_armor(v: &mut Vec<PartSpec>, f: Form) {
    let dorsal_height = if f.a && f.t2 { 0.70 } else { 0.50 };
    v.push(
        part(
            Secondary(Spring::Back),
            Cone(0.22, dorsal_height),
            if f.a && f.t2 { BONE } else { SHARK_SLATE },
            [0.0, dorsal_height * 0.5, -0.07],
        )
        .scale([0.26, 1.0, 1.0])
        .rot([-16.0, 0.0, 0.0]),
    );
    for (upper, fore, shin, side) in [
        (UpperArmL, ForearmL, ShinL, 1.0),
        (UpperArmR, ForearmR, ShinR, -1.0),
    ] {
        v.push(
            part(
                Body(fore),
                Cone(0.075, 0.19),
                SHARK_SLATE,
                [side * 0.075, -0.10, -0.03],
            )
            .scale([0.25, 1.0, 1.0])
            .rot([0.0, 0.0, side * 30.0]),
        );
        v.push(part(
            Body(fore),
            Cyl(0.061, 0.08),
            DARK_LEATHER,
            [0.0, -0.27, 0.0],
        ));
        if f.a {
            v.push(part(
                Body(upper),
                Pill(0.11, 0.045),
                BONE,
                [0.0, -0.01, 0.0],
            ));
            v.push(part(
                Body(upper),
                Ring(0.012, 0.108, 240.0),
                RUST,
                [0.0, 0.015, 0.0],
            ));
        }
        if f.a && f.t2 {
            v.push(part(
                Body(shin),
                Box(0.135, 0.17, 0.035),
                BONE,
                [0.0, -0.10, 0.065],
            ));
        }
        if f.b && f.t2 {
            v.push(part(
                Body(upper),
                Box(0.17, 0.06, 0.17),
                RUST,
                [0.0, -0.025, 0.0],
            ));
        }
    }
}
