//! Stone anatomy, celestial mantle and five distinctive carved-obelisk forms.
//!
//! GD-04 refinement: carved stone plates, a readable face and hands, and
//! structural Evolutions. Branch A (damage, range, splash) follows the
//! offensive "Pillar of Wrath / Destroyer Obelisk" direction with gold blades;
//! branch B (+HP) the defensive "Marble Bastion / Divine Fortress" direction
//! and keeps its dark charcoal stone (owner rule). Scale and the obelisk core
//! are unchanged, so GD-02 clearance and the GD-03 strike tip still hold.
use crate::art::{
    figure::{part, FigureSpec, PartSpec},
    kits::common::{body, Form},
    mesh::Shape::*,
    palette::*,
    rig::{
        Joint::*,
        Node::{Body, Prop},
        Proportions, Socket,
    },
};
use bevy::prelude::Color;

pub(super) fn plan(t: u8) -> Proportions {
    let mut p = body::plan(true, 1.85, t);
    p.torso.x = 0.52;
    p.arm = 0.30;
    p.leg = 0.46;
    p.head = 0.15;
    p
}

#[derive(Clone, Copy)]
struct Stone {
    main: Color,
    shade: Color,
    metal: Color,
    glow: Color,
}

fn stone(f: Form) -> Stone {
    if f.b {
        Stone {
            main: CHARCOAL,
            shade: GRAPHITE,
            metal: GOLD,
            glow: AMBER,
        }
    } else {
        Stone {
            main: IVORY,
            shade: ROCK,
            metal: GOLD,
            glow: AMBER,
        }
    }
}

pub(super) fn spec(t: u8, b: u8) -> FigureSpec {
    let p = plan(t);
    let f = Form::of(t, b);
    let s = stone(f);
    let mut v = Vec::new();
    body::humanoid(
        &mut v,
        &p,
        &body::Look {
            skin: s.main,
            cloth: s.main,
            legs: FIELD_SLATE,
            boots: s.main,
        },
    );
    v.retain(|a| !matches!(a.node, Body(Head)));
    face(&mut v, s, f);
    torso(&mut v, &p, s);
    limbs(&mut v, &p, s);
    if f.a {
        wrath(&mut v, s, f);
    }
    if f.b {
        bastion(&mut v, s, f);
    }
    obelisk(&mut v, s, f);
    FigureSpec { plan: p, parts: v }
}

fn face(v: &mut Vec<PartSpec>, s: Stone, f: Form) {
    let head = Body(Head);
    v.extend([
        part(
            head,
            Bevel(0.25, 0.29, 0.23, 0.035),
            s.main,
            [0.0, 0.0, 0.0],
        ),
        // Brow ridge, nose and jaw give a simple carved expression.
        part(
            head,
            Bevel(0.23, 0.045, 0.06, 0.012),
            s.shade,
            [0.0, 0.065, 0.10],
        ),
        part(head, Prism(0.022, 0.07), s.main, [0.0, 0.0, 0.122]).scale([1.0, 1.0, 0.8]),
        part(
            head,
            Bevel(0.19, 0.07, 0.07, 0.015),
            s.main,
            [0.0, -0.11, 0.08],
        ),
        part(head, Prism(0.035, 0.11), s.metal, [0.0, -0.14, 0.10]).scale([1.0, 1.0, 0.5]),
    ]);
    for side in [-1.0f32, 1.0] {
        v.push(
            part(
                head,
                Bevel(0.075, 0.035, 0.028, 0.008),
                s.glow,
                [side * 0.065, 0.02, 0.122],
            )
            .glow(),
        );
        v.push(
            part(
                head,
                Prism(0.065, 0.20),
                s.metal,
                [side * 0.16, 0.09, -0.02],
            )
            .scale([0.6, 1.0, 0.5])
            .rot([0.0, 0.0, -side * 20.0]),
        );
    }
    if f.a || f.b {
        // A gilded diadem on every Evolution; the crown differs per branch below.
        v.push(part(
            head,
            Bevel(0.30, 0.065, 0.26, 0.015),
            s.metal,
            [0.0, 0.15, 0.0],
        ));
    }
}

fn torso(v: &mut Vec<PartSpec>, p: &Proportions, s: Stone) {
    let t = p.torso;
    // Carved pectoral and abdominal plates, a gilded collar and a girdle.
    for side in [-1.0f32, 1.0] {
        v.push(
            part(
                Body(Chest),
                Bevel(0.20, 0.11, 0.11, 0.025),
                s.metal,
                [side * 0.13, 0.25, 0.10],
            )
            .rot([0.0, 0.0, -side * 17.0]),
        );
        v.push(
            part(
                Body(Chest),
                Bevel(0.19, 0.15, 0.05, 0.02),
                s.main,
                [side * 0.12, 0.16, t.z * 0.5],
            )
            .rot([0.0, side * 8.0, 0.0]),
        );
        v.push(
            part(
                Body(Hips),
                Prism(0.12, 0.30),
                FIELD_SLATE,
                [side * 0.12, -0.15, 0.08],
            )
            .scale([1.0, 1.0, 0.35])
            .rot([0.0, 0.0, side * 8.0]),
        );
    }
    for (i, y) in [0.05f32, -0.02].into_iter().enumerate() {
        v.push(part(
            Body(Spine),
            Bevel(0.26 - 0.03 * i as f32, 0.06, 0.05, 0.012),
            s.shade,
            [0.0, y + 0.08, t.z * 0.48],
        ));
    }
    v.extend([
        part(
            Body(Chest),
            Bevel(t.x * 0.62, 0.05, t.z * 1.05, 0.012),
            s.metal,
            [0.0, t.y * 0.97, 0.0],
        ),
        part(
            Body(Hips),
            Bevel(t.x * 1.04, 0.07, t.z * 1.12, 0.014),
            s.metal,
            [0.0, 0.05, 0.0],
        ),
        part(
            Body(Hips),
            Bevel(0.08, 0.08, 0.03, 0.01),
            s.glow,
            [0.0, 0.05, t.z * 0.58],
        )
        .glow(),
    ]);
}

fn limbs(v: &mut Vec<PartSpec>, p: &Proportions, s: Stone) {
    for (arm, fore, hand) in [(UpperArmL, ForearmL, HandL), (UpperArmR, ForearmR, HandR)] {
        v.push(part(Body(arm), Ball(0.135), s.main, [0.0, -0.01, 0.0]).scale([0.85, 1.0, 1.0]));
        v.push(part(
            Body(arm),
            Frustum(0.12, 0.085, 0.28),
            s.main,
            [0.0, -0.14, 0.0],
        ));
        v.push(part(
            Body(arm),
            Bevel(0.15, 0.035, 0.15, 0.01),
            s.metal,
            [0.0, -0.27, 0.0],
        ));
        v.push(part(
            Body(fore),
            Bevel(0.15, 0.20, 0.17, 0.025),
            s.main,
            [0.0, -0.12, 0.0],
        ));
        v.push(part(
            Body(fore),
            Bevel(0.17, 0.045, 0.18, 0.012),
            s.metal,
            [0.0, -0.17, 0.0],
        ));
        for finger in 0..4 {
            v.push(part(
                Body(hand),
                Bevel(0.031, 0.065, 0.055, 0.008),
                s.main,
                [-0.055 + finger as f32 * 0.036, -0.035, 0.055],
            ));
        }
        // A thumb and knuckle band make the large hands read as hands.
        let thumb = if matches!(hand, HandL) { 0.075 } else { -0.075 };
        v.push(part(
            Body(hand),
            Bevel(0.03, 0.05, 0.045, 0.008),
            s.main,
            [thumb, -0.01, 0.03],
        ));
        v.push(part(
            Body(hand),
            Bevel(0.15, 0.025, 0.06, 0.006),
            s.metal,
            [0.0, 0.005, 0.05],
        ));
    }
    // Gold knee and ankle bands, stone shin plates.
    for (shin, foot) in [(ShinL, FootL), (ShinR, FootR)] {
        v.push(part(
            Body(shin),
            Bevel(0.15, 0.05, 0.13, 0.012),
            s.metal,
            [0.0, 0.0, 0.02],
        ));
        v.push(part(
            Body(shin),
            Bevel(0.12, 0.17, 0.05, 0.015),
            s.main,
            [0.0, -p.leg * 0.25, 0.07],
        ));
        v.push(part(
            Body(foot),
            Bevel(0.15, 0.035, 0.18, 0.01),
            s.metal,
            [0.0, 0.03, 0.02],
        ));
    }
}

/// Branch A: an offensive silhouette of angular gold blades over the stone.
fn wrath(v: &mut Vec<PartSpec>, s: Stone, f: Form) {
    let blades = if f.t2 { 3 } else { 2 };
    for (upper, fore, side) in [(UpperArmL, ForearmL, 1.0f32), (UpperArmR, ForearmR, -1.0)] {
        v.push(
            part(
                Body(upper),
                Bevel(0.22, 0.13, 0.23, 0.03),
                s.main,
                [0.0, 0.03, 0.0],
            )
            .rot([0.0, 0.0, side * 12.0]),
        );
        for i in 0..blades {
            let k = i as f32;
            v.push(
                part(
                    Body(upper),
                    Prism(0.05, 0.20 + 0.04 * k),
                    s.metal,
                    [side * (0.10 + 0.02 * k), 0.12, -0.04 + 0.05 * k],
                )
                .scale([0.5, 1.0, 0.35])
                .rot([0.0, 0.0, -side * (30.0 + 14.0 * k)]),
            );
        }
        // Heavier gauntlets for the overhead impact.
        v.push(part(
            Body(fore),
            Bevel(0.19, 0.11, 0.20, 0.02),
            s.metal,
            [0.0, -0.06, 0.0],
        ));
        if f.t2 {
            v.push(
                part(
                    Body(fore),
                    Prism(0.045, 0.22),
                    s.metal,
                    [side * 0.11, -0.10, 0.0],
                )
                .scale([0.45, 1.0, 0.35])
                .rot([0.0, 0.0, -side * 18.0]),
            );
        }
    }
    let rays: &[f32] = if f.t2 {
        &[-0.10, -0.05, 0.0, 0.05, 0.10]
    } else {
        &[-0.08, 0.0, 0.08]
    };
    for &x in rays {
        v.push(
            part(
                Body(Head),
                Prism(0.04, 0.22 + 0.4 * (0.1 - x.abs())),
                s.metal,
                [x, 0.26, -0.01],
            )
            .scale([0.7, 1.0, 0.5])
            .rot([0.0, 0.0, -x * 220.0]),
        );
    }
}

/// Branch B: stacked dark-stone armor, gold-bound joints and, at tier 2,
/// forearm tower shields with a protective halo and banner.
fn bastion(v: &mut Vec<PartSpec>, s: Stone, f: Form) {
    for (upper, fore, side) in [(UpperArmL, ForearmL, 1.0f32), (UpperArmR, ForearmR, -1.0)] {
        for (i, y) in [0.04f32, -0.04].into_iter().enumerate() {
            let color = if i == 0 { s.main } else { s.metal };
            v.push(
                part(
                    Body(upper),
                    Bevel(0.24 - 0.03 * i as f32, 0.10, 0.25, 0.03),
                    color,
                    [side * 0.02, y, 0.0],
                )
                .rot([0.0, 0.0, side * 16.0]),
            );
        }
        v.push(part(
            Body(fore),
            Bevel(0.20, 0.16, 0.20, 0.025),
            s.shade,
            [0.0, -0.10, 0.0],
        ));
        if f.t2 {
            // A tower shield on the outer forearm, gold-rimmed with a navy field.
            v.extend([
                part(
                    Body(fore),
                    Bevel(0.04, 0.34, 0.24, 0.012),
                    s.main,
                    [side * 0.13, -0.12, 0.0],
                ),
                part(
                    Body(fore),
                    Bevel(0.045, 0.36, 0.035, 0.008),
                    s.metal,
                    [side * 0.13, -0.12, 0.12],
                ),
                part(
                    Body(fore),
                    Bevel(0.045, 0.36, 0.035, 0.008),
                    s.metal,
                    [side * 0.13, -0.12, -0.12],
                ),
                part(
                    Body(fore),
                    Bevel(0.046, 0.12, 0.10, 0.008),
                    NAVY,
                    [side * 0.135, -0.10, 0.0],
                ),
                part(
                    Body(fore),
                    Prism(0.03, 0.08),
                    s.metal,
                    [side * 0.142, -0.10, 0.0],
                )
                .scale([0.4, 1.0, 1.0]),
            ]);
        }
    }
    v.push(part(
        Body(Head),
        Bevel(0.27, 0.10, 0.25, 0.02),
        s.main,
        [0.0, 0.20, -0.01],
    ));
    if f.t2 {
        v.extend([
            part(
                Body(Head),
                Ring(0.012, 0.22, 360.0),
                s.glow,
                [0.0, 0.16, -0.20],
            )
            .glow(),
            part(
                Body(Hips),
                Bevel(0.18, 0.30, 0.03, 0.01),
                NAVY,
                [0.0, -0.20, 0.15],
            ),
            part(
                Body(Hips),
                Bevel(0.19, 0.035, 0.035, 0.008),
                s.metal,
                [0.0, -0.06, 0.16],
            ),
            part(Body(Hips), Prism(0.035, 0.10), s.metal, [0.0, -0.22, 0.165])
                .scale([1.0, 1.0, 0.3]),
        ]);
    }
}

fn obelisk(v: &mut Vec<PartSpec>, s: Stone, f: Form) {
    let n = Prop(Socket::WeaponR);
    let width = if f.a { 0.27 } else { 0.23 };
    let body = if f.b { CHARCOAL } else { ROCK };
    // The unchanged GD-01 core: shaft, pyramidion, base cap and four bands.
    v.push(
        part(
            n,
            Frustum(width * 0.75, width, 1.58),
            body,
            [0.0, 0.38, 0.0],
        )
        .scale([0.8, 1.0, 0.7]),
    );
    v.push(part(n, Cone(width * 0.75, 0.30), s.metal, [0.0, 1.32, 0.0]).scale([0.8, 1.0, 0.7]));
    v.push(part(
        n,
        Bevel(width * 1.85, 0.10, width * 1.45, 0.025),
        s.metal,
        [0.0, -0.40, 0.0],
    ));
    for y in [-0.23, 0.19, 0.61, 1.04] {
        v.push(part(
            n,
            Bevel(width * 1.7, 0.045, width * 1.4, 0.012),
            s.metal,
            [0.0, y, 0.0],
        ));
        v.push(
            part(
                n,
                Prism(0.055, 0.09),
                if f.b { AMBER } else { GOLD },
                [0.0, y + 0.13, width * 0.74],
            )
            .scale([0.65, 1.0, 0.25])
            .rot([0.0, 0.0, 45.0]),
        );
    }
    if f.a {
        // Offensive fins along both edges, sharper and longer at tier 2.
        let fin = if f.t2 { 0.30 } else { 0.22 };
        for side in [-1.0f32, 1.0] {
            for y in [0.05f32, 0.48, 0.90] {
                v.push(
                    part(n, Prism(0.04, fin), s.metal, [side * width * 0.78, y, 0.0])
                        .scale([0.5, 1.0, 0.3])
                        .rot([0.0, 0.0, -side * 35.0]),
                );
            }
        }
    }
    if f.b {
        // Structural buttresses and a heavier foot; tier 2 a broad crowned cap.
        for side in [-1.0f32, 1.0] {
            v.push(part(
                n,
                Bevel(0.05, 0.85, width * 0.9, 0.012),
                s.metal,
                [side * width * 0.72, 0.30, 0.0],
            ));
        }
        if f.t2 {
            v.push(part(
                n,
                Bevel(width * 1.5, 0.06, width * 1.25, 0.015),
                s.metal,
                [0.0, 1.16, 0.0],
            ));
        }
    }
    if f.t2 {
        v.push(part(n, Ball(0.055), AMBER, [0.0, 1.24, 0.08]).glow());
    }
}
