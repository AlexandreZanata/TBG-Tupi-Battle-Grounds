//! Artillery: an open timber gun deck on stone piers with one big cannon
//! and exposed gears. Branch A (Heavy Cannon) swaps in ever larger barrels
//! with a recoil sled; branch B (Rapid Fire) adds a hopper, more gears and
//! a shot magazine. The barrel kicks and the gears notch on every shot.

use super::aim::Turret;
use super::motion::Gun;
use super::parts::{along_x, along_z, gear, piece, t, whole, Form, Motion, PartDef, Piece};
use crate::arena::Part;
use crate::art::{mesh::Shape::*, palette::*};
use crate::vfx::{impacts::Muzzle, shots::ShotKind};
use bevy::prelude::*;
use tbg_core::{Ruleset, StrongholdKind};

static DECK: PartDef = PartDef {
    name: "artillery deck",
    build: deck,
};
static CARRIAGE: PartDef = PartDef {
    name: "artillery carriage",
    build: carriage,
};
static PARAPET: PartDef = PartDef {
    name: "artillery parapet",
    build: parapet,
};
static GEAR: PartDef = PartDef {
    name: "artillery gear",
    build: |_| gear(0.16, BRASS),
};
static BARREL: PartDef = PartDef {
    name: "artillery barrel",
    build: |_| barrel(0.12, 1.0, 2, BRASS),
};
static HEAVY_BARREL: PartDef = PartDef {
    name: "heavy cannon barrel",
    build: |_| barrel(0.17, 1.15, 3, GUNMETAL),
};
static GREAT_BARREL: PartDef = PartDef {
    name: "great cannon barrel",
    build: |_| barrel(0.24, 1.4, 4, GOLD),
};
static SANDBAGS: PartDef = PartDef {
    name: "heavy cannon sandbags",
    build: sandbags,
};
static BRACING: PartDef = PartDef {
    name: "great cannon bracing",
    build: bracing,
};
static RECOIL_SLED: PartDef = PartDef {
    name: "great cannon recoil sled",
    build: recoil_sled,
};
static HOPPER: PartDef = PartDef {
    name: "rapid fire hopper",
    build: hopper,
};
static MAGAZINE: PartDef = PartDef {
    name: "rapid fire magazine",
    build: magazine,
};
static FLYWHEEL: PartDef = PartDef {
    name: "rapid fire flywheel",
    build: flywheel,
};

/// Muzzle raised a little above the horizon.
const ELEVATION: f32 = 0.12;
/// One gear tooth per shot.
const TOOTH: f32 = std::f32::consts::TAU / 8.0;

/// Barrel radius and length of a form (`barrel` builds them).
fn bore(form: Form) -> (f32, f32) {
    match form {
        Form::A1 => (0.17, 1.15),
        Form::A2 => (0.24, 1.4),
        _ => (0.12, 1.0),
    }
}

/// The aimed gun: trunnion, rest elevation, muzzle (barrel space), shot.
pub(super) fn turret(form: Form) -> Turret {
    let (r, len) = bore(form);
    Turret {
        pivot: Vec3::new(0.05, 1.05 + r, 0.0),
        rest_pitch: ELEVATION,
        muzzle: Vec3::new(len * 0.6 + r * 0.5, 0.0, 0.0),
        shot: match form {
            Form::A2 => ShotKind::GreatCannon,
            Form::B1 | Form::B2 => ShotKind::RapidCannon,
            _ => ShotKind::Cannon,
        },
        flash: if form == Form::A2 {
            Muzzle::GreatCannon
        } else {
            Muzzle::Cannon
        },
    }
}

pub(super) fn pieces(form: Form) -> Vec<Piece> {
    let (barrel, carriage_width) = match form {
        Form::A1 => (&HEAVY_BARREL, 1.3),
        Form::A2 => (&GREAT_BARREL, 1.6),
        _ => (&BARREL, 1.0),
    };
    let trunnion = Transform::from_translation(turret(form).pivot)
        .with_rotation(Quat::from_rotation_z(ELEVATION));
    let mut p = vec![
        whole(&DECK),
        piece(
            &CARRIAGE,
            Transform::from_scale(Vec3::new(1.0, 1.0, carriage_width)),
        )
        .aimed()
        .moving(Motion::Swivel),
        piece(barrel, trunnion)
            .aimed()
            .moving(Motion::Kick(gun(form).kick)),
        gear_at(-0.32, 1.04, 0.36),
        whole(&PARAPET).lost(1),
    ];
    match form {
        Form::A1 => p.push(whole(&SANDBAGS).lost(2).aimed()),
        Form::A2 => p.extend([
            whole(&SANDBAGS).lost(2).aimed(),
            whole(&BRACING),
            whole(&RECOIL_SLED).aimed().moving(Motion::Swivel),
        ]),
        Form::B1 => p.extend([
            whole(&HOPPER).lost(2).aimed().moving(Motion::Swivel),
            gear_at(-0.32, 1.04, -0.36),
        ]),
        Form::B2 => p.extend([
            whole(&HOPPER).lost(2).aimed().moving(Motion::Swivel),
            gear_at(-0.32, 1.04, -0.36),
            whole(&MAGAZINE).lost(2).aimed(),
            piece(&FLYWHEEL, along_z(-0.62, 1.3, 0.3))
                .lost(1)
                .mirrored()
                .moving(Motion::Notch(TOOTH * 0.5)),
        ]),
        Form::Base => {}
    }
    p
}

fn gear_at(x: f32, y: f32, z: f32) -> Piece {
    piece(&GEAR, along_z(x, y, z))
        .lost(2)
        .mirrored()
        .moving(Motion::Notch(TOOTH))
}

/// Recoil of each form: heavy shots kick far and settle slowly, rapid ones
/// are short so a 14-tick cadence still reads shot by shot. The interval is
/// the authoritative one for the form.
pub(super) fn gun(form: Form) -> Gun {
    let (kick, recover) = match form {
        Form::Base => (0.12, 0.45),
        Form::A1 => (0.2, 0.6),
        Form::A2 => (0.3, 0.75),
        Form::B1 => (0.08, 0.35),
        Form::B2 => (0.07, 0.28),
    };
    let branch = u8::from(form.b());
    let stats = tbg_core::stronghold::stats(
        Ruleset::canon(),
        StrongholdKind::Artillery,
        form.tier(),
        branch,
    );
    Gun {
        kick,
        recover,
        interval: f32::from(stats.interval) / tbg_core::content::TICK_HZ as f32,
    }
}

/// Stone piers under an open plank deck, cross-braced, with a team skirt.
fn deck(team: Color) -> Vec<Part> {
    let mut p = vec![
        (Box(1.6, 0.12, 1.6), WOOD, t(0.0, 0.81, 0.0)),
        (Box(1.64, 0.1, 1.64), team, t(0.0, 0.7, 0.0)),
    ];
    for (x, z) in [(-0.6, -0.6), (-0.6, 0.6), (0.6, -0.6), (0.6, 0.6)] {
        p.push((Box(0.28, 0.66, 0.28), ROCK, t(x, 0.33, z)));
    }
    for s in [-1.0, 1.0] {
        for lean in [-0.45, 0.45] {
            let across = Quat::from_rotation_z(lean);
            p.push((
                Box(1.3, 0.07, 0.06),
                TIMBER,
                t(0.0, 0.36, 0.6 * s).with_rotation(across),
            ));
            let along = Quat::from_rotation_x(lean);
            p.push((
                Box(0.06, 0.07, 1.3),
                TIMBER,
                t(0.6 * s, 0.36, 0.0).with_rotation(along),
            ));
        }
    }
    p
}

/// Two cheek boards on an axle with spoked wheels.
fn carriage(_team: Color) -> Vec<Part> {
    let mut p = vec![(Cyl(0.04, 0.6), IRON, along_z(0.1, 1.07, 0.0))];
    for z in [-0.2, 0.2] {
        p.push((Box(0.75, 0.28, 0.06), OAK, t(-0.05, 1.01, z)));
        p.push((Cyl(0.2, 0.06), WOOD, along_z(0.1, 1.07, z * 1.35)));
        p.push((Cyl(0.06, 0.08), IRON, along_z(0.1, 1.07, z * 1.35)));
    }
    p
}

/// A cast barrel along +X around its trunnion, banded with `trim`.
fn barrel(r: f32, len: f32, bands: usize, trim: Color) -> Vec<Part> {
    let front = len * 0.6;
    let mut p = vec![
        (Cyl(r, len), GUNMETAL, along_x(len * 0.1, 0.0, 0.0)),
        (Ball(r * 1.15), GUNMETAL, t(-len * 0.4, 0.0, 0.0)),
        (Ball(r * 0.5), DARK, t(-len * 0.4 - r, 0.0, 0.0)),
        (
            Frustum(r * 1.3, r * 1.05, r),
            GUNMETAL,
            along_x(front, 0.0, 0.0),
        ),
        (Cyl(r * 0.7, 0.02), DARK, along_x(front + r * 0.5, 0.0, 0.0)),
    ];
    for i in 0..bands {
        let x = -len * 0.3 + i as f32 * len * 0.7 / bands as f32;
        p.push((Cyl(r * 1.1, 0.05), trim, along_x(x, 0.0, 0.0)));
    }
    p
}

/// Wicker gabions, powder kegs and a pile of shot at the deck edge.
fn parapet(_team: Color) -> Vec<Part> {
    let mut p = vec![
        (Cyl(0.11, 0.22), OAK, t(-0.6, 0.98, 0.58)),
        (Cyl(0.115, 0.04), IRON, t(-0.6, 1.02, 0.58)),
        (Ball(0.08), DARK, t(-0.6, 0.95, -0.25)),
        (Ball(0.08), DARK, t(-0.6, 0.95, -0.08)),
        (Ball(0.08), DARK, t(-0.6, 1.08, -0.165)),
    ];
    for (x, z) in [(-0.62, -0.62), (0.62, -0.62), (0.62, 0.62)] {
        p.push((Cyl(0.14, 0.32), TIMBER, t(x, 1.03, z)));
    }
    p
}

/// Heavy Cannon: a sandbag berm along the front edge.
fn sandbags(_team: Color) -> Vec<Part> {
    (0..5)
        .map(|i| {
            let z = -0.48 + i as f32 * 0.24;
            (Box(0.18, 0.13, 0.22), LINEN, t(0.72, 0.94, z))
        })
        .collect()
}

/// Great Cannon: iron sleeves on the piers and struts under the deck.
fn bracing(_team: Color) -> Vec<Part> {
    let mut p = Vec::new();
    for (x, z) in [(-0.6, -0.6), (-0.6, 0.6), (0.6, -0.6), (0.6, 0.6)] {
        p.push((Box(0.32, 0.18, 0.32), IRON, t(x, 0.12, z)));
        p.push((Box(0.32, 0.12, 0.32), IRON, t(x, 0.6, z)));
    }
    for z in [-0.3, 0.3] {
        p.push((Box(1.3, 0.08, 0.08), IRON, t(0.0, 0.7, z)));
    }
    p
}

/// Great Cannon: rails and twin recoil cylinders behind the breech.
fn recoil_sled(_team: Color) -> Vec<Part> {
    let mut p = Vec::new();
    for z in [-0.14, 0.14] {
        p.push((Box(1.2, 0.06, 0.06), IRON, t(-0.2, 0.9, z)));
        p.push((Cyl(0.06, 0.5), STEEL, along_x(-0.62, 1.05, z)));
        p.push((Cyl(0.075, 0.05), BRASS, along_x(-0.4, 1.05, z)));
    }
    p.push((Box(0.12, 0.3, 0.42), IRON, t(-0.9, 1.0, 0.0)));
    p
}

/// Rapid Fire: a timber hopper feeding the breech.
fn hopper(_team: Color) -> Vec<Part> {
    vec![
        (Box(0.05, 0.5, 0.05), WOOD, t(-0.3, 1.3, 0.18)),
        (Box(0.05, 0.5, 0.05), WOOD, t(-0.3, 1.3, -0.18)),
        (Frustum(0.2, 0.08, 0.24), WOOD, t(-0.3, 1.52, 0.0)),
        (Ball(0.07), DARK, t(-0.3, 1.66, 0.05)),
        (Ball(0.07), DARK, t(-0.32, 1.66, -0.08)),
    ]
}

/// Advanced Mechanism: a shot magazine tower with a chute into the hopper.
fn magazine(_team: Color) -> Vec<Part> {
    let mut p = vec![(
        Box(0.42, 0.04, 0.14),
        TIMBER,
        t(-0.47, 1.76, 0.0).with_rotation(Quat::from_rotation_z(-0.5)),
    )];
    for (x, z) in [(-0.72, -0.1), (-0.72, 0.1), (-0.54, -0.1), (-0.54, 0.1)] {
        p.push((Box(0.05, 1.1, 0.05), WOOD, t(x, 1.42, z)));
    }
    for i in 0..5 {
        p.push((Cyl(0.075, 0.13), DARK, t(-0.63, 1.0 + i as f32 * 0.16, 0.0)));
    }
    p.push((Box(0.3, 0.06, 0.3), IRON, t(-0.63, 1.98, 0.0)));
    p
}

/// Advanced Mechanism: a spoked brass flywheel with a crank.
fn flywheel(_team: Color) -> Vec<Part> {
    vec![
        (Ring(0.03, 0.28, 360.0), BRASS, Transform::IDENTITY),
        (Box(0.56, 0.04, 0.04), IRON, Transform::IDENTITY),
        (Box(0.04, 0.04, 0.56), IRON, Transform::IDENTITY),
        (Cyl(0.05, 0.08), DARK, Transform::IDENTITY),
        (Cyl(0.02, 0.18), STEEL, t(0.2, 0.09, 0.0)),
    ]
}
