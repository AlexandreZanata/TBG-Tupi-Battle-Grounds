//! Smuggler figure: a shifty runner in a dark coat, tricorn and eye patch,
//! red sash, a lantern at the belt and a crate on a back frame. Branch A
//! gets richer (coin pouches and stacked crates, then a gilded chest, a
//! plumed tricorn and a team cape); branch B gets faster (a light satchel
//! and bandana, then two crates and a lantern on a pole).

use bevy::prelude::*;

use crate::art::{
    figure::{part, FigureSpec, PartSpec},
    kits::common::{body, gear, Form},
    mesh::Shape::*,
    palette::*,
    rig::{Joint::*, Node::Body, Node::Prop, Node::Secondary, Proportions, Socket, Spring},
};

const COAT: Color = Color::srgb(0.32, 0.24, 0.2);
const HAT: Color = Color::srgb(0.17, 0.14, 0.13);

pub fn plan(tier: u8) -> Proportions {
    body::plan(false, 1.0, tier)
}

pub fn spec(tier: u8, branch: u8) -> FigureSpec {
    let f = Form::of(tier, branch);
    let p = plan(tier);
    let mut v = Vec::with_capacity(110);
    let look = body::Look {
        skin: SKIN,
        cloth: COAT,
        legs: DARK,
        boots: DARK_LEATHER,
    };
    body::humanoid(&mut v, &p, &look);
    body::tunic(&mut v, &p, COAT, DARK_LEATHER, RED_CLOTH);
    head(&mut v, &p, f);
    pack(&mut v, f);
    // Lantern at the belt, team kerchief at the neck.
    v.extend(gear::lantern(Body(Hips), [0.19, -0.08, 0.06], 1.1));
    v.extend([part(
        Body(Chest),
        Box(0.18, 0.05, 0.05),
        ALLY,
        [0.0, p.torso.y * 0.95, p.torso.z * 0.3],
    )]);
    if f.a {
        for x in [-0.12, 0.1] {
            v.extend(gear::sack(Body(Hips), [x, -0.05, 0.13], GOLD, 0.45));
        }
        for y in [0.15, 0.25] {
            v.push(part(
                Body(Chest),
                Ball(0.016),
                BRASS,
                [0.07, y, p.torso.z * 0.47],
            ));
        }
    }
    if f.a && f.t2 {
        body::cape(&mut v, ALLY, 0.26, 0.42);
        v.push(
            part(
                Body(Chest),
                Ring(0.008, 0.1, 200.0),
                GOLD,
                [0.0, p.torso.y * 0.8, 0.06],
            )
            .rot([80.0, 0.0, 0.0]),
        );
    }
    if f.b {
        // Runner's boots striped in team color.
        for shin in [ShinL, ShinR] {
            v.push(part(
                Body(shin),
                Box(0.1, 0.03, 0.1),
                ALLY,
                [0.0, -0.05, 0.0],
            ));
        }
    }
    FigureSpec { plan: p, parts: v }
}

fn head(v: &mut Vec<PartSpec>, p: &Proportions, f: Form) {
    let h = p.head;
    v.extend([
        part(
            Body(Head),
            Box(h * 0.5, h * 0.12, 0.02),
            DARK,
            [0.0, h * 0.18, h * 0.93],
        )
        .rot([0.0, 0.0, 20.0]),
        part(
            Body(Head),
            Box(0.05, 0.045, 0.02),
            DARK,
            [h * 0.36, h * 0.1, h * 0.96],
        ),
        part(
            Body(Head),
            Box(h * 1.1, h * 0.25, h * 0.4),
            OAK,
            [0.0, -h * 0.5, h * 0.55],
        ),
    ]);
    if f.b {
        // A bandana knotted at the back.
        v.push(
            part(
                Body(Head),
                Ball(h * 1.04),
                RED_CLOTH,
                [0.0, h * 0.25, -h * 0.05],
            )
            .scale([1.03, 0.6, 1.03]),
        );
        v.push(part(
            Secondary(Spring::HeadGear),
            Box(0.03, h * 0.6, h * 0.25),
            RED_CLOTH,
            [0.0, h * 0.1, -h * 1.05],
        ));
        return;
    }
    // Tricorn: a low crown and three upturned brim boards.
    v.push(part(Body(Head), Ball(h * 1.0), HAT, [0.0, h * 0.55, 0.0]).scale([1.05, 0.7, 1.05]));
    for a in [0.0f32, 120.0, 240.0] {
        let r = a.to_radians();
        v.push(
            part(
                Body(Head),
                Box(h * 1.75, h * 0.5, 0.03),
                HAT,
                [r.sin() * h * 0.82, h * 0.62, r.cos() * h * 0.82],
            )
            .rot([-30.0, a, 0.0]),
        );
    }
    if f.a && f.t2 {
        v.push(
            part(
                Secondary(Spring::HeadGear),
                Box(0.03, h * 0.4, h * 1.4),
                GOLD,
                [h * 0.5, h * 0.95, -h * 0.2],
            )
            .rot([0.0, 0.0, -20.0]),
        );
    }
}

/// The back frame and its cargo.
fn pack(v: &mut Vec<PartSpec>, f: Form) {
    let b = Secondary(Spring::Back);
    if f.b && !f.t2 {
        // Faster Deliveries: only a satchel across the body.
        v.push(part(
            Body(Hips),
            Box(0.16, 0.14, 0.08),
            LEATHER,
            [-0.17, 0.0, -0.06],
        ));
        v.push(
            part(
                Body(Chest),
                Box(0.04, 0.45, 0.02),
                LEATHER,
                [0.0, 0.16, 0.13],
            )
            .rot([0.0, 0.0, -35.0]),
        );
        return;
    }
    v.extend([
        part(b, Box(0.04, 0.5, 0.04), WOOD, [0.12, -0.05, -0.04]),
        part(b, Box(0.04, 0.5, 0.04), WOOD, [-0.12, -0.05, -0.04]),
    ]);
    if f.a && f.t2 {
        // Black Market: a gilded chest with a glowing lock.
        v.extend([
            part(b, Box(0.3, 0.2, 0.2), OAK, [0.0, 0.0, -0.14]),
            part(b, Cyl(0.1, 0.3), OAK, [0.0, 0.1, -0.14])
                .rot([0.0, 0.0, 90.0])
                .scale([1.0, 1.0, 0.95]),
            part(b, Box(0.32, 0.04, 0.22), GOLD, [0.0, 0.02, -0.14]),
            part(b, Box(0.06, 0.06, 0.02), SAP, [0.0, 0.0, -0.25]).glow(),
        ]);
        return;
    }
    v.extend(gear::crate_box(b, [0.0, 0.0, -0.13], 0.24));
    if f.a || f.t2 {
        v.extend(gear::crate_box(b, [0.0, 0.22, -0.13], 0.18));
    }
    if f.b && f.t2 {
        let w = Prop(Socket::OffhandL);
        v.extend(gear::crate_box(w, [0.12, -0.02, 0.0], 0.2));
        v.push(part(b, Cyl(0.012, 0.6), WOOD, [-0.14, 0.35, -0.06]));
        v.extend(gear::lantern(b, [-0.14, 0.58, -0.06], 1.5));
    }
    v.push(part(b, Ring(0.015, 0.07, 360.0), LINEN, [0.12, -0.25, -0.1]).rot([0.0, 0.0, 90.0]));
}
