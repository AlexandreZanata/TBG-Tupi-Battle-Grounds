//! Versioned personal terminal data; unsupported metrics remain absent, never inferred.

use super::MatchMode;
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MatchOrigin {
    #[default]
    PublicQueue,
    PrivateRoom,
    Invitation,
    Rematch,
    LegacyRanked,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ChampionPurchases {
    pub champion: String,
    pub purchases: u32,
    pub evolution_tiers: [u32; 2],
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct AppliedAugment {
    pub id: u16,
    pub copies: u16,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct TerminalStronghold {
    pub kind: String,
    pub hp: i32,
    pub max_hp: i32,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct PersonalMatchSummary {
    pub version: u16,
    pub quality: String,
    pub mode: MatchMode,
    pub origin: MatchOrigin,
    pub ended_unix: i64,
    pub arena: Option<String>,
    pub ruleset_version: Option<u32>,
    pub source_hash: Option<String>,
    pub combat_ticks: Option<u32>,
    pub tick_hz: u16,
    pub wall_ms: Option<u64>,
    pub outcome: String,
    pub reason: String,
    pub purchases: Option<u32>,
    pub champions: Option<Vec<ChampionPurchases>>,
    pub evolutions: Option<u32>,
    pub fortifications: Option<u32>,
    pub augments: Option<Vec<AppliedAugment>>,
    pub stronghold: Option<TerminalStronghold>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum HistorySaveState {
    Pending,
    Stored,
    Delayed,
}
