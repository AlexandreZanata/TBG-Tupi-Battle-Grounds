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
    /// D-64 (P58 §13.5): the three new Factions, appended in this order.
    Monsters,
    Celestials,
    Insects,
}

impl Faction {
    pub fn index(self) -> usize {
        self as usize
    }

    /// The English name used by the HUD, the stats API and the ranked JSON.
    pub fn name(self) -> &'static str {
        match self {
            Faction::Humans => "Humans",
            Faction::Undead => "Undead",
            Faction::Machines => "Machines",
            Faction::Plants => "Plants",
            Faction::Monsters => "Monsters",
            Faction::Celestials => "Celestials",
            Faction::Insects => "Insects",
        }
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
    // P47 (Proposed, staged): wave D. Same staging rules.
    Oilpress,
    Drumguard,
    // P48-A (Proposed, staged): wave E1. Same staging rules.
    Leechroot,
    CargoCrab,
    // P48-B (Proposed, staged): wave E2. Same staging rules.
    Puppetmaster,
    Smuggler,
    /// Smuggler cargo crate (BR-U08); never in the Shop.
    Crate,
    // D-66 (P59 pilot, Proposed): the Monsters base kinds, appended so older kinds
    // keep their indices. Their authored kits and rules are active in the consolidated Shop roster.
    Bonehorn,
    Ravager,
    Acidmaw,
    Whisperer,
    /// D-72: R01, active in the consolidated roster.
    Bulwark,
    /// D-73: R02, active in the consolidated roster.
    Demolisher,
    /// D-74: R03, active in the consolidated roster.
    Crowcaller,
    /// D-75: R04, active in the consolidated roster.
    Trigger,
    /// D-76: R05, active in the consolidated roster.
    RattleTick,
    /// D-77: R06, active in the consolidated roster.
    Capybara,
    /// D-78: R07, active in the consolidated roster.
    Trident,
    /// D-79: R12, active in the consolidated roster.
    SolarSentinel,
    /// D-80: R13, active in the consolidated roster.
    LuminousOracle,
    /// D-81: R14, active in the consolidated roster.
    CelestialArcher,
    /// D-82: R15, active in the consolidated roster.
    Justiciar,
    /// D-83: R16, active in the consolidated roster.
    Jawguard,
    /// D-84: R17, active in the consolidated roster.
    Mantis,
    /// D-85: R18, active in the consolidated roster.
    Hornet,
    /// D-86: R19, active in the consolidated roster.
    Weaver,
    /// D-87: R20, active in the consolidated roster.
    PyreWarden,
    /// D-88: R21, active in the consolidated roster.
    Drillnest,
    Microdrill,
    Duodrill,
    Megadrill,
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
pub const ALL_KINDS: [UnitKind; 51] = [
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
    UnitKind::Oilpress,
    UnitKind::Drumguard,
    UnitKind::Leechroot,
    UnitKind::CargoCrab,
    UnitKind::Puppetmaster,
    UnitKind::Smuggler,
    UnitKind::Crate,
    UnitKind::Bonehorn,
    UnitKind::Ravager,
    UnitKind::Acidmaw,
    UnitKind::Whisperer,
    UnitKind::Bulwark,
    UnitKind::Demolisher,
    UnitKind::Crowcaller,
    UnitKind::Trigger,
    UnitKind::RattleTick,
    UnitKind::Capybara,
    UnitKind::Trident,
    UnitKind::SolarSentinel,
    UnitKind::LuminousOracle,
    UnitKind::CelestialArcher,
    UnitKind::Justiciar,
    UnitKind::Jawguard,
    UnitKind::Mantis,
    UnitKind::Hornet,
    UnitKind::Weaver,
    UnitKind::PyreWarden,
    UnitKind::Drillnest,
    UnitKind::Microdrill,
    UnitKind::Duodrill,
    UnitKind::Megadrill,
];

impl UnitKind {
    /// Summon kinds: created by abilities, never bought (BR-U08). Beetlings
    /// hatch from Nesters; crates drop from Smugglers.
    pub fn is_summon(self) -> bool {
        matches!(
            self,
            UnitKind::Beetling
                | UnitKind::Crate
                | UnitKind::Microdrill
                | UnitKind::Duodrill
                | UnitKind::Megadrill
        )
    }

    /// Conventional units: everything the lab duels and favors (BR-U08:
    /// summons are never bought, evolved, or counted).
    pub fn is_conventional(self) -> bool {
        !self.is_summon()
    }
}

/// The full conventional roster in `UnitKind` order: the staging pool the
/// laboratory, Practice and authoritative online matches draw from.
#[rustfmt::skip]
pub const FULL_POOL: [UnitKind; 46] = [
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
    UnitKind::Oilpress,
    UnitKind::Drumguard,
    UnitKind::Leechroot,
    UnitKind::CargoCrab,
    UnitKind::Puppetmaster,
    UnitKind::Smuggler,
    UnitKind::Bonehorn,
    UnitKind::Ravager,
    UnitKind::Acidmaw,
    UnitKind::Whisperer,
    UnitKind::Bulwark,
    UnitKind::Demolisher,
    UnitKind::Crowcaller,
    UnitKind::Trigger,
    UnitKind::RattleTick,
    UnitKind::Capybara,
    UnitKind::Trident,
    UnitKind::SolarSentinel,
    UnitKind::LuminousOracle,
    UnitKind::CelestialArcher,
    UnitKind::Justiciar,
    UnitKind::Jawguard,
    UnitKind::Mantis,
    UnitKind::Hornet,
    UnitKind::Weaver,
    UnitKind::PyreWarden,
    UnitKind::Drillnest,
];

/// All playable archetypes, in glossary order.
pub const ARCHETYPES: [Archetype; 5] = [
    Archetype::Tank,
    Archetype::Dps,
    Archetype::Support,
    Archetype::Ranged,
    Archetype::Control,
];

/// Number of Factions: the four original ones and the three of D-64.
pub const FACTION_COUNT: usize = 7;

/// All factions, in glossary order (also their synergy index).
pub const FACTIONS: [Faction; FACTION_COUNT] = [
    Faction::Humans,
    Faction::Undead,
    Faction::Machines,
    Faction::Plants,
    Faction::Monsters,
    Faction::Celestials,
    Faction::Insects,
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

/// The match's arena (ARENAS-G): drawn by the server at pairing, chosen in
/// practice, fixed for the whole match. Order is a serialization contract.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Arena {
    /// Verdant Forest: the neutral reference.
    #[default]
    Forest,
    Tundra,
    Volcanic,
    Ruins,
    Storm,
}

impl Arena {
    pub const ALL: [Arena; 5] = [
        Arena::Forest,
        Arena::Tundra,
        Arena::Volcanic,
        Arena::Ruins,
        Arena::Storm,
    ];

    pub fn index(self) -> usize {
        self as usize
    }

    pub fn from_index(i: usize) -> Option<Arena> {
        Arena::ALL.get(i).copied()
    }

    /// Stable lowercase identifier for URLs, storage and catalogs.
    pub fn slug(self) -> &'static str {
        match self {
            Arena::Forest => "forest",
            Arena::Tundra => "tundra",
            Arena::Volcanic => "volcanic",
            Arena::Ruins => "ruins",
            Arena::Storm => "storm",
        }
    }

    pub fn from_slug(slug: &str) -> Option<Arena> {
        Arena::ALL.into_iter().find(|a| a.slug() == slug)
    }
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
