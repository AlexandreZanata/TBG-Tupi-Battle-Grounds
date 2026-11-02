//! Volthar's storm-god anatomy, blue-and-gold armor and rune hammer across five forms.

use super::hammer;
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

/// Gods share the Colossus monument scale (GD-01): a slightly smaller,
/// athletic frame so Colossus stays the largest of the three.
pub fn plan(tier: u8) -> Proportions {
    let mut p = body::plan(true, 1.66, tier);
    p.torso = Vec3::new(0.50, 0.36, 0.26);
    p.arm = 0.31;
    p.leg = 0.46;
    p.head = 0.145;
    p
}

pub fn parent(node: Node) -> Option<Node> {
    match node {
        // The hammer owns its frame; the wrists solve onto its grip (no forearm clipping).
        Prop(Socket::WeaponR) => Some(Body(Chest)),
        // Idle sparks orbit the hammer head.
        Prop(Socket::OffhandL) => Some(Prop(Socket::WeaponR)),
        _ => node.parent(),
    }
}

pub fn rest(node: Node, p: &Proportions) -> Vec3 {
    match node {
        Prop(Socket::OffhandL) => Vec3::new(0.0, 0.0, hammer::HEAD_Z),
        Secondary(Spring::Back) => Vec3::new(0.0, 0.31, -0.15),
        _ => rig::rest(node, p),
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
            skin: SKIN,
            cloth: ROYAL_BLUE,
            legs: NAVY,
            boots: if f.b { GRAPHITE } else { NAVY },
        },
    );
    head(&mut v, &p, f);
    armor(&mut v, &p, f);
    cape(&mut v, f);
    hammer::build(&mut v, f);
    FigureSpec { plan: p, parts: v }
}

fn head(v: &mut Vec<PartSpec>, p: &Proportions, f: Form) {
    let h = p.head;
    // Full white hair and a broad two-layer beard frame the glowing eyes.
    v.extend([
        part(
            Body(Head),
            Ball(h * 1.08),
            IVORY,
            [0.0, h * 0.18, -h * 0.28],
        )
        .scale([1.05, 1.0, 0.95]),
        part(
            Body(Head),
            Bevel(h * 1.5, h * 0.9, h * 0.5, 0.012),
            IVORY,
            [0.0, -h * 0.35, -h * 0.75],
        ),
        part(
            Body(Head),
            Bevel(h * 1.2, h * 0.24, h * 0.30, 0.01),
            IVORY,
            [0.0, -h * 0.30, h * 0.80],
        ),
    ]);
    body::beard(v, p, IVORY, h * if f.t2 { 2.6 } else { 2.2 });
    v.push(
        part(
            Body(Head),
            Cone(h * 0.55, h * 1.6),
            IVORY,
            [0.0, -h * 0.95, h * 0.62],
        )
        .rot([172.0, 0.0, 0.0])
        .scale([1.25, 1.0, 0.7]),
    );
    body::glowing_eyes(v, p, STORM_BLUE);
    // Crown band with prongs; A grows a celestial halo, B a war crest.
    v.push(part(
        Body(Head),
        Bevel(h * 2.1, h * 0.32, h * 1.9, 0.012),
        GOLD,
        [0.0, h * 0.62, 0.0],
    ));
    let prongs: &[(f32, f32)] = if f.a && f.t2 {
        &[(-0.6, 1.1), (-0.3, 1.5), (0.0, 1.9), (0.3, 1.5), (0.6, 1.1)]
    } else {
        &[(-0.45, 1.0), (0.0, 1.35), (0.45, 1.0)]
    };
    for &(x, len) in prongs {
        v.push(
            part(
                Body(Head),
                Prism(h * 0.18, h * len),
                GOLD,
                [x * h, h * (0.75 + len * 0.45), h * 0.55],
            )
            .scale([0.7, 1.0, 0.4]),
        );
    }
    for s in [-1.0f32, 1.0] {
        // Side horns sweep back and up, longer and darker on the wrath branch.
        let len = if f.b { 1.9 } else { 1.4 };
        v.push(
            part(
                Body(Head),
                Cone(h * 0.2, h * len),
                if f.b { BRASS } else { GOLD },
                [s * h * 1.0, h * 0.95, -h * 0.1],
            )
            .rot([-20.0, 0.0, -s * 28.0]),
        );
    }
    if f.a && f.t2 {
        v.push(
            part(
                Body(Head),
                Ring(0.012, h * 1.9, 360.0),
                STORM_BLUE,
                [0.0, h * 1.35, -h * 0.9],
            )
            .rot([80.0, 0.0, 0.0])
            .glow(),
        );
    }
}

fn armor(v: &mut Vec<PartSpec>, p: &Proportions, f: Form) {
    let t = p.torso;
    let trim = if f.b { BRASS } else { GOLD };
    // Breastplate with a lightning emblem, belt and plated skirt.
    v.extend([
        part(
            Body(Chest),
            Bevel(t.x * 0.9, t.y * 0.55, 0.06, 0.012),
            trim,
            [0.0, t.y * 0.62, t.z * 0.5],
        ),
        part(
            Body(Chest),
            Prism(0.035, 0.16),
            STORM_BLUE,
            [0.0, t.y * 0.62, t.z * 0.5 + 0.035],
        )
        .rot([0.0, 0.0, 28.0])
        .glow(),
        part(
            Body(Hips),
            Bevel(t.x * 1.05, 0.08, t.z * 1.15, 0.012),
            trim,
            [0.0, 0.05, 0.0],
        ),
        part(
            Body(Hips),
            Bevel(0.10, 0.07, 0.03, 0.008),
            STORM_BLUE,
            [0.0, 0.05, t.z * 0.6],
        )
        .glow(),
        part(
            Body(Hips),
            Bevel(t.x * 0.42, 0.22, 0.04, 0.01),
            NAVY,
            [0.0, -0.10, t.z * 0.55],
        ),
        part(
            Body(Hips),
            Bevel(0.05, 0.22, 0.045, 0.008),
            trim,
            [0.0, -0.10, t.z * 0.58],
        ),
    ]);
    // Angular pauldrons in layers; every tier adds a plate, B adds spikes.
    let layers = 1 + usize::from(f.a || f.b) + usize::from(f.t2);
    for (upper, s) in [(UpperArmL, 1.0f32), (UpperArmR, -1.0)] {
        for i in 0..layers {
            let k = i as f32;
            v.push(
                part(
                    Body(upper),
                    Bevel(0.26 - 0.03 * k, 0.07, 0.24 - 0.02 * k, 0.012),
                    if i == 0 { trim } else { ROYAL_BLUE },
                    [s * (0.03 + 0.02 * k), 0.06 - 0.055 * k, 0.0],
                )
                .rot([0.0, 0.0, s * (18.0 + 8.0 * k)]),
            );
        }
        if f.b {
            v.push(
                part(
                    Body(upper),
                    Cone(0.04, 0.16 + 0.06 * f32::from(u8::from(f.t2))),
                    trim,
                    [s * 0.10, 0.12, 0.0],
                )
                .rot([0.0, 0.0, -s * 35.0]),
            );
        }
    }
    body::bracers(v, p, trim);
    for fore in [ForearmL, ForearmR] {
        v.push(
            part(
                Body(fore),
                Bevel(0.13, 0.03, 0.13, 0.008),
                STORM_BLUE,
                [0.0, -p.arm * 0.32, 0.0],
            )
            .glow(),
        );
    }
    body::greaves(v, trim);
    if f.t2 {
        // Final forms: gilded collar and greaves extended over the knees.
        v.push(part(
            Body(Chest),
            Bevel(t.x * 0.75, 0.06, t.z * 1.05, 0.01),
            trim,
            [0.0, t.y * 0.98, 0.0],
        ));
        for shin in [ShinL, ShinR] {
            v.push(part(
                Body(shin),
                Bevel(0.13, 0.16, 0.07, 0.01),
                trim,
                [0.0, -0.10, 0.06],
            ));
        }
    }
}

fn cape(v: &mut Vec<PartSpec>, f: Form) {
    let color = if f.b { GRAPHITE } else { NAVY };
    let length = if f.t2 { 0.95 } else { 0.85 };
    body::cape(v, color, 0.46, length);
    v.push(part(
        Secondary(Spring::Back),
        Bevel(0.48, 0.05, 0.05, 0.01),
        GOLD,
        [0.0, 0.0, -0.02],
    ));
    if f.a {
        // Sovereign's cape lining shows a gold storm border.
        v.push(part(
            Secondary(Spring::Back),
            Box(0.47, 0.035, 0.022),
            GOLD,
            [0.0, -length * 0.85, -0.035],
        ));
    }
}
