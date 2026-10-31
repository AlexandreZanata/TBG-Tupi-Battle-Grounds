//! P61 §5: the optional raw summary row, written only after P60's
//! authoritative settlement committed. Insert-or-verify: an identical
//! duplicate is fine, different content for the same match is an error and
//! never overwrites evidence. A failure here never touches ratings.

use rusqlite::{params, Connection, OptionalExtension};

use crate::match_task::summary::{validate, SUMMARY_CAP};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum StatsWriteError {
    Unavailable,
    InvalidSummary,
    ConflictingDuplicate,
}

pub(super) fn save(
    conn: &Connection,
    match_id: i64,
    summary: &serde_json::Value,
) -> Result<(), StatsWriteError> {
    let payload = summary.to_string();
    if payload.len() > SUMMARY_CAP || validate(summary).is_err() {
        return Err(StatsWriteError::InvalidSummary);
    }
    let version = summary["schema"]
        .as_u64()
        .ok_or(StatsWriteError::InvalidSummary)?;
    let stored: Option<String> = conn
        .query_row(
            "SELECT payload FROM ranked_stats WHERE match_id = ?1",
            params![match_id],
            |r| r.get(0),
        )
        .optional()
        .map_err(|_| StatsWriteError::Unavailable)?;
    match stored {
        Some(p) if p == payload => Ok(()),
        Some(_) => Err(StatsWriteError::ConflictingDuplicate),
        None => conn
            .execute(
                "INSERT INTO ranked_stats (match_id, summary_version, payload) VALUES (?1, ?2, ?3)",
                params![match_id, version as i64, payload],
            )
            .map(|_| ())
            .map_err(|_| StatsWriteError::Unavailable),
    }
}
