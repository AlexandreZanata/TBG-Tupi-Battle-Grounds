//! Pyre Warden's coffin furnace, rigid hose, braced nozzle and five funerary armor silhouettes.
use crate::art::{
    figure::{part, FigureSpec},
    kits::common::{body, Form},
    mesh::Shape::*,
    palette::*,
    rig::{
        Joint::*,
        Node::{Body, Prop, Secondary},
        Proportions, Socket, Spring,
    },
};
use bevy::prelude::*;
pub fn plan(t: u8) -> Proportions {
    let mut p = body::plan(false, 1.08, t);
    p.torso.x *= 1.2;
    p
}
pub fn spec(t: u8, b: u8) -> FigureSpec {
    let f = Form::of(t, b);
    let p = plan(t);
    let mut v = Vec::new();
    body::skeleton(&mut v, &p, IVORY);
    v.extend([
        part(
            Body(Chest),
            Box(0.36, 0.30, 0.14),
            if f.a { IVORY } else { IRON },
            [0.0, 0.10, 0.02],
        ),
        part(
            Secondary(Spring::Back),
            Box(0.32, 0.62, 0.22),
            if f.b { CHITIN } else { IRON },
            [0.0, 0.20, -0.22],
        ),
        part(
            Secondary(Spring::Back),
            Box(0.20, 0.48, 0.03),
            OAK,
            [0.0, 0.20, -0.35],
        ),
        part(
            Secondary(Spring::Back),
            Box(0.025, 0.30, 0.04),
            BRASS,
            [0.0, 0.22, -0.38],
        ),
        part(
            Secondary(Spring::Back),
            Box(0.15, 0.025, 0.04),
            BRASS,
            [0.0, 0.28, -0.38],
        ),
        part(
            Prop(Socket::WeaponR),
            Cyl(0.055, if f.a && f.t2 { 0.60 } else { 0.46 }),
            IRON,
            [0.0, 0.02, if f.a && f.t2 { 0.30 } else { 0.23 }],
        )
        .rot([90.0, 0.0, 0.0]),
        part(
            Prop(Socket::WeaponR),
            Frustum(0.055, 0.09, 0.10),
            BRASS,
            [0.0, 0.02, if f.a && f.t2 { 0.55 } else { 0.41 }],
        )
        .rot([90.0, 0.0, 0.0]),
        part(
            Prop(Socket::WeaponR),
            Box(0.035, 0.14, 0.06),
            OAK,
            [0.0, -0.06, 0.05],
        ),
        part(Body(Chest), Box(0.13, 0.11, 0.035), ALLY, [0.0, 0.07, 0.10]),
        part(Body(Head), Box(0.20, 0.055, 0.22), IRON, [0.0, 0.11, 0.0]),
    ]);
    for s in [-1.0, 1.0] {
        v.push(part(Body(Head), Ball(0.025), LEAF, [s * 0.055, 0.025, 0.13]).glow());
        v.push(part(
            Body(Chest),
            Box(if f.a { 0.18 } else { 0.13 }, 0.075, 0.17),
            if f.a { IVORY } else { IRON },
            [s * 0.22, 0.18, 0.0],
        ));
        v.push(part(
            Secondary(Spring::Back),
            Cyl(0.04, 0.32),
            BRASS,
            [s * 0.18, 0.37, -0.22],
        ));
        v.push(
            part(
                Secondary(Spring::Back),
                Cone(0.03, 0.09),
                LEAF,
                [s * 0.18, 0.57, -0.22],
            )
            .glow(),
        );
        for y in [0.0, 0.11, 0.22] {
            v.push(
                part(
                    Secondary(Spring::Back),
                    Box(0.07, 0.025, 0.03),
                    LEAF,
                    [s * 0.08, y, -0.35],
                )
                .glow(),
            );
        }
        if f.b {
            v.push(
                part(
                    Prop(Socket::WeaponR),
                    Cyl(0.06, 0.12),
                    LEAF,
                    [s * 0.08, 0.02, 0.18],
                )
                .rot([90.0, 0.0, 0.0]),
            );
        }
        if f.b && f.t2 {
            v.push(
                part(
                    Prop(Socket::WeaponR),
                    Cyl(0.04, 0.20),
                    IRON,
                    [s * 0.09, 0.02, 0.34],
                )
                .rot([90.0, 0.0, 0.0]),
            );
        }
    }
    // A rigid, segmented hose remains in weapon space and cannot jitter like a rope.
    for (x, y, z, angle) in [
        (0.06, -0.02, -0.06, 65.0),
        (0.12, -0.08, -0.12, 30.0),
        (0.15, -0.17, -0.12, 0.0),
    ] {
        v.push(
            part(Prop(Socket::WeaponR), Cyl(0.025, 0.12), CHITIN, [x, y, z]).rot([0.0, 0.0, angle]),
        );
    }
    if f.a && f.t2 {
        v.push(part(
            Secondary(Spring::Back),
            Box(0.39, 0.065, 0.28),
            IVORY,
            [0.0, 0.50, -0.20],
        ));
    }
    FigureSpec { plan: p, parts: v }
}
