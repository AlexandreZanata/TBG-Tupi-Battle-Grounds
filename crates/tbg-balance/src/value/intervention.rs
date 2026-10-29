//! One controlled change for the offline value engine: a ruleset knob, an
//! on-hit ability, or a one-sided Energy grant. Every change is applied to a
//! cloned `Ruleset`; the canonical ruleset is never modified.

use tbg_core::content::{ENERGY_SCALE, MIN_INTERVAL};
use tbg_core::{OnHit, Ruleset, UnitKind, ALL_KINDS, UNIT_KINDS};

use crate::knobs::{self, Knob};
use crate::runner::Setup;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Intervention {
    /// Move a registered `Ruleset` knob by `delta` canonical units.
    Knob { knob: Knob, delta: i64 },
    /// Replace the base form's on-hit status.
    OnHit {
        kind: UnitKind,
        value: Option<OnHit>,
    },
    /// Give one participant Energy at a tick; measured as a shadow price.
    Energy {
        seat: usize,
        tick: u32,
        sub_energy: u32,
    },
}

impl Intervention {
    pub fn knob(kind: UnitKind, field: knobs::UnitField, delta: i64) -> Intervention {
        Intervention::Knob {
            knob: Knob::Unit(kind as usize, field),
            delta,
        }
    }

    pub fn form(kind: UnitKind, form: usize, field: knobs::FormField, delta: i64) -> Intervention {
        Intervention::Knob {
            knob: Knob::Form(kind as usize, form, field),
            delta,
        }
    }

    /// The ruleset used by the treated world. `None` means the canonical
    /// ruleset is enough and the change rides in `setup`.
    pub fn rules(self) -> Result<Option<&'static Ruleset>, String> {
        self.rules_from(Ruleset::canon())
    }

    /// Apply the change on top of a baseline ruleset (the lab may pass an
    /// `--apply` variant). `Energy` never changes the ruleset.
    pub fn rules_from(self, base: &Ruleset) -> Result<Option<&'static Ruleset>, String> {
        match self {
            Intervention::Energy { .. } => Ok(None),
            Intervention::Knob { knob, delta } => {
                let mut rules = base.clone();
                let next = knobs::get(&rules, knob) + delta;
                knobs::set(&mut rules, knob, next);
                validate(&rules)?;
                // ponytail: one small leak per candidate, freed at exit; a
                // lifetime would touch every runner and gate caller.
                Ok(Some(Box::leak(Box::new(rules))))
            }
            Intervention::OnHit { kind, value } => {
                let mut rules = base.clone();
                rules.units[kind as usize].on_hit = value;
                validate(&rules)?;
                Ok(Some(Box::leak(Box::new(rules))))
            }
        }
    }

    pub fn setup(self) -> Setup {
        match self {
            Intervention::Energy {
                seat,
                tick,
                sub_energy,
            } => Setup::Energy {
                seat,
                tick,
                sub_energy,
            },
            _ => Setup::None,
        }
    }

    /// The unit a one-sided experiment treats, if any.
    pub fn kind(self) -> Option<UnitKind> {
        match self {
            Intervention::Knob { knob, .. } => match knob {
                Knob::Unit(k, _) | Knob::Form(k, _, _) => Some(ALL_KINDS[k]),
                _ => None,
            },
            Intervention::OnHit { kind, .. } => Some(kind),
            Intervention::Energy { .. } => None,
        }
    }
}

/// Reject changes that would make the simulation's fixed bounds meaningless.
/// This is deliberately weaker than `knobs::valid`: value experiments may
/// test a role-breaking change, but never a negative HP, a zero interval, or
/// a slow of 100 % that would divide by zero.
fn validate(r: &Ruleset) -> Result<(), String> {
    for k in UNIT_KINDS {
        let d = r.def(k);
        if d.cost < ENERGY_SCALE {
            return Err(format!("{k:?}: cost below 1 Energy"));
        }
        if d.hp < 0 || d.damage < 0 || d.heal < 0 {
            return Err(format!("{k:?}: negative HP, damage, or heal"));
        }
        if d.interval < MIN_INTERVAL {
            return Err(format!("{k:?}: interval below the minimum"));
        }
        if d.range < 0 || d.speed < 0 || d.splash < 0 {
            return Err(format!("{k:?}: negative range, speed, or splash"));
        }
        if let Some(OnHit::Slow { bp, .. }) = d.on_hit {
            if bp >= 10_000 {
                return Err(format!("{k:?}: a slow of 100 % divides by zero"));
            }
        }
    }
    if r.stronghold_hp <= 0 || r.stronghold_damage < 0 {
        return Err("Stronghold HP must be positive and damage non-negative".into());
    }
    if r.stronghold_interval < MIN_INTERVAL || r.stronghold_range < 0 {
        return Err("Stronghold interval below the minimum or negative range".into());
    }
    if r.energy_start == 0 || r.energy_cap.is_some_and(|cap| cap < r.energy_start) {
        return Err("Energy start or cap outside its bounds".into());
    }
    for (_, bp) in r.synergies {
        if bp.iter().any(|&v| !(0..=10_000).contains(&v)) {
            return Err("Synergy basis points outside 0..=10 000".into());
        }
    }
    Ok(())
}
