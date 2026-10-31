//! Rotcap figure: a squat mushroom-folk spitter, pale stem body, a broad
//! spotted cap with gills, a ring collar and a round spitting mouth.
//! Branch A (Concentrated) darkens the cap with glowing toxin spots, then
//! drips venom from its rim; branch B (Contamination) sprouts caps on the
//! shoulders, then a cluster of caps and glowing spore pods.

use bevy::prelude::*;

use crate::art::{
    figure::{part, FigureSpec, PartSpec},
    kits::common::{body, gear, Form},
    mesh::Shape::*,
    palette::*,
    rig::{Joint::*, Node::Body, Node::Secondary, Proportions, Spring},
};

const STEM: Color = Color::srgb(0.9, 0.86, 0.74);
const STEM_DARK: Color = Color::srgb(0.74, 0.68, 0.54);
const GILLS: Color = Color::srgb(0.62, 0.5, 0.4);
const SPOT: Color = Color::srgb(0.95, 0.9, 0.78);
const TOXIN: Color = Color::srgb(0.55, 0.95, 0.2);
const VENOM_CAP: Color = Color::srgb(0.42, 0.2, 0.36);
const SPORE: Color = Color::srgb(0.85, 0.95, 0.4);

pub fn plan(tier: u8) -> Proportions {
    let mut p = body::plan(false, 0.95, tier);
    p.leg *= 0.8;
    p.torso.x *= 1.1;
    p
}

/// The glob leaves the mouth (head space).
pub fn mouth(p: &Proportions) -> Vec3 {
    Vec3::new(0.0, -p.head * 0.38, p.head * 1.02)
}

pub fn spec(tier: u8, branch: u8) -> FigureSpec {
    let f = Form::of(tier, branch);
    let p = plan(tier);
    let mut v = Vec::with_capacity(120);
    let look = body::Look {
        skin: STEM,
        cloth: STEM_DARK,
        legs: STEM_DARK,
        boots: BARK,
    };
    body::humanoid(&mut v, &p, &look);
    let t = p.torso;
    v.extend([
        // The mushroom ring as a collar, and the team's sash.
        part(
            Body(Chest),
            Frustum(t.x * 0.62, t.x * 0.45, 0.08),
            SPOT,
            [0.0, t.y * 0.88, 0.0],
        )
        .scale([1.0, 1.0, 0.8]),
        part(
            Body(Chest),
            Box(t.x * 1.0, 0.06, t.z * 0.8),
            ALLY,
            [0.0, t.y * 0.25, 0.02],
        )
        .rot([0.0, 0.0, 12.0]),
    ]);
    face(&mut v, &p, f);
    cap(&mut v, &p, f);
    if f.a {
        for x in [-0.12, 0.12] {
            v.extend(gear::flask(Body(Hips), [x, -0.03, t.z * 0.6], TOXIN, 0.8));
        }
    }
    if f.b {
        for arm in [UpperArmL, UpperArmR] {
            v.extend([
                part(Body(arm), Cyl(0.025, 0.06), STEM, [0.0, 0.07, 0.0]),
                part(Body(arm), Ball(0.075), MUSHROOM, [0.0, 0.11, 0.0]).scale([1.0, 0.5, 1.0]),
            ]);
        }
    }
    FigureSpec { plan: p, parts: v }
}

fn face(v: &mut Vec<PartSpec>, p: &Proportions, f: Form) {
    let h = p.head;
    let m = mouth(p);
    v.extend([
        part(Body(Head), Ball(h * 0.24), DARK, m.to_array()).scale([1.0, 0.8, 0.5]),
        part(
            Body(Head),
            Ring(0.012, h * 0.24, 360.0),
            STEM_DARK,
            m.to_array(),
        )
        .rot([90.0, 0.0, 0.0]),
    ]);
    if f.a && f.t2 {
        body::glowing_eyes(v, p, TOXIN);
    }
}

fn cap(v: &mut Vec<PartSpec>, p: &Proportions, f: Form) {
    let h = p.head;
    let r = h * if f.a && f.t2 { 3.0 } else { 2.6 };
    let color = match (f.a, f.t2) {
        (true, true) => VENOM_CAP,
        (true, false) => Color::srgb(0.6, 0.24, 0.24),
        _ => MUSHROOM,
    };
    let y = h * 0.75;
    v.extend([
        part(Body(Head), Ball(r), color, [0.0, y, 0.0]).scale([1.0, 0.5, 1.0]),
        part(
            Body(Head),
            Cyl(r * 0.96, 0.02),
            GILLS,
            [0.0, y - 0.005, 0.0],
        ),
    ]);
    let glow = f.a;
    for (a, d, s) in [
        (20.0f32, 0.55, 1.0),
        (140.0, 0.6, 0.8),
        (250.0, 0.5, 0.9),
        (320.0, 0.35, 0.7),
        (90.0, 0.2, 0.8),
    ] {
        let ang = a.to_radians();
        let at = [
            ang.sin() * r * d,
            y + r * 0.5 * (1.0 - d * d).sqrt(),
            ang.cos() * r * d,
        ];
        let spot = part(
            Body(Head),
            Ball(r * 0.16 * s),
            if glow { TOXIN } else { SPOT },
            at,
        )
        .scale([1.0, 0.45, 1.0]);
        v.push(if glow { spot.glow() } else { spot });
    }
    if f.a && f.t2 {
        // Venom drips from the rim.
        for a in [30.0f32, 100.0, 200.0, 290.0] {
            let ang = a.to_radians();
            v.push(
                part(
                    Body(Head),
                    Cone(0.018, 0.08),
                    TOXIN,
                    [ang.sin() * r * 0.92, y - 0.04, ang.cos() * r * 0.92],
                )
                .rot([180.0, 0.0, 0.0])
                .glow(),
            );
        }
    }
    if f.b && f.t2 {
        // Toxic Rain: a cluster of caps and spore pods on the back.
        for (x, z, s) in [(0.12, -0.08, 0.6), (-0.14, -0.05, 0.5), (0.0, 0.1, 0.45)] {
            v.push(
                part(Body(Head), Ball(r * s), MUSHROOM, [x, y + r * 0.42, z])
                    .scale([1.0, 0.5, 1.0]),
            );
        }
        let b = Secondary(Spring::Back);
        for (x, y) in [(-0.08, 0.05), (0.08, 0.0), (0.0, 0.16)] {
            v.push(part(b, Ball(0.05), SPORE, [x, y, -0.06]).glow());
        }
    }
}
