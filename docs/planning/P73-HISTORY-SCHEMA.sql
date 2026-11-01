-- P73 conceptual schema contract, NOT an installed migration.
-- Allocate the next free schema version and integrate with the existing
-- migration runner. Apply only to a disposable database for plan validation.
-- Existing accounts/ranked_matches tables are prerequisites.

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

-- The writer must set the side's denormalized ended_unix equal to its
-- parent's frozen value in the same transaction and verify duplicates.
-- The public API must use a strict participant projection, never payload_json.
-- Example owner paging query (parameters, not interpolated values):
-- SELECT s.entry_id, s.ended_unix, h.mode, h.origin, h.quality
-- FROM player_match_history_sides s
-- JOIN player_match_history h ON h.history_key = s.history_key
-- WHERE s.account_id = :account
--   AND (s.ended_unix, s.entry_id) < (:cursor_time, :cursor_id)
-- ORDER BY s.ended_unix DESC, s.entry_id DESC LIMIT :limit_plus_one;
-- No destructive down migration or automatic retention/purge is authorized.
