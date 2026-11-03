-- P75 conceptual schema contract, NOT an installed migration.
-- Owner request 2026-11-02: required player email, email-code key recovery,
-- operator console, bans/moderation, reports, match chat, emojis and
-- moderated custom emojis. See docs/phases/P75-admin-moderation-and-match-social.md.
--
-- Integration rules (immutability contracts IC-13/IC-14 in P75-CONTRACTS.md):
-- * Additive only. No existing table, column, index, trigger or row is
--   renamed, dropped, rewritten or reinterpreted. `accounts` gains NO column:
--   every P75 fact lives in its own table keyed by `accounts.id`.
-- * Allocate the next free version at S0 (audited free value: 12, after the
--   flag-gated P73 history v11). Install these tables as one shape-verified
--   feature (`schema::ensure`, like RELEASE/AUTH) inside `migrate()`.
-- * Fix `migrate_history` first: it must test for the v11 row/shape, never
--   `MAX(version) < 11`, or a v12 database would silently skip P73 history.
-- * The P67 `#[cfg(test)]` fixture tables are promoted here with the same
--   names so `store/recovery` code and its tests carry over; their extra
--   columns are listed explicitly. Fixture-only `schema::install` is deleted.
-- * Timestamps are Unix seconds from the server clock (`auth::Clock`).
-- * Never store plaintext AccountKeys, AdminKeys, codes, session tokens or
--   SMTP secrets. Codes/tokens are stored as SHA-256 digests only.

------------------------------------------------------------------------------
-- 1. Player identity: email, verification, key epoch (promoted P67 tables)
------------------------------------------------------------------------------

CREATE TABLE recovery_identity (
  account_id INTEGER PRIMARY KEY REFERENCES accounts(id),
  -- Address given at registration or the last confirmed change, normalized
  -- by store/recovery/email.rs (ASCII, lowercase domain, local part kept).
  email TEXT CHECK (email IS NULL OR length(email) BETWEEN 3 AND 254),
  -- Set only by a consumed 'verify' challenge for exactly `email`.
  verified_email TEXT CHECK (verified_email IS NULL OR length(verified_email) BETWEEN 3 AND 254),
  verified_unix INTEGER,
  -- Requested replacement during a routine change (BR-ID05); never usable
  -- for recovery until its own verification and old-mailbox approval.
  pending_email TEXT CHECK (pending_email IS NULL OR length(pending_email) BETWEEN 3 AND 254),
  pending_unix INTEGER,
  -- Incremented by every key rotation; challenges carry the epoch they were
  -- issued under and die when it moves.
  epoch INTEGER NOT NULL DEFAULT 0 CHECK (epoch >= 0),
  compromised INTEGER NOT NULL DEFAULT 0 CHECK (compromised IN (0, 1)),
  -- Language of identity emails, chosen from the client language at
  -- registration or at the last email change.
  locale TEXT NOT NULL DEFAULT 'en' CHECK (locale IN ('en', 'pt-BR')),
  updated_unix INTEGER NOT NULL
);
CREATE INDEX recovery_identity_by_email ON recovery_identity(email);
CREATE INDEX recovery_identity_by_verified ON recovery_identity(verified_email);

-- One active challenge per (account, purpose). Codes are short and typed in
-- game, so they are bound to the account and limited in attempts.
CREATE TABLE recovery_challenges (
  account_id INTEGER NOT NULL REFERENCES accounts(id),
  purpose TEXT NOT NULL CHECK (purpose IN ('verify', 'rotate', 'change_old', 'change_new')),
  -- SHA-256(account_id || 0x00 || purpose || 0x00 || code)
  digest BLOB NOT NULL CHECK (length(digest) = 32),
  email TEXT NOT NULL,
  epoch INTEGER NOT NULL,
  expires_unix INTEGER NOT NULL,
  attempts INTEGER NOT NULL DEFAULT 0 CHECK (attempts BETWEEN 0 AND 5),
  consumed INTEGER NOT NULL DEFAULT 0 CHECK (consumed IN (0, 1)),
  PRIMARY KEY (account_id, purpose)
);

CREATE TABLE recovery_dispatch_limits (
  bucket BLOB NOT NULL CHECK (length(bucket) = 32),
  issued INTEGER NOT NULL
);
CREATE INDEX recovery_limits_by_bucket ON recovery_dispatch_limits(bucket, issued);

CREATE TABLE recovery_audit (
  id INTEGER PRIMARY KEY,
  account_id INTEGER NOT NULL REFERENCES accounts(id),
  at_unix INTEGER NOT NULL,
  epoch INTEGER NOT NULL,
  action TEXT NOT NULL CHECK (action IN (
    'email_registered', 'verify_sent', 'verified', 'rotate_sent', 'rotated',
    'change_requested', 'change_approved', 'email_changed', 'change_cancelled',
    'operator_recovery_sent', 'marked_compromised'))
);
CREATE INDEX recovery_audit_by_account ON recovery_audit(account_id, id);
CREATE TRIGGER recovery_audit_no_update BEFORE UPDATE ON recovery_audit
BEGIN SELECT RAISE(ABORT, 'append-only recovery audit'); END;
CREATE TRIGGER recovery_audit_no_delete BEFORE DELETE ON recovery_audit
BEGIN SELECT RAISE(ABORT, 'append-only recovery audit'); END;

CREATE TABLE recovery_revocations (
  account_id INTEGER NOT NULL REFERENCES accounts(id),
  epoch INTEGER NOT NULL,
  at_unix INTEGER NOT NULL,
  PRIMARY KEY (account_id, epoch)
);

------------------------------------------------------------------------------
-- 2. Mail outbox: intent only, never a secret (BR-RC07)
------------------------------------------------------------------------------

-- The dispatcher creates the challenge and its code inside the same
-- transaction that leases a row, sends, then marks it sent. A crash between
-- send and mark re-sends with a NEW code; the old one is replaced (PK above).
CREATE TABLE mail_outbox (
  id INTEGER PRIMARY KEY,
  purpose TEXT NOT NULL CHECK (purpose IN (
    'account_verify', 'account_recovery', 'account_key_rotated',
    'account_change_old', 'account_change_new', 'account_email_changed',
    'operator_claim', 'operator_login', 'operator_key_rotated',
    'operator_login_notice')),
  principal_kind TEXT NOT NULL CHECK (principal_kind IN ('account', 'operator')),
  principal_id INTEGER NOT NULL,
  to_email TEXT NOT NULL CHECK (length(to_email) BETWEEN 3 AND 254),
  locale TEXT NOT NULL CHECK (locale IN ('en', 'pt-BR')),
  created_unix INTEGER NOT NULL,
  not_before_unix INTEGER NOT NULL,
  attempts INTEGER NOT NULL DEFAULT 0 CHECK (attempts BETWEEN 0 AND 8),
  status TEXT NOT NULL DEFAULT 'pending'
    CHECK (status IN ('pending', 'sending', 'sent', 'failed', 'cancelled')),
  lease_owner TEXT,
  lease_until_unix INTEGER,
  -- Stable code such as 'smtp_auth', 'smtp_timeout', 'rejected_recipient';
  -- never a server reply body, address list or credential.
  last_error TEXT CHECK (last_error IS NULL OR length(last_error) <= 64),
  sent_unix INTEGER
);
CREATE INDEX mail_outbox_due ON mail_outbox(status, not_before_unix, id);
CREATE INDEX mail_outbox_by_principal ON mail_outbox(principal_kind, principal_id, id);

-- Exactly one process (release slot) dispatches at a time.
CREATE TABLE mail_dispatch_lease (
  singleton INTEGER PRIMARY KEY CHECK (singleton = 1),
  holder TEXT NOT NULL CHECK (length(holder) <= 64),
  until_unix INTEGER NOT NULL
);

------------------------------------------------------------------------------
-- 3. Operators (administrators) — never player accounts (BR-AD01)
------------------------------------------------------------------------------

CREATE TABLE operators (
  id INTEGER PRIMARY KEY,
  email TEXT NOT NULL UNIQUE CHECK (length(email) BETWEEN 3 AND 254),
  role TEXT NOT NULL CHECK (role IN ('owner', 'moderator')),
  status TEXT NOT NULL CHECK (status IN ('pending_claim', 'active', 'disabled')),
  -- SHA-256 of the 32 secret characters of the AdminKey; NULL until claimed.
  key_hash BLOB CHECK (key_hash IS NULL OR length(key_hash) = 32),
  key_epoch INTEGER NOT NULL DEFAULT 0 CHECK (key_epoch >= 0),
  locale TEXT NOT NULL DEFAULT 'pt-BR' CHECK (locale IN ('en', 'pt-BR')),
  created_unix INTEGER NOT NULL,
  claimed_unix INTEGER,
  disabled_unix INTEGER,
  -- 'bootstrap' (TBG_ADMIN_BOOTSTRAP_EMAIL), 'invite:<operator id>' or 'cli'.
  origin TEXT NOT NULL CHECK (length(origin) <= 64)
);
-- Exactly zero or one Owner (BR-AD02).
CREATE UNIQUE INDEX operators_single_owner ON operators(role) WHERE role = 'owner';

CREATE TABLE operator_challenges (
  operator_id INTEGER NOT NULL REFERENCES operators(id),
  purpose TEXT NOT NULL CHECK (purpose IN ('claim', 'login')),
  digest BLOB NOT NULL CHECK (length(digest) = 32),
  key_epoch INTEGER NOT NULL,
  expires_unix INTEGER NOT NULL,
  attempts INTEGER NOT NULL DEFAULT 0 CHECK (attempts BETWEEN 0 AND 5),
  consumed INTEGER NOT NULL DEFAULT 0 CHECK (consumed IN (0, 1)),
  -- 'login' challenges are bound to one pending login attempt.
  attempt_hash BLOB CHECK (attempt_hash IS NULL OR length(attempt_hash) = 32),
  PRIMARY KEY (operator_id, purpose)
);

CREATE TABLE operator_sessions (
  token_hash BLOB PRIMARY KEY CHECK (length(token_hash) = 32),
  operator_id INTEGER NOT NULL REFERENCES operators(id),
  csrf_hash BLOB NOT NULL CHECK (length(csrf_hash) = 32),
  key_epoch INTEGER NOT NULL,
  created_unix INTEGER NOT NULL,
  last_seen_unix INTEGER NOT NULL,
  absolute_expires_unix INTEGER NOT NULL,
  -- Last AdminKey re-entry; sensitive actions need it within 5 minutes.
  reauth_unix INTEGER NOT NULL,
  revoked_unix INTEGER,
  revoke_reason TEXT CHECK (revoke_reason IS NULL OR length(revoke_reason) <= 32)
);
CREATE INDEX operator_sessions_by_operator ON operator_sessions(operator_id, revoked_unix);

-- Every privileged mutation writes exactly one row in the SAME transaction
-- (BR-AD07). Payloads hold safe metadata only (ids, codes, before/after state),
-- never keys, codes, cookies, message bodies or full emails.
CREATE TABLE admin_audit (
  id INTEGER PRIMARY KEY,
  at_unix INTEGER NOT NULL,
  operator_id INTEGER REFERENCES operators(id),
  actor TEXT NOT NULL CHECK (length(actor) BETWEEN 1 AND 64),
  action TEXT NOT NULL CHECK (length(action) BETWEEN 1 AND 64),
  target_kind TEXT NOT NULL CHECK (target_kind IN (
    'account', 'operator', 'report', 'emoji', 'term', 'setting', 'email_block', 'system')),
  target_id TEXT NOT NULL CHECK (length(target_id) <= 64),
  reason_code TEXT CHECK (reason_code IS NULL OR length(reason_code) <= 32),
  note TEXT CHECK (note IS NULL OR length(CAST(note AS BLOB)) <= 1024),
  before_json TEXT NOT NULL CHECK (length(CAST(before_json AS BLOB)) <= 4096),
  after_json TEXT NOT NULL CHECK (length(CAST(after_json AS BLOB)) <= 4096),
  request_id TEXT NOT NULL CHECK (length(request_id) = 32)
);
CREATE INDEX admin_audit_by_target ON admin_audit(target_kind, target_id, id);
CREATE INDEX admin_audit_by_operator ON admin_audit(operator_id, id);
CREATE TRIGGER admin_audit_no_update BEFORE UPDATE ON admin_audit
BEGIN SELECT RAISE(ABORT, 'append-only admin audit'); END;
CREATE TRIGGER admin_audit_no_delete BEFORE DELETE ON admin_audit
BEGIN SELECT RAISE(ABORT, 'append-only admin audit'); END;

------------------------------------------------------------------------------
-- 4. Moderation state, history and the cross-slot feed
------------------------------------------------------------------------------

-- Current projection; absent row == no restriction. Never deleted.
CREATE TABLE account_moderation (
  account_id INTEGER PRIMARY KEY REFERENCES accounts(id),
  -- NULL: not banned. 253402300799 (9999-12-31) encodes a permanent ban.
  ban_until_unix INTEGER,
  ban_reason TEXT CHECK (ban_reason IS NULL OR ban_reason IN (
    'cheating', 'abusive_chat', 'offensive_name', 'offensive_emoji',
    'harassment', 'ban_evasion', 'impersonation', 'other')),
  chat_until_unix INTEGER,
  emoji_until_unix INTEGER,
  version INTEGER NOT NULL DEFAULT 1 CHECK (version >= 1),
  updated_unix INTEGER NOT NULL
);
CREATE INDEX account_moderation_bans ON account_moderation(ban_until_unix)
  WHERE ban_until_unix IS NOT NULL;

CREATE TABLE moderation_actions (
  id INTEGER PRIMARY KEY,
  at_unix INTEGER NOT NULL,
  operator_id INTEGER NOT NULL REFERENCES operators(id),
  account_id INTEGER NOT NULL REFERENCES accounts(id),
  action TEXT NOT NULL CHECK (action IN (
    'ban', 'unban', 'chat_restrict', 'chat_unrestrict', 'emoji_restrict',
    'emoji_unrestrict', 'revoke_sessions', 'send_recovery', 'warn')),
  until_unix INTEGER,
  reason TEXT NOT NULL CHECK (length(reason) BETWEEN 1 AND 32),
  note TEXT CHECK (note IS NULL OR length(CAST(note AS BLOB)) <= 1024),
  report_id INTEGER REFERENCES reports(id)
);
CREATE INDEX moderation_actions_by_account ON moderation_actions(account_id, id);
CREATE TRIGGER moderation_actions_no_update BEFORE UPDATE ON moderation_actions
BEGIN SELECT RAISE(ABORT, 'append-only moderation history'); END;
CREATE TRIGGER moderation_actions_no_delete BEFORE DELETE ON moderation_actions
BEGIN SELECT RAISE(ABORT, 'append-only moderation history'); END;

-- Release slots share this file; each slot follows the feed about once per
-- second (same pattern as the P71 W3 revocation feed). Pruned after 24 h.
CREATE TABLE moderation_events (
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  at_unix INTEGER NOT NULL,
  kind TEXT NOT NULL CHECK (kind IN (
    'ban', 'unban', 'chat', 'emoji', 'settings', 'terms', 'emoji_revoked',
    'emoji_approved')),
  account_id INTEGER,
  payload_json TEXT NOT NULL CHECK (length(CAST(payload_json AS BLOB)) <= 1024)
);

-- Exact normalized addresses refused for NEW registrations and email changes.
CREATE TABLE email_blocks (
  email TEXT PRIMARY KEY CHECK (length(email) BETWEEN 3 AND 254),
  created_unix INTEGER NOT NULL,
  operator_id INTEGER NOT NULL REFERENCES operators(id),
  reason TEXT NOT NULL CHECK (length(reason) BETWEEN 1 AND 32)
);

------------------------------------------------------------------------------
-- 5. Player reports
------------------------------------------------------------------------------

CREATE TABLE reports (
  id INTEGER PRIMARY KEY,
  created_unix INTEGER NOT NULL,
  reporter_id INTEGER NOT NULL REFERENCES accounts(id),
  target_id INTEGER NOT NULL REFERENCES accounts(id),
  reason TEXT NOT NULL CHECK (reason IN (
    'cheating', 'abusive_chat', 'offensive_name', 'offensive_emoji',
    'harassment', 'griefing', 'impersonation', 'other')),
  context TEXT NOT NULL CHECK (context IN ('match', 'profile', 'emoji')),
  result_id BLOB CHECK (result_id IS NULL OR length(result_id) = 16),
  emoji_id INTEGER REFERENCES emoji_assets(id),
  comment TEXT CHECK (comment IS NULL OR length(CAST(comment AS BLOB)) <= 1000),
  -- Server-captured at filing time: match facts, transcript excerpt, names.
  evidence_json TEXT NOT NULL CHECK (length(CAST(evidence_json AS BLOB)) <= 49152),
  status TEXT NOT NULL DEFAULT 'open' CHECK (status IN ('open', 'actioned', 'dismissed')),
  resolved_unix INTEGER,
  resolved_by INTEGER REFERENCES operators(id),
  resolution_note TEXT CHECK (resolution_note IS NULL OR length(CAST(resolution_note AS BLOB)) <= 1024),
  CHECK (reporter_id <> target_id)
);
CREATE UNIQUE INDEX reports_one_open ON reports(reporter_id, target_id, COALESCE(result_id, x''))
  WHERE status = 'open';
CREATE INDEX reports_queue ON reports(status, created_unix, id);
CREATE INDEX reports_by_target ON reports(target_id, id);

------------------------------------------------------------------------------
-- 6. Match social: transcripts, blocked terms, kill switches
------------------------------------------------------------------------------

-- One row per finished online PvP match with at least one account seat,
-- written fire-and-forget at match end (like `insert_match`). It is the
-- only source for report eligibility/evidence (`matches` has no result id
-- and P73 history may be disabled). Expires after 7 days (Proposed);
-- reports copy what they need into `reports.evidence_json`.
CREATE TABLE match_social_records (
  result_id BLOB PRIMARY KEY CHECK (length(result_id) = 16),
  ended_unix INTEGER NOT NULL,
  expires_unix INTEGER NOT NULL,
  mode TEXT NOT NULL CHECK (mode IN ('casual', 'ranked', 'challenge')),
  a_account INTEGER REFERENCES accounts(id),
  b_account INTEGER REFERENCES accounts(id),
  a_name TEXT NOT NULL CHECK (length(a_name) <= 16),
  b_name TEXT NOT NULL CHECK (length(b_name) <= 16),
  -- {"winner":0|1|null,"reason":"...","duration_s":n,"arena":n}
  outcome_json TEXT NOT NULL CHECK (length(CAST(outcome_json AS BLOB)) <= 1024),
  -- Both frozen emoji loadouts, by seat index.
  loadouts_json TEXT NOT NULL CHECK (length(CAST(loadouts_json AS BLOB)) <= 1024),
  -- [] when nobody chatted; at most 200 entries (masked text, emoji refs).
  transcript_json TEXT NOT NULL CHECK (length(CAST(transcript_json AS BLOB)) <= 32768),
  CHECK (a_account IS NOT NULL OR b_account IS NOT NULL)
);
CREATE INDEX match_social_expiry ON match_social_records(expires_unix);
CREATE INDEX match_social_by_a ON match_social_records(a_account, ended_unix);
CREATE INDEX match_social_by_b ON match_social_records(b_account, ended_unix);

CREATE TABLE chat_blocked_terms (
  term TEXT PRIMARY KEY CHECK (length(term) BETWEEN 2 AND 32),
  created_unix INTEGER NOT NULL,
  operator_id INTEGER NOT NULL REFERENCES operators(id)
);

CREATE TABLE admin_settings (
  key TEXT PRIMARY KEY CHECK (key IN (
    'chat_enabled', 'emoji_enabled', 'custom_emoji_enabled',
    'emoji_uploads_enabled', 'reports_enabled', 'registration_enabled')),
  value INTEGER NOT NULL CHECK (value IN (0, 1)),
  updated_unix INTEGER NOT NULL,
  operator_id INTEGER REFERENCES operators(id)
);

------------------------------------------------------------------------------
-- 7. Emojis: moderated custom assets and the three equipped slots
------------------------------------------------------------------------------

CREATE TABLE emoji_assets (
  id INTEGER PRIMARY KEY,
  -- Random, assigned at approval; the only public handle (URL and wire).
  public_id BLOB UNIQUE CHECK (public_id IS NULL OR length(public_id) = 16),
  owner_account_id INTEGER NOT NULL REFERENCES accounts(id),
  content_hash BLOB NOT NULL CHECK (length(content_hash) = 32),
  status TEXT NOT NULL CHECK (status IN ('pending', 'approved', 'rejected', 'revoked', 'withdrawn')),
  label TEXT NOT NULL CHECK (length(label) BETWEEN 1 AND 24),
  -- Normalized PNG sprite sheet produced by the server (never the upload).
  sheet BLOB CHECK (sheet IS NULL OR length(sheet) <= 262144),
  frame_px INTEGER NOT NULL CHECK (frame_px BETWEEN 16 AND 128),
  frames INTEGER NOT NULL CHECK (frames BETWEEN 1 AND 24),
  columns INTEGER NOT NULL CHECK (columns BETWEEN 1 AND 6),
  -- JSON array of per-frame delays in ms (each 50..1000), length = frames.
  delays_json TEXT NOT NULL CHECK (length(delays_json) <= 256),
  source_format TEXT NOT NULL CHECK (source_format IN ('png', 'gif', 'webp')),
  source_bytes INTEGER NOT NULL CHECK (source_bytes BETWEEN 1 AND 1048576),
  created_unix INTEGER NOT NULL,
  reviewed_unix INTEGER,
  reviewer_id INTEGER REFERENCES operators(id),
  decision_reason TEXT CHECK (decision_reason IS NULL OR length(decision_reason) <= 32),
  UNIQUE (owner_account_id, content_hash)
);
CREATE INDEX emoji_queue ON emoji_assets(status, created_unix, id);
CREATE INDEX emoji_by_owner ON emoji_assets(owner_account_id, status);

CREATE TABLE account_emoji_slots (
  account_id INTEGER NOT NULL REFERENCES accounts(id),
  slot INTEGER NOT NULL CHECK (slot IN (0, 1, 2)),
  standard_id INTEGER CHECK (standard_id IS NULL OR standard_id BETWEEN 0 AND 255),
  emoji_id INTEGER REFERENCES emoji_assets(id),
  updated_unix INTEGER NOT NULL,
  PRIMARY KEY (account_id, slot),
  CHECK ((standard_id IS NULL) <> (emoji_id IS NULL))
);
