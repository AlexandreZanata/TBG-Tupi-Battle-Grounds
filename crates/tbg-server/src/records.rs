//! Match records in SQLite (WAL): one row per finished match, built from
//! the same JSON summary that is logged. No personal data, no IPs.
//! The only module that touches SQL (ARCHITECTURE §8).

use std::path::Path;
use std::sync::Mutex;

use rusqlite::{params, Connection};
use serde_json::Value;

const SCHEMA: &str = "CREATE TABLE IF NOT EXISTS matches (
    id INTEGER PRIMARY KEY,
    ended_at TEXT NOT NULL,
    duration_s INTEGER NOT NULL,
    winner TEXT,
    reason TEXT NOT NULL,
    seed INTEGER NOT NULL,
    seat_a TEXT NOT NULL,
    seat_b TEXT NOT NULL,
    step_us_mean INTEGER NOT NULL,
    step_us_max INTEGER NOT NULL,
    late_ticks INTEGER NOT NULL,
    max_lateness_ms INTEGER NOT NULL
);";

pub struct Records {
    conn: Mutex<Connection>,
}

impl Records {
    pub fn open(path: &Path) -> rusqlite::Result<Records> {
        let conn = Connection::open(path)?;
        conn.pragma_update(None, "journal_mode", "WAL")?;
        conn.execute_batch(SCHEMA)?;
        Ok(Records {
            conn: Mutex::new(conn),
        })
    }

    /// Insert one match summary (the `match_end` JSON line).
    pub fn insert(&self, m: &Value) -> rusqlite::Result<()> {
        let int = |k: &str| m[k].as_i64().unwrap_or(0);
        let text = |k: &str| m[k].as_str().map(str::to_string);
        let seat = |i: usize| m["seats"][i].to_string();
        let conn = self
            .conn
            .lock()
            .map_err(|_| rusqlite::Error::InvalidQuery)?;
        conn.execute(
            "INSERT INTO matches (ended_at, duration_s, winner, reason, seed, seat_a, seat_b,
                step_us_mean, step_us_max, late_ticks, max_lateness_ms)
             VALUES (strftime('%Y-%m-%dT%H:%M:%SZ', 'now'), ?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)",
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
            ],
        )?;
        Ok(())
    }

    pub fn count(&self) -> rusqlite::Result<i64> {
        let conn = self
            .conn
            .lock()
            .map_err(|_| rusqlite::Error::InvalidQuery)?;
        conn.query_row("SELECT count(*) FROM matches", [], |r| r.get(0))
    }
}
