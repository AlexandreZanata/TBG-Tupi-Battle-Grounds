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
}

/// The purchasable roster: the Shop's pool (summons excluded).
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
pub const ALL_KINDS: [UnitKind; 11] = [
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
];

impl UnitKind {
    /// Summons are created by abilities, never bought (BR-U08).
    pub fn is_summon(self) -> bool {
        self == UnitKind::Beetling
    }
}

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

/// Kind of a seat's Stronghold, chosen free before the match and frozen at
/// pairing (GAME-DESIGN §10.4, BR-SK01). Order is a serialization contract.
/// `Classic` is the reference tower every kind is measured against: the
/// tests and the balance lab use it; players never pick it online.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum StrongholdKind {
    #[default]
    Classic,
    Treasury,
    Bastion,
    Artillery,
    Command,
}

impl StrongholdKind {
    /// The four kinds a player can choose (BR-SK01).
    pub const SELECTABLE: [StrongholdKind; 4] = [
        StrongholdKind::Treasury,
        StrongholdKind::Bastion,
        StrongholdKind::Artillery,
        StrongholdKind::Command,
    ];
    /// What a player who never chose fights with (BR-SK01).
    pub const DEFAULT: StrongholdKind = StrongholdKind::Bastion;

    pub fn index(self) -> usize {
        self as usize
    }

    pub fn from_index(i: usize) -> Option<StrongholdKind> {
        [
            StrongholdKind::Classic,
            StrongholdKind::Treasury,
            StrongholdKind::Bastion,
            StrongholdKind::Artillery,
            StrongholdKind::Command,
        ]
        .get(i)
        .copied()
    }

    pub fn selectable(self) -> bool {
        self != StrongholdKind::Classic
    }
}
