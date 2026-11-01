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

/// One post-cutover ranked match of a revision, as stored (P61 §6). The
/// pure aggregation in `crate::stats` decides eligibility from these facts.
#[derive(Clone, Debug, PartialEq)]
pub struct StoredMatch {
    pub id: i64,
    pub accounts: [i64; 2],
    pub winner: Option<usize>,
    pub end_detail: String,
    pub combat_ticks: u32,
    pub ended_unix: i64,
    /// Settlement-time ratings, never today's (residuals).
    pub before: [u16; 2],
    pub risk: u8,
    pub disposition: u8,
    pub reached_running: bool,
    pub payload: Option<String>,
}

/// Bounded rebuild (P61 §5): at most this many matches per report.
pub const COHORT_CAP: usize = 100_000;

/// The stored catalog of `version` and its post-cutover matches, in
/// completion order, read on one (read-only) connection. `Err` when the
/// cohort exceeds `COHORT_CAP`: never publish an undocumented sample.
pub fn read_cohort(
    conn: &Connection,
    version: u32,
) -> rusqlite::Result<(Option<String>, Vec<StoredMatch>)> {
    let catalog: Option<String> = conn
        .query_row(
            "SELECT catalog_json FROM ranked_rulesets WHERE version = ?1",
            params![version],
            |r| r.get(0),
        )
        .optional()?;
    let mut stmt = conn.prepare(
        "SELECT m.id, m.a_id, m.b_id, m.winner, m.end_detail, m.combat_ticks, m.ended_unix,
                m.a_before, m.b_before, i.risk, i.disposition, i.facts_json, s.payload
         FROM ranked_matches m
         JOIN ranked_integrity i ON i.match_id = m.id
         LEFT JOIN ranked_stats s ON s.match_id = m.id
         WHERE m.ruleset_version = ?1 AND m.match_key IS NOT NULL
         ORDER BY m.ended_unix, m.id LIMIT ?2",
    )?;
    let mut rows = stmt.query(params![version, COHORT_CAP as i64 + 1])?;
    let mut out = Vec::new();
    while let Some(row) = rows.next()? {
        if out.len() == COHORT_CAP {
            return Err(rusqlite::Error::QueryReturnedMoreThanOneRow);
        }
        let facts: String = row.get(11)?;
        let reached_running = serde_json::from_str::<serde_json::Value>(&facts)
            .ok()
            .and_then(|f| f["reached_running"].as_bool())
            .unwrap_or(false);
        out.push(StoredMatch {
            id: row.get(0)?,
            accounts: [row.get(1)?, row.get(2)?],
            winner: row.get::<_, Option<i64>>(3)?.map(|w| w as usize),
            end_detail: row.get::<_, Option<String>>(4)?.unwrap_or_default(),
            combat_ticks: row.get::<_, Option<u32>>(5)?.unwrap_or(0),
            ended_unix: row.get::<_, Option<i64>>(6)?.unwrap_or(0),
            before: [row.get(7)?, row.get(8)?],
            risk: row.get(9)?,
            disposition: row.get(10)?,
            reached_running,
            payload: row.get(12)?,
        });
    }
    Ok((catalog, out))
}
