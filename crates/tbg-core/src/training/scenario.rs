//! Training scenario schema and its atomic validation (P62 §3.3, R06–R07).
//! A scenario names exact unit instances; it never becomes a Shop pool.

use crate::content::{LANE_LEN, SPAWN_OFFSET, TICK_HZ};
use crate::ruleset::CANON;
use crate::types::{Arena, UnitKind};

use super::dummy::{DummyHp, DummyMode};

/// Schema version of `TrainingScenario`; part of a trial's identity.
pub const SCHEMA: u16 = 1;
/// Initial units per side (GAME-DESIGN §16, **Proposed**, O02).
pub const MAX_UNITS_PER_SIDE: usize = 12;
/// Dummies on the target side (**Proposed**, O02).
pub const MAX_DUMMIES: usize = 5;
/// Living-entity safety ceiling, summons included (**Proposed**, O02).
pub const MAX_LIVING: usize = 256;
/// Active run limit: 300 s (**Proposed**, O02). Pauses; never a winner.
pub const RUN_LIMIT_TICKS: u32 = 300 * TICK_HZ;
/// Valid placement bounds: the purchase spawn points of both seats.
pub const LANE_MIN: i32 = SPAWN_OFFSET;
pub const LANE_MAX: i32 = LANE_LEN - SPAWN_OFFSET;

/// One initial instance: a playable kind in one available form, optionally
/// at an explicit lane position (default spacing otherwise).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct TrainingUnitSpec {
    pub kind: UnitKind,
    /// 0 = base, 1 and 2 = Evolution tiers.
    pub tier: u8,
    /// 0 = branch A, 1 = branch B; must be 0 at the base tier.
    pub branch: u8,
    pub x: Option<i32>,
}

impl TrainingUnitSpec {
    pub fn base(kind: UnitKind) -> TrainingUnitSpec {
        TrainingUnitSpec {
            kind,
            tier: 0,
            branch: 0,
            x: None,
        }
    }
}

/// What the allied team fights.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Opponent {
    Champions(Vec<TrainingUnitSpec>),
    /// One dummy per position entry (`None` = default spacing).
    Dummies {
        mode: DummyMode,
        hp: DummyHp,
        positions: Vec<Option<i32>>,
    },
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TrainingScenario {
    pub schema: u16,
    pub seed: u64,
    pub arena: Arena,
    /// Seat A, the player's team.
    pub allies: Vec<TrainingUnitSpec>,
    /// Seat B.
    pub opponent: Opponent,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Side {
    Allies,
    Opponents,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ScenarioError {
    Schema(u16),
    EmptySide(Side),
    TooMany(Side),
    /// Not in the active Shop pool (summons and staged kinds included).
    NotPlayable {
        side: Side,
        index: usize,
    },
    BadForm {
        side: Side,
        index: usize,
    },
    BadPosition {
        side: Side,
        index: usize,
    },
}

impl TrainingScenario {
    /// Validate the whole draft before anything is allocated.
    pub fn validate(&self) -> Result<(), ScenarioError> {
        if self.schema != SCHEMA {
            return Err(ScenarioError::Schema(self.schema));
        }
        validate_units(Side::Allies, &self.allies)?;
        match &self.opponent {
            Opponent::Champions(units) => validate_units(Side::Opponents, units),
            Opponent::Dummies { positions, .. } => {
                bounded(Side::Opponents, positions.len(), MAX_DUMMIES)?;
                for (index, x) in positions.iter().enumerate() {
                    position(Side::Opponents, index, *x)?;
                }
                Ok(())
            }
        }
    }
}

fn validate_units(side: Side, units: &[TrainingUnitSpec]) -> Result<(), ScenarioError> {
    bounded(side, units.len(), MAX_UNITS_PER_SIDE)?;
    for (index, spec) in units.iter().enumerate() {
        if spec.kind.is_summon() || !CANON.pool.contains(&spec.kind) {
            return Err(ScenarioError::NotPlayable { side, index });
        }
        let form_ok = spec.tier <= 2 && spec.branch <= 1 && (spec.tier > 0 || spec.branch == 0);
        if !form_ok {
            return Err(ScenarioError::BadForm { side, index });
        }
        position(side, index, spec.x)?;
    }
    Ok(())
}

fn bounded(side: Side, len: usize, max: usize) -> Result<(), ScenarioError> {
    match len {
        0 => Err(ScenarioError::EmptySide(side)),
        n if n > max => Err(ScenarioError::TooMany(side)),
        _ => Ok(()),
    }
}

fn position(side: Side, index: usize, x: Option<i32>) -> Result<(), ScenarioError> {
    match x {
        Some(x) if !(LANE_MIN..=LANE_MAX).contains(&x) => {
            Err(ScenarioError::BadPosition { side, index })
        }
        _ => Ok(()),
    }
}

/// Deterministic default spacing (**Proposed**): allies fan out behind
/// x = 6,000, opponents behind x = 10,000, dummies from x = 9,000.
pub fn default_x(side: Side, dummy: bool, index: usize) -> i32 {
    let i = index as i32;
    match (side, dummy) {
        (Side::Allies, _) => 6_000 - 350 * i,
        (Side::Opponents, false) => 10_000 + 350 * i,
        (Side::Opponents, true) => 9_000 + 500 * i,
    }
}
