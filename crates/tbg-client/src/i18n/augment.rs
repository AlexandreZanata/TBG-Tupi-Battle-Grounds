//! Augment descriptions: catalog sentence templates filled with the numbers
//! of `AugmentEffect`, so a balance change never needs two catalog edits.
//! The English rendering equals `AugmentDef::text` (contract in `tests`).

use tbg_core::content::{AugmentDef, AugmentEffect, ENERGY_SCALE, TICK_HZ};

use super::{Lang, AUGMENT_TEXT};

/// Basis points as a percentage (`1500` → `15`).
fn pct(lang: Lang, bp: i32) -> String {
    lang.compact(f64::from(bp) / 100.0, 2)
}

fn secs(lang: Lang, ticks: u16) -> String {
    lang.compact(f64::from(ticks) / f64::from(TICK_HZ), 1)
}

/// Sub-energy as Energy (`90` → `0.15`).
fn energy(lang: Lang, sub: u32) -> String {
    lang.compact(f64::from(sub) / f64::from(ENERGY_SCALE), 2)
}

/// Named template arguments for one effect; a template uses a subset.
fn args(lang: Lang, effect: &AugmentEffect) -> Vec<(&'static str, String)> {
    use AugmentEffect as E;
    match *effect {
        E::UnitStat {
            hp_bp,
            dmg_bp,
            atk_bp,
            move_bp,
            heal_bp,
            summon_add,
            ..
        } => vec![
            ("hp", pct(lang, hp_bp)),
            ("dmg", pct(lang, dmg_bp)),
            ("atk", pct(lang, atk_bp)),
            ("move", pct(lang, move_bp)),
            ("heal", pct(lang, heal_bp)),
            ("summon", summon_add.to_string()),
        ],
        E::StrongholdStat { dmg_bp: bp }
        | E::VsStronghold { bp, .. }
        | E::VsArchetype { bp, .. }
        | E::Comeback { bp } => vec![("dmg", pct(lang, bp))],
        E::Bounty { per_kill, cap } => {
            vec![
                ("energy", energy(lang, per_kill)),
                ("cap", energy(lang, cap)),
            ]
        }
        // Discounts never take a price below 1 Energy (`intent.rs`).
        E::EvolveDiscount { amount } => vec![
            ("energy", energy(lang, amount)),
            ("min", energy(lang, ENERGY_SCALE)),
        ],
        E::Lifesteal { bp } | E::StatusDuration { bp } => vec![("pct", pct(lang, bp))],
        E::LastStand {
            below_bp, ticks, ..
        } => vec![("below", pct(lang, below_bp)), ("secs", secs(lang, ticks))],
        E::DeployHaste { move_bp, ticks } => {
            vec![("move", pct(lang, move_bp)), ("secs", secs(lang, ticks))]
        }
        E::KillMend { hp, cap, window } => vec![
            ("hp", hp.to_string()),
            ("cap", cap.to_string()),
            ("secs", secs(lang, window)),
        ],
        E::DamageTakenAura { bp, below_bp, .. } => {
            vec![("pct", pct(lang, bp)), ("below", pct(lang, below_bp))]
        }
        E::HealBonus { bp, below_bp } => {
            vec![("heal", pct(lang, bp)), ("below", pct(lang, below_bp))]
        }
        E::TowerFury {
            bp,
            max_stacks,
            ticks,
        } => vec![
            ("dmg", pct(lang, bp)),
            ("stacks", max_stacks.to_string()),
            ("secs", secs(lang, ticks)),
        ],
        E::HealCut { bp, ticks } => vec![("pct", pct(lang, bp)), ("secs", secs(lang, ticks))],
        E::Executioner { below_bp, bp } => {
            vec![("dmg", pct(lang, bp)), ("below", pct(lang, below_bp))]
        }
        E::SecondWind => Vec::new(),
        E::Convergence { hp_bp, dmg_bp } => {
            vec![("hp", pct(lang, hp_bp)), ("dmg", pct(lang, dmg_bp))]
        }
    }
}

/// The Augment's localized description with its canonical numbers.
pub fn text(lang: Lang, def: &AugmentDef) -> String {
    let Some(msg) = AUGMENT_TEXT.get(usize::from(def.id)) else {
        return def.text.to_string();
    };
    let values = args(lang, &def.effect);
    let named: Vec<(&str, &dyn std::fmt::Display)> = values
        .iter()
        .map(|(name, value)| (*name, value as &dyn std::fmt::Display))
        .collect();
    lang.tf(*msg, &named)
}
