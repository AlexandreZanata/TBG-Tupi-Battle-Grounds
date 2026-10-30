//! Bastion: a low, thick curtain wall laid across the BattleLine, hung with
//! team shields. Branch A (Fortress) adds drum towers, plates and
//! buttresses; branch B (Sentinel) raises a watchtower and then a ballista deck on it.

use super::parts::{along_x, along_z, around, piece, t, whole, Form, Motion, PartDef, Piece};
use crate::arena::Part;
use crate::art::{mesh::Shape::*, palette::*};
use bevy::prelude::*;

static WALLS: PartDef = PartDef {
    name: "bastion walls",
    build: walls,
};
static MERLONS: PartDef = PartDef {
    name: "bastion merlons",
    build: merlons,
};
static KEEP: PartDef = PartDef {
    name: "bastion keep",
    build: keep,
};
static DRUMS: PartDef = PartDef {
    name: "fortress drum towers",
    build: drums,
};
static BUTTRESSES: PartDef = PartDef {
    name: "fortress buttresses",
    build: buttresses,
};
static CAPS: PartDef = PartDef {
    name: "fortress caps",
    build: caps,
};
static SHAFT: PartDef = PartDef {
    name: "sentinel shaft",
    build: shaft,
};
static STAGE_PART: PartDef = PartDef {
    name: "royal sentinel stage",
    build: stage,
};
static LOOKOUT: PartDef = PartDef {
    name: "sentinel lookout",
    build: lookout,
};
static LOOKOUT_ROOF: PartDef = PartDef {
    name: "sentinel lookout roof",
    build: lookout_roof,
};
static DECK: PartDef = PartDef {
    name: "sentinel deck",
    build: deck,
};
static BALLISTA: PartDef = PartDef {
    name: "sentinel ballista",
    build: ballista,
};

/// Top of the keep.
const KEEP_TOP: f32 = 1.97;
/// The Sentinel watchtower: its depth on the wall, its shaft height, and
/// the extra stage the Royal Sentinel adds.
const SENTRY_Z: f32 = -0.98;
const SENTRY_TOP: f32 = 1.95;
const STAGE: f32 = 0.5;

/// The Royal Sentinel ballista: its mount on the stage and its bolt tip.
pub(super) fn turret() -> super::aim::Turret {
    super::aim::Turret {
        pivot: Vec3::new(0.0, SENTRY_TOP + STAGE + 0.22, SENTRY_Z),
        rest_pitch: 0.0,
        muzzle: Vec3::new(0.47, 0.24, 0.0),
        shot: crate::vfx::shots::ShotKind::Ballista,
        flash: crate::vfx::impacts::Muzzle::Bolt,
        orbit: false,
        arc: std::f32::consts::PI,
    }
}

pub(super) fn pieces(form: Form) -> Vec<Piece> {
    let mut p = vec![whole(&WALLS), whole(&MERLONS).lost(1), whole(&KEEP).lost(2)];
    if form.a() {
        p.push(whole(&DRUMS));
    }
    if form == Form::A2 {
        p.push(whole(&BUTTRESSES));
        p.push(whole(&CAPS).lost(1));
    }
    if form.b() {
        p.push(whole(&SHAFT));
    }
    let top = t(0.0, SENTRY_TOP, SENTRY_Z);
    let staged = t(0.0, SENTRY_TOP + STAGE, SENTRY_Z);
    match form {
        Form::B1 => p.extend([
            piece(&LOOKOUT, top).lost(1),
            piece(&LOOKOUT_ROOF, top).lost(1),
        ]),
        Form::B2 => p.extend([
            whole(&STAGE_PART).lost(1),
            piece(&DECK, staged).lost(1),
            piece(&BALLISTA, Transform::from_translation(turret().pivot))
                .lost(1)
                .aimed()
                .moving(Motion::Aim),
        ]),
        _ => {}
    }
    p
}

/// Battered plinth, a thick wall block, a gate, and team kite shields.
fn walls(team: Color) -> Vec<Part> {
    let mut p = vec![
        (Box(1.9, 0.3, 2.5), ROCK, t(0.0, 0.15, 0.0)),
        (Box(1.6, 0.95, 2.2), STONE, t(0.0, 0.775, 0.0)),
        (Box(1.66, 0.08, 2.26), ROCK, t(0.0, 0.55, 0.0)),
        (Box(1.7, 0.1, 2.3), ROCK, t(0.0, 1.3, 0.0)),
        (Box(0.5, 0.6, 0.05), WOOD, t(0.0, 0.6, 1.11)),
        (Box(0.54, 0.05, 0.06), IRON, t(0.0, 0.45, 1.13)),
        (Box(0.54, 0.05, 0.06), IRON, t(0.0, 0.75, 1.13)),
    ];
    for z in [-0.7, 0.0, 0.7] {
        for x in [-0.82, 0.82] {
            // Each shield faces out of its own wall face.
            let out = Quat::from_rotation_y(f32::signum(x) * 1.571);
            p.extend(shield(team, t(x, 0.88, z).with_rotation(out)));
        }
    }
    for x in [-0.5, 0.5] {
        p.extend(shield(team, t(x, 0.88, 1.12)));
    }
    p.extend(gatehouse(team));
    p
}

/// The lobby portrait's dress: an arched gate up a flight of steps, and
/// round corner turrets with conical roofs and team pennants.
fn gatehouse(team: Color) -> Vec<Part> {
    let up = Quat::from_rotation_x(-std::f32::consts::FRAC_PI_2);
    let mut p = vec![
        (Box(0.72, 0.1, 0.3), ROCK, t(0.0, 0.05, 1.4)),
        (Box(0.72, 0.2, 0.14), ROCK, t(0.0, 0.1, 1.3)),
        (
            Ring(0.045, 0.27, 180.0),
            ROCK,
            t(0.0, 0.9, 1.12).with_rotation(up),
        ),
        (Box(0.08, 0.6, 0.08), ROCK, t(-0.29, 0.6, 1.13)),
        (Box(0.08, 0.6, 0.08), ROCK, t(0.29, 0.6, 1.13)),
    ];
    for (x, z) in [(-0.78, -1.08), (-0.78, 1.08), (0.78, -1.08), (0.78, 1.08)] {
        p.push((Cyl(0.16, 0.5), STONE, t(x, 1.55, z)));
        p.push((Cyl(0.18, 0.05), ROCK, t(x, 1.8, z)));
        p.push((Cone(0.21, 0.34), RUST, t(x, 1.99, z)));
        if z > 0.0 {
            p.push((Cyl(0.012, 0.3), WOOD, t(x, 2.26, z)));
            p.push((Box(0.15, 0.09, 0.012), team, t(x + 0.075, 2.34, z)));
        }
    }
    p
}

fn shield(team: Color, at: Transform) -> [Part; 3] {
    let tip = at.mul_transform(t(0.0, -0.24, 0.0).with_rotation(Quat::from_rotation_z(0.785)));
    [
        (Box(0.34, 0.34, 0.05), team, at),
        (Box(0.2, 0.2, 0.05), team, tip),
        (
            Cyl(0.07, 0.04),
            STEEL,
            at.mul_transform(along_z(0.0, 0.0, 0.03)),
        ),
    ]
}

fn merlons(_team: Color) -> Vec<Part> {
    let mut p = Vec::new();
    for i in 0..5 {
        let z = -1.0 + i as f32 * 0.5;
        for x in [-0.72, 0.72] {
            p.push((Box(0.24, 0.26, 0.24), STONE, t(x, 1.48, z)));
        }
    }
    for x in [-0.36, 0.0, 0.36] {
        for z in [-1.03, 1.03] {
            p.push((Box(0.24, 0.26, 0.24), STONE, t(x, 1.48, z)));
        }
    }
    p
}

/// The central keep, crenellated, its team flag flying.
fn keep(team: Color) -> Vec<Part> {
    let mut p = vec![
        (Box(0.9, 0.62, 1.0), STONE, t(0.0, 1.66, 0.0)),
        (Box(1.0, 0.08, 1.1), ROCK, t(0.0, KEEP_TOP, 0.0)),
        (Box(0.08, 0.22, 0.04), DARK, t(-0.2, 1.7, 0.51)),
        (Box(0.08, 0.22, 0.04), DARK, t(0.2, 1.7, 0.51)),
        (Cyl(0.02, 0.6), WOOD, t(-0.25, KEEP_TOP + 0.3, -0.3)),
        (Box(0.3, 0.18, 0.015), team, t(-0.1, KEEP_TOP + 0.5, -0.3)),
        (Ball(0.03), GOLD, t(-0.25, KEEP_TOP + 0.62, -0.3)),
    ];
    for (x, z) in [(-0.4, -0.45), (-0.4, 0.45), (0.4, -0.45), (0.4, 0.45)] {
        p.push((Box(0.17, 0.17, 0.17), STONE, t(x, KEEP_TOP + 0.12, z)));
    }
    p
}

/// Fortress: drum towers closing both ends of the wall and iron plates.
fn drums(_team: Color) -> Vec<Part> {
    let mut p = Vec::new();
    for z in [-1.3, 1.3] {
        p.push((Cyl(0.56, 0.3), ROCK, t(0.0, 0.15, z)));
        p.push((Cyl(0.5, 1.5), STONE, t(0.0, 0.85, z)));
        p.push((Cyl(0.53, 0.08), ROCK, t(0.0, 1.6, z)));
        for at in around(6, 0.42, 1.74) {
            p.push((
                Box(0.2, 0.22, 0.2),
                STONE,
                at.with_translation(at.translation + Vec3::Z * z),
            ));
        }
    }
    for x in [-0.96, 0.96] {
        p.push((Box(0.06, 0.24, 2.3), IRON, t(x, 0.18, 0.0)));
        for z in [-0.35, 0.35] {
            p.push((Box(0.05, 0.4, 0.3), IRON, t(x * 0.86, 0.5, z)));
        }
    }
    p
}

/// Imperial Fortress: stone buttresses leaning on both long faces.
fn buttresses(_team: Color) -> Vec<Part> {
    let mut p = Vec::new();
    for z in [-0.95, -0.35, 0.35, 0.95] {
        for (x, lean) in [(0.88, 0.3), (-0.88, -0.3)] {
            p.push((
                Box(0.28, 1.2, 0.22),
                ROCK,
                t(x, 0.6, z).with_rotation(Quat::from_rotation_z(lean)),
            ));
        }
    }
    p
}

/// Imperial Fortress: conical drum roofs and a great banner on the keep.
fn caps(team: Color) -> Vec<Part> {
    let mut p = vec![
        (Box(0.5, 0.7, 0.03), team, t(0.0, 1.5, 0.53)),
        (
            Box(0.16, 0.16, 0.035),
            GOLD,
            t(0.0, 1.6, 0.55).with_rotation(Quat::from_rotation_z(0.785)),
        ),
    ];
    for z in [-1.3, 1.3] {
        p.push((Cone(0.58, 0.6), RUST, t(0.0, 2.15, z)));
        p.push((Cyl(0.02, 0.3), WOOD, t(0.0, 2.55, z)));
        p.push((Box(0.22, 0.14, 0.02), team, t(0.11, 2.62, z)));
    }
    p
}

/// Sentinel: a square stone watchtower rising from the back of the wall.
fn shaft(team: Color) -> Vec<Part> {
    vec![
        (
            Box(0.62, SENTRY_TOP, 0.62),
            STONE,
            t(0.0, SENTRY_TOP / 2.0, SENTRY_Z),
        ),
        (Box(0.66, 0.1, 0.66), team, t(0.0, 1.62, SENTRY_Z)),
        (Box(0.08, 0.22, 0.04), DARK, t(-0.12, 1.2, SENTRY_Z + 0.32)),
        (Box(0.08, 0.22, 0.04), DARK, t(0.12, 1.2, SENTRY_Z + 0.32)),
    ]
}

/// Sentinel: an open timber lookout on the shaft (origin at its top).
fn lookout(_team: Color) -> Vec<Part> {
    let mut p = vec![(Box(0.85, 0.08, 0.85), WOOD, t(0.0, 0.04, 0.0))];
    for (x, z) in [(-0.38, -0.38), (-0.38, 0.38), (0.38, -0.38), (0.38, 0.38)] {
        p.push((Box(0.05, 0.36, 0.05), WOOD, t(x, 0.26, z)));
    }
    p
}

fn lookout_roof(team: Color) -> Vec<Part> {
    vec![
        (Cone(0.6, 0.38), WOOD, t(0.0, 0.6, 0.0)),
        (Cyl(0.02, 0.2), WOOD, t(0.0, 0.85, 0.0)),
        (Box(0.18, 0.12, 0.02), team, t(0.09, 0.89, 0.0)),
    ]
}

/// Royal Sentinel: one more stone stage on the shaft.
fn stage(_team: Color) -> Vec<Part> {
    vec![
        (
            Box(0.56, STAGE, 0.56),
            STONE,
            t(0.0, SENTRY_TOP + STAGE / 2.0, SENTRY_Z),
        ),
        (
            Box(0.6, 0.06, 0.6),
            ROCK,
            t(0.0, SENTRY_TOP + 0.03, SENTRY_Z),
        ),
    ]
}

/// Royal Sentinel: a crenellated stone fighting deck (origin at its base).
fn deck(team: Color) -> Vec<Part> {
    let mut p = vec![
        (Cyl(0.66, 0.1), ROCK, t(0.0, 0.12, 0.0)),
        (Cyl(0.68, 0.05), team, t(0.0, 0.05, 0.0)),
    ];
    for at in around(8, 0.58, 0.26) {
        p.push((Box(0.15, 0.2, 0.15), STONE, at));
    }
    p
}

/// A ballista on the deck, its bolt pointing at the enemy (+X).
fn ballista(_team: Color) -> Vec<Part> {
    vec![
        (Box(0.08, 0.16, 0.08), WOOD, t(0.0, 0.08, 0.0)),
        (Box(0.62, 0.07, 0.1), OAK, t(0.05, 0.18, 0.0)),
        (Box(0.06, 0.06, 0.8), WOOD, t(0.24, 0.2, 0.0)),
        (
            Box(0.04, 0.04, 0.2),
            WOOD,
            t(0.19, 0.2, 0.38).with_rotation(Quat::from_rotation_y(0.6)),
        ),
        (
            Box(0.04, 0.04, 0.2),
            WOOD,
            t(0.19, 0.2, -0.38).with_rotation(Quat::from_rotation_y(-0.6)),
        ),
        (Cyl(0.02, 0.62), STEEL, along_x(0.08, 0.24, 0.0)),
        (Cone(0.04, 0.1), STEEL, along_x(0.42, 0.24, 0.0)),
    ]
}
