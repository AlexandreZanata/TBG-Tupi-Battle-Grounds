//! Atomic immutable history writes and durable account preferences on the existing store worker.

use super::AccountId;
use rusqlite::{params, Connection, OptionalExtension};
use serde::{Deserialize, Serialize};
use tbg_core::protocol::{PersonalMatchSummary, ResultId};

mod query;
pub use query::{detail, list, Query, Subject};
pub mod backfill;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct HistorySide {
    pub entry_id: [u8; 16],
    pub account: Option<AccountId>,
    pub name: String,
    pub summary: PersonalMatchSummary,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Envelope {
    pub format: u16,
    pub key: String,
    pub result_id: Option<ResultId>,
    pub ranked_key: Option<[u8; 16]>,
    pub ranked_row: Option<i64>,
    pub winner: Option<u8>,
    pub sides: [HistorySide; 2],
}

#[derive(Debug, PartialEq, Eq)]
pub enum Error {
    Store,
    Unauthorized,
    Conflict,
    Invalid,
    NotFound,
    Private,
    Revision,
}
impl From<rusqlite::Error> for Error {
    fn from(error: rusqlite::Error) -> Self {
        if error.sqlite_error_code() == Some(rusqlite::ErrorCode::ConstraintViolation) {
            Self::Conflict
        } else {
            Self::Store
        }
    }
}

pub fn insert(conn: &mut Connection, e: &Envelope) -> Result<(), Error> {
    let payload = serde_json::to_string(e).map_err(|_| Error::Invalid)?;
    if e.format != 1
        || payload.len() > 32768
        || e.key.len() > 96
        || e.sides[0].summary.mode != e.sides[1].summary.mode
        || e.sides[0].summary.origin != e.sides[1].summary.origin
        || e.sides
            .iter()
            .any(|s| s.name.is_empty() || s.name.len() > 64)
        || e.sides[0].summary.ended_unix != e.sides[1].summary.ended_unix
        || e.sides[0].account.is_some() && e.sides[0].account == e.sides[1].account
    {
        return Err(Error::Invalid);
    }
    let tx = conn.transaction_with_behavior(rusqlite::TransactionBehavior::Immediate)?;
    let previous: Option<String> = tx
        .query_row(
            "SELECT payload_json FROM player_match_history WHERE history_key=?1",
            [&e.key],
            |r| r.get(0),
        )
        .optional()?;
    if let Some(previous) = previous {
        // Random participant IDs are frozen in the journal and checked on every retry.
        return if previous == payload {
            Ok(())
        } else {
            Err(Error::Conflict)
        };
    }
    let summary = &e.sides[0].summary;
    let origin = serde_json::to_value(summary.origin).map_err(|_| Error::Invalid)?;
    let mode = match summary.mode {
        tbg_core::protocol::MatchMode::Casual => "casual",
        tbg_core::protocol::MatchMode::Ranked => "ranked",
        tbg_core::protocol::MatchMode::Challenge => "challenge",
    };
    use sha2::{Digest, Sha256};
    let hash = Sha256::digest(payload.as_bytes());
    tx.execute("INSERT INTO player_match_history(history_key,result_id,ranked_match_key,ranked_row_id,mode,origin,ended_unix,combat_ticks,wall_ms,winner_seat,end_reason,arena_id,ruleset_version,source_hash,summary_version,quality,canonical_hash,payload_json) VALUES(?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?11,?12,?13,?14,?15,?16,?17,?18)", params![
        e.key, e.result_id.as_ref().map(|v| v.as_slice()), e.ranked_key.as_ref().map(|v| v.as_slice()),e.ranked_row,mode, origin.as_str(),summary.ended_unix,
        summary.combat_ticks,summary.wall_ms.and_then(|n| i64::try_from(n).ok()), e.winner,summary.reason,summary.arena,summary.ruleset_version,summary.source_hash,summary.version,summary.quality,hash.as_slice(),payload])?;
    for (seat, side) in e.sides.iter().enumerate() {
        if let Some(account) = side.account {
            tx.execute("INSERT INTO player_match_history_sides(entry_id,history_key,seat,account_id,name_at_match,ended_unix) VALUES(?1,?2,?3,?4,?5,?6)",params![side.entry_id.as_slice(),e.key,seat as i64,account,side.name,summary.ended_unix])?;
        }
    }
    tx.commit()?;
    Ok(())
}

pub fn preference(conn: &Connection, account: AccountId) -> Result<serde_json::Value, Error> {
    conn.query_row("SELECT history_visibility,history_privacy_revision FROM accounts WHERE id=?1",[account], |r| Ok(serde_json::json!({"visibility": r.get::<_,String>(0)?, "revision":r.get::<_,i64>(1)?}))).optional()?.ok_or(Error::NotFound)
}

pub fn set_preference(
    conn: &mut Connection,
    account: AccountId,
    visibility: &str,
    revision: i64,
) -> Result<serde_json::Value, Error> {
    if !matches!(visibility, "private" | "public") || revision < 0 {
        return Err(Error::Invalid);
    }
    let tx = conn.transaction_with_behavior(rusqlite::TransactionBehavior::Immediate)?;
    if tx.execute("UPDATE accounts SET history_visibility=?1,history_privacy_revision=history_privacy_revision+1 WHERE id=?2 AND history_privacy_revision=?3 AND history_privacy_revision < 9223372036854775807",params![visibility,account,revision])? != 1 { return Err(Error::Revision); }
    let saved = preference(&tx, account)?;
    tx.commit()?;
    Ok(saved)
}
