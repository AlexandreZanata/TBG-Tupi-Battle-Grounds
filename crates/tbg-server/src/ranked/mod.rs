//! Authoritative ranked settlement (P60, D-87): the frozen result envelope a
//! match task produces, the receipt the store returns, and the durable
//! journal plus single finalizer that settle results in completion order.
//! Nothing here is accepted from a client.

pub(crate) mod catalog;
pub(crate) mod finalizer;
pub mod journal;
pub mod policy;
pub mod signals;

use tbg_core::protocol::RatingResults;
use tbg_core::ranked_integrity::{RatingDisposition, RiskState};
use tbg_core::{ArcanaPreset, Outcome};

use crate::store::AccountId;

/// OS-random at pairing; separate from the public `LiveId`.
pub type MatchKey = [u8; 16];

/// The cause that actually ended the match (first terminal cause wins).
/// Stored beside the legacy `reason`, which stays `Forfeit` for both
/// Surrender and Disconnect.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum EndDetail {
    Stronghold,
    Surrender,
    Disconnect,
}

impl EndDetail {
    pub fn as_str(self) -> &'static str {
        match self {
            EndDetail::Stronghold => "stronghold",
            EndDetail::Surrender => "surrender",
            EndDetail::Disconnect => "disconnect",
        }
    }

    pub fn parse(s: &str) -> Option<EndDetail> {
        [
            EndDetail::Stronghold,
            EndDetail::Surrender,
            EndDetail::Disconnect,
        ]
        .into_iter()
        .find(|d| d.as_str() == s)
    }
}

/// Fixed-size per-seat counters from successful commands (P60 §5). Damage
/// is not attributable from current events, so it is absent, never zero.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct IntegritySeat {
    pub purchases: u32,
    pub evolutions: u32,
    pub fortifications: u32,
    pub augments_applied: u32,
    pub reconnects: u32,
    /// Combat-relative tick of a Surrender that ended the match.
    pub surrender_tick: Option<u32>,
    /// Terminal Stronghold progress, not cumulative damage.
    pub final_hp: i32,
    pub final_max_hp: i32,
}

impl IntegritySeat {
    /// No successful purchase, Evolution, Fortification or applied Augment.
    pub fn low_activity(&self) -> bool {
        self.purchases == 0
            && self.evolutions == 0
            && self.fortifications == 0
            && self.augments_applied == 0
    }
}

/// The minimal authoritative envelope of one completed ranked match.
#[derive(Clone, Debug, PartialEq)]
pub struct RankedFinal {
    pub key: MatchKey,
    pub accounts: [AccountId; 2],
    pub outcome: Outcome,
    pub end_detail: EndDetail,
    /// `state.tick - RUNNING_START`, saturating; seconds use `TICK_HZ`.
    pub combat_ticks: u32,
    pub wall_ms: u64,
    pub reached_running: bool,
    pub ruleset_version: u32,
    pub ruleset_source_hash: u64,
    pub seats: [IntegritySeat; 2],
    pub factions: [[u32; 4]; 2],
    pub arcanas: [ArcanaPreset; 2],
    /// P61 §4: the optional, size-bounded side summary; `None` when
    /// collection is off or the summary was a coverage gap.
    pub stats: Option<serde_json::Value>,
}

/// A `RankedFinal` with its completion order and frozen UTC second.
#[derive(Clone, Debug, PartialEq)]
pub struct JournalEntry {
    pub completion_seq: i64,
    pub ended_unix: i64,
    pub result: RankedFinal,
}

/// What a settlement (or an idempotent replay of it) produced.
#[derive(Clone, Debug, PartialEq)]
pub struct RankedReceipt {
    pub key: MatchKey,
    /// The `ranked_matches` id; strictly increasing in completion order.
    pub sequence: i64,
    pub accounts: [AccountId; 2],
    /// Per seat `(before, after, current profile)`.
    pub results: RatingResults,
    pub disposition: RatingDisposition,
    pub risk: RiskState,
    pub reason_bits: u32,
}

/// The binary's gameplay source hash, computed once per process (P61 §3).
pub fn source_hash() -> u64 {
    static HASH: std::sync::OnceLock<u64> = std::sync::OnceLock::new();
    *HASH.get_or_init(tbg_core::ruleset_version::ruleset_source_hash)
}

/// Lowercase hex, for keys and hashes in SQL text and file names.
pub fn hex(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{b:02x}")).collect()
}

pub fn parse_key(hex: &str) -> Option<MatchKey> {
    if hex.len() != 32 {
        return None;
    }
    let mut key = [0u8; 16];
    for (i, byte) in key.iter_mut().enumerate() {
        *byte = u8::from_str_radix(hex.get(2 * i..2 * i + 2)?, 16).ok()?;
    }
    Some(key)
}
