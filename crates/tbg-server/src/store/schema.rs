//! SQLite schema and ordered migrations. v1 is the P5 `matches` table
//! (kept byte-compatible so old databases keep their rows); v2 adds the
//! P11 `accounts` table; v3 adds P12 `ranked_matches` plus the rating
//! leaderboard index. Timestamps come from SQLite itself.

use rusqlite::Connection;

const MATCHES_V1: &str = "CREATE TABLE IF NOT EXISTS matches (
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

const RANKED_V3: &str = "CREATE TABLE IF NOT EXISTS ranked_matches (
    id INTEGER PRIMARY KEY,
    ended_at TEXT NOT NULL,
    a_id INTEGER NOT NULL REFERENCES accounts(id),
    b_id INTEGER NOT NULL REFERENCES accounts(id),
    winner INTEGER,
    reason TEXT NOT NULL,
    duration_s INTEGER NOT NULL,
    a_before INTEGER NOT NULL, a_after INTEGER NOT NULL,
    b_before INTEGER NOT NULL, b_after INTEGER NOT NULL,
    a_factions TEXT NOT NULL,
    b_factions TEXT NOT NULL
);
CREATE INDEX IF NOT EXISTS ranked_by_a ON ranked_matches(a_id, id);
CREATE INDEX IF NOT EXISTS ranked_by_b ON ranked_matches(b_id, id);
CREATE INDEX IF NOT EXISTS accounts_by_rating ON accounts(rating DESC, wins DESC, name_lower);";

const ACCOUNTS_V2: &str = "CREATE TABLE IF NOT EXISTS accounts (
    id INTEGER PRIMARY KEY,
    name TEXT NOT NULL,
    name_lower TEXT NOT NULL UNIQUE,
    key_hash BLOB NOT NULL,
    created_at TEXT NOT NULL,
    last_login_at TEXT,
    rating INTEGER NOT NULL DEFAULT 1000,
    peak_rating INTEGER NOT NULL DEFAULT 1000,
    ranked_games INTEGER NOT NULL DEFAULT 0,
    wins INTEGER NOT NULL DEFAULT 0,
    losses INTEGER NOT NULL DEFAULT 0,
    draws INTEGER NOT NULL DEFAULT 0
);";

/// Pragmas plus every missing migration in one transaction. Re-running on
/// a migrated database is a no-op.
pub(super) fn migrate(conn: &mut Connection) -> rusqlite::Result<()> {
    conn.pragma_update(None, "journal_mode", "WAL")?;
    conn.pragma_update(None, "synchronous", "NORMAL")?;
    conn.pragma_update(None, "foreign_keys", "ON")?;
    conn.execute_batch("CREATE TABLE IF NOT EXISTS schema_version(version INTEGER NOT NULL)")?;
    let version: i64 = conn.query_row(
        "SELECT COALESCE(MAX(version), 0) FROM schema_version",
        [],
        |r| r.get(0),
    )?;
    let tx = conn.transaction()?;
    if version < 1 {
        tx.execute_batch(MATCHES_V1)?;
        tx.execute("INSERT INTO schema_version(version) VALUES (1)", [])?;
    }
    if version < 2 {
        tx.execute_batch(ACCOUNTS_V2)?;
        tx.execute("INSERT INTO schema_version(version) VALUES (2)", [])?;
    }
    if version < 3 {
        tx.execute_batch(RANKED_V3)?;
        tx.execute("INSERT INTO schema_version(version) VALUES (3)", [])?;
    }
    tx.commit()
}
