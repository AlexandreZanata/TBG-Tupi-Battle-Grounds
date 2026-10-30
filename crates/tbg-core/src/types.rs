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

/// Every unit kind (GAME-DESIGN §5): the purchasable roster, then summons.
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
    // P16-B (D-32, Proposed): appended so older kinds keep their indices.
    Nester,
    Rampart,
    /// Summon hatched by a Nester (BR-U08); never in the Shop.
    Beetling,
    // P44 (Proposed, staged): wave A. Appended so older kinds keep their
    // indices; not in the Shop pool until the P53 owner switch.
    Longaxe,
    Farwatch,
    Bombard,
    // P45 (Proposed, staged): wave B. Same staging rules.
    Rotcap,
    OldFinn,
    AntidoteBrewer,
    // P46 (Proposed, staged): wave C. Same staging rules.
    Shatterback,
    BloodFurnace,
    Bloodweaver,
}

/// The staged Shop pool (summons excluded): what offers draw from until the
/// P53 owner switch grows it. Staged kinds exist in `ALL_KINDS` first.
pub const UNIT_KINDS: [UnitKind; 10] = [
    UnitKind::Shieldbearer,
    UnitKind::Archer,
    UnitKind::BoneBrute,
    UnitKind::Hexer,
    UnitKind::Sawbot,
    UnitKind::SiegeWalker,
    UnitKind::Bloomkeeper,
    UnitKind::Rootcaller,
    UnitKind::Nester,
    UnitKind::Rampart,
];

/// Every kind, in `UnitKind` order (content tables, art, audio).
#[rustfmt::skip]
pub const ALL_KINDS: [UnitKind; 20] = [
    UnitKind::Shieldbearer,
    UnitKind::Archer,
    UnitKind::BoneBrute,
    UnitKind::Hexer,
    UnitKind::Sawbot,
    UnitKind::SiegeWalker,
    UnitKind::Bloomkeeper,
    UnitKind::Rootcaller,
    UnitKind::Nester,
    UnitKind::Rampart,
    UnitKind::Beetling,
    UnitKind::Longaxe,
    UnitKind::Farwatch,
    UnitKind::Bombard,
    UnitKind::Rotcap,
    UnitKind::OldFinn,
    UnitKind::AntidoteBrewer,
    UnitKind::Shatterback,
    UnitKind::BloodFurnace,
    UnitKind::Bloodweaver,
];

impl UnitKind {
    /// Summons are created by abilities, never bought (BR-U08).
    pub fn is_summon(self) -> bool {
        self == UnitKind::Beetling
    }

    /// Conventional units: everything the lab duels and favors (BR-U08:
    /// summons are never bought, evolved, or counted).
    pub fn is_conventional(self) -> bool {
        !self.is_summon()
    }
}

/// The full conventional roster in `UnitKind` order: the staging pool the
/// lab and the Practice preview draw from (P46: 19 kinds; grows per wave
/// until the P53 owner switch moves it into `CANON.pool`).
#[rustfmt::skip]
pub const FULL_POOL: [UnitKind; 19] = [
    UnitKind::Shieldbearer,
    UnitKind::Archer,
    UnitKind::BoneBrute,
    UnitKind::Hexer,
    UnitKind::Sawbot,
    UnitKind::SiegeWalker,
    UnitKind::Bloomkeeper,
    UnitKind::Rootcaller,
    UnitKind::Nester,
    UnitKind::Rampart,
    UnitKind::Longaxe,
    UnitKind::Farwatch,
    UnitKind::Bombard,
    UnitKind::Rotcap,
    UnitKind::OldFinn,
    UnitKind::AntidoteBrewer,
    UnitKind::Shatterback,
    UnitKind::BloodFurnace,
    UnitKind::Bloodweaver,
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
