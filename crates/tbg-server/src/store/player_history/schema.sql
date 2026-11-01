-- Modern Ranked writers mark an expected complete envelope in their existing settlement transaction.
ALTER TABLE ranked_matches ADD COLUMN history_expected INTEGER NOT NULL DEFAULT 0 CHECK(history_expected IN (0,1));
-- P73 schema 11: additive participant history and revisioned account privacy.
ALTER TABLE accounts ADD COLUMN history_visibility TEXT NOT NULL DEFAULT 'private'
  CHECK (history_visibility IN ('private', 'public'));
ALTER TABLE accounts ADD COLUMN history_privacy_revision INTEGER NOT NULL DEFAULT 0
  CHECK (history_privacy_revision >= 0);

CREATE TABLE player_match_history (
  history_key TEXT PRIMARY KEY NOT NULL,
  result_id BLOB UNIQUE CHECK (result_id IS NULL OR length(result_id) = 16),
  ranked_match_key BLOB UNIQUE CHECK (ranked_match_key IS NULL OR length(ranked_match_key) = 16),
  ranked_row_id INTEGER UNIQUE REFERENCES ranked_matches(id),
  mode TEXT NOT NULL CHECK (mode IN ('ranked', 'casual', 'challenge')),
  origin TEXT NOT NULL CHECK (origin IN ('public_queue', 'private_room', 'invitation', 'rematch', 'legacy_ranked')),
  ended_unix INTEGER NOT NULL CHECK (ended_unix >= 0),
  combat_ticks INTEGER CHECK (combat_ticks IS NULL OR combat_ticks >= 0),
  wall_ms INTEGER CHECK (wall_ms IS NULL OR wall_ms >= 0),
  winner_seat INTEGER CHECK (winner_seat IS NULL OR winner_seat IN (0, 1)),
  end_reason TEXT NOT NULL,
  arena_id TEXT,
  ruleset_version INTEGER,
  source_hash TEXT,
  summary_version INTEGER NOT NULL CHECK (summary_version >= 0),
  quality TEXT NOT NULL CHECK (quality IN ('complete', 'partial', 'result_only')),
  canonical_hash BLOB NOT NULL CHECK (length(canonical_hash) = 32),
  payload_json TEXT NOT NULL CHECK (length(CAST(payload_json AS BLOB)) <= 32768)
);

CREATE TABLE player_match_history_sides (
  entry_id BLOB PRIMARY KEY NOT NULL CHECK (length(entry_id) = 16),
  history_key TEXT NOT NULL REFERENCES player_match_history(history_key),
  seat INTEGER NOT NULL CHECK (seat IN (0, 1)),
  account_id INTEGER NOT NULL REFERENCES accounts(id),
  name_at_match TEXT NOT NULL,
  ended_unix INTEGER NOT NULL CHECK (ended_unix >= 0),
  UNIQUE (history_key, seat),
  UNIQUE (history_key, account_id)
);
CREATE INDEX player_match_history_by_account
  ON player_match_history_sides(account_id, ended_unix DESC, entry_id DESC);

CREATE TABLE player_history_meta (singleton INTEGER PRIMARY KEY CHECK(singleton=1), backfill_id INTEGER NOT NULL DEFAULT 0, backfill_target INTEGER NOT NULL);
INSERT INTO player_history_meta SELECT 1,0,COALESCE(MAX(id),0) FROM ranked_matches;
