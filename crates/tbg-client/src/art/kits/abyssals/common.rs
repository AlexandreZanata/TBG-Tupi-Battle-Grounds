//! Marine anatomy, shell pieces and material progression shared by the Abyssal kits.
pub(super) use super::super::common::{body, Form};
pub(super) use crate::art::{
    figure::{part, FigureSpec, PartSpec},
    mesh::Shape::*,
    palette::*,
    rig::{
        Joint::*,
        Node::{Body, Prop},
        Proportions, Socket,
    },
};
pub(super) use bevy::prelude::*;
pub(super) use tbg_core::UnitKind::{self, *};
pub const SEA: Color = Color::srgb(0.33, 0.58, 0.53);
pub const PALE: Color = Color::srgb(0.77, 0.84, 0.74);
pub const PLUM: Color = Color::srgb(0.29, 0.20, 0.34);
pub const FOAM: Color = Color::srgb(0.58, 0.90, 0.79);
pub fn plan(k: UnitKind, t: u8) -> Proportions {
    let (size, leg, width, arm) = match k {
        NacreDuelist => (1.08, 0.46, 0.34, 0.23),
        ReefLancer => (1.11, 0.48, 0.32, 0.24),
        TrenchFist => (1.04, 0.34, 0.45, 0.25),
        PearlSlinger => (0.96, 0.36, 0.33, 0.20),
        AbyssHarpooner => (1.08, 0.45, 0.35, 0.24),
        UndertowHerald => (1.10, 0.41, 0.43, 0.24),
        TrenchLamplighter => (1.12, 0.48, 0.29, 0.24),
        FoamWeaver => (1.05, 0.46, 0.31, 0.22),
        _ => (0.99, 0.34, 0.35, 0.22),
    };
    let mut p = body::plan(false, size, t);
    p.leg = leg;
    p.torso.x = width;
    p.arm = arm;
    p.head = 0.14;
    p
}
pub fn shell(
    v: &mut Vec<PartSpec>,
    node: crate::art::rig::Node,
    at: [f32; 3],
    size: f32,
    color: Color,
) {
    v.push(part(node, Ball(size), color, at).scale([1.0, 0.70, 0.45]));
    for i in 0..5 {
        let a = (-50.0 + i as f32 * 25.0).to_radians();
        v.push(
            part(
                node,
                Prism(size * 0.19, size * 1.5),
                color,
                [
                    at[0] + a.sin() * size * 0.50,
                    at[1] + a.cos() * size * 0.37,
                    at[2] + size * 0.22,
                ],
            )
            .scale([1.0, 1.0, 0.5])
            .rot([0.0, 0.0, -a.to_degrees()]),
        );
    }
}
pub fn marine(k: UnitKind, p: &Proportions, f: Form) -> Vec<PartSpec> {
    let skin = match k {
        AbyssHarpooner => Color::srgb(0.38, 0.33, 0.42),
        TrenchFist => Color::srgb(0.35, 0.42, 0.30),
        UndertowHerald => Color::srgb(0.30, 0.44, 0.31),
        _ => SEA,
    };
    let cloth = if f.b {
        PLUM
    } else if f.a {
        IVORY
    } else {
        match k {
            FoamWeaver => IVORY,
            ReefLancer | PearlSlinger => RUST,
            TrenchFist => OILSKIN_YELLOW,
            NacreDuelist | AbyssHarpooner | TrenchLamplighter => PLUM,
            _ => FIELD_SLATE,
        }
    };
    let mut v = Vec::new();
    body::humanoid(
        &mut v,
        p,
        &body::Look {
            skin,
            cloth: skin,
            legs: cloth,
            boots: DARK_LEATHER,
        },
    );
    v.retain(|a| !matches!(a.node, Body(Head)));
    for part in &mut v {
        match part.node {
            Body(ThighL | ThighR) => {
                part.shape = Frustum(0.071, 0.055, p.leg * 0.5);
                part.color = skin;
            }
            Body(ShinL | ShinR) => part.color = skin,
            Body(FootL | FootR) => {
                part.shape = Bevel(0.145, 0.075, 0.24, 0.025);
                part.color = skin;
            }
            _ => {}
        }
    }
    v.push(part(
        Body(Head),
        Frustum(0.055, 0.075, 0.16),
        skin,
        [0.0, -0.13, -0.025],
    ));
    v.push(
        part(
            Body(Chest),
            Frustum(p.torso.x * 0.29, p.torso.x * 0.25, p.torso.y * 0.78),
            PALE,
            [0.0, p.torso.y * 0.47, 0.09],
        )
        .scale([1.0, 1.0, 0.45]),
    );
    v.push(
        part(
            Body(Hips),
            Cyl(p.torso.x * 0.48, 0.045),
            LEATHER,
            [0.0, 0.025, 0.0],
        )
        .scale([1.0, 1.0, 0.78]),
    );
    for s in [-1.0f32, 1.0] {
        v.push(
            part(
                Body(Chest),
                Ball(p.torso.x * 0.24),
                skin,
                [s * p.torso.x * 0.24, p.torso.y * 0.72, 0.05],
            )
            .scale([1.0, 0.68, 0.70]),
        );
        v.push(
            part(
                Body(Hips),
                Prism(0.10, if k == TrenchLamplighter { 0.48 } else { 0.25 }),
                cloth,
                [s * 0.09, -0.14, 0.05],
            )
            .scale([1.0, 1.0, 0.3])
            .rot([0.0, 0.0, -s * 10.0]),
        );
    }
    face(&mut v, k, skin, f);
    if f.a || f.b {
        shell(
            &mut v,
            Body(UpperArmL),
            [0.0, 0.015, 0.0],
            0.125,
            if f.a { IVORY } else { PLUM },
        );
        for shin in [ShinL, ShinR] {
            v.push(
                part(
                    Body(shin),
                    Prism(0.085, 0.20),
                    if f.a { IVORY } else { PLUM },
                    [0.0, -0.05, 0.055],
                )
                .scale([1.0, 1.0, 0.45]),
            );
        }
    }
    if f.t2 {
        v.push(
            part(
                Body(UpperArmL),
                Ball(0.041),
                if f.a { AMBER } else { FOAM },
                [0.0, 0.07, 0.08],
            )
            .glow(),
        );
    }
    v
}
fn face(v: &mut Vec<PartSpec>, k: UnitKind, skin: Color, f: Form) {
    let hammer = k == TrenchFist;
    let small = k == PearlSlinger;
    v.push(part(
        Body(Head),
        Bevel(
            if hammer { 0.38 } else { 0.20 },
            if small { 0.22 } else { 0.17 },
            0.24,
            0.035,
        ),
        skin,
        [0.0, 0.0, 0.025],
    ));
    v.push(part(
        Body(Head),
        Bevel(
            if hammer { 0.18 } else { 0.14 },
            0.075,
            if k == AbyssHarpooner { 0.22 } else { 0.16 },
            0.025,
        ),
        PALE,
        [0.0, -0.06, 0.105],
    ));
    let eye_x = if hammer { 0.165 } else { 0.075 };
    for s in [-1.0f32, 1.0] {
        v.push(
            part(
                Body(Head),
                Ball(if small { 0.043 } else { 0.026 }),
                if small { DARK } else { AMBER },
                [s * eye_x, 0.035, 0.145],
            )
            .scale([1.0, 1.0, 0.35]),
        );
        if !small {
            v.push(part(
                Body(Head),
                Box(0.012, 0.025, 0.007),
                DARK,
                [s * eye_x, 0.035, 0.155],
            ));
        }
        let fan = matches!(k, ReefLancer | FoamWeaver | PearlSlinger);
        for i in 0..if fan { 3 } else { 2 } {
            v.push(
                part(
                    Body(Head),
                    Prism(0.06, if fan { 0.22 } else { 0.12 }),
                    if f.b { PLUM } else { skin },
                    [s * (0.12 + i as f32 * 0.02), 0.04 - i as f32 * 0.04, -0.035],
                )
                .scale([0.6, 1.0, 0.35])
                .rot([25.0, 0.0, -s * (55.0 + i as f32 * 15.0)]),
            );
        }
    }
    if k == TrenchLamplighter {
        v.push(
            part(
                Body(Head),
                Ring(0.012, 0.10, 200.0),
                skin,
                [0.0, 0.19, 0.05],
            )
            .rot([90.0, 0.0, 90.0]),
        );
        v.push(part(Body(Head), Ball(0.025), AMBER, [0.0, 0.17, 0.15]).glow());
    } else if matches!(k, NacreDuelist | AbyssHarpooner) {
        v.push(
            part(
                Body(Head),
                Prism(0.13, 0.035),
                if f.b { PLUM } else { skin },
                [0.0, 0.12, -0.09],
            )
            .scale([0.4, 1.0, 1.0])
            .rot([90.0, 0.0, 0.0]),
        );
    }
    if k == UndertowHerald {
        for x in [-0.065, 0.0, 0.065] {
            v.push(
                part(Body(Head), Cone(0.02, 0.11), PALE, [x, -0.14, 0.11]).rot([165.0, 0.0, 0.0]),
            );
        }
    }
}
pub fn finish(v: &mut Vec<PartSpec>, f: Form, node: crate::art::rig::Node, at: [f32; 3]) {
    if f.t2 {
        v.push(part(node, Ball(0.035), if f.a { AMBER } else { FOAM }, at).glow());
    }
}
