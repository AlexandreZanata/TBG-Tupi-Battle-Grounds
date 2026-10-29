//! Match records (P5 format, unchanged): one row per finished match.
//! Non-critical — a failed insert is logged by the caller, never fatal.

use rusqlite::{params, Connection};
use serde_json::Value;

pub(super) fn insert(conn: &Connection, m: &Value) -> rusqlite::Result<()> {
    let int = |k: &str| m[k].as_i64().unwrap_or(0);
    let text = |k: &str| m[k].as_str().map(str::to_string);
    let seat = |i: usize| m["seats"][i].to_string();
    // D-40 (BR-AR07): six rank bytes per seat; NULL when the line lacks them.
    let arcanas = |i: usize| -> Option<Vec<u8>> {
        let ranks = m["arcanas"][i].as_array()?;
        let bytes: Option<Vec<u8>> = ranks
            .iter()
            .map(|r| u8::try_from(r.as_u64()?).ok())
            .collect();
        bytes.filter(|b| b.len() == 6)
    };
    conn.execute(
        "INSERT INTO matches (ended_at, duration_s, winner, reason, seed, seat_a, seat_b,
            step_us_mean, step_us_max, late_ticks, max_lateness_ms, a_arcanas, b_arcanas)
         VALUES (strftime('%Y-%m-%dT%H:%M:%SZ', 'now'), ?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10,
            ?11, ?12)",
        params![
            int("duration_s"),
            text("winner"),
            text("reason").unwrap_or_default(),
            // SQLite integers are i64; keep the seed's bits.
            m["seed"].as_u64().unwrap_or(0) as i64,
            seat(0),
            seat(1),
            int("step_us_mean"),
            int("step_us_max"),
            int("late_ticks"),
            int("max_lateness_ms"),
            arcanas(0),
            arcanas(1),
        ],
    )?;
    Ok(())
}

pub(super) fn count(conn: &Connection) -> rusqlite::Result<i64> {
    conn.query_row("SELECT count(*) FROM matches", [], |r| r.get(0))
}
