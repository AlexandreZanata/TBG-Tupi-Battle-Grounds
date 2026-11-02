# P75 requirements — administration, moderation, recovery and match social

Companion to the executable phase
[P75](../phases/P75-admin-moderation-and-match-social.md). Contracts and
tests: [P75-CONTRACTS](P75-CONTRACTS.md). Interfaces: [P75-API](P75-API.md).
Storage: [P75-SCHEMA.sql](P75-SCHEMA.sql). Host steps:
[P75-OPERATIONS](P75-OPERATIONS.md). Executor handoff:
[P75-EXECUTION-PROMPT](P75-EXECUTION-PROMPT.md).

**Status:** planning only, 2026-11-02. Nothing here is implemented, deployed
or activated. Every product value below is **Proposed** until the owner
confirms it (section 6); the plan's defaults apply when no answer is given.

## 1. Owner request (2026-11-02, paraphrased from Portuguese)

1. A complete administrative suite: account bans, moderation and the tools
   around them.
2. A **Report account** action in the game client.
3. Real-time **chat** during the match.
4. **Emojis**, including **custom emojis/GIFs** that enter the game only
   after an administrator has seen the content and approved it.
5. **Three emoji slots** per player, usable during the match.
6. An **email is required** to create an account.
7. If a player loses the account key, they recover it **by email** and get
   a **new key inside the game**; the old key is revoked. Recovery must
   never lose anything: match history, ratings, settings — nothing.
8. An **administrator with the email `zanataxandyzv07@gmail.com`** and no
   key. After the owner installs the email-server credentials on the VPS
   and the new version starts, the recovery process is emailed to that
   address and generates the **single administrator key** the owner will
   use. Losing it later is solved by the same email recovery.
9. Investigate the current structure, gather every requirement, define the
   immutability contracts that keep this behavior from breaking later, and
   leave the plan ready for another AI to execute; publish it as a PR.

A follow-up instruction on the same day: **always use GitHub `main` as the
base.** The audit base of this plan is `origin/main` `b964875`.

## 2. Scope

**In scope:** player email identity and verification; email-code AccountKey
recovery with in-place rotation; SMTP delivery with a durable outbox; a
separate operator (administrator) identity bootstrapped from configuration
and claimed by email; the operator console on `admin.tupibg.com`; account
bans, chat/emoji restrictions, session revocation, email blocks; player
reports with server-captured evidence; in-match text chat; a standard emoji
set, three equipped slots, custom emoji upload, normalization, premoderation
and revocation; kill switches; audit; retention; client UI (Linux reference,
identical Windows and desktop web); the operator web console; deploy
templates and the owner runbook.

**Out of scope (do not build):** P65 news/forum/support tickets (its
foundations are reused, its content features stay in P65); chat for
spectators or practice; private messages, friends, global/lobby chat; rich
text, links, attachments in chat; custom emojis shared with other players
or sold; renaming accounts (BR-P01 keeps names permanent); rating/history
edits from the console (P60 `tbg-ranked-admin` stays the only correction
path); automatic punishment from report counts; IP bans; phone/mobile UI;
third-party identity providers; account deletion (a legal follow-up, §6
O-P75-13); any production deploy, migration or secret handling by the
executor.

## 3. Actors

| Actor | Identity | Notes |
|---|---|---|
| Guest | Session only | Plays casual; reads chat; sends only standard emojis; cannot report, upload or be reported |
| Player | `Account` (PlayerName + AccountKey, D-97 tokens) | Email required for new accounts; owns reports, uploads, slots |
| Owner operator | `Operator` role `owner`, email + AdminKey | Exactly one; bootstrapped from host configuration; full capabilities |
| Moderator operator | `Operator` role `moderator` | Optional; invited by the Owner; reduced capabilities (§4.4) |
| Server | Authoritative `tbg-server` slots a/b | Owns every decision; shares one SQLite file |
| Mail provider | SMTP submission service chosen by the owner | Credentials only on the VPS |

## 4. Functional requirements

IDs are stable; tests cite them. "Proposed" values live in GAME-DESIGN §22.

### 4.1 Player identity and email (ID)

| ID | Requirement |
|---|---|
| ID-01 | New accounts cannot be created without an email address. Registration takes name + email (+ client language for emails) in one request; the account, its key and its unverified email are created in one transaction. |
| ID-02 | Normalization reuses `store/recovery/email.rs`: trimmed ASCII, ≤ 254 bytes, local part kept as typed, domain lowercased, no IDN, no provider-specific rewriting (dots/plus tags are never merged). Invalid shapes return `email_invalid`. |
| ID-03 | After registration the server queues a verification email with a single-use code. Entering it in game marks the address verified. Codes: 10 Crockford characters shown as `XXXXX-XXXXX`, 24 h for verification, 5 attempts. |
| ID-04 | Email addresses are private: never in public DTOs (leaderboard, profiles, live, directory, history, match messages), logs, metrics, crash text or URLs. The owner's own account panel shows a masked form (`z***7@gmail.com`). |
| ID-05 | Accounts created before P75 keep playing. They see a non-blocking prompt to add an email; adding one requires the current AccountKey (fresh proof) and then verification. |
| ID-06 | Routine change of a verified email requires the AccountKey, a code sent to the old address and a code sent to the new address; both addresses are notified. |
| ID-07 | A player who lost the old mailbox but still has the key may change it with the key plus new-address verification, but the change takes effect only after a 7-day hold during which the old address is notified and can cancel. |
| ID-08 | At most 5 accounts may use the same normalized email (Proposed, O-P75-01). The 6th registration returns `email_limit`. Emails are not unique, so registration is not an existence oracle. |
| ID-09 | Normalized addresses on the operator-managed block list are refused at registration and email change with `email_blocked`. |
| ID-10 | Resend of a verification code: at most 3 per hour per account and per destination, DB-backed (both slots share the limit). |
| ID-11 | Text chat and custom emoji upload require a verified email (Proposed, O-P75-02); standard emojis and play do not. |

### 4.2 Key recovery (RC)

| ID | Requirement |
|---|---|
| RC-01 | "Lost your key?" in game asks for PlayerName + email. The reply is identical for unknown names, unverified/mismatched addresses, throttling, mail outage and success (`202 {"ok":true}`). |
| RC-02 | A recovery code is mailed only to the **verified** address of the named account. Unverified addresses never recover an account. |
| RC-03 | Recovery codes are single-use, bound to account + purpose + key epoch, valid 15 minutes, 5 wrong attempts invalidate them; a newer request replaces the older code. |
| RC-04 | Completion rotates the key **in place**: same `accounts.id`, name, ratings, ranked rows, history, Arcanas, Stronghold kind, emoji library/slots, moderation state, email. Only `accounts.key_hash`, the identity epoch and the recovery/audit/session rows change. |
| RC-05 | In the same server operation all auth families and live sockets of the account are revoked on every slot (existing `revoke_account` barrier plus the P71 W3 feed). The old key stops working immediately. |
| RC-06 | The new AccountKey is returned once in the completion response, which also signs that device in (same grant shape as registration, `account_key` field). The client shows the existing "save your key" screen. The key is never emailed or stored in plaintext. |
| RC-07 | After rotation the verified address receives a notice without the key. The mail outbox never stores codes or keys; the dispatcher creates the code when it sends. |
| RC-08 | Recovery never creates an account, never lifts a ban and never changes restrictions. A banned account can rotate its key but stays banned. |
| RC-09 | Lost key **and** lost mailbox is not recoverable through the game. The Owner may only re-send a code to the verified address (MD-09); the console can never view, set or generate a player key or change a player email. |
| RC-10 | The existing manual-backup/vault features keep working: after recovery the client replaces any stored key backup for that account with the new key (coordinate with the open `codex/native-settings-updates` front). |

### 4.3 Operator identity and console access (AD)

| ID | Requirement |
|---|---|
| AD-01 | Operators are a separate principal from player accounts. Player keys, Access/refresh tokens or names never authorize an operator action, and operator credentials never sign into the game. |
| AD-02 | `TBG_ADMIN_BOOTSTRAP_EMAIL` (host configuration, never compiled in) creates the single Owner operator idempotently at startup in state `pending_claim` with **no key**. An existing Owner is never replaced by a different configured address (warning only). |
| AD-03 | While the Owner is `pending_claim`, an `operator_claim` email is queued at most once per 24 h; it is sent as soon as SMTP is configured. No manual step is needed beyond installing the mail settings and the bootstrap address. |
| AD-04 | The console's **Claim or recover key** form takes the email + emailed code (24 h, 5 attempts) and shows the AdminKey once. |
| AD-05 | AdminKey: `TBGADM-` + 32 Crockford characters (160 bits) in groups of 4; stored as SHA-256 of the secret; one valid key per operator. Every claim/recovery increments `key_epoch`, invalidates the previous key and revokes all of that operator's sessions. |
| AD-06 | A lost AdminKey is recovered by the same claim flow to the operator's email. |
| AD-07 | Console sign-in: AdminKey, then a login code emailed to the operator (second factor, Proposed default on, O-P75-04), then a session. |
| AD-08 | Sessions: opaque 256-bit token in `__Host-tbg_admin` (Secure, HttpOnly, SameSite=Strict, Path=/), idle 30 min, absolute 8 h, at most 3 per operator, sensitive actions need AdminKey re-entry within 5 min. |
| AD-09 | Every mutating admin request needs the exact admin Origin, `Content-Type: application/json` and the session-bound `X-TBG-CSRF` header. No CORS. No state change on GET. |
| AD-10 | Sign-in, claim and code attempts are rate limited per IP and per operator in the database (both slots). Failures never lock an operator out permanently. |
| AD-11 | Break-glass host command (`tbg-server operator …`, run as the service user over SSH; a subcommand because the backend artifact may only ship `tbg-server` and `tbg-release-admin`) can print a claim code without SMTP, revoke sessions, disable/enable an operator and change the Owner email with confirmation. Every CLI action is audited. |
| AD-12 | Every successful operator sign-in and key change emails a notice to that operator. |
| AD-13 | The Owner can invite moderators by email (same claim flow), disable or re-enable them. Moderators cannot manage operators. The Owner cannot be disabled from the web. |
| AD-14 | Admin handlers run on a dedicated loopback listener per process; `admin.tupibg.com` proxies only `/admin-api/*` to it. No admin route exists on the public game router or static fallback. |
| AD-15 | Every privileged mutation writes one append-only audit row in the same transaction; if the audit insert fails the action fails. |

### 4.4 Moderation and bans (MD)

| ID | Requirement |
|---|---|
| MD-01 | Ban an account temporarily (1 hour to 365 days) or permanently with a reason category and a private note. |
| MD-02 | A ban takes effect on every slot within about 2 s: the account's sessions are revoked, queues/rooms/invitations/spectating are cancelled, and a seat in an admitted match loses by **forfeit** at once (Proposed, O-P75-03). Login, refresh, registration of a blocked email and WebSocket claims are refused while banned. |
| MD-03 | The banned player sees a localized suspension screen with the end time (or "permanent") and the reason category; the private note is never shown. |
| MD-04 | While banned the account is absent from the leaderboard, active-player directory, live list and public profile/history routes (same response as an unknown name). Opponents' stored history rows and ratings are never rewritten. |
| MD-05 | Unban (or expiry) restores every surface; nothing was deleted. |
| MD-06 | Chat restriction and emoji restriction are separate temporary measures (1 hour to 90 days); they apply to new messages immediately, including in a running match. |
| MD-07 | Revoke sessions forces logout on every device without a ban. |
| MD-08 | Email block list: add/remove exact normalized addresses; optional "also block email" when banning. |
| MD-09 | Owner-only support action: send a recovery code to the account's verified address. No other credential action exists. |
| MD-10 | No rating, match result or history edit exists in the console. Ranked corrections remain the P60 CLI plan/apply flow (D-89). |
| MD-11 | Every action records operator, reason, optional report link, before/after state and an optimistic `expected_version`; stale versions return `409 conflict`. |
| MD-12 | Reports never punish automatically. |

Capability matrix (default deny; the server enforces every request):

| Capability | Owner | Moderator |
|---|---|---|
| View accounts (masked email), reports, emoji queue, transcripts attached to reports | yes | yes |
| View full email of an account | yes | no |
| Resolve reports, approve/reject emojis, manage blocked terms | yes | yes |
| Chat/emoji restriction, revoke sessions | yes | yes |
| Temporary ban up to 30 days | yes | yes |
| Longer or permanent ban, unban | yes | no |
| Revoke an approved emoji | yes | yes |
| Email block list, kill switches, send recovery code | yes | no |
| Invite/disable operators, read the full audit trail | yes | no |

### 4.5 Reports (RP)

| ID | Requirement |
|---|---|
| RP-01 | A signed-in player can report another account from: the Result screen of a finished online match (opponent), a match-history entry, a public profile or the active-players directory, and an opponent's custom emoji shown in that match's Result. During a match the opponent menu offers **Mute** at once and marks the Result screen's Report action, so evidence is complete when the report is filed. |
| RP-02 | Reasons: cheating, abusive chat, offensive name, offensive emoji, harassment, griefing/AFK, impersonation, other. Optional comment ≤ 500 characters (Latin-1 printable). |
| RP-03 | The server captures evidence at filing: match mode/arena/result facts, both names, the stored chat transcript (if any) and the reported emoji, frozen in the report. Client-supplied evidence is never trusted. |
| RP-04 | Eligibility: match context requires that the reporter played that match against the target within the 7-day transcript window; profile context requires a public account; emoji context requires that the emoji was equipped by the target in a match with the reporter. |
| RP-05 | One open report per reporter, target and match; at most 10 reports per reporter per 24 h; duplicates return `409 duplicate`. |
| RP-06 | Reporter identity is visible only to operators; the target is never told who reported. The reporter gets a localized "Report sent" confirmation. |
| RP-07 | Guests cannot report and guest seats show no Report action (they cannot type text). |

### 4.6 Match chat (CH)

| ID | Requirement |
|---|---|
| CH-01 | Real-time text chat between the two seats of an online PvP match (Casual, Ranked, Challenge, private room, rematch), relayed by the match task in the same turn it is received. Never through the simulation, replay or state hash. |
| CH-02 | Spectators, practice/training and the lobby have no chat (v1). |
| CH-03 | Sending text requires an account with a verified email (ID-11) and no active chat restriction; guests receive and display text. |
| CH-04 | Text: 1–120 Unicode scalars after trimming, printable ASCII plus Latin-1 Supplement (U+00A0–U+00FF), no control characters, runs of spaces collapsed. Other characters are refused (`invalid_text`); the client filters input to the same set. |
| CH-05 | Limits per seat: 3 messages per 5 s, 20 per minute, identical text refused within 10 s. Refusals reach only the sender. |
| CH-06 | Blocked terms (operator-managed, seeded from `BLOCKED_SUBSTRINGS`) are masked whole-word with `*` after case/leetspeak folding; the stored transcript keeps the masked text. |
| CH-07 | Transcript: at most 200 entries and 32 KiB per match, stored at match end in the match's social record (participants, outcome, loadouts) for every online PvP match with an account seat, kept 7 days, then deleted; reports copy what they need. |
| CH-08 | Client: chat log (last 50 lines, fades when idle); **Enter** opens the input, Enter sends, Esc cancels; game hotkeys are suspended while typing; opponent mute per match; Settings: Chat **All / Emojis only / Off** (default All). |
| CH-11 | Keyboard safety: today Enter in a match activates the focused control and the implicit first focus is Surrender (`ui/keyboard.rs`), so Enter twice can surrender. With P75, Enter activates a control only after the player moved focus with Tab in this match; otherwise Enter opens chat (or does nothing when chat is unavailable). A regression test proves Enter never surrenders without explicit Tab focus. |
| CH-09 | Kill switch `chat_enabled` (default **off** until the Owner enables it). |
| CH-10 | Social state survives reconnects: `MatchSocial` is re-sent on every attach. |

### 4.7 Emojis and slots (EM)

| ID | Requirement |
|---|---|
| EM-01 | A standard set of 16 emojis ships with the client (art source O-P75-07). Standard IDs 0–15 are permanent; future additions append IDs; a removed emoji keeps its ID reserved. |
| EM-02 | Each account has exactly 3 slots (default standard 0, 1, 2) edited in an **Emojis** screen outside matches. A slot holds one standard emoji or one of the account's approved custom emojis. |
| EM-03 | Loadouts are frozen at pairing for both seats and sent in `MatchSocial`; edits apply from the next match. Guests use the default loadout. |
| EM-04 | In match: three emoji buttons near the chat with hotkeys 1, 2, 3 (inactive while typing); the emoji appears as a bubble at the sender's name/HP bar for 2.5 s; animated emojis play their frames. |
| EM-05 | Limits per seat: 1 emoji per 2 s, 8 per 30 s. |
| EM-06 | Mute opponent hides their emojis and text; "Emojis only" hides text. Own sends still render locally. |
| EM-07 | Kill switches: `emoji_enabled` (default on) and `custom_emoji_enabled` (default **off**); with custom display off, custom slots show their standard fallback (slot default). |

### 4.8 Custom emojis (CE)

| ID | Requirement |
|---|---|
| CE-01 | Accounts with a verified email and no emoji restriction upload PNG, GIF (animated allowed) or static WebP, ≤ 1 MiB, ≤ 1024×1024 px, ≤ 120 source frames, with a label of 1–24 Latin-1 characters. |
| CE-02 | The server decodes with strict limits and produces a normalized PNG sprite sheet: frames ≤ 128×128 px (aspect kept, transparent padding), ≤ 24 frames sampled by time, per-frame delay 50–1000 ms, loop ≤ 4 s, ≤ 6 columns, ≤ 256 KiB. Anything else is refused with a reason. The original upload is discarded. |
| CE-03 | New uploads are `pending`, visible only to their owner (private preview) and operators. |
| CE-04 | Review shows every frame as a grid, the animation at real speed, the label, the uploader and their history; approve or reject with a reason category. |
| CE-05 | Approval assigns a random public id; only approved sheets are served publicly (`/api/emoji/{id}.png`, immutable cache). Pending, rejected, withdrawn and revoked content is never served publicly. |
| CE-06 | Owners may withdraw their uploads; rejected/withdrawn sheets are deleted after 30 days (metadata and audit stay). |
| CE-07 | Revoking an approved emoji removes it from every slot (replaced by the slot default), stops relaying it on every slot within ~2 s and records the action. |
| CE-08 | Limits: 3 uploads per account per 24 h, 10 non-rejected custom emojis per account, 500 pending globally (`503 queue_full` beyond). |
| CE-09 | Custom emojis are usable only by their owner (no sharing, trading or sale). |
| CE-10 | Kill switch `emoji_uploads_enabled` (default **off** until the Owner enables it). |

### 4.9 Operations (OP)

| ID | Requirement |
|---|---|
| OP-01 | SMTP settings come from host files outside git: non-secret values in an `EnvironmentFile`, the password through systemd `LoadCredential`. `Config` (Debug-logged) never contains secrets. |
| OP-02 | The mail dispatcher runs in exactly one process via a DB lease, retries with backoff (1 min, 5 min, 30 min, 2 h, 6 h; at most 8 attempts), records stable error codes and exposes counters. Missing SMTP leaves mail queued, never crashes or blocks play. |
| OP-03 | Outbox bound: 2,000 pending rows; above it new verification/notice rows are dropped with a metric; recovery requests stay generic. |
| OP-04 | Delivery is attended: backup, rehearsed migration on a copy, published clients, coordinated contract switch (schema and protocol both change), host configuration, Caddy route. The automatic pipeline refuses schema changes by design. |
| OP-05 | Nightly backups must be running before activation (they currently are not installed on the live host). |
| OP-06 | Kill switches in `admin_settings` take effect on every slot within ~2 s and are audited. |
| OP-07 | Retention jobs (transcripts, rejected sheets, outbox, feed, expired challenges/sessions) delete at most 256 rows per maintenance turn, outside match ticks. |
| OP-08 | Metrics: chat sent/refused, emoji sent, reports filed, uploads, approvals, bans, mail queued/sent/failed, admin sign-ins/failures. No personal data in metrics. |

## 5. Non-functional requirements (NF)

| ID | Requirement |
|---|---|
| NF-01 | Authority: combat, Energy, Shop, Evolution, Synergy, outcomes stay server-side and unchanged. Social features never enter `tbg-core` simulation state; `contract_determinism` and replay hashes stay identical. |
| NF-02 | Security: OWASP-aligned (authorization per request, session management, CSRF, plain-text rendering, logging hygiene) as in P65's S01–S17 plus P75 cases. Constant-shape comparisons for every secret digest. |
| NF-03 | Privacy: collect only email, locale and the listed moderation data; publish none of it; documented retention; privacy page update before activation (O-P75-13). |
| NF-04 | Performance: no SQL on the tick path; chat/emoji relay O(1) per message; image normalization in `spawn_blocking` behind a 2-permit semaphore and a 2 s budget; emoji sheets cached in memory (≤ 32 MiB LRU). |
| NF-05 | Every queue, cache, map and table has a bound and an expiry. |
| NF-06 | EN and pt-BR complete for every new string (client catalogs, emails, console). |
| NF-07 | Linux is the visual reference; Windows and desktop web identical (AGENTS rule 8), with `docs/ui/DESKTOP-UI-CONTRACT.md` entries for every new surface. Desktop only. |
| NF-08 | Keyboard: chat, emoji hotkeys, report modal and recovery forms fully usable without a mouse; focus never traps game controls. |
| NF-09 | Compatibility: wire changes append only (one protocol bump); schema additive; legacy clients receive the existing `VersionMismatch`/UpdateRequired path. |
| NF-10 | Code standards: one responsibility per module, `//!` headers, no god files, no new trait for a single implementation, new dependencies with a reason comment. |

## 6. Owner decisions (defaults used if unanswered)

| ID | Decision | Default in this plan |
|---|---|---|
| O-P75-01 | Accounts per email | Up to 5; emails not unique |
| O-P75-02 | Verified email needed for text chat and uploads | Yes |
| O-P75-03 | Ban during a match | Immediate forfeit of the banned seat |
| O-P75-04 | Console second factor | Emailed login code on every sign-in |
| O-P75-05 | Console exposure | `admin.tupibg.com/admin-api/*` → loopback listener; optional IP allow-list |
| O-P75-06 | Chat modes | All online PvP modes; no spectators/practice |
| O-P75-07 | Standard emoji art | 16 Twemoji graphics (CC-BY 4.0, attributed) until in-house art replaces them under the same IDs |
| O-P75-08 | Retention | Transcripts 7 d; closed reports 180 d; rejected sheets 30 d; outbox 30 d; audit and moderation history kept |
| O-P75-09 | Limits | As in §4 (chat 120 chars, 3/5 s; emoji 1/2 s; uploads 3/day, 10 total) |
| O-P75-10 | Mail provider and sender | Owner supplies an SMTP submission account; sender `no-reply@tupibg.com` with SPF/DKIM/DMARC, or the provider's own address |
| O-P75-11 | Moderator role | Implemented; no moderator exists until the Owner invites one |
| O-P75-12 | Initial kill-switch values | chat off, emoji on, custom display off, uploads off, reports on, registration on |
| O-P75-13 | Legal | Update the privacy page (email, chat retention, moderation) and decide an account-deletion process before activation; no compliance claim |

## 7. Traceability

| Area | Steps | Contract tests (P75-CONTRACTS §4) |
|---|---|---|
| ID | S1, S2, S3, S11 | `contract_p75_identity` |
| RC | S2, S4, S11 | `contract_p75_recovery` |
| AD | S6, S7, S12 | `contract_p75_admin` |
| MD | S5, S7 | `contract_p75_moderation` |
| RP | S8, S11 | `contract_p75_reports` |
| CH, EM | S9, S11 | `contract_p75_chat`, `contract_protocol` (append) |
| CE | S10, S11, S12 | `contract_p75_emoji` |
| OP | S2, S13 | `schema_upgrade` (append), operations rehearsal |
| NF-01 | all | `contract_determinism` unchanged |
