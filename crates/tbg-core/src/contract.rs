//! Match contract identity for releases (P66 §6.5, D-92): the wire epoch
//! and the online ruleset revision plus gameplay source hash a match is
//! admitted under. Computed from `PROTOCOL_VERSION` and the watched
//! gameplay sources (`ruleset_version`), never typed by hand; two builds
//! with equal contracts may play each other, any difference may not.

use crate::protocol::PROTOCOL_VERSION;
use crate::ruleset_version::{ruleset_source_hash, RULESET_VERSION};

/// Wire epoch, ruleset revision and gameplay source hash.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct MatchContract {
    pub wire: u16,
    pub ruleset: u32,
    pub source_hash: u64,
}

impl MatchContract {
    /// This build's contract. Hashes the watched sources: call once and
    /// keep the value, never per tick.
    pub fn current() -> Self {
        MatchContract {
            wire: PROTOCOL_VERSION,
            ruleset: RULESET_VERSION,
            source_hash: ruleset_source_hash(),
        }
    }

    /// Stable text form for admission URLs and release descriptors:
    /// `w<wire>-r<ruleset>-<16 hex>`.
    pub fn id(&self) -> String {
        format!("w{}-r{}-{:016x}", self.wire, self.ruleset, self.source_hash)
    }

    /// Strict inverse of [`MatchContract::id`]; anything else is `None`.
    pub fn parse(id: &str) -> Option<Self> {
        let rest = id.strip_prefix('w')?;
        let (wire, rest) = rest.split_once("-r")?;
        let (ruleset, hash) = rest.split_once('-')?;
        let digits = |s: &str| !s.is_empty() && s.bytes().all(|b| b.is_ascii_digit());
        if !digits(wire) || !digits(ruleset) || hash.len() != 16 {
            return None;
        }
        if !hash.bytes().all(|b| matches!(b, b'0'..=b'9' | b'a'..=b'f')) {
            return None;
        }
        Some(MatchContract {
            wire: wire.parse().ok()?,
            ruleset: ruleset.parse().ok()?,
            source_hash: u64::from_str_radix(hash, 16).ok()?,
        })
    }
}
