//! Figure specs for every unit form: one shared base body plus faction
//! props. Evolutions add or swap props and grow the figure 8% per tier.

use bevy::prelude::*;
use tbg_core::UnitKind;

use super::legacy::{part, BodyPlan, FigureSpec, Joint::*, PartSpec};
use super::mesh::Shape::*;
use super::palette::{BARK, DARK, FRAME, LEAF, LEAF_DARK, LEATHER, SKIN, STEEL, TRIM};

pub mod blockout;
mod body;
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
    (LEATHER, Color::srgb_u8(0x2E, 0x4A, 0x66)),
    (TRIM, Color::srgb_u8(0x7F, 0xA8, 0xC9)),
];
const BOMBARD: &[(Color, Color)] = &[
    (SKIN, Color::srgb_u8(0x3A, 0x3F, 0x45)),
    (TRIM, Color::srgb_u8(0xD9, 0x5A, 0x2B)),
];

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
        v.push(part(
            leg_j,
            Pill(0.078, leg * 0.5),
            look.limbs,
            [0.0, -leg / 2.0, 0.0],
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
