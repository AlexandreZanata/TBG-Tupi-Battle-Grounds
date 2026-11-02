//! Stone anatomy, celestial mantle and five distinctive carved-obelisk forms.
use crate::art::{
    figure::{part, FigureSpec},
    kits::common::{body, Form},
    mesh::Shape::*,
    palette::*,
    rig::{
        Joint::*,
        Node::{Body, Prop},
        Proportions, Socket,
    },
};
pub(super) fn plan(t: u8) -> Proportions {
    let mut p = body::plan(true, 1.85, t);
    p.torso.x = 0.52;
    p.arm = 0.30;
    p.leg = 0.46;
    p.head = 0.15;
    p
}
pub(super) fn spec(t: u8, b: u8) -> FigureSpec {
    let p = plan(t);
    let f = Form::of(t, b);
    let mut v = Vec::new();
    let stone = if f.b { CHARCOAL } else { IVORY };
    let metal = if f.a { GOLD } else { BRASS };
    body::humanoid(
        &mut v,
        &p,
        &body::Look {
            skin: stone,
            cloth: stone,
            legs: FIELD_SLATE,
            boots: stone,
        },
    );
    v.retain(|a| !matches!(a.node, Body(Head)));
    v.push(part(
        Body(Head),
        Bevel(0.25, 0.29, 0.23, 0.035),
        stone,
        [0.0, 0.0, 0.0],
    ));
    for s in [-1.0f32, 1.0] {
        v.push(
            part(
                Body(Head),
                Bevel(0.075, 0.035, 0.028, 0.008),
                AMBER,
                [s * 0.065, 0.02, 0.122],
            )
            .glow(),
        );
        v.push(
            part(
                Body(Head),
                Prism(0.065, 0.20),
                metal,
                [s * 0.16, 0.09, -0.02],
            )
            .scale([0.6, 1.0, 0.5])
            .rot([0.0, 0.0, -s * 20.0]),
        );
        v.push(
            part(
                Body(Chest),
                Bevel(0.20, 0.11, 0.11, 0.025),
                metal,
                [s * 0.13, 0.25, 0.10],
            )
            .rot([0.0, 0.0, -s * 17.0]),
        );
        v.push(
            part(
                Body(Hips),
                Prism(0.12, 0.30),
                FIELD_SLATE,
                [s * 0.12, -0.15, 0.08],
            )
            .scale([1.0, 1.0, 0.35])
            .rot([0.0, 0.0, s * 8.0]),
        );
    }
    v.push(part(Body(Head), Prism(0.035, 0.11), metal, [0.0, -0.14, 0.10]).scale([1.0, 1.0, 0.5]));
    for (arm, fore, hand) in [(UpperArmL, ForearmL, HandL), (UpperArmR, ForearmR, HandR)] {
        v.push(part(Body(arm), Ball(0.135), stone, [0.0, -0.01, 0.0]).scale([0.85, 1.0, 1.0]));
        v.push(part(
            Body(arm),
            Frustum(0.12, 0.085, 0.28),
            stone,
            [0.0, -0.14, 0.0],
        ));
        v.push(part(
            Body(fore),
            Bevel(0.15, 0.20, 0.17, 0.025),
            stone,
            [0.0, -0.12, 0.0],
        ));
        v.push(part(
            Body(fore),
            Bevel(0.17, 0.045, 0.18, 0.012),
            metal,
            [0.0, -0.17, 0.0],
        ));
        for finger in 0..4 {
            v.push(part(
                Body(hand),
                Bevel(0.031, 0.065, 0.055, 0.008),
                stone,
                [-0.055 + finger as f32 * 0.036, -0.035, 0.055],
            ));
        }
    }
    if t > 0 {
        v.push(part(
            Body(Head),
            Bevel(0.30, 0.065, 0.26, 0.015),
            metal,
            [0.0, 0.15, 0.0],
        ));
        v.push(part(
            Body(UpperArmL),
            Bevel(0.22, 0.15, 0.23, 0.03),
            if f.a { IVORY } else { CHARCOAL },
            [0.0, 0.02, 0.0],
        ));
    }
    if t == 2 {
        for x in [-0.08, 0.0, 0.08] {
            v.push(
                part(
                    Body(Head),
                    Prism(0.04, 0.22),
                    if f.a { GOLD } else { AMBER },
                    [x, 0.26, -0.01],
                )
                .scale([0.7, 1.0, 0.5]),
            );
        }
        v.push(part(
            Body(UpperArmR),
            Bevel(0.22, 0.13, 0.23, 0.03),
            metal,
            [0.0, 0.02, 0.0],
        ));
    }
    let n = Prop(Socket::WeaponR);
    let width = if f.a { 0.27 } else { 0.23 };
    v.push(
        part(
            n,
            Frustum(width * 0.75, width, 1.58),
            if f.b { CHARCOAL } else { ROCK },
            [0.0, 0.38, 0.0],
        )
        .scale([0.8, 1.0, 0.7]),
    );
    v.push(part(n, Cone(width * 0.75, 0.30), metal, [0.0, 1.32, 0.0]).scale([0.8, 1.0, 0.7]));
    v.push(part(
        n,
        Bevel(width * 1.85, 0.10, width * 1.45, 0.025),
        metal,
        [0.0, -0.40, 0.0],
    ));
    for y in [-0.23, 0.19, 0.61, 1.04] {
        v.push(part(
            n,
            Bevel(width * 1.7, 0.045, width * 1.4, 0.012),
            metal,
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
    if t == 2 {
        v.push(part(n, Ball(0.055), AMBER, [0.0, 1.24, 0.08]).glow());
    }
    FigureSpec { plan: p, parts: v }
}
