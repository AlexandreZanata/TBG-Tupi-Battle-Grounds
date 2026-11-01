//! SQLite schema and ordered migrations. v1 is the P5 `matches` table
//! (kept byte-compatible so old databases keep their rows); v2 adds the
//! P11 `accounts` table; v3 adds P12 `ranked_matches` plus the rating
//! leaderboard index; v4 (D-40) adds each account's saved Arcana preset
//! and both applied presets on match rows (NULL for older rows); v5/v6 add
//! the Stronghold kind and arena; v7 (P60) adds authoritative ranked
//! records, integrity projections and the append-only decision log; v8
//! (P61) adds the optional per-match statistics summary. v9 was allocated
//! twice by parallel fronts: P66 (release directory, audit log, cross-slot
//! account ownership) and P68/D-97 (hash-only auth families). Either
//! variant may already be marked 9, so v10 recognizes both feature table
//! sets by their actual shape, creates whichever is absent and refuses a
//! partial or foreign shape instead of trusting the version number.
//! Timestamps come from SQLite itself, except P60's frozen `ended_unix`.

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

const ARCANAS_V4: &str = "ALTER TABLE accounts ADD COLUMN arcanas BLOB;
ALTER TABLE matches ADD COLUMN a_arcanas BLOB;
ALTER TABLE matches ADD COLUMN b_arcanas BLOB;
ALTER TABLE ranked_matches ADD COLUMN a_arcanas BLOB;
ALTER TABLE ranked_matches ADD COLUMN b_arcanas BLOB;";

/// D-46: the account's chosen Stronghold kind (`StrongholdKind::index`).
const STRONGHOLD_V5: &str = "ALTER TABLE accounts ADD COLUMN stronghold INTEGER;";

/// D-55: the match's arena (`Arena::index`); NULL on rows from before arenas.
const ARENA_V6: &str = "ALTER TABLE matches ADD COLUMN arena INTEGER;";

/// P60 (D-87): authoritative ranked records. Additive only: original
/// before/after columns never change; old rows keep NULL keys and stay out
/// of new analyses. The integrity tables are append/projection records of
/// the same settlement transaction; `ranked_decisions` is append-only.
const RANKED_V7: &str = "CREATE TABLE ranked_rulesets (
  version INTEGER PRIMARY KEY,
  source_hash TEXT NOT NULL UNIQUE,
  catalog_json TEXT NOT NULL CHECK(length(catalog_json)<=65536)
);
ALTER TABLE ranked_matches ADD COLUMN match_key BLOB;
ALTER TABLE ranked_matches ADD COLUMN ruleset_version INTEGER REFERENCES ranked_rulesets(version);
ALTER TABLE ranked_matches ADD COLUMN ruleset_source_hash TEXT;
ALTER TABLE ranked_matches ADD COLUMN ended_unix INTEGER;
ALTER TABLE ranked_matches ADD COLUMN end_detail TEXT;
ALTER TABLE ranked_matches ADD COLUMN combat_ticks INTEGER;
CREATE UNIQUE INDEX ranked_match_key ON ranked_matches(match_key)
  WHERE match_key IS NOT NULL;
CREATE TABLE ranked_integrity (
  match_id INTEGER PRIMARY KEY REFERENCES ranked_matches(id),
  policy_version INTEGER NOT NULL,
  mode TEXT NOT NULL CHECK(mode IN ('shadow','enforce')),
  risk INTEGER NOT NULL CHECK(risk IN (0,1,2)),
  disposition INTEGER NOT NULL CHECK(disposition IN (0,1)),
  reason_bits INTEGER NOT NULL,
  facts_json TEXT NOT NULL CHECK(length(facts_json) <= 8192)
);
CREATE TABLE ranked_sides (
  match_id INTEGER NOT NULL REFERENCES ranked_matches(id),
  seat INTEGER NOT NULL CHECK(seat IN (0,1)),
  account_id INTEGER NOT NULL REFERENCES accounts(id),
  opponent_id INTEGER NOT NULL REFERENCES accounts(id),
  ended_unix INTEGER NOT NULL,
  opponent_mature INTEGER NOT NULL CHECK(opponent_mature IN (0,1)),
  original_delta INTEGER NOT NULL,
  effective_delta INTEGER NOT NULL,
  low_activity INTEGER NOT NULL CHECK(low_activity IN (0,1)),
  early_surrender INTEGER NOT NULL CHECK(early_surrender IN (0,1)),
  PRIMARY KEY(match_id, seat), CHECK(account_id <> opponent_id)
);
CREATE INDEX ranked_side_history ON ranked_sides(account_id, ended_unix, match_id);
CREATE TABLE ranked_baseline (
  account_id INTEGER PRIMARY KEY REFERENCES accounts(id),
  rating INTEGER NOT NULL, peak_rating INTEGER NOT NULL,
  ranked_games INTEGER NOT NULL, wins INTEGER NOT NULL,
  losses INTEGER NOT NULL, draws INTEGER NOT NULL,
  mature_games INTEGER NOT NULL
);
INSERT INTO ranked_baseline
  SELECT id,rating,peak_rating,ranked_games,wins,losses,draws,ranked_games FROM accounts;
ALTER TABLE accounts ADD COLUMN mature_games INTEGER NOT NULL DEFAULT 0;
UPDATE accounts SET mature_games=ranked_games;
CREATE TABLE ranked_meta (
  singleton INTEGER PRIMARY KEY CHECK(singleton=1),
  last_utc INTEGER NOT NULL, revision INTEGER NOT NULL
);
INSERT INTO ranked_meta VALUES(1,0,0);
CREATE TABLE ranked_decisions (
  id INTEGER PRIMARY KEY,
  operation_key TEXT NOT NULL UNIQUE,
  match_id INTEGER NOT NULL REFERENCES ranked_matches(id),
  decided_unix INTEGER NOT NULL, actor TEXT NOT NULL,
  action TEXT NOT NULL CHECK(action IN ('invalidate','clear','review','restore')),
  reason TEXT NOT NULL CHECK(length(reason) BETWEEN 1 AND 512),
  before_json TEXT NOT NULL CHECK(length(before_json)<=4096),
  after_json TEXT NOT NULL CHECK(length(after_json)<=4096)
);
CREATE TRIGGER ranked_decisions_no_update BEFORE UPDATE ON ranked_decisions
BEGIN SELECT RAISE(ABORT,'append-only decision log'); END;
CREATE TRIGGER ranked_decisions_no_delete BEFORE DELETE ON ranked_decisions
BEGIN SELECT RAISE(ABORT,'append-only decision log'); END;";

/// P61 (D-88): one optional raw summary per ranked match, written after the
/// authoritative settlement, plus the per-patch cohort index.
const RANKED_STATS_V8: &str = "CREATE TABLE ranked_stats (
  match_id INTEGER PRIMARY KEY REFERENCES ranked_matches(id),
  summary_version INTEGER NOT NULL,
  payload TEXT NOT NULL CHECK(length(payload)<=16384)
);
CREATE INDEX ranked_stats_cohort ON ranked_matches(ruleset_version, ended_unix, id);";

const RELEASE_V9: &str = "CREATE TABLE release_slots (
  slot TEXT PRIMARY KEY,
  generation TEXT NOT NULL,
  server_build TEXT NOT NULL,
  contract TEXT NOT NULL,
  endpoint TEXT NOT NULL,
  state TEXT NOT NULL,
  canary_pct INTEGER NOT NULL DEFAULT 0,
  ranked_authority INTEGER NOT NULL DEFAULT 0,
  matches_running INTEGER NOT NULL DEFAULT 0,
  results_pending INTEGER NOT NULL DEFAULT 0,
  ranked_inflight INTEGER NOT NULL DEFAULT 0,
  heartbeat_unix INTEGER NOT NULL,
  version INTEGER NOT NULL DEFAULT 1
);
CREATE TABLE release_audit (
  id INTEGER PRIMARY KEY,
  at_unix INTEGER NOT NULL,
  actor TEXT NOT NULL,
  action TEXT NOT NULL,
  slot TEXT NOT NULL,
  detail TEXT NOT NULL
);
CREATE TABLE account_owner (
  account_id INTEGER PRIMARY KEY REFERENCES accounts(id),
  slot TEXT NOT NULL,
  generation TEXT NOT NULL,
  epoch INTEGER NOT NULL,
  heartbeat_unix INTEGER NOT NULL
);
";

/// P66 index, verified on every open like the D-97 ones (idempotent).
const RELEASE_INDEXES: &str =
    "CREATE INDEX IF NOT EXISTS account_owner_by_slot ON account_owner(slot);";

/// One feature's tables and the columns its code reads and writes.
struct Feature {
    tables: &'static str,
    indexes: &'static str,
    shape: &'static [(&'static str, &'static [&'static str])],
}

const RELEASE: Feature = Feature {
    tables: RELEASE_V9,
    indexes: RELEASE_INDEXES,
    shape: &[
        (
            "release_slots",
            &[
                "slot",
                "generation",
                "server_build",
                "contract",
                "endpoint",
                "state",
                "canary_pct",
                "ranked_authority",
                "matches_running",
                "results_pending",
                "ranked_inflight",
                "heartbeat_unix",
                "version",
            ],
        ),
        (
            "release_audit",
            &["id", "at_unix", "actor", "action", "slot", "detail"],
        ),
        (
            "account_owner",
            &[
                "account_id",
                "slot",
                "generation",
                "epoch",
                "heartbeat_unix",
            ],
        ),
    ],
};

const AUTH: Feature = Feature {
    tables: super::auth_sessions::TABLES,
    indexes: super::auth_sessions::INDEXES,
    shape: &[
        (
            "auth_sessions",
            &[
                "id",
                "account_id",
                "device_id",
                "client_kind",
                "keep_signed_in",
                "created_at",
                "expires_at",
                "last_used_at",
                "revoked_at",
                "revoke_reason",
            ],
        ),
        (
            "auth_refresh_tokens",
            &[
                "token_hash",
                "session_id",
                "generation",
                "issued_at",
                "consumed_at",
                "operation_hash",
            ],
        ),
    ],
};

/// A table's column names, or `None` when it does not exist.
fn columns(tx: &Connection, table: &str) -> rusqlite::Result<Option<Vec<String>>> {
    let exists: bool = tx.query_row(
        "SELECT EXISTS(SELECT 1 FROM sqlite_master WHERE type = 'table' AND name = ?1)",
        [table],
        |r| r.get(0),
    )?;
    if !exists {
        return Ok(None);
    }
    let mut stmt = tx.prepare("SELECT name FROM pragma_table_info(?1)")?;
    let names = stmt
        .query_map([table], |r| r.get(0))?
        .collect::<rusqlite::Result<Vec<String>>>()?;
    Ok(Some(names))
}

fn foreign_shape(table: &str) -> rusqlite::Error {
    rusqlite::Error::SqliteFailure(
        rusqlite::ffi::Error::new(rusqlite::ffi::SQLITE_SCHEMA),
        Some(format!("unexpected existing shape for {table}")),
    )
}

/// Create a feature's tables when none exists; accept them only when every
/// table exists with every expected column. A partial set fails closed.
fn ensure(tx: &Connection, feature: &Feature) -> rusqlite::Result<()> {
    let mut present = 0;
    for (table, expected) in feature.shape {
        if let Some(found) = columns(tx, table)? {
            if !expected.iter().all(|c| found.iter().any(|f| f == c)) {
                return Err(foreign_shape(table));
            }
            present += 1;
        }
    }
    if present == 0 {
        tx.execute_batch(feature.tables)?;
    } else if present != feature.shape.len() {
        return Err(foreign_shape(feature.shape[0].0));
    }
    tx.execute_batch(feature.indexes)
}

/// Pragmas plus every missing migration in one transaction. Re-running on
/// a migrated database is a no-op.
pub(super) fn migrate(conn: &mut Connection) -> rusqlite::Result<()> {
    // P66: two slots may create the same new file at once. Switching it to
    // WAL can then lose a lock race that SQLite reports at once, without
    // consulting the busy handler; the migration is idempotent, so retry
    // briefly (about 5 s in total) instead of opening without a store.
    let mut attempt: u64 = 0;
    loop {
        match migrate_once(conn) {
            Err(rusqlite::Error::SqliteFailure(e, _))
                if attempt < 30
                    && matches!(
                        e.code,
                        rusqlite::ErrorCode::DatabaseBusy | rusqlite::ErrorCode::DatabaseLocked
                    ) =>
            {
                attempt += 1;
                std::thread::sleep(std::time::Duration::from_millis(20 + 10 * attempt));
            }
            other => return other,
        }
    }
}

fn migrate_once(conn: &mut Connection) -> rusqlite::Result<()> {
    // P66: two slot processes may share this file; wait for the other
    // writer instead of failing, and migrate under one write lock so both
    // read the same starting version.
    conn.busy_timeout(std::time::Duration::from_secs(5))?;
    conn.pragma_update(None, "journal_mode", "WAL")?;
    conn.pragma_update(None, "synchronous", "NORMAL")?;
    conn.pragma_update(None, "foreign_keys", "ON")?;
    conn.execute_batch("CREATE TABLE IF NOT EXISTS schema_version(version INTEGER NOT NULL)")?;
    let tx = conn.transaction_with_behavior(rusqlite::TransactionBehavior::Immediate)?;
    let version: i64 = tx.query_row(
        "SELECT COALESCE(MAX(version), 0) FROM schema_version",
        [],
        |r| r.get(0),
    )?;
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
    if version < 4 {
        tx.execute_batch(ARCANAS_V4)?;
        tx.execute("INSERT INTO schema_version(version) VALUES (4)", [])?;
    }
    if version < 5 {
        tx.execute_batch(STRONGHOLD_V5)?;
        tx.execute("INSERT INTO schema_version(version) VALUES (5)", [])?;
    }
    if version < 6 {
        tx.execute_batch(ARENA_V6)?;
        tx.execute("INSERT INTO schema_version(version) VALUES (6)", [])?;
    }
    if version < 7 {
        tx.execute_batch(RANKED_V7)?;
        tx.execute("INSERT INTO schema_version(version) VALUES (7)", [])?;
    }
    if version < 8 {
        tx.execute_batch(RANKED_STATS_V8)?;
        tx.execute("INSERT INTO schema_version(version) VALUES (8)", [])?;
    }
    // v9 then v10: whichever v9 variant exists, both feature sets must.
    if version < 10 {
        ensure(&tx, &RELEASE)?;
        ensure(&tx, &AUTH)?;
        if version < 9 {
            tx.execute("INSERT INTO schema_version(version) VALUES (9)", [])?;
        }
        tx.execute("INSERT INTO schema_version(version) VALUES (10)", [])?;
    } else {
        // Index definitions stay verified even on an already combined file.
        tx.execute_batch(RELEASE_INDEXES)?;
        tx.execute_batch(super::auth_sessions::INDEXES)?;
    }
    tx.commit()
}
