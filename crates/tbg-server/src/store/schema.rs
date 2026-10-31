//! SQLite schema and ordered migrations. v1 is the P5 `matches` table
//! (kept byte-compatible so old databases keep their rows); v2 adds the
//! P11 `accounts` table; v3 adds P12 `ranked_matches` plus the rating
//! leaderboard index; v4 (D-40) adds each account's saved Arcana preset
//! and both applied presets on match rows (NULL for older rows); v5/v6 add
//! the Stronghold kind and arena; v7 (P60) adds authoritative ranked
//! records, integrity projections and the append-only decision log.
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
    tx.commit()
}
