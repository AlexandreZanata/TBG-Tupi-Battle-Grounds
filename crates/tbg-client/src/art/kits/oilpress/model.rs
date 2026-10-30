//! Oilpress figure: the Ironworks' walking press, an ogre sealed in a
//! riveted boiler with a smokestack, piston arms and iron press plates for
//! fists. Branch A (Fortress Engine) bolts on hull plates, then a bulwark,
//! spiked anchor feet and a lit furnace; branch B (Tar Bastion) carries
//! leaking oil barrels, then a tar tank with tar-coated plates.

use bevy::prelude::*;

use crate::art::{
    figure::{part, FigureSpec, PartSpec},
    kits::common::{body, gear, Form},
    mesh::Shape::*,
    palette::*,
    rig::{Joint::*, Node::Body, Node::Secondary, Proportions, Spring},
};

const HOT: Color = Color::srgb(1.0, 0.55, 0.18);
const OIL: Color = Color::srgb(0.07, 0.06, 0.08);
const AMBER: Color = Color::srgb(0.95, 0.62, 0.12);

pub fn plan(tier: u8) -> Proportions {
    let mut p = body::plan(true, 0.95, tier);
    p.torso = Vec3::new(p.torso.x * 1.15, p.torso.y * 1.05, p.torso.z * 1.35);
    p.head *= 0.8;
    p.arm *= 1.15;
    p
}

/// Where the press plate lands (right hand space).
pub fn tip() -> Vec3 {
    Vec3::new(0.0, -0.14, 0.0)
}

pub fn spec(tier: u8, branch: u8) -> FigureSpec {
    let f = Form::of(tier, branch);
    let p = plan(tier);
    let mut v = Vec::with_capacity(150);
    let look = body::Look {
        skin: OGRE_SKIN,
        cloth: GUNMETAL,
        legs: DARK_LEATHER,
        boots: IRON,
    };
    body::humanoid(&mut v, &p, &look);
    boiler(&mut v, &p, f);
    head(&mut v, &p, f);
    arms(&mut v, &p, f);
    legs(&mut v, f);
    back(&mut v, f);
    FigureSpec { plan: p, parts: v }
}

fn boiler(v: &mut Vec<PartSpec>, p: &Proportions, f: Form) {
    let t = p.torso;
    let r = t.x * 0.56;
    let depth = t.z / t.x;
    v.push(
        part(
            Body(Chest),
            Cyl(r, t.y * 0.95),
            GUNMETAL,
            [0.0, t.y * 0.5, 0.0],
        )
        .scale([1.0, 1.0, depth * 1.1]),
    );
    for y in [0.12, 0.88] {
        v.push(
            part(
                Body(Chest),
                Cyl(r * 1.03, 0.045),
                BRASS,
                [0.0, t.y * y, 0.0],
            )
            .scale([1.0, 1.0, depth * 1.1]),
        );
    }
    let front = r * depth * 1.1;
    // Pressure gauge and the furnace grille.
    v.extend([
        part(
            Body(Chest),
            Cyl(0.06, 0.03),
            BRASS,
            [t.x * 0.22, t.y * 0.72, front],
        )
        .rot([90.0, 0.0, 0.0]),
        part(
            Body(Chest),
            Cyl(0.048, 0.035),
            LINEN,
            [t.x * 0.22, t.y * 0.72, front + 0.002],
        )
        .rot([90.0, 0.0, 0.0]),
        part(
            Body(Chest),
            Box(0.008, 0.045, 0.01),
            DARK,
            [t.x * 0.22, t.y * 0.73, front + 0.02],
        )
        .rot([0.0, 0.0, -35.0]),
        part(
            Body(Chest),
            Box(t.x * 0.42, t.y * 0.26, 0.04),
            DARK,
            [-t.x * 0.06, t.y * 0.36, front - 0.01],
        ),
    ]);
    let lit = f.t2 && f.a;
    for i in 0..3 {
        let y = t.y * (0.29 + 0.07 * i as f32);
        let slit = part(
            Body(Chest),
            Box(t.x * 0.36, 0.018, 0.02),
            if lit { HOT } else { RUST },
            [-t.x * 0.06, y, front + 0.01],
        );
        v.push(if lit { slit.glow() } else { slit });
    }
    for a in [-50.0f32, -25.0, 25.0, 50.0] {
        let s = a.to_radians();
        v.push(part(
            Body(Chest),
            Ball(0.018),
            BRASS,
            [s.sin() * r, t.y * 0.12, s.cos() * front],
        ));
    }
    if f.a {
        // Reinforced Hull: a riveted plate over the boiler's front.
        v.push(
            part(
                Body(Chest),
                Box(t.x * 0.8, t.y * 0.3, 0.05),
                IRON,
                [0.0, t.y * 0.92, front * 0.8],
            )
            .rot([-12.0, 0.0, 0.0]),
        );
    }
    if f.a && f.t2 {
        // Immovable Mass: a bulwark on the hips with the team's stripe.
        v.extend([
            part(
                Body(Hips),
                Box(t.x * 1.05, 0.22, 0.06),
                IRON,
                [0.0, -0.06, t.z * 0.7],
            ),
            part(
                Body(Hips),
                Box(t.x * 0.2, 0.2, 0.065),
                ALLY,
                [0.0, -0.06, t.z * 0.7],
            ),
        ]);
    }
    if f.b {
        // Black oil runs down the boiler.
        for (x, y, len) in [(-0.12, 0.6, 1.0), (0.05, 0.7, 0.7), (0.16, 0.5, 1.2)] {
            v.push(
                part(
                    Body(Chest),
                    Ball(0.03),
                    OIL,
                    [t.x * x, t.y * y, front * 0.98],
                )
                .scale([0.8, 2.2 * len, 0.6]),
            );
        }
    }
}

fn head(v: &mut Vec<PartSpec>, p: &Proportions, f: Form) {
    let h = p.head;
    v.extend([
        part(Body(Head), Ball(h * 1.15), IRON, [0.0, h * 0.15, -h * 0.05]).scale([1.0, 0.9, 1.0]),
        part(
            Body(Head),
            Box(h * 1.3, h * 0.2, 0.04),
            DARK,
            [0.0, h * 0.15, h * 1.0],
        ),
        part(Body(Head), Cyl(h * 0.3, 0.04), BRASS, [0.0, h * 1.1, 0.0]),
    ]);
    for s in [-1.0f32, 1.0] {
        let eye = part(
            Body(Head),
            Box(0.035, 0.022, 0.02),
            HOT,
            [s * h * 0.35, h * 0.15, h * 1.03],
        );
        v.push(if f.t2 { eye.glow() } else { eye });
    }
}

fn arms(v: &mut Vec<PartSpec>, p: &Proportions, f: Form) {
    let a = p.arm;
    let plate = if f.b && f.t2 { OIL } else { IRON };
    for (upper, fore, hand, s) in [
        (UpperArmL, ForearmL, HandL, 1.0f32),
        (UpperArmR, ForearmR, HandR, -1.0),
    ] {
        v.extend([
            part(Body(upper), Ball(0.13), GUNMETAL, [0.02 * s, 0.0, 0.0]).scale([1.2, 0.9, 1.1]),
            gear::strut(
                Body(upper),
                [0.07 * s, -0.02, -0.06],
                [0.07 * s, -a * 0.95, -0.05],
                0.02,
                BRASS,
            ),
            part(
                Body(fore),
                Cyl(0.1, a * 0.75),
                GUNMETAL,
                [0.0, -a * 0.45, 0.0],
            ),
            part(Body(fore), Cyl(0.105, 0.03), BRASS, [0.0, -a * 0.2, 0.0]),
            part(Body(hand), Cyl(0.06, 0.08), IRON, [0.0, -0.03, 0.0]),
            part(Body(hand), Box(0.24, 0.07, 0.24), plate, [0.0, -0.1, 0.0]),
        ]);
        if f.a {
            v.push(
                part(Body(upper), Box(0.2, 0.06, 0.2), IRON, [0.03 * s, 0.1, 0.0]).rot([
                    0.0,
                    0.0,
                    18.0 * s,
                ]),
            );
        }
        if f.a && f.t2 {
            for z in [-0.06, 0.06] {
                v.push(
                    part(Body(upper), Cone(0.03, 0.12), STEEL, [0.08 * s, 0.17, z]).rot([
                        0.0,
                        0.0,
                        -30.0 * s,
                    ]),
                );
            }
        }
        if f.b && f.t2 {
            // Tar drips from the plates.
            v.push(
                part(Body(hand), Ball(0.04), OIL, [0.06 * s, -0.16, 0.05]).scale([0.8, 1.6, 0.8]),
            );
        }
    }
}

fn legs(v: &mut Vec<PartSpec>, f: Form) {
    for (shin, foot) in [(ShinL, FootL), (ShinR, FootR)] {
        v.extend([
            part(Body(shin), Box(0.16, 0.16, 0.16), IRON, [0.0, -0.06, 0.02]),
            part(Body(foot), Box(0.22, 0.07, 0.3), IRON, [0.0, -0.02, 0.05]),
        ]);
        if f.a && f.t2 {
            // Anchor spikes on the foot plates.
            for x in [-0.08, 0.08] {
                v.push(
                    part(Body(foot), Cone(0.025, 0.08), STEEL, [x, -0.05, 0.18])
                        .rot([180.0, 0.0, 0.0]),
                );
            }
        }
    }
}

/// The smokestack, the oil barrels or the tar tank.
fn back(v: &mut Vec<PartSpec>, f: Form) {
    let b = Secondary(Spring::Back);
    v.extend([
        part(b, Cyl(0.065, 0.36), IRON, [0.14, 0.24, -0.04]),
        part(b, Frustum(0.1, 0.07, 0.07), DARK, [0.14, 0.45, -0.04]),
    ]);
    if f.a && f.t2 {
        v.push(part(b, Cyl(0.07, 0.02), HOT, [0.14, 0.49, -0.04]).glow());
    }
    if f.b && !f.t2 {
        for x in [-0.1, 0.08] {
            v.extend([
                part(b, Cyl(0.09, 0.22), RUST, [x - 0.06, -0.02, -0.12]),
                part(b, Cyl(0.095, 0.02), IRON, [x - 0.06, 0.06, -0.12]),
                part(b, Ball(0.025), OIL, [x - 0.06, -0.14, -0.12]).scale([1.0, 1.8, 1.0]),
            ]);
        }
    }
    if f.b && f.t2 {
        v.extend([
            part(b, Cyl(0.16, 0.36), OIL, [-0.08, 0.02, -0.16]),
            part(b, Cyl(0.165, 0.03), BRASS, [-0.08, 0.15, -0.16]),
            part(b, Box(0.05, 0.22, 0.02), AMBER, [-0.08, 0.0, -0.33]).glow(),
            gear::strut(b, [-0.08, 0.2, -0.16], [-0.3, 0.28, 0.06], 0.025, BRASS),
        ]);
    }
}
