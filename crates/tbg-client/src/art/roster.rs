//! Figure specs for every unit form: one shared base body plus faction
//! props. Evolutions add or swap props and grow the figure 8% per tier.

use bevy::prelude::*;
use tbg_core::UnitKind;

use super::legacy::{part, BodyPlan, FigureSpec, Joint::*, PartSpec};
use super::mesh::Shape::*;
use super::palette::{
    BARK, BONE, BRASS, DARK, FRAME, GLOW, GUNMETAL, HOOD_GREEN, IRON, LEAF, LEAF_DARK, LEATHER,
    OAK, ORANGE, RED_CLOTH, SKIN, STEEL, TIMBER, TRIM, VIOLET,
};

pub mod blockout;
pub(crate) mod body;
mod humans;
mod machines;
mod plants;
mod undead;

/// Base-body colors.
pub struct Look {
    pub skin: Color,
    pub body: Color,
    pub limbs: Color,
    pub feet: Color,
    pub eyes: Color,
}

impl Look {
    pub const fn new(skin: Color, body: Color, limbs: Color, feet: Color) -> Look {
        Look {
            skin,
            body,
            limbs,
            feet,
            eyes: DARK,
        }
    }
}

/// Which evolution props a form has.
#[derive(Clone, Copy)]
pub struct Form {
    pub a: bool,
    pub b: bool,
    pub t2: bool,
}

impl Form {
    fn of(tier: u8, branch: u8) -> Form {
        Form {
            a: tier >= 1 && branch == 0,
            b: tier >= 1 && branch == 1,
            t2: tier >= 2,
        }
    }
}

/// Default native Archer forms: 0 base, 1 Longbow, 2 Marksman.
pub fn archer_v2(tier: u8) -> super::figure::FigureSpec {
    humans::archer_v2::spec(tier)
}

// ponytail: P16-B placeholder looks, recolored so the new units read as
// their own units on the lane and in Shop portraits until their final art.
/// Hive druid: honey and amber instead of leaf green.
const NESTER: &[(Color, Color)] = &[
    (LEAF, Color::srgb_u8(0xE0, 0x9B, 0x2A)),
    (LEAF_DARK, Color::srgb_u8(0x8A, 0x55, 0x1C)),
];
/// Clockwork wall: iron plates and brass instead of skin and leather.
const RAMPART: &[(Color, Color)] = &[
    (SKIN, Color::srgb_u8(0x5C, 0x63, 0x6B)),
    (LEATHER, Color::srgb_u8(0x3E, 0x44, 0x4C)),
    (TRIM, Color::srgb_u8(0xC8, 0x8F, 0x2E)),
];
/// Beetle shell: dark lacquer instead of bark.
const BEETLING: &[(Color, Color)] = &[
    (BARK, Color::srgb_u8(0x2B, 0x33, 0x24)),
    (LEAF, Color::srgb_u8(0xC8, 0x8F, 0x2A)),
];
/// P44 placeholders: recolored figures until the character phases.
const LONGAXE: &[(Color, Color)] = &[
    (SKIN, Color::srgb_u8(0x8A, 0x6F, 0x4D)),
    (LEATHER, Color::srgb_u8(0x4E, 0x5A, 0x66)),
];
const FARWATCH: &[(Color, Color)] = &[
    (RED_CLOTH, Color::srgb_u8(0x2E, 0x4A, 0x66)),
    (TRIM, Color::srgb_u8(0x7F, 0xA8, 0xC9)),
];
const BOMBARD: &[(Color, Color)] = &[
    (GUNMETAL, Color::srgb_u8(0x3A, 0x3F, 0x45)),
    (BRASS, Color::srgb_u8(0xD9, 0x5A, 0x2B)),
];
/// P45 placeholders: recolored figures until the character phases.
const ROTCAP: &[(Color, Color)] = &[
    (BARK, Color::srgb_u8(0x4A, 0x2E, 0x5A)),
    (LEAF, Color::srgb_u8(0x7F, 0xC9, 0x4A)),
];
const OLDFINN: &[(Color, Color)] = &[
    (SKIN, Color::srgb_u8(0x6B, 0x7B, 0x8A)),
    (LEATHER, Color::srgb_u8(0x2E, 0x4A, 0x5A)),
];
const ANTIDOTE: &[(Color, Color)] = &[
    (LEAF, Color::srgb_u8(0xC9, 0xB8, 0x4A)),
    (LEAF_DARK, Color::srgb_u8(0x6B, 0x8A, 0x2E)),
];
/// P46 placeholders: recolored figures until the character phases.
const SHATTERBACK: &[(Color, Color)] = &[
    (BARK, Color::srgb_u8(0x5A, 0x4A, 0x2E)),
    (LEAF, Color::srgb_u8(0x8A, 0x6B, 0x3A)),
];
const FURNACE: &[(Color, Color)] = &[
    (BONE, Color::srgb_u8(0x3A, 0x2E, 0x2E)),
    (VIOLET, Color::srgb_u8(0x6B, 0x2A, 0x22)),
    (GLOW, Color::srgb_u8(0xD9, 0x5A, 0x2B)),
];
const WEAVER: &[(Color, Color)] = &[
    (BONE, Color::srgb_u8(0x8A, 0x7B, 0x9B)),
    (VIOLET, Color::srgb_u8(0x5A, 0x1E, 0x2A)),
    (GLOW, Color::srgb_u8(0xC8, 0x3A, 0x4A)),
];
/// P47 placeholders: recolored figures until the character phases.
const OILPRESS: &[(Color, Color)] = &[
    (GUNMETAL, Color::srgb_u8(0x2E, 0x2A, 0x26)),
    (BRASS, Color::srgb_u8(0x8A, 0x6B, 0x3A)),
    (ORANGE, Color::srgb_u8(0xC9, 0xA1, 0x3A)),
];
const DRUMGUARD: &[(Color, Color)] = &[
    (SKIN, Color::srgb_u8(0xC9, 0x8A, 0x6B)),
    (TRIM, Color::srgb_u8(0x7F, 0xA8, 0xC9)),
];
/// P48-A placeholders: recolored figures until the character phases.
const LEECHROOT: &[(Color, Color)] = &[
    (BARK, Color::srgb_u8(0x2B, 0x4A, 0x2E)),
    (LEAF, Color::srgb_u8(0x4A, 0x8A, 0x3A)),
];
const CRAB: &[(Color, Color)] = &[
    (GUNMETAL, Color::srgb_u8(0xA8, 0x43, 0x2F)),
    (IRON, Color::srgb_u8(0x6B, 0x2E, 0x22)),
];
/// P48-B placeholders: recolored figures until the character phases.
const PUPPET: &[(Color, Color)] = &[
    (BONE, Color::srgb_u8(0x9B, 0x7B, 0xB8)),
    (VIOLET, Color::srgb_u8(0x2A, 0x22, 0x3A)),
    (GLOW, Color::srgb_u8(0xE0, 0xC2, 0x4A)),
];
const SMUGGLER: &[(Color, Color)] = &[
    (HOOD_GREEN, Color::srgb_u8(0x2E, 0x2A, 0x26)),
    (RED_CLOTH, Color::srgb_u8(0x5C, 0x4A, 0x32)),
    (TRIM, Color::srgb_u8(0xC9, 0x8A, 0x2E)),
];

/// The Smuggler's cargo crate: a banded wooden box, no body (it never
/// moves or attacks; it must read as loot, not as a unit).
fn supply_crate() -> FigureSpec {
    let plan = BodyPlan::SMALL;
    // The hips sit at leg height: the box rests on the ground below them.
    let y = 0.21 - plan.leg;
    let mut parts = vec![part(Hips, Box(0.5, 0.42, 0.5), TIMBER, [0.0, y, 0.0])];
    for dy in [-0.17, 0.17] {
        parts.push(part(Hips, Box(0.53, 0.06, 0.53), OAK, [0.0, y + dy, 0.0]));
    }
    for dx in [-0.16, 0.16] {
        parts.push(part(Hips, Box(0.05, 0.43, 0.53), BRASS, [dx, y, 0.0]));
    }
    FigureSpec { plan, parts }
}

fn recolor(mut spec: FigureSpec, map: &[(Color, Color)]) -> FigureSpec {
    for part in &mut spec.parts {
        if let Some((_, to)) = map.iter().find(|(from, _)| *from == part.color) {
            part.color = *to;
        }
    }
    spec
}

/// Units drawn with the large body plan. Visual only: since D-32 no unit
/// takes more army space than another (the old two-slot units plus Rampart).
pub fn large(kind: UnitKind) -> bool {
    matches!(
        kind,
        UnitKind::BoneBrute | UnitKind::SiegeWalker | UnitKind::Rampart
    )
}

pub fn figure_spec(kind: UnitKind, tier: u8, branch: u8) -> FigureSpec {
    let f = Form::of(tier, branch);
    let mut spec = match kind {
        UnitKind::Shieldbearer => humans::shieldbearer(f),
        UnitKind::Archer => humans::archer(f),
        UnitKind::BoneBrute => undead::bone_brute(f),
        UnitKind::Hexer => undead::hexer(f),
        UnitKind::Sawbot => machines::sawbot(f),
        UnitKind::SiegeWalker => machines::siege_walker(f),
        UnitKind::Bloomkeeper => plants::bloomkeeper(f),
        UnitKind::Rootcaller => plants::rootcaller(f),
        // ponytail: P16-B placeholders reuse the nearest figure; final art for
        // Nester, Rampart, and the Beetling comes in their character phases.
        UnitKind::Nester => recolor(plants::bloomkeeper(f), NESTER),
        UnitKind::Rampart => recolor(humans::shieldbearer(f), RAMPART),
        UnitKind::Beetling => recolor(plants::rootcaller(f), BEETLING),
        // P44 placeholders (blockout recolors); final art in the wave's
        // character phase.
        UnitKind::Longaxe => recolor(humans::shieldbearer(f), LONGAXE),
        UnitKind::Farwatch => recolor(humans::archer(f), FARWATCH),
        UnitKind::Bombard => recolor(machines::siege_walker(f), BOMBARD),
        // P45 placeholders (blockout recolors); final art in the wave's
        // character phase.
        UnitKind::Rotcap => recolor(plants::rootcaller(f), ROTCAP),
        UnitKind::OldFinn => recolor(humans::shieldbearer(f), OLDFINN),
        UnitKind::AntidoteBrewer => recolor(plants::bloomkeeper(f), ANTIDOTE),
        // P46 placeholders (blockout recolors); final art in the wave's
        // character phase.
        UnitKind::Shatterback => recolor(plants::rootcaller(f), SHATTERBACK),
        UnitKind::BloodFurnace => recolor(undead::bone_brute(f), FURNACE),
        UnitKind::Bloodweaver => recolor(undead::hexer(f), WEAVER),
        // P47 placeholders (blockout recolors); final art in the wave's
        // character phase.
        UnitKind::Oilpress => recolor(machines::siege_walker(f), OILPRESS),
        UnitKind::Drumguard => recolor(humans::shieldbearer(f), DRUMGUARD),
        // P48-A placeholders (blockout recolors); final art in the wave's
        // character phase.
        UnitKind::Leechroot => recolor(plants::bloomkeeper(f), LEECHROOT),
        UnitKind::CargoCrab => recolor(machines::sawbot(f), CRAB),
        // P48-B placeholders (blockout recolors); final art in the wave's
        // character phase.
        UnitKind::Puppetmaster => recolor(undead::hexer(f), PUPPET),
        UnitKind::Smuggler => recolor(humans::archer(f), SMUGGLER),
        UnitKind::Crate => supply_crate(),
        // P43: staged kinds render as a plain gray silhouette until their
        // wave lands (art/mod.rs builds every kind in ALL_KINDS).
        #[allow(unreachable_patterns)]
        _ => {
            let plan = BodyPlan::SMALL;
            let look = Look::new(STEEL, STEEL, STEEL, FRAME);
            FigureSpec {
                plan,
                parts: base_body(&plan, &look),
            }
        }
    };
    if large(kind) {
        spec.plan.scale = spec.plan.scale.max(BodyPlan::BIG.scale);
    }
    if kind == UnitKind::Rampart {
        // ponytail: a stretched Shieldbearer at full large scale towers over
        // the Bone Brute; keep the placeholder between human and large.
        spec.plan.scale = 1.25;
    }
    if kind.is_summon() {
        spec.plan.scale *= 0.55;
    }
    // Figurines read 20% larger than life on the lane (toy-diorama style).
    spec.plan.scale *= 1.2 * (1.0 + 0.08 * f32::from(tier.min(2)));
    spec
}

/// Hips, torso, head with eyes, arms, hands, legs, feet.
pub fn base_body(p: &BodyPlan, look: &Look) -> Vec<PartSpec> {
    let (t, arm, leg, h) = (p.torso, p.arm, p.leg, p.head);
    let mut v = vec![
        part(
            Hips,
            Box(t.x * 0.92, 0.13, t.z * 0.92),
            look.body,
            [0.0, 0.0, 0.0],
        ),
        part(Torso, Box(t.x, t.y, t.z), look.body, [0.0, t.y / 2.0, 0.0]),
        part(Head, Ball(h), look.skin, [0.0, 0.0, 0.0]),
    ];
    for s in [-1.0f32, 1.0] {
        v.push(part(
            Head,
            Box(0.035, 0.055, 0.02),
            look.eyes,
            [s * h * 0.36, h * 0.12, h * 0.9],
        ));
    }
    for (upper, fore, hand, leg_j) in [(ArmL, ForearmL, HandL, LegL), (ArmR, ForearmR, HandR, LegR)]
    {
        v.push(part(
            upper,
            Pill(0.062, arm * 0.55),
            look.limbs,
            [0.0, -arm / 2.0, 0.0],
        ));
        v.push(part(
            fore,
            Pill(0.056, arm * 0.5),
            look.limbs,
            [0.0, -arm * 0.45, 0.0],
        ));
        v.push(part(hand, Ball(0.078), look.skin, [0.0, -0.035, 0.0]));
        // Thigh and shin overlap at the knee so two-bone walking bends cleanly.
        v.push(part(
            leg_j,
            Pill(0.078, leg * 0.25),
            look.limbs,
            [0.0, -leg * 0.25, 0.0],
        ));
        v.push(part(
            leg_j,
            Pill(0.072, leg * 0.25),
            look.limbs,
            [0.0, -leg * 0.74, 0.0],
        ));
        v.push(part(
            leg_j,
            Box(0.14, 0.075, 0.2),
            look.feet,
            [0.0, -leg - 0.02, 0.035],
        ));
    }
    v
}
