//! The defenders of the gunless towers, each animated for its tower's role:
//! a crossbowman pacing the Classic battlement onto his target, a gun port
//! in the Bastion wall whose shutter flies open for a swivel falconet, a
//! roof hatch on the Treasury that opens for a brass mortar lobbing purses
//! of coin, and an archer on the Command platform turning to loose. The
//! Artillery keeps its own gun (`artillery.rs`).

use std::f32::consts::PI;

use bevy::prelude::*;
use tbg_core::StrongholdKind;

use super::aim::Turret;
use super::parts::{along_x, piece, t, Motion, PartDef, Piece};
use crate::arena::Part;
use crate::art::{mesh::Shape::*, palette::*};
use crate::vfx::{impacts::Muzzle, shots::ShotKind};

static SENTRY: PartDef = PartDef {
    name: "classic sentry",
    build: |team| figure(team, STEEL),
};
static CROSSBOW: PartDef = PartDef {
    name: "classic crossbow",
    build: crossbow,
};
static PORT: PartDef = PartDef {
    name: "bastion gun port",
    build: port,
};
static SHUTTER: PartDef = PartDef {
    name: "bastion port shutter",
    build: shutter,
};
static FALCONET: PartDef = PartDef {
    name: "bastion falconet",
    build: falconet,
};
static HATCH: PartDef = PartDef {
    name: "treasury roof hatch",
    build: hatch,
};
static TRAPDOOR: PartDef = PartDef {
    name: "treasury trapdoor",
    build: trapdoor,
};
static MORTAR: PartDef = PartDef {
    name: "treasury coin mortar",
    build: mortar,
};
static ARCHER: PartDef = PartDef {
    name: "command archer",
    build: |team| figure(team, HOOD_GREEN),
};
static LONGBOW: PartDef = PartDef {
    name: "command longbow",
    build: longbow,
};

/// Classic: the sentry's feet on the crown floor, the crossbow at his chest.
const SENTRY_AT: Vec3 = Vec3::new(0.42, 1.86, 0.0);
const CROSSBOW_AT: Vec3 = Vec3::new(0.52, 2.12, 0.0);
/// Bastion: a gun port between the shields of the wall facing the enemy.
const PORT_AT: Vec3 = Vec3::new(0.8, 0.88, 0.35);
/// The falconet waits inside the wall, its muzzle behind the shutter.
const FALCONET_AT: Vec3 = Vec3::new(0.52, 0.88, 0.35);
/// Treasury: a hatch on a dormer at the enemy end of the hipped roof.
const HATCH_AT: Vec3 = Vec3::new(0.6, 1.55, 0.0);
const MORTAR_AT: Vec3 = Vec3::new(0.6, 1.59, 0.0);
/// Command: an archer at the platform rail, clear of the standard's pole.
const ARCHER_AT: Vec3 = Vec3::new(0.72, 1.5, 0.0);
const LONGBOW_AT: Vec3 = Vec3::new(0.8, 1.77, 0.0);

/// The defender pieces a kind adds (every form keeps its defender).
pub(super) fn pieces(kind: StrongholdKind) -> Vec<Piece> {
    match kind {
        StrongholdKind::Classic => vec![
            piece(&SENTRY, t(SENTRY_AT.x, SENTRY_AT.y, SENTRY_AT.z))
                .lost(1)
                .aimed()
                .moving(Motion::Swivel),
            carried(&CROSSBOW, CROSSBOW_AT, 0.03).lost(1),
        ],
        StrongholdKind::Bastion => vec![
            piece(&PORT, t(PORT_AT.x, PORT_AT.y, PORT_AT.z)).aimed(),
            piece(&SHUTTER, hinge(PORT_AT + Vec3::new(0.02, 0.12, 0.0)))
                .aimed()
                .moving(Motion::Hatch(1.3)),
            // Negative recoil: it runs out through the open port, then back.
            fixed(&FALCONET, FALCONET_AT, -0.2),
        ],
        StrongholdKind::Treasury => vec![
            piece(&HATCH, t(HATCH_AT.x, HATCH_AT.y, HATCH_AT.z))
                .lost(1)
                .mirrored(),
            // Two leaves hinged on the hatch's long edges, opening upward.
            piece(&TRAPDOOR, t(HATCH_AT.x, HATCH_AT.y + 0.02, -0.12))
                .lost(1)
                .mirrored()
                .moving(Motion::Hatch(1.9)),
            piece(
                &TRAPDOOR,
                t(HATCH_AT.x, HATCH_AT.y + 0.02, 0.12).with_rotation(Quat::from_rotation_y(PI)),
            )
            .lost(1)
            .mirrored()
            .moving(Motion::Hatch(1.9)),
            fixed(&MORTAR, MORTAR_AT, 0.05).lost(1),
        ],
        StrongholdKind::Command => vec![
            piece(&ARCHER, t(ARCHER_AT.x, ARCHER_AT.y, ARCHER_AT.z))
                .lost(2)
                .aimed()
                .moving(Motion::Swivel),
            carried(&LONGBOW, LONGBOW_AT, 0.025).lost(2),
        ],
        StrongholdKind::Artillery => Vec::new(),
    }
}

/// The aimed weapon of a gunless kind, if its defender still stands.
pub(super) fn turret(kind: StrongholdKind, damage: u8) -> Option<Turret> {
    let aimed = |pivot, orbit, arc, muzzle, (shot, flash)| Turret {
        pivot,
        rest_pitch: 0.0,
        muzzle,
        shot,
        flash,
        orbit,
        arc,
    };
    match kind {
        StrongholdKind::Classic if damage < 1 => Some(aimed(
            CROSSBOW_AT,
            true,
            PI,
            Vec3::new(0.18, 0.02, 0.0),
            (ShotKind::Crossbow, Muzzle::Bolt),
        )),
        StrongholdKind::Bastion => Some(aimed(
            FALCONET_AT,
            false,
            0.9,
            Vec3::new(0.27, 0.0, 0.0),
            (ShotKind::RapidCannon, Muzzle::Bombard),
        )),
        StrongholdKind::Treasury if damage < 1 => Some(Turret {
            rest_pitch: 0.8,
            ..aimed(
                MORTAR_AT,
                false,
                PI,
                Vec3::new(0.16, 0.0, 0.0),
                (ShotKind::Coins, Muzzle::Coin),
            )
        }),
        StrongholdKind::Command if damage < 2 => Some(aimed(
            LONGBOW_AT,
            true,
            PI,
            Vec3::new(0.2, 0.0, 0.0),
            (ShotKind::Crossbow, Muzzle::Bolt),
        )),
        _ => None,
    }
}

/// A weapon in a defender's hands: it walks round the tower with him,
/// aims, and recoils along its barrel.
fn carried(def: &'static PartDef, at: Vec3, kick: f32) -> Piece {
    piece(def, t(at.x, at.y, at.z))
        .aimed()
        .moving(Motion::Mounted(kick))
}

/// A gun on a fixed mount: it aims where it stands and recoils.
fn fixed(def: &'static PartDef, at: Vec3, kick: f32) -> Piece {
    piece(def, t(at.x, at.y, at.z))
        .aimed()
        .moving(Motion::Kick(kick))
}

/// A shutter's hinge: its local Z points out of the +X wall face.
fn hinge(at: Vec3) -> Transform {
    t(at.x, at.y, at.z).with_rotation(Quat::from_rotation_y(PI / 2.0))
}

/// A small defender facing +X, feet at the origin: team tabard, belt,
/// arms reaching for the weapon, and a hat (`hat`).
fn figure(team: Color, hat: Color) -> Vec<Part> {
    let mut p = vec![
        (Box(0.1, 0.17, 0.15), team, t(0.0, 0.27, 0.0)),
        (Box(0.105, 0.025, 0.155), LEATHER, t(0.0, 0.2, 0.0)),
        (Ball(0.058), SKIN, t(0.01, 0.41, 0.0)),
        (
            Ball(0.064),
            hat,
            t(0.0, 0.44, 0.0).with_scale(Vec3::new(1.0, 0.65, 1.0)),
        ),
        (Cyl(0.085, 0.012), hat, t(0.0, 0.425, 0.0)),
    ];
    for z in [-0.04, 0.04] {
        p.push((Box(0.06, 0.19, 0.06), DARK_LEATHER, t(0.0, 0.095, z)));
    }
    for z in [-0.085, 0.085] {
        let reach = Quat::from_rotation_z(-1.0);
        p.push((
            Box(0.045, 0.15, 0.045),
            team,
            t(0.06, 0.3, z).with_rotation(reach),
        ));
    }
    p
}

/// A crossbow along +X: stock, steel prod, string, a bolt on the rail.
fn crossbow(_team: Color) -> Vec<Part> {
    vec![
        (Box(0.24, 0.035, 0.035), WOOD, t(0.03, 0.0, 0.0)),
        (Box(0.025, 0.025, 0.24), STEEL, t(0.14, 0.0, 0.0)),
        (Box(0.008, 0.008, 0.22), LINEN, t(0.09, 0.0, 0.0)),
        (Box(0.14, 0.012, 0.012), TIMBER, t(0.1, 0.025, 0.0)),
    ]
}

/// A dark opening framed in timber, set into the wall face.
fn port(_team: Color) -> Vec<Part> {
    vec![
        (Box(0.04, 0.22, 0.22), DARK, t(0.005, 0.0, 0.0)),
        (Box(0.05, 0.04, 0.3), TIMBER, t(0.01, 0.13, 0.0)),
        (Box(0.05, 0.04, 0.3), TIMBER, t(0.01, -0.13, 0.0)),
    ]
}

/// The port's shutter, hanging from its hinge (local Z points outward).
fn shutter(team: Color) -> Vec<Part> {
    vec![
        (Box(0.24, 0.24, 0.03), OAK, t(0.0, -0.12, 0.0)),
        (Box(0.26, 0.03, 0.035), IRON, t(0.0, -0.04, 0.0)),
        (Box(0.26, 0.03, 0.035), IRON, t(0.0, -0.2, 0.0)),
        (Box(0.08, 0.08, 0.036), team, t(0.0, -0.12, 0.0)),
    ]
}

/// A swivel falconet along +X on a short iron fork.
fn falconet(_team: Color) -> Vec<Part> {
    vec![
        (Cyl(0.04, 0.34), GUNMETAL, along_x(0.1, 0.0, 0.0)),
        (Cyl(0.05, 0.04), BRASS, along_x(0.26, 0.0, 0.0)),
        (Ball(0.05), GUNMETAL, t(-0.07, 0.0, 0.0)),
        (Box(0.04, 0.12, 0.1), IRON, t(0.0, -0.06, 0.0)),
    ]
}

/// The roof hatch: a stone dormer rising out of the tiles, an iron rim
/// round a dark opening on top.
fn hatch(_team: Color) -> Vec<Part> {
    let mut p = vec![
        (Box(0.36, 0.38, 0.36), STONE, t(0.0, -0.2, 0.0)),
        (Box(0.26, 0.02, 0.26), DARK, t(0.0, 0.0, 0.0)),
    ];
    for (x, z, w, d) in [
        (0.0, 0.14, 0.32, 0.04),
        (0.0, -0.14, 0.32, 0.04),
        (0.14, 0.0, 0.04, 0.32),
        (-0.14, 0.0, 0.04, 0.32),
    ] {
        p.push((Box(w, 0.05, d), IRON, t(x, 0.02, z)));
    }
    p
}

/// One trapdoor leaf, hinged along its local X edge, lying toward +Z.
fn trapdoor(_team: Color) -> Vec<Part> {
    vec![
        (Box(0.24, 0.025, 0.12), OAK, t(0.0, 0.0, 0.06)),
        (Box(0.25, 0.03, 0.025), BRASS, t(0.0, 0.0, 0.1)),
        (Ball(0.018), GOLD, t(0.0, 0.02, 0.07)),
    ]
}

/// A squat brass mortar along +X on a ball mount.
fn mortar(_team: Color) -> Vec<Part> {
    vec![
        (Ball(0.075), IRON, t(0.0, 0.0, 0.0)),
        (Cyl(0.065, 0.2), BRASS, along_x(0.07, 0.0, 0.0)),
        (Cyl(0.08, 0.04), GOLD, along_x(0.16, 0.0, 0.0)),
        (Cyl(0.04, 0.02), DARK, along_x(0.185, 0.0, 0.0)),
    ]
}

/// A longbow held upright in front of the archer, an arrow nocked along +X.
fn longbow(_team: Color) -> Vec<Part> {
    vec![
        (
            Box(0.022, 0.22, 0.022),
            WOOD,
            t(0.07, 0.1, 0.0).with_rotation(Quat::from_rotation_z(0.25)),
        ),
        (
            Box(0.022, 0.22, 0.022),
            WOOD,
            t(0.07, -0.1, 0.0).with_rotation(Quat::from_rotation_z(-0.25)),
        ),
        (Box(0.006, 0.42, 0.006), LINEN, t(0.025, 0.0, 0.0)),
        (Box(0.26, 0.01, 0.01), TIMBER, t(0.1, 0.0, 0.0)),
        (Box(0.03, 0.02, 0.02), STEEL, t(0.23, 0.0, 0.0)),
    ]
}
