//! Astreon's rupture-god anatomy: graphite, gold and ivory armor in five forms.

use super::lance;
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

/// Gods monument scale (GD-01), tall and athletic beside the stone Colossus.
pub fn plan(tier: u8) -> Proportions {
    let mut p = body::plan(true, 1.64, tier);
    p.torso = Vec3::new(0.46, 0.37, 0.25);
    p.arm = 0.33;
    p.leg = 0.48;
    p.head = 0.14;
    p
}

pub fn parent(node: Node) -> Option<Node> {
    match node {
        // A rigid two-hand prop: both wrists solve onto it, the shaft stays clear of the torso.
        Prop(Socket::WeaponR) => Some(Body(Chest)),
        _ => node.parent(),
    }
}

pub fn rest(node: Node, p: &Proportions) -> Vec3 {
    match node {
        Secondary(Spring::Back) => Vec3::new(0.0, 0.32, -0.15),
        Secondary(Spring::HeadGear) => Vec3::new(0.0, 0.17, -0.02),
        _ => rig::rest(node, p),
    }
}

pub fn spec(tier: u8, branch: u8) -> FigureSpec {
    let p = plan(tier);
    let f = Form::of(tier, branch);
    let mut v = Vec::new();
    // B forms darken toward graphite and crimson (owner reference, Sentença Final).
    let plate = if f.b { GRAPHITE } else { IVORY };
    body::humanoid(
        &mut v,
        &p,
        &body::Look {
            skin: SKIN,
            cloth: if f.b { CHARCOAL } else { GRAPHITE },
            legs: GRAPHITE,
            boots: plate,
        },
    );
    helm(&mut v, &p, f);
    armor(&mut v, &p, f, plate);
    lance::build(&mut v, f);
    FigureSpec { plan: p, parts: v }
}

fn helm(v: &mut Vec<PartSpec>, p: &Proportions, f: Form) {
    let h = p.head;
    let metal = GOLD;
    body::beard(v, p, IVORY, h * 1.7);
    v.push(part(
        Body(Head),
        Bevel(h * 1.3, h * 0.25, h * 0.32, 0.01),
        IVORY,
        [0.0, -h * 0.32, h * 0.78],
    ));
    // A geometric closed helm with cheek guards and a T-visor.
    v.extend([
        part(
            Body(Head),
            Bevel(h * 2.05, h * 1.15, h * 2.05, 0.02),
            metal,
            [0.0, h * 0.42, -h * 0.05],
        ),
        part(
            Body(Head),
            Bevel(h * 0.5, h * 0.9, h * 0.25, 0.01),
            metal,
            [h * 0.78, -h * 0.05, h * 0.62],
        ),
        part(
            Body(Head),
            Bevel(h * 0.5, h * 0.9, h * 0.25, 0.01),
            metal,
            [-h * 0.78, -h * 0.05, h * 0.62],
        ),
        part(
            Body(Head),
            Bevel(h * 0.22, h * 0.6, h * 0.12, 0.008),
            metal,
            [0.0, h * 0.25, h * 1.0],
        ),
    ]);
    body::glowing_eyes(v, p, if f.b { RED_CLOTH } else { STORM_BLUE });
    // Swept horns; A gains a tall central fin, B longer forward horns.
    for s in [-1.0f32, 1.0] {
        let len = if f.b { 2.0 } else { 1.5 };
        v.push(
            part(
                Body(Head),
                Cone(h * 0.2, h * len),
                metal,
                [s * h * 0.95, h * 0.95, -h * 0.05],
            )
            .rot([-25.0, 0.0, -s * 22.0]),
        );
    }
    if f.a {
        let fin = if f.t2 { 1.7 } else { 1.2 };
        v.push(
            part(
                Secondary(Spring::HeadGear),
                Prism(h * 0.16, h * fin),
                metal,
                [0.0, h * fin * 0.45, 0.0],
            )
            .scale([0.6, 1.0, 1.6]),
        );
    }
    if f.t2 {
        v.push(
            part(
                Body(Head),
                Prism(h * 0.12, h * 0.4),
                if f.b { RED_CLOTH } else { STORM_BLUE },
                [0.0, h * 0.8, h * 0.95],
            )
            .glow(),
        );
    }
}

fn armor(v: &mut Vec<PartSpec>, p: &Proportions, f: Form, plate: Color) {
    let t = p.torso;
    let accent = if f.b { CRIMSON } else { NAVY };
    // Cuirass in two tones with a gold rupture star, belt and armored skirt.
    v.extend([
        part(
            Body(Chest),
            Bevel(t.x * 0.92, t.y * 0.62, 0.06, 0.012),
            plate,
            [0.0, t.y * 0.6, t.z * 0.5],
        ),
        part(
            Body(Chest),
            Bevel(t.x * 0.30, t.y * 0.42, 0.065, 0.01),
            GOLD,
            [0.0, t.y * 0.6, t.z * 0.52],
        ),
        part(
            Body(Chest),
            Prism(0.03, 0.10),
            if f.b { RED_CLOTH } else { STORM_BLUE },
            [0.0, t.y * 0.66, t.z * 0.5 + 0.04],
        )
        .glow(),
        part(
            Body(Hips),
            Bevel(t.x * 1.05, 0.07, t.z * 1.1, 0.012),
            GOLD,
            [0.0, 0.05, 0.0],
        ),
        part(
            Body(Hips),
            Bevel(t.x * 0.38, 0.24, 0.04, 0.01),
            accent,
            [0.0, -0.11, t.z * 0.55],
        ),
        part(
            Body(Hips),
            Bevel(t.x * 0.38, 0.22, 0.04, 0.01),
            plate,
            [t.x * 0.4, -0.10, t.z * 0.3],
        )
        .rot([0.0, 25.0, 0.0]),
        part(
            Body(Hips),
            Bevel(t.x * 0.38, 0.22, 0.04, 0.01),
            plate,
            [-t.x * 0.4, -0.10, t.z * 0.3],
        )
        .rot([0.0, -25.0, 0.0]),
    ]);
    // Angular pauldrons: one blade per form level, upswept on the B branch.
    let layers = 1 + usize::from(f.a || f.b) + usize::from(f.t2);
    for (upper, s) in [(UpperArmL, 1.0f32), (UpperArmR, -1.0)] {
        for i in 0..layers {
            let k = i as f32;
            v.push(
                part(
                    Body(upper),
                    Bevel(0.24 - 0.025 * k, 0.065, 0.22 - 0.02 * k, 0.012),
                    if i == 0 { GOLD } else { plate },
                    [s * (0.03 + 0.02 * k), 0.06 - 0.05 * k, 0.0],
                )
                .rot([0.0, 0.0, s * (if f.b { 30.0 } else { 18.0 } + 8.0 * k)]),
            );
        }
    }
    body::bracers(v, p, GOLD);
    body::greaves(v, GOLD);
    // A short split cape; the B branch wears crimson.
    body::cape(v, accent, 0.40, if f.t2 { 0.92 } else { 0.80 });
    v.push(part(
        Secondary(Spring::Back),
        Bevel(0.42, 0.05, 0.05, 0.01),
        GOLD,
        [0.0, 0.0, -0.02],
    ));
    if f.t2 {
        for shin in [ShinL, ShinR] {
            v.push(part(
                Body(shin),
                Bevel(0.13, 0.16, 0.07, 0.01),
                GOLD,
                [0.0, -0.10, 0.06],
            ));
        }
        v.push(part(
            Body(Chest),
            Bevel(t.x * 0.7, 0.06, t.z * 1.05, 0.01),
            GOLD,
            [0.0, t.y * 0.98, 0.0],
        ));
    }
}
