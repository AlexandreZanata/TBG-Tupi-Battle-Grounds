//! P16-D offline value engine: contextual marginal value with an Energy
//! numeraire (PVE). The scalar is always match-based; micro-scenarios are
//! deterministic features and diagnostics, and no production rule reads this
//! module.

pub mod estimate;
pub mod intervention;
pub mod measure;
pub mod report;
pub mod scenario;
pub mod validate;

use tbg_core::{OnHit, UnitKind, UnitKind::*};

use crate::knobs::{self, Knob};
use crate::policy::Policy;

use intervention::Intervention;

#[derive(Clone, Copy, Debug)]
pub struct Candidate {
    pub name: &'static str,
    pub intervention: Intervention,
    pub kind: Option<UnitKind>,
}

impl Candidate {
    pub fn policies(self) -> [Policy; 2] {
        match self.kind {
            Some(k) => [Policy::Favor(k), Policy::BotAvoid(k)],
            None => [Policy::Bot, Policy::Bot],
        }
    }
}

/// A fixed, pre-registered set of changes. It is intentionally small: the
/// first run proves the measurement, not the whole roster.
pub fn catalog() -> Vec<Candidate> {
    let unit = |name, kind, field, delta| Candidate {
        name,
        intervention: Intervention::knob(kind, field, delta),
        kind: Some(kind),
    };
    vec![
        unit(
            "shieldbearer_hp_20",
            Shieldbearer,
            knobs::UnitField::Hp,
            130,
        ),
        unit("archer_damage_20", Archer, knobs::UnitField::Damage, 9),
        unit(
            "archer_range_plus_1",
            Archer,
            knobs::UnitField::Range,
            1_000,
        ),
        unit(
            "sawbot_interval_minus_10",
            Sawbot,
            knobs::UnitField::Interval,
            -2,
        ),
        unit(
            "siegewalker_splash_plus_1",
            SiegeWalker,
            knobs::UnitField::Splash,
            1_000,
        ),
        unit(
            "bloomkeeper_heal_20",
            Bloomkeeper,
            knobs::UnitField::Heal,
            8,
        ),
        unit("rampart_hp_20", Rampart, knobs::UnitField::Hp, 300),
        Candidate {
            name: "rootcaller_root_plus_0_4s",
            intervention: Intervention::OnHit {
                kind: Rootcaller,
                value: Some(OnHit::Root { ticks: 24 }),
            },
            kind: Some(Rootcaller),
        },
        Candidate {
            name: "hexer_slow_60",
            intervention: Intervention::OnHit {
                kind: Hexer,
                value: Some(OnHit::Slow {
                    bp: 6_000,
                    ticks: 40,
                }),
            },
            kind: Some(Hexer),
        },
        // Canaries: these must be caught, not balanced around.
        Candidate {
            name: "canary_archer_damage_x2",
            intervention: Intervention::knob(Archer, knobs::UnitField::Damage, 45),
            kind: Some(Archer),
        },
        Candidate {
            name: "canary_shieldbearer_cost_plus_1",
            intervention: Intervention::knob(Shieldbearer, knobs::UnitField::Cost, 600),
            kind: Some(Shieldbearer),
        },
        Candidate {
            name: "global_evolution_cost_minus_1",
            intervention: Intervention::Knob {
                knob: Knob::EvolveCost(0),
                delta: -600,
            },
            kind: None,
        },
    ]
}

/// A small, deterministic interaction set: one row per known family plus a
/// null pair. Each scenario puts the treated units on seat A only.
pub struct Pair {
    pub name: &'static str,
    pub scenario: scenario::Scenario,
    pub a: Intervention,
    pub b: Intervention,
}

pub fn pairs() -> Vec<Pair> {
    vec![
        Pair {
            name: "hp_plus_heal",
            scenario: scenario::Scenario {
                name: "shieldbearer_vs_archer_with_bloomkeeper",
                spawns: vec![
                    scenario::Spawn {
                        tick: 60,
                        seat: tbg_core::Seat::A,
                        kind: Shieldbearer,
                        tier: 0,
                        branch: 0,
                        x: 5_000,
                        hp: None,
                    },
                    scenario::Spawn {
                        tick: 60,
                        seat: tbg_core::Seat::A,
                        kind: Bloomkeeper,
                        tier: 0,
                        branch: 0,
                        x: 4_000,
                        hp: None,
                    },
                    scenario::Spawn {
                        tick: 60,
                        seat: tbg_core::Seat::B,
                        kind: Archer,
                        tier: 0,
                        branch: 0,
                        x: 11_000,
                        hp: None,
                    },
                ],
                horizon: 1_200,
                stronghold_fire: false,
                regen: false,
            },
            a: Intervention::knob(Shieldbearer, knobs::UnitField::Hp, 130),
            b: Intervention::knob(Bloomkeeper, knobs::UnitField::Heal, 8),
        },
        Pair {
            name: "damage_plus_attack_speed",
            scenario: scenario::Scenario {
                name: "archer_vs_shieldbearer",
                spawns: vec![
                    scenario::Spawn {
                        tick: 60,
                        seat: tbg_core::Seat::A,
                        kind: Archer,
                        tier: 0,
                        branch: 0,
                        x: 5_000,
                        hp: None,
                    },
                    scenario::Spawn {
                        tick: 60,
                        seat: tbg_core::Seat::B,
                        kind: Shieldbearer,
                        tier: 0,
                        branch: 0,
                        x: 10_000,
                        hp: None,
                    },
                ],
                horizon: 900,
                stronghold_fire: false,
                regen: false,
            },
            a: Intervention::knob(Archer, knobs::UnitField::Damage, 9),
            b: Intervention::knob(Archer, knobs::UnitField::Interval, -2),
        },
        Pair {
            name: "root_plus_ranged_damage",
            scenario: scenario::Scenario {
                name: "rootcaller_archer_vs_bonebrute",
                spawns: vec![
                    scenario::Spawn {
                        tick: 60,
                        seat: tbg_core::Seat::A,
                        kind: Rootcaller,
                        tier: 0,
                        branch: 0,
                        x: 5_000,
                        hp: None,
                    },
                    scenario::Spawn {
                        tick: 60,
                        seat: tbg_core::Seat::A,
                        kind: Archer,
                        tier: 0,
                        branch: 0,
                        x: 4_000,
                        hp: None,
                    },
                    scenario::Spawn {
                        tick: 60,
                        seat: tbg_core::Seat::B,
                        kind: BoneBrute,
                        tier: 0,
                        branch: 0,
                        x: 10_000,
                        hp: None,
                    },
                ],
                horizon: 1_200,
                stronghold_fire: false,
                regen: false,
            },
            a: Intervention::OnHit {
                kind: Rootcaller,
                value: Some(OnHit::Root { ticks: 24 }),
            },
            b: Intervention::knob(Archer, knobs::UnitField::Damage, 9),
        },
        Pair {
            name: "null_hp_plus_unrelated_range",
            scenario: scenario::Scenario {
                name: "shieldbearer_vs_sawbot",
                spawns: vec![
                    scenario::Spawn {
                        tick: 60,
                        seat: tbg_core::Seat::A,
                        kind: Shieldbearer,
                        tier: 0,
                        branch: 0,
                        x: 5_000,
                        hp: None,
                    },
                    scenario::Spawn {
                        tick: 60,
                        seat: tbg_core::Seat::B,
                        kind: Sawbot,
                        tier: 0,
                        branch: 0,
                        x: 10_000,
                        hp: None,
                    },
                ],
                horizon: 900,
                stronghold_fire: false,
                regen: false,
            },
            a: Intervention::knob(Shieldbearer, knobs::UnitField::Hp, 130),
            b: Intervention::knob(Archer, knobs::UnitField::Range, 1_000),
        },
    ]
}
