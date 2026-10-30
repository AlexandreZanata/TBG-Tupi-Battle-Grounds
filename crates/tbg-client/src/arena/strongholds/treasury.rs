//! Treasury: a fortified trading post around a round vault door. Branch A
//! (Armored Vault) grows the door and bolts on iron; branch B (Mint) adds
//! coin presses, gears and gold.

use super::parts::{along_x, along_z, gear, piece, t, whole, Form, Motion, PartDef, Piece};
use crate::arena::Part;
use crate::art::{mesh::Shape::*, palette::*};
use bevy::prelude::*;

/// Terracotta roof tiles and their darker eave.
const TILE: Color = Color::srgb(0.69, 0.35, 0.22);
const TILE_EDGE: Color = Color::srgb(0.48, 0.24, 0.16);

static HOUSE: PartDef = PartDef {
    name: "treasury house",
    build: house,
};
static DOOR: PartDef = PartDef {
    name: "treasury vault door",
    build: door,
};
static ROOF: PartDef = PartDef {
    name: "treasury roof",
    build: roof,
};
static GOODS: PartDef = PartDef {
    name: "treasury goods",
    build: goods,
};
static BANDS: PartDef = PartDef {
    name: "armored vault bands",
    build: bands,
};
static TURRETS: PartDef = PartDef {
    name: "grand vault turrets",
    build: turrets,
};
static PRESS: PartDef = PartDef {
    name: "mint press",
    build: press,
};
static LEVER: PartDef = PartDef {
    name: "mint press lever",
    build: lever,
};
static GEAR: PartDef = PartDef {
    name: "mint gear",
    build: |_| gear(0.26, BRASS),
};
static FORGE: PartDef = PartDef {
    name: "royal mint forge",
    build: forge,
};
static MEDALLION: PartDef = PartDef {
    name: "royal mint medallion",
    build: medallion,
};

/// Where the round door hangs; its piece scales up on the Armored Vault.
const DOOR_AT: Vec3 = Vec3::new(0.0, 0.6, 0.655);

pub(super) fn pieces(form: Form) -> Vec<Piece> {
    let door_scale = match form {
        Form::A1 => 1.2,
        Form::A2 => 1.4,
        _ => 1.0,
    };
    let mut p = vec![
        whole(&HOUSE),
        piece(
            &DOOR,
            Transform::from_translation(DOOR_AT).with_scale(Vec3::splat(door_scale)),
        ),
        whole(&ROOF).lost(1),
    ];
    // The Mint's presses take the storage corners.
    if !form.b() {
        p.push(whole(&GOODS).lost(2));
    }
    if form.a() {
        p.push(whole(&BANDS));
    }
    if form == Form::A2 {
        p.push(whole(&TURRETS));
    }
    if form.b() {
        p.extend(mint(0.62));
        p.push(
            piece(&GEAR, along_x(0.8, 0.62, -0.1))
                .lost(2)
                .moving(Motion::Turn(0.9)),
        );
    }
    if form == Form::B2 {
        p.extend(mint(-0.62));
        p.push(whole(&FORGE).lost(1));
        p.push(
            piece(&MEDALLION, t(0.0, 2.72, 0.0))
                .lost(1)
                .moving(Motion::Turn(0.8)),
        );
    }
    p
}

/// One screw press in front of the house at `x`, its lever turning.
fn mint(x: f32) -> [Piece; 2] {
    [
        piece(&PRESS, t(x, 0.0, 0.98)),
        piece(&LEVER, t(x, 1.02, 0.98))
            .lost(2)
            .moving(Motion::Turn(1.6)),
    ]
}

fn house(team: Color) -> Vec<Part> {
    let mut p = vec![
        (Box(1.8, 0.2, 1.5), ROCK, t(0.0, 0.1, 0.0)),
        (Box(1.5, 0.9, 1.25), STONE, t(0.0, 0.65, 0.0)),
        // Team band under the eaves, seen from every side and every damage state.
        (Box(1.56, 0.12, 1.31), team, t(0.0, 1.04, 0.0)),
        (Box(0.28, 0.24, 0.05), BRASS, t(0.0, 1.04, 0.66)),
    ];
    for (x, z) in [(-0.72, -0.6), (-0.72, 0.6), (0.72, -0.6), (0.72, 0.6)] {
        p.push((Box(0.18, 1.0, 0.18), ROCK, t(x, 0.65, z)));
    }
    for x in [-0.5, 0.5] {
        p.push((Cyl(0.05, 0.14), BRASS, t(x, 0.92, 0.66)));
    }
    p.extend(trimmings(team));
    p
}

/// The trading post's dress: gold finials and team pennants on the
/// pillars, team banners flanking the vault, entry steps, gold-framed
/// windows and striped market awnings over the goods.
fn trimmings(team: Color) -> Vec<Part> {
    let mut p = vec![
        (Box(0.72, 0.1, 0.18), STONE, t(0.0, 0.05, 0.84)),
        (Box(0.72, 0.04, 0.2), ROCK, t(0.0, 0.12, 0.8)),
    ];
    for (x, z) in [(-0.72, -0.6), (-0.72, 0.6), (0.72, -0.6), (0.72, 0.6)] {
        p.push((Cone(0.075, 0.2), GOLD, t(x, 1.25, z)));
        p.push((Ball(0.035), GOLD, t(x, 1.37, z)));
    }
    for x in [-0.72, 0.72] {
        p.push((Cyl(0.012, 0.36), WOOD, t(x, 1.52, 0.6)));
        p.push((
            Box(0.17, 0.1, 0.012),
            team,
            t(x + 0.085 * x.signum(), 1.63, 0.6),
        ));
    }
    for x in [-0.5, 0.5] {
        p.push((Box(0.2, 0.46, 0.02), team, t(x, 0.64, 0.645)));
        p.push((
            Box(0.08, 0.08, 0.025),
            GOLD,
            t(x, 0.7, 0.655).with_rotation(Quat::from_rotation_z(0.785)),
        ));
        p.push((Box(0.2, 0.03, 0.025), GOLD, t(x, 0.42, 0.655)));
    }
    for x in [-0.755, 0.755] {
        for z in [-0.3, 0.3] {
            p.push((Box(0.02, 0.26, 0.2), BRASS, t(x, 0.72, z)));
            p.push((Box(0.03, 0.2, 0.14), DARK, t(x * 1.006, 0.72, z)));
        }
    }
    for x in [-0.72, 0.72] {
        for (i, dz) in [-0.12, 0.0, 0.12].into_iter().enumerate() {
            let stripe = if i % 2 == 0 { RED_CLOTH } else { LINEN };
            p.push((
                Box(0.42, 0.02, 0.13),
                stripe,
                t(x, 0.82, 0.92 + dz).with_rotation(Quat::from_rotation_x(0.35)),
            ));
        }
        for dx in [-0.18, 0.18] {
            p.push((Box(0.03, 0.62, 0.03), WOOD, t(x + dx, 0.5, 1.06)));
        }
    }
    p
}

/// Bronze-framed round iron door with a gold hub and three spokes.
fn door(_team: Color) -> Vec<Part> {
    let mut p = vec![
        (Box(0.74, 0.74, 0.05), BRASS, t(0.0, 0.0, -0.02)),
        (Cyl(0.31, 0.06), GUNMETAL, along_z(0.0, 0.0, 0.0)),
        (Cyl(0.08, 0.06), GOLD, along_z(0.0, 0.0, 0.05)),
    ];
    for a in [0.0, 1.047, 2.094] {
        p.push((
            Box(0.5, 0.04, 0.03),
            STEEL,
            t(0.0, 0.0, 0.04).with_rotation(Quat::from_rotation_z(a)),
        ));
    }
    p
}

/// Terracotta tiles on an eight-sided hipped roof, a stone drum with
/// windows, and a gold dome with a lantern spire (lobby portrait).
fn roof(_team: Color) -> Vec<Part> {
    let flat = Vec3::new(1.0, 1.0, 0.84);
    let mut p = vec![
        (Box(1.62, 0.08, 1.36), ROCK, t(0.0, 1.14, 0.0)),
        (
            Frustum(0.36, 1.05, 0.03),
            TILE_EDGE,
            t(0.0, 1.19, 0.0).with_scale(flat),
        ),
        (
            Frustum(0.34, 1.02, 0.42),
            TILE,
            t(0.0, 1.39, 0.0).with_scale(flat),
        ),
        (Cyl(0.26, 0.2), STONE, t(0.0, 1.68, 0.0)),
        (Cyl(0.28, 0.04), GOLD, t(0.0, 1.79, 0.0)),
        (
            Ball(0.26),
            GOLD,
            t(0.0, 1.8, 0.0).with_scale(Vec3::new(1.0, 0.85, 1.0)),
        ),
        (Cyl(0.05, 0.12), GOLD, t(0.0, 2.08, 0.0)),
        (Cone(0.06, 0.16), GOLD, t(0.0, 2.22, 0.0)),
    ];
    for a in [0.4f32, 1.97, 3.54, 5.11] {
        let at = t(a.cos() * 0.255, 1.68, a.sin() * 0.255).with_rotation(Quat::from_rotation_y(-a));
        p.push((Box(0.02, 0.1, 0.07), DARK, at));
    }
    p
}

/// Crates, a coin chest and grain sacks on the plinth corners.
fn goods(_team: Color) -> Vec<Part> {
    vec![
        (Box(0.28, 0.28, 0.28), WOOD, t(-0.72, 0.34, 0.86)),
        (Box(0.2, 0.2, 0.2), TIMBER, t(-0.72, 0.58, 0.86)),
        (Box(0.34, 0.2, 0.24), OAK, t(0.72, 0.3, 0.86)),
        (Box(0.36, 0.05, 0.26), GOLD, t(0.72, 0.42, 0.86)),
        (Cyl(0.06, 0.08), GOLD, t(0.66, 0.48, 0.86)),
        (Ball(0.13), LINEN, t(-0.95, 0.3, 0.45)),
    ]
}

/// Armored Vault: riveted iron bands around the vault house.
fn bands(_team: Color) -> Vec<Part> {
    let mut p = vec![
        (Box(1.54, 0.07, 1.29), IRON, t(0.0, 0.36, 0.0)),
        (Box(1.54, 0.07, 1.29), IRON, t(0.0, 0.84, 0.0)),
    ];
    for x in [-0.68, -0.6, 0.6, 0.68] {
        for y in [0.36, 0.84] {
            p.push((Box(0.05, 0.05, 0.05), DARK, t(x, y, 0.66)));
        }
    }
    p
}

/// Grand Vault: iron corner turrets and side plates, a gold halo on the door.
fn turrets(_team: Color) -> Vec<Part> {
    let mut p = vec![(Ring(0.035, 0.5, 360.0), GOLD, along_z(0.0, DOOR_AT.y, 0.72))];
    for (x, z) in [(-0.76, -0.62), (-0.76, 0.62), (0.76, -0.62), (0.76, 0.62)] {
        p.push((Cyl(0.21, 1.45), GUNMETAL, t(x, 0.92, z)));
        p.push((Cone(0.27, 0.38), IRON, t(x, 1.83, z)));
    }
    for x in [-0.77, 0.77] {
        p.push((Box(0.05, 0.75, 0.95), IRON, t(x, 0.62, 0.0)));
    }
    p
}

/// Mint: a timber screw press on a stone pad with a coin stack.
fn press(_team: Color) -> Vec<Part> {
    vec![
        (Box(0.5, 0.2, 0.36), ROCK, t(0.0, 0.1, 0.0)),
        (Box(0.07, 0.75, 0.07), WOOD, t(-0.18, 0.57, 0.0)),
        (Box(0.07, 0.75, 0.07), WOOD, t(0.18, 0.57, 0.0)),
        (Box(0.5, 0.1, 0.12), WOOD, t(0.0, 0.94, 0.0)),
        (Box(0.24, 0.1, 0.2), DARK, t(0.0, 0.25, 0.0)),
        (Cyl(0.13, 0.05), BRASS, t(0.0, 0.5, 0.0)),
        (Cyl(0.035, 0.45), IRON, t(0.0, 0.74, 0.0)),
        (Cyl(0.06, 0.1), GOLD, t(0.3, 0.25, 0.08)),
    ]
}

fn lever(_team: Color) -> Vec<Part> {
    vec![
        (Box(0.66, 0.035, 0.035), IRON, Transform::IDENTITY),
        (Ball(0.05), BRASS, t(0.33, 0.0, 0.0)),
        (Ball(0.05), BRASS, t(-0.33, 0.0, 0.0)),
    ]
}

/// Royal Mint: a smelting chimney and a gold ridge on the roof.
fn forge(_team: Color) -> Vec<Part> {
    vec![
        (Box(0.3, 0.8, 0.3), STONE, t(-0.5, 1.75, -0.3)),
        (Cyl(0.13, 0.08), GOLD, t(-0.5, 2.18, -0.3)),
        (Ring(0.03, 0.9, 360.0), GOLD, t(0.0, 1.21, 0.0)),
        (Cyl(0.03, 0.35), BRASS, t(0.0, 2.45, 0.0)),
    ]
}

/// Royal Mint signature: a giant gold coin turning above the dome.
fn medallion(_team: Color) -> Vec<Part> {
    vec![
        (Cyl(0.4, 0.07), GOLD, along_z(0.0, 0.0, 0.0)),
        (Cyl(0.24, 0.09), SAP, along_z(0.0, 0.0, 0.0)),
        (Ring(0.03, 0.4, 360.0), BRASS, along_z(0.0, 0.0, 0.0)),
    ]
}
