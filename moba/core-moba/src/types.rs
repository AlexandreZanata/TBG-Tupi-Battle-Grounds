//! Glossary types of the MOBA mode (`docs/moba/GAME-DESIGN-MOBA.md` §2).

use serde::{Deserialize, Serialize};

/// Number of seats in a match: Blue 0..5, Red 5..10.
pub const SEATS: usize = 10;
pub const TEAM_SIZE: usize = 5;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
pub enum Team {
    Blue,
    Red,
}

impl Team {
    pub const ALL: [Team; 2] = [Team::Blue, Team::Red];

    pub fn index(self) -> usize {
        self as usize
    }

    pub fn enemy(self) -> Team {
        match self {
            Team::Blue => Team::Red,
            Team::Red => Team::Blue,
        }
    }
}

/// Who a unit fights for. Neutral camp monsters are enemies of both teams.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Side {
    Team(Team),
    Neutral,
}

impl Side {
    pub fn hostile(self, other: Side) -> bool {
        match (self, other) {
            (Side::Team(a), Side::Team(b)) => a != b,
            (Side::Neutral, Side::Neutral) => false,
            _ => true,
        }
    }

    pub fn team(self) -> Option<Team> {
        match self {
            Side::Team(t) => Some(t),
            Side::Neutral => None,
        }
    }
}

/// A seat index `0..10`; always assigned by the host, never by message content.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
pub struct Seat(pub u8);

impl Seat {
    pub fn all() -> impl Iterator<Item = Seat> {
        (0..SEATS as u8).map(Seat)
    }

    pub fn valid(self) -> bool {
        (self.0 as usize) < SEATS
    }

    pub fn team(self) -> Team {
        if (self.0 as usize) < TEAM_SIZE {
            Team::Blue
        } else {
            Team::Red
        }
    }

    pub fn role(self) -> Role {
        Role::ALL[self.0 as usize % TEAM_SIZE]
    }

    pub fn of(team: Team, role: Role) -> Seat {
        Seat((team.index() * TEAM_SIZE + role as usize) as u8)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Role {
    Top,
    Jungle,
    Mid,
    Bot,
    Support,
}

impl Role {
    pub const ALL: [Role; 5] = [Role::Top, Role::Jungle, Role::Mid, Role::Bot, Role::Support];

    /// The lane a role's Commander walks in Auto mode (Jungle has none).
    pub fn lane(self) -> Option<Lane> {
        match self {
            Role::Top => Some(Lane::Top),
            Role::Mid => Some(Lane::Mid),
            Role::Bot | Role::Support => Some(Lane::Bot),
            Role::Jungle => None,
        }
    }

    pub fn ranged(self) -> bool {
        matches!(self, Role::Mid | Role::Bot | Role::Support)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Lane {
    Top,
    Mid,
    Bot,
}

impl Lane {
    pub const ALL: [Lane; 3] = [Lane::Top, Lane::Mid, Lane::Bot];
}

/// Where a purchased Troop is sent ("Destino da Tropa").
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Destination {
    Lane(Lane),
    Jungle,
}

impl Destination {
    pub const ALL: [Destination; 4] = [
        Destination::Lane(Lane::Top),
        Destination::Lane(Lane::Mid),
        Destination::Lane(Lane::Bot),
        Destination::Jungle,
    ];
}

/// A purchasable Troop is one of the 1v1 champions (`tbg_core::UnitKind`):
/// both modes share the same roster, stats, names and figures.
pub type Champion = tbg_core::UnitKind;

/// The 1v1 online champion pool the MOBA Shop draws from.
pub const POOL: &[Champion] = &tbg_core::types::FULL_POOL;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum MinionKind {
    Melee,
    Ranged,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum NeutralKind {
    Small,
    Big,
}

/// Tower position in its lane, from the river towards the base, plus the
/// two Nexus towers guarding the Stronghold.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum TowerTier {
    Outer,
    Inner,
    Base,
    Nexus,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum UnitKind {
    Commander(Role),
    Minion(MinionKind),
    Troop(Champion),
    Neutral(NeutralKind),
    /// `None` lane for the Nexus towers.
    Tower(TowerTier, Option<Lane>),
    Stronghold,
}

impl UnitKind {
    pub fn structure(self) -> bool {
        matches!(self, UnitKind::Tower(..) | UnitKind::Stronghold)
    }
}
