# P75 contracts — immutability, security cases and acceptance

Companion to [P75](../phases/P75-admin-moderation-and-match-social.md) and
[P75-REQUIREMENTS](P75-REQUIREMENTS.md). This file defines what must stay
true **forever** once P75 lands, how each guarantee is enforced by an
append-only contract test, and what the consolidation must observe.

AGENTS rule: contract tests (`tests/contract_*.rs`) are **append-only**.
A test here changes only in the same commit as the GAME-DESIGN/DECISIONS
change it guards, and that commit message names the change. Tests assert
the documented rule (cite its ID), never an implementation detail.

## 1. Immutability contracts (IC)

| ID | Guarantee (never weaken without an owner decision) | Enforced by |
|---|---|---|
| IC-01 | **Recovery loses nothing.** Rotating a key changes only `accounts.key_hash` (and `last_login_at` on the next login), the identity epoch and rows in recovery/auth/outbox tables. Every other column of `accounts` and **every row of every table with a foreign key to `accounts(id)`** — discovered at test time through `pragma_foreign_key_list`, so tables added later are covered automatically — is byte-identical before and after. No account is created or deleted; the leaderboard snapshot and the P73 history rows are unchanged. | `contract_p75_recovery::br_rc04_recovery_preserves_every_account_row` |
| IC-02 | **Exactly one valid key.** After recovery the old key fails like any wrong key (`BadKey`/`auth_required`), the new key works, every pre-recovery refresh token and Access token is refused, and the account's sockets on **both** release slots are closed within 2 s. | `…::br_rc05_old_key_and_sessions_die_on_every_slot` |
| IC-03 | **No enumeration.** `recovery-request`, `claim/request` and `session/start` return byte-identical bodies and statuses for unknown names/emails, unverified or mismatched addresses, throttled buckets, mail outage and success (per-IP 429 aside). | `…::br_rc01_request_is_indistinguishable`, `contract_p75_admin::br_ad04_claim_is_indistinguishable` |
| IC-04 | **Email stays private.** A random canary address registered in a test never appears in any public HTTP response (leaderboard, players, live, online, public history, patch notes), any WebSocket message, any metrics output or any captured log line. | `contract_p75_identity::br_id04_email_never_leaves_private_routes` |
| IC-05 | **Wire is append-only.** Golden encodings of every pre-P75 `ClientMsg`/`ServerMsg` sample stay byte-identical; P75 variants occupy the frozen next tags; `PROTOCOL_VERSION` moves exactly once for P75; golden encodings of the new variants are frozen for the future. | `crates/tbg-core/tests/contract_protocol.rs` (append `p75_*` tests) |
| IC-06 | **Simulation untouched.** The existing `contract_determinism` suite passes unchanged; a seeded online match flooded with chat/emoji traffic produces the same state hash, events and outcome as the same match without social traffic. | `contract_determinism` (unchanged) + `contract_p75_chat::br_ch01_social_traffic_never_changes_the_match` |
| IC-07 | **Moderation loses nothing.** Ban → unban leaves every account-referencing row identical except moderation/audit/session tables; opponents' ranked rows, ratings and history are never rewritten; after unban the leaderboard and public profile equal the pre-ban state. | `contract_p75_moderation::br_md05_ban_and_unban_preserve_every_row` |
| IC-08 | **Audit is atomic and append-only.** A forced audit-insert failure rolls back the privileged action; `UPDATE`/`DELETE` on `admin_audit`, `moderation_actions` and `recovery_audit` abort. | `contract_p75_admin::br_ad15_audit_failure_aborts_the_action`, `…::audit_tables_are_append_only` |
| IC-09 | **One Owner, idempotent bootstrap.** Restarting with the same bootstrap address, starting two slots at once, or configuring a different address never yields a second Owner or replaces the existing one. | `contract_p75_admin::br_ad02_bootstrap_is_idempotent_and_single` |
| IC-10 | **Separate authorities.** Player keys/Access/refresh cookies never authorize `/admin-api`; the admin cookie never authorizes `/api/me`, `/ws` or auth routes; the public router has no `/admin-api` route (route inventory test). | `contract_p75_admin::br_ad01_player_and_operator_credentials_never_cross` |
| IC-11 | **Emoji identities are stable.** Standard ids 0–15 map to the frozen asset list forever; an approved custom sheet served at its public id always hashes to its stored `content_hash`; public ids are never reused. | `contract_p75_emoji::br_em01_standard_ids_are_permanent`, `…::br_ce05_served_sheet_matches_content_hash` |
| IC-12 | **Chat boundary.** Chat/emoji messages reach only the two seats of their match — never spectators, views, replays, other matches or logs; text is sanitized and bounded before relay. | `contract_p75_chat::br_ch01_only_the_two_seats_receive_social_messages` |
| IC-13 | **Schema is additive and ordered.** A v10 database upgrades to the P75 version without changing any existing row; the flag-gated P73 v11 history migration still installs **after** P75 (presence-based gate, never `MAX(version)`); re-running migrations is a no-op; a partial/foreign P75 shape fails closed. | `crates/tbg-server/tests/schema_upgrade.rs` (append `p75_*`) |
| IC-14 | **Registration contract preserved.** Name rules, case-insensitive uniqueness, refund of refused attempts and the per-IP cap behave exactly as before; an invalid/blocked/over-limit email never consumes the register slot; email is required. | `contract_registration.rs` (helper gains email in the BR-P01 amendment commit) + `contract_p75_identity::br_id01_email_is_required` |
| IC-15 | **Codes are narrow.** Every emailed code is single-use, bound to principal + purpose + epoch, expires, dies after 5 wrong attempts and is replaced by a newer one; account codes never work for operators or other purposes and vice versa. | `contract_p75_recovery::br_rc03_codes_are_single_use_and_bound`, `contract_p75_admin::br_ad05_claim_codes_are_operator_only` |
| IC-16 | **No secret at rest or in logs.** After every flow the database file, WAL and captured logs contain no plaintext AccountKey, AdminKey, code, session token or SMTP password canary. | `contract_p75_recovery::no_plaintext_secret_in_db_or_logs`, `contract_p75_admin::no_operator_secret_in_db_or_logs` |
| IC-17 | **Only approved emojis are public.** Pending, rejected, withdrawn and revoked assets return 404 on the public route; private previews require the owner or an operator; original upload bytes are never stored or served. | `contract_p75_emoji::br_ce05_only_approved_sheets_are_public` |
| IC-18 | **Kill switches converge.** Each `admin_settings` switch blocks its feature on both slots within the feed interval and restores it when turned back on. | `contract_p75_moderation::op06_kill_switches_converge_on_both_slots` |
| IC-19 | **Moderation never invents outcomes.** The only outcome a moderation action can cause is the documented ban forfeit (BR-MD02), settled exactly like a surrender for rating (BR-R04); no other action ends, pauses or edits a match. | `contract_p75_moderation::br_md02_ban_forfeit_settles_like_a_surrender` |
| IC-20 | **AccountKey format unchanged.** Recovered keys use the existing `Name-XXXX-XXXX-XXXX-XXXX` format and `tbg_core::account::parse_key`; no stored `key_hash` is migrated; pre-P75 keys keep working. | `contract_p75_recovery::br_p02_recovered_keys_keep_the_key_format` |

## 2. Security negative cases (S)

Adapted from P65 S01–S17 plus P75 specifics. Each needs a behavioral test;
a 200 response or a screenshot is not evidence. Use temporary databases,
`Transport::Capture`, the controllable `auth::Clock` and two servers on one
database for cross-slot cases (pattern: `tests/revocation_feed.rs`).

| ID | Scenario | Required result |
|---|---|---|
| S01 | Every admin route × {none, expired, disabled operator, moderator, owner} including wrong methods | Exactly the capability table's answer; a route missing from the matrix fails the test (inventory completeness) |
| S02 | Bodies naming another account id, operator id, role, version or report owner | Ignored or rejected; targets are derived server-side |
| S03 | Missing/wrong `X-TBG-CSRF`, foreign/missing `Origin`, `text/plain` or form posts, mutations via GET | Refused with no state change and no audit row |
| S04 | Session reuse after logout, claim (key rotation), disable, idle/absolute expiry, restart | Refused; cookies `__Host-`, Secure, HttpOnly, SameSite=Strict |
| S05 | Code guessing (account and operator), AdminKey guessing from many IPs, across both slots | 5 attempts per code; DB-backed per-IP/per-operator budgets; the real owner can always start a fresh claim |
| S06 | Enumeration through claim/request, recovery-request, session/start, register email limit | Uniform answers (IC-03); `email_limit` only at the 6th account for that address |
| S07 | Script/markup/control/zero-width/RTL characters in chat, labels, comments, notes, terms, names | Client and console render text nodes only; server refuses out-of-charset chat/labels |
| S08 | Upload bombs (huge declared dimensions, 10,000-frame GIF), truncated files, polyglots, wrong `Content-Type`, zero delays, oversize | Rejected before full decode or bounded by decoder limits; never stored raw; never served raw |
| S09 | Any URL in chat/comments/labels/emails | Never fetched; emails carry no token-bearing link |
| S10 | Secret canaries in logs/DB/WAL/metrics/errors | Absent (IC-16); `Config` Debug output has no mail secret |
| S11 | Floods: chat, emojis, reports, uploads, outbox, feed, sessions | Every limiter/table/cache bounded; 429/503 with Retry-After; no unbounded task spawn |
| S12 | Races: ban vs unban, approve vs revoke, claim twice with one code, recover twice, duplicate report | One effect; others `409`/`401`; single audit row per effect |
| S13 | Store/SMTP failure mid-action | Privileged writes fail closed; play and chat relay continue; mail retries later |
| S14 | Forged `X-Forwarded-For`/`Host` from a non-loopback peer; allow-list bypass attempts | Trusted-proxy rule unchanged; allow-list enforced on the derived client IP |
| S15 | Banned account: refresh, resume handle, lease reconnect, guest-then-claim, spectate, register again with a blocked email | All refused; no seat, no queue, no new account with that email |
| S16 | CR/LF or header syntax in names, labels, addresses, locales | Refused at validation; mail built only through typed builders |
| S17 | Reporter identity, full emails and private notes in target-visible or moderator-visible outputs | Reporter never shown to the target; moderators see masked emails; notes stay operator-only |

## 3. Acceptance scenarios (consolidation)

Run once on the integrated tree under the shared lock (VALIDATION §3).
Real clients: native Linux (reference) and desktop web; Windows evidence
per the visual-correspondence contract (pending is recorded as pending).

| ID | Scenario |
|---|---|
| A01 | Register with email on web and native; verify with the captured code; account panel shows verified |
| A02 | Pre-P75 account adds an email with its key, verifies, later changes it through both codes |
| A03 | Lost key on native and web: request → captured code → recover → new key shown once → old key refused → second device signed out → rating/history/Arcanas/emoji slots unchanged |
| A04 | Bootstrap: start with `TBG_ADMIN_BOOTSTRAP_EMAIL` and capture transport; claim mail sent once; claim → AdminKey; sign in with key + login code; overview loads |
| A05 | Two slots on one DB: ban from slot a's admin listener ends the banned player's match on slot b by forfeit within 2 s; suspension screen shows; unban restores the leaderboard row |
| A06 | Chat between native Linux and desktop web in a casual match: send/receive, masking, rate limit, mid-match chat restriction, reconnect keeps chat working; Enter never surrenders without Tab focus |
| A07 | Emoji: standard slots with hotkeys; upload a GIF → console approve → equip → opponent sees the animation on web and native → revoke mid-match → fallback emoji |
| A08 | Report from Result with transcript evidence → console shows frozen evidence → ban from the report → report resolved, audit rows present |
| A09 | Each kill switch toggled from the console takes effect on both slots |
| A10 | Mail unconfigured: registration and play work, rows queue; configure capture → rows sent with fresh codes |
| A11 | Migration rehearsal on a synthetic production-shaped copy (v10 → P75), then `TBG_HISTORY_MIGRATE` still installs v11 |
| A12 | Desktop UI contract entries and captures for every new surface (§5) on Linux; Windows/web recorded or explicitly pending |

## 4. Test inventory to create

Server integration targets (new, append-only once merged):
`contract_p75_identity`, `contract_p75_recovery`, `contract_p75_admin`,
`contract_p75_moderation`, `contract_p75_reports`, `contract_p75_chat`,
`contract_p75_emoji`, plus shared helpers `tests/common/mail.rs` (capture
access, code extraction) and `tests/common/operator.rs` (claim + session
helper). Appends: `crates/tbg-core/tests/contract_protocol.rs`,
`crates/tbg-server/tests/schema_upgrade.rs`. Unit tests next to code for
email normalization, code codec, chat sanitizer/masker, limiters, image
normalization limits and capability lookup. Client unit tests: chat input
charset filter, Enter/Tab keyboard safety (CH-11), sheet header limits
before decode, loadout resolution and mute. Web console: extend
`web/scripts/check.py` so the admin app keeps text-only rendering and every
non-GET call sends `X-TBG-CSRF`.

Consolidation command shape (targets exist only after implementation):

```bash
flock -n -E 75 "/tmp/tbg-validation-$(id -u).lock" \
  env CARGO_BUILD_JOBS=2 cargo test -p tbg-server \
  --test contract_p75_identity --test contract_p75_recovery \
  --test contract_p75_admin --test contract_p75_moderation \
  --test contract_p75_reports --test contract_p75_chat \
  --test contract_p75_emoji --test contract_registration \
  --test schema_upgrade --test revocation_feed -- --test-threads=1
flock -n -E 75 "/tmp/tbg-validation-$(id -u).lock" \
  env CARGO_BUILD_JOBS=2 cargo test -p tbg-core \
  --test contract_protocol --test contract_determinism -- --test-threads=1
```

## 5. Visual correspondence surfaces (AGENTS rule 8)

Every surface below gets a change record in
`docs/ui/DESKTOP-UI-CONTRACT.md` and cases in `desktop-ui-contract.json`
(Linux reference, Windows and web propagation, motion where animated):
registration email step; account email panel (unverified/verified/pending
change/hold); lost-key request, code and new-key screens; suspension
screen; report modal (all reasons, errors, sent); battle chat log, input
(focused/unfocused/refused/restricted), emoji bar and hotkey hints, emoji
bubbles (standard and animated custom, both seats), mute menu; Emojis
screen (standard grid, uploads with each status, slots editor, upload
errors); Settings chat mode. Spectator HUD must show none of the social
controls (`Hud::PrivateControls`).
