//! Match records (P5 format, unchanged): one row per finished match.
//! Non-critical — a failed insert is logged by the caller, never fatal.

use rusqlite::{params, Connection};
use serde_json::Value;

pub(super) fn insert(conn: &Connection, m: &Value) -> rusqlite::Result<()> {
    let int = |k: &str| m[k].as_i64().unwrap_or(0);
    let text = |k: &str| m[k].as_str().map(str::to_string);
    let seat = |i: usize| m["seats"][i].to_string();
    // D-40/P40 (BR-AR07): fifteen rank bytes per seat; NULL when absent.
    let arcanas = |i: usize| -> Option<Vec<u8>> {
        let ranks = m["arcanas"][i].as_array()?;
        let bytes: Option<Vec<u8>> = ranks
            .iter()
            .map(|r| u8::try_from(r.as_u64()?).ok())
            .collect();
        bytes.filter(|b| b.len() == tbg_core::arcana::NODES)
    };
    conn.execute(
        "INSERT INTO matches (ended_at, duration_s, winner, reason, seed, seat_a, seat_b,
            step_us_mean, step_us_max, late_ticks, max_lateness_ms, a_arcanas, b_arcanas, arena)
         VALUES (strftime('%Y-%m-%dT%H:%M:%SZ', 'now'), ?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10,
            ?11, ?12, ?13)",
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
            // D-55: `Arena::index`, NULL when absent.
            m["arena"].as_i64(),
        ],
    )?;
    Ok(())
}

pub(super) fn count(conn: &Connection) -> rusqlite::Result<i64> {
    conn.query_row("SELECT count(*) FROM matches", [], |r| r.get(0))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_arena_is_recorded_and_older_rows_stay_null() {
        // D-55: schema v6 adds a nullable `matches.arena`.
        let mut conn = Connection::open_in_memory().expect("memory db");
        super::super::schema::migrate(&mut conn).expect("migrate");
        let row = |arena: Option<u64>| {
            let mut m = serde_json::json!({
                "duration_s": 10, "winner": "A", "reason": "StrongholdDestroyed",
                "seed": 7, "seats": [{}, {}],
            });
            if let Some(a) = arena {
                m["arena"] = a.into();
            }
            m
        };
        insert(&conn, &row(Some(3))).expect("insert with arena");
        insert(&conn, &row(None)).expect("insert without");
        let arenas: Vec<Option<i64>> = conn
            .prepare("SELECT arena FROM matches ORDER BY id")
            .expect("query")
            .query_map([], |r| r.get(0))
            .expect("rows")
            .map(|r| r.expect("row"))
            .collect();
        assert_eq!(arenas, [Some(3), None]);
        let version: i64 = conn
            .query_row("SELECT MAX(version) FROM schema_version", [], |r| r.get(0))
            .expect("version");
        assert_eq!(version, 6);
    }
}
