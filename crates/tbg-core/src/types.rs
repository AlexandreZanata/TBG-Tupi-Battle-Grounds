//! Domain vocabulary shared by every module (docs/GAME-DESIGN.md §2).

use serde::{Deserialize, Serialize};

/// One simulation step (50 ms).
pub type Tick = u32;
/// Unique id of a unit within one match; ids only grow.
pub type UnitId = u32;

/// A player's side. `A` owns the Stronghold at x = 0, `B` the one at x = 16 000.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Seat {
    A,
    B,
}

impl Seat {
    pub const BOTH: [Seat; 2] = [Seat::A, Seat::B];

    pub fn index(self) -> usize {
        match self {
            Seat::A => 0,
            Seat::B => 1,
        }
    }

    pub fn other(self) -> Seat {
        match self {
            Seat::A => Seat::B,
            Seat::B => Seat::A,
        }
    }

    /// Walking direction along the lane: A walks +x, B walks -x.
    pub fn dir(self) -> i32 {
        match self {
            Seat::A => 1,
            Seat::B => -1,
        }
    }
}

/// Mechanical role of a unit (rule BR-U02).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Archetype {
    Tank,
    Dps,
    Support,
    Ranged,
    Control,
}

/// Thematic group defining visual language and synergy rules (rule BR-Y01).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Faction {
    Humans,
    Undead,
    Machines,
    Plants,
}

impl Faction {
    pub fn index(self) -> usize {
        self as usize
    }
}

/// Space a unit occupies on the `BattleLine`: one or two slots (rule BR-U03).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum SlotCount {
    One,
    Two,
}

impl SlotCount {
    pub fn count(self) -> u8 {
        match self {
            SlotCount::One => 1,
            SlotCount::Two => 2,
        }
    }
}

/// The eight units of the MVP starter roster (GAME-DESIGN §5).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum UnitKind {
    Shieldbearer,
    Archer,
    BoneBrute,
    Hexer,
    Sawbot,
    SiegeWalker,
    Bloomkeeper,
    Rootcaller,
}

pub const UNIT_KINDS: [UnitKind; 8] = [
    UnitKind::Shieldbearer,
    UnitKind::Archer,
    UnitKind::BoneBrute,
    UnitKind::Hexer,
    UnitKind::Sawbot,
    UnitKind::SiegeWalker,
    UnitKind::Bloomkeeper,
    UnitKind::Rootcaller,
];

/// All playable archetypes, in glossary order.
pub const ARCHETYPES: [Archetype; 5] = [
    Archetype::Tank,
    Archetype::Dps,
    Archetype::Support,
    Archetype::Ranged,
    Archetype::Control,
];

/// All factions, in glossary order (also their synergy index).
pub const FACTIONS: [Faction; 4] = [
    Faction::Humans,
    Faction::Undead,
    Faction::Machines,
    Faction::Plants,
];

/// Status effect applied to the primary target of a hit.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum OnHit {
    Slow { bp: u16, ticks: u16 },
    Root { ticks: u16 },
}

/// Current status of a unit (statuses never stack).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Status {
    None,
    Slowed { bp: u16, ticks_left: u16 },
    Rooted { ticks_left: u16 },
}
