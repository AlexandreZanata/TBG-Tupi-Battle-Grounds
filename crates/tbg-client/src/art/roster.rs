//! Figure specs for every unit form: one shared base body plus faction
//! props. Evolutions add or swap props and grow the figure 8% per tier.

use bevy::prelude::*;
use tbg_core::UnitKind;

use super::legacy::{part, BodyPlan, FigureSpec, Joint::*, PartSpec};
use super::mesh::Shape::*;
use super::palette::DARK;

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
        UnitKind::Nester => plants::bloomkeeper(f),
        UnitKind::Rampart => humans::shieldbearer(f),
        UnitKind::Beetling => plants::rootcaller(f),
    };
    if large(kind) {
        spec.plan.scale = spec.plan.scale.max(BodyPlan::BIG.scale);
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
