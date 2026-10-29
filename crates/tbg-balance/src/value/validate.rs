//! A deliberately crude static baseline for the value engine. If full-match
//! PVE cannot beat this ranking out of sample, the report says the model adds
//! no useful information.

use tbg_core::{OnHit, Ruleset, UnitKind};

use crate::knobs::{self, FormField, Knob, UnitField};

use super::intervention::Intervention;
use super::Candidate;

/// Static marginal power of a candidate: the changed stat ratio times the
/// unit's canonical `HP * damage / interval / cost`.
pub fn static_power(c: &Candidate) -> f64 {
    let canon = Ruleset::canon();
    let kind = c.kind.unwrap_or(UnitKind::Shieldbearer);
    let base = super::estimate::static_power(canon, kind);
    match c.intervention {
        Intervention::Knob { knob, delta } => {
            let current = knobs::get(canon, knob);
            base * ratio(knob, current, current + delta)
        }
        Intervention::OnHit { value, .. } => base * on_hit_ratio(canon, kind, value),
        Intervention::Energy { .. } => 0.0,
    }
}

fn ratio(knob: Knob, base: i64, next: i64) -> f64 {
    let positive = |v: i64| v.max(1) as f64;
    match knob {
        Knob::Unit(_, UnitField::Interval) | Knob::StrongholdInterval => {
            positive(base) / positive(next)
        }
        Knob::Unit(_, UnitField::Cost) | Knob::EvolveCost(_) => positive(base) / positive(next),
        Knob::Form(_, _, FormField::AtkBp) => positive(base) / positive(next),
        Knob::Unit(_, UnitField::Hp)
        | Knob::Unit(_, UnitField::Damage)
        | Knob::Unit(_, UnitField::Range)
        | Knob::Unit(_, UnitField::Speed)
        | Knob::Unit(_, UnitField::Splash)
        | Knob::Unit(_, UnitField::Heal)
        | Knob::Form(..)
        | Knob::StrongholdHp
        | Knob::StrongholdDamage
        | Knob::StrongholdRange
        | Knob::EnergyStart
        | Knob::RegenRunning
        | Knob::InterestBand
        | Knob::InterestPerBand
        | Knob::InterestMaxBands
        | Knob::AugmentRoundTicks
        | Knob::AugmentWindow
        | Knob::AugmentForgo
        | Knob::AugmentCost(_)
        | Knob::Synergy(..)
        | Knob::HatchEvery
        | Knob::HatchEveryHive
        | Knob::BeetleLifetime
        | Knob::Arcana(_) => positive(next) / positive(base),
    }
}

fn on_hit_ratio(canon: &Ruleset, kind: UnitKind, value: Option<OnHit>) -> f64 {
    let before = canon.def(kind).on_hit;
    match (before, value) {
        (Some(OnHit::Root { ticks: a }), Some(OnHit::Root { ticks: b })) => {
            f64::from(b) / f64::from(a.max(1))
        }
        (Some(OnHit::Slow { bp: a, .. }), Some(OnHit::Slow { bp: b, .. })) => {
            f64::from(b) / f64::from(a.max(1))
        }
        (None, Some(_)) => 1.5,
        (Some(_), None) => 0.5,
        _ => 1.0,
    }
}
