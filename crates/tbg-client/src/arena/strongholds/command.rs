//! Command: a slim round tower with a team crest, a ringed observation
//! platform and a tall war standard. Branch A (War Standard) grows the
//! standard into a gold command symbol; branch B (Forward Banner) extends
//! the post toward the lane and then raises an observatory.

use super::parts::{along_z, around, t, whole, Form, PartDef, Piece};
use crate::arena::Part;
use crate::art::{mesh::Shape::*, palette::*};
use bevy::prelude::*;

static TOWER: PartDef = PartDef {
    name: "command tower",
    build: tower,
};
static PLATFORM: PartDef = PartDef {
    name: "command platform",
    build: platform,
};
static STANDARD: PartDef = PartDef {
    name: "command standard",
    build: |team| standard(team, 1.5, 0.62, 0.8),
};
static WAR_STANDARD: PartDef = PartDef {
    name: "war standard",
    build: |team| standard(team, 1.7, 0.9, 1.0),
};
static WAR_ARMS: PartDef = PartDef {
    name: "war standard arms",
    build: war_arms,
};
static SUPREME_SYMBOL: PartDef = PartDef {
    name: "supreme command symbol",
    build: supreme_symbol,
};
static SUPREME_BANNERS: PartDef = PartDef {
    name: "supreme command banners",
    build: supreme_banners,
};
static OUTPOST: PartDef = PartDef {
    name: "forward outpost",
    build: outpost,
};
static OBSERVATORY: PartDef = PartDef {
    name: "high command observatory",
    build: observatory,
};

const PROFILE: &[[f32; 2]] = &[
    [0.82, 0.0],
    [0.82, 0.16],
    [0.66, 0.24],
    [0.6, 1.35],
    [0.7, 1.42],
];
/// Platform deck height and the standard's pole position on it.
const DECK: f32 = 1.5;
const POLE: Vec3 = Vec3::new(0.45, DECK, -0.35);

pub(super) fn pieces(form: Form) -> Vec<Piece> {
    let mut p = vec![whole(&TOWER), whole(&PLATFORM).lost(2)];
    p.push(
        if form.a() {
            whole(&WAR_STANDARD)
        } else {
            whole(&STANDARD)
        }
        .lost(1),
    );
    if form.a() {
        p.push(whole(&WAR_ARMS));
    }
    if form == Form::A2 {
        p.push(whole(&SUPREME_SYMBOL).lost(1));
        p.push(whole(&SUPREME_BANNERS).lost(2));
    }
    if form.b() {
        p.push(whole(&OUTPOST).aimed());
    }
    if form == Form::B2 {
        p.push(whole(&OBSERVATORY).lost(1).aimed());
    }
    p
}

/// Round stone tower with a door, arrow slits and the team crest.
fn tower(team: Color) -> Vec<Part> {
    let mut p = vec![
        (Lathe(PROFILE), STONE, Transform::IDENTITY),
        (Box(0.3, 0.42, 0.05), WOOD, t(0.0, 0.3, 0.7)),
        (Box(0.42, 0.42, 0.05), team, t(0.0, 0.92, 0.64)),
        (
            Box(0.3, 0.3, 0.05),
            team,
            t(0.0, 0.72, 0.64).with_rotation(Quat::from_rotation_z(0.785)),
        ),
        (
            Box(0.14, 0.14, 0.03),
            LINEN,
            t(0.0, 0.88, 0.675).with_rotation(Quat::from_rotation_z(0.785)),
        ),
        (Box(0.3, 0.07, 0.03), GOLD, t(0.0, 1.11, 0.67)),
    ];
    for at in around(3, 0.61, 1.0).skip(1) {
        p.push((Box(0.07, 0.24, 0.04), DARK, at));
    }
    // Masonry courses, an arched door up two steps (lobby portrait).
    let up = Quat::from_rotation_x(-std::f32::consts::FRAC_PI_2);
    p.extend([
        (Ring(0.018, 0.655, 360.0), ROCK, t(0.0, 0.55, 0.0)),
        (Ring(0.018, 0.63, 360.0), ROCK, t(0.0, 1.0, 0.0)),
        (
            Ring(0.035, 0.16, 180.0),
            ROCK,
            t(0.0, 0.51, 0.7).with_rotation(up),
        ),
        (Box(0.44, 0.08, 0.22), ROCK, t(0.0, 0.04, 0.86)),
    ]);
    p
}

/// Observation platform: a deck on struts, railing posts and a command tent.
fn platform(team: Color) -> Vec<Part> {
    let mut p = vec![
        (Cyl(0.95, 0.08), WOOD, t(0.0, DECK - 0.04, 0.0)),
        (Ring(0.02, 0.92, 360.0), WOOD, t(0.0, DECK + 0.3, 0.0)),
        (Cone(0.3, 0.45), LINEN, t(-0.5, DECK + 0.22, 0.35)),
        (Cyl(0.02, 0.2), WOOD, t(-0.5, DECK + 0.5, 0.35)),
        (Box(0.12, 0.08, 0.01), team, t(-0.44, DECK + 0.56, 0.35)),
    ];
    for at in around(10, 0.92, DECK + 0.15) {
        p.push((Box(0.04, 0.3, 0.04), WOOD, at));
    }
    for at in around(4, 0.75, DECK - 0.25) {
        let lean = at.rotation * Quat::from_rotation_z(-0.6);
        p.push((Box(0.06, 0.5, 0.06), TIMBER, at.with_rotation(lean)));
    }
    // Cloth panels between the rail posts, team and linen in turn.
    for i in 0..10 {
        let a = (i as f32 + 0.5) / 10.0 * std::f32::consts::TAU;
        let at =
            t(a.cos() * 0.92, DECK + 0.17, a.sin() * 0.92).with_rotation(Quat::from_rotation_y(-a));
        let cloth = if i % 2 == 0 { team } else { LINEN };
        p.push((Box(0.015, 0.2, 0.5), cloth, at));
    }
    // A heraldic banner hanging from the deck over the door.
    p.extend([
        (Box(0.44, 0.03, 0.03), GOLD, t(0.0, DECK - 0.07, 0.93)),
        (Box(0.34, 0.36, 0.02), team, t(0.0, DECK - 0.25, 0.93)),
        (
            Box(0.13, 0.13, 0.025),
            GOLD,
            t(0.0, DECK - 0.24, 0.945).with_rotation(Quat::from_rotation_z(0.785)),
        ),
        (
            Box(0.12, 0.12, 0.02),
            team,
            t(-0.09, DECK - 0.44, 0.93).with_rotation(Quat::from_rotation_z(0.785)),
        ),
        (
            Box(0.12, 0.12, 0.02),
            team,
            t(0.09, DECK - 0.44, 0.93).with_rotation(Quat::from_rotation_z(0.785)),
        ),
    ]);
    p
}

/// A pole with a crossbar, a hanging team banner and a gold finial.
fn standard(team: Color, height: f32, width: f32, drop: f32) -> Vec<Part> {
    let top = POLE.y + height;
    vec![
        (
            Cyl(0.04, height),
            WOOD,
            t(POLE.x, POLE.y + height / 2.0, POLE.z),
        ),
        (
            Box(width + 0.1, 0.05, 0.05),
            WOOD,
            t(POLE.x, top - 0.1, POLE.z + 0.04),
        ),
        (
            Box(width, drop, 0.03),
            team,
            t(POLE.x, top - 0.12 - drop / 2.0, POLE.z + 0.06),
        ),
        (
            Box(width * 0.3, width * 0.3, 0.035),
            LINEN,
            t(POLE.x, top - 0.12 - drop * 0.45, POLE.z + 0.06)
                .with_rotation(Quat::from_rotation_z(0.785)),
        ),
        (Ball(0.07), GOLD, t(POLE.x, top + 0.05, POLE.z)),
    ]
}

/// War Standard: crossed spears and a gold laurel behind the crest.
fn war_arms(_team: Color) -> Vec<Part> {
    let mut p = vec![(Ring(0.025, 0.32, 300.0), GOLD, along_z(0.0, 0.88, 0.63))];
    for lean in [-0.6, 0.6] {
        let at = t(0.0, 0.88, 0.61).with_rotation(Quat::from_rotation_z(lean));
        p.push((Box(0.03, 0.95, 0.03), STEEL, at));
        p.push((Cone(0.05, 0.14), STEEL, at.mul_transform(t(0.0, 0.52, 0.0))));
    }
    p
}

/// Supreme Command: a huge gold sun disc crowning the standard.
fn supreme_symbol(_team: Color) -> Vec<Part> {
    let center = Vec3::new(POLE.x, POLE.y + 2.0, POLE.z);
    let mut p = vec![
        (Cyl(0.26, 0.06), GOLD, along_z(center.x, center.y, center.z)),
        (Cyl(0.16, 0.08), SAP, along_z(center.x, center.y, center.z)),
        (Cyl(0.035, 0.3), WOOD, t(center.x, center.y - 0.3, center.z)),
    ];
    for i in 0..8 {
        let a = i as f32 / 8.0 * std::f32::consts::TAU;
        let at = Transform::from_translation(center + Vec3::new(a.cos(), a.sin(), 0.0) * 0.34)
            .with_rotation(Quat::from_rotation_z(a - std::f32::consts::FRAC_PI_2));
        p.push((Cone(0.07, 0.2), GOLD, at));
    }
    p
}

/// Supreme Command: four team pennants around the platform rim.
fn supreme_banners(team: Color) -> Vec<Part> {
    let mut p = Vec::new();
    for i in 0..4 {
        let a = (i as f32 + 0.5) / 4.0 * std::f32::consts::TAU;
        let (x, z) = (a.cos() * 0.85, a.sin() * 0.85);
        p.push((Cyl(0.025, 0.95), WOOD, t(x, DECK + 0.47, z)));
        p.push((Box(0.3, 0.22, 0.02), team, t(x + 0.15, DECK + 0.82, z)));
        p.push((Ball(0.04), GOLD, t(x, DECK + 0.97, z)));
    }
    p
}

/// Forward Banner: a gangway toward the lane with a map table and a
/// team awning.
fn outpost(team: Color) -> Vec<Part> {
    let mut p = vec![
        (Box(0.8, 0.08, 0.64), WOOD, t(1.05, DECK - 0.04, 0.0)),
        (Box(0.3, 0.04, 0.24), OAK, t(1.0, DECK + 0.28, 0.0)),
        (Box(0.04, 0.26, 0.04), OAK, t(1.0, DECK + 0.13, 0.0)),
        (Cyl(0.03, 0.22), LINEN, along_z(1.0, DECK + 0.33, 0.0)),
        (Box(0.78, 0.04, 0.7), team, t(1.08, DECK + 0.62, 0.0)),
    ];
    for z in [-0.28, 0.28] {
        p.push((Box(0.07, DECK, 0.07), TIMBER, t(1.38, DECK / 2.0, z)));
        p.push((Box(0.04, 0.62, 0.04), WOOD, t(1.4, DECK + 0.31, z)));
        p.push((Box(0.04, 0.62, 0.04), WOOD, t(0.72, DECK + 0.31, z)));
    }
    p
}

/// High Command: a stone observatory with a brass dome and a telescope
/// trained on the lane.
fn observatory(team: Color) -> Vec<Part> {
    vec![
        (Cyl(0.3, 1.0), STONE, t(0.0, DECK + 0.5, 0.0)),
        (Cyl(0.33, 0.1), team, t(0.0, DECK + 0.95, 0.0)),
        (Ball(0.3), BRASS, t(0.0, DECK + 1.05, 0.0)),
        (
            Cyl(0.05, 0.6),
            STEEL,
            // Tilted 25 degrees up from +X.
            t(0.32, DECK + 1.18, 0.0).with_rotation(Quat::from_rotation_z(-1.12)),
        ),
        (Cyl(0.02, 0.3), WOOD, t(0.0, DECK + 1.45, 0.0)),
        (Box(0.24, 0.16, 0.02), team, t(0.12, DECK + 1.52, 0.0)),
    ]
}
