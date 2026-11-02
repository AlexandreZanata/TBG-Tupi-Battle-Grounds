# P75 — Administration, moderation, email key recovery and match social

**Status:** planning only, 2026-11-02. No runtime code, migration, VPS
change or secret is part of this delivery.
**Planning branch:** `docs/p75-admin-moderation-plan`.
**Execution branch:** `phase/p75-admin-moderation-social`, created from the
freshly fetched GitHub `origin/main` (owner rule 2026-11-02: always base on
GitHub main).
**Audited base:** `origin/main` `b964875` (wire 38, schema 10 + flag-gated
P73 v11, highest decision D-106).
**Companions:** [requirements](../planning/P75-REQUIREMENTS.md) ·
[contracts and acceptance](../planning/P75-CONTRACTS.md) ·
[interfaces](../planning/P75-API.md) · [schema](../planning/P75-SCHEMA.sql) ·
[operations runbook](../planning/P75-OPERATIONS.md) ·
[execution prompt](../planning/P75-EXECUTION-PROMPT.md).
**Execution publication:** when the owner sends the P75 execution prompt,
that is the explicit authorization to commit per step, push the execution
branch normally and open a **Draft** PR to `main`. Never force-push,
rewrite history, push to `main`, merge, enable auto-merge, deploy, access
the VPS, touch DNS or handle real secrets.

## 1. Outcome

Players create accounts with an email, verify it with a code, and recover
a lost AccountKey in game: a code sent to the verified address yields a new
key on the **same account** — nothing is lost and the old key dies. During
online matches the two players chat and send emojis from three personal
slots, including custom emojis/GIFs that an administrator approved after
seeing every frame. Players report accounts from the Result screen,
history, profiles and the directory. The Owner — an operator identified
only by the configured address, with no key until the first emailed claim —
runs a console on `admin.tupibg.com` to review reports and emojis, ban or
restrict accounts, manage blocked terms, kill switches and moderators,
with every action audited.

Full catalog: [P75-REQUIREMENTS](../planning/P75-REQUIREMENTS.md) (ID, RC,
AD, MD, RP, CH, EM, CE, OP, NF). Owner decisions O-P75-01…13 have defaults;
record answers at S0, use the defaults otherwise.

## 2. Read first

- `AGENTS.md`; `docs/VALIDATION.md`; this phase and its five companions.
- GAME-DESIGN §2 glossary, §3 BR-P01–P05, §19–§21 (BR-UP, BR-AS, BR-CO);
  DECISIONS D-17, D-94, D-97, D-99, D-103, D-104 and the new D-107–D-111.
- `docs/planning/WEB-ADMIN-AUTH-CONTRACT.md` (superseded parts are listed
  in D-107/D-109) and P65 on branch `codex/p65-community-admin-foundation-plan`
  (`docs/planning/P65-SECURITY-CONTRACTS.md` S01–S17) for the security model.
- `docs/modules/commerce/accounts/README.md` (P67 recovery design) and the
  `#[cfg(test)]` module `crates/tbg-server/src/store/recovery/` it executes.
- `docs/ui/DESKTOP-UI-CONTRACT.md` (rule 8) before any client change.
- Code in the order each step needs it (§3 lists the exact files).
  Never read or edit `archive/`.

## 3. Source audit (what exists, what to reuse, real gaps)

Audited on `b964875`; refresh every line reference at S0.

**Accounts and auth.** `tbg-core/src/account.rs`: names (3–16, reserved list
already includes admin/moderator/staff/owner), `encode_secret` (80 bits),
`format_key`, `parse_key`, `BLOCKED_SUBSTRINGS`. `store/accounts.rs`:
`register_in_transaction`, constant-shape `login`. `auth/` (D-97): HTTP
operations in `auth/http.rs` (`login`, `register`, `refresh`, `confirm`,
`logout`, `logout-all`; 4 KiB bodies; 5 s deadline; `deny_unknown_fields`),
issuance in `auth/rotation.rs`, revocation barriers in `auth/access.rs`
(`revoke_account` = DB revoke + Access index + lobby `AuthRevoke`; the
lobby detaches sockets and **never fabricates forfeits**, `lobby/auth.rs`),
cross-slot revocation follower in `auth/mod.rs` (`follow_revocations`, ~1 s).
`history_binding` (`auth/http.rs`) is the Bearer helper for `/api/me/*`.

**P67 recovery fixture.** `store/recovery/` is compiled only under
`#[cfg(test)]` (`store/mod.rs`): email normalization, purpose-bound
challenges, DB-backed dispatch limits (3/account, 3/destination, 12/IP per
hour), transactional key rotation on the same AccountId with epoch and
revocation rows, in-memory mail capture, 617 lines of tests. It uses
64-hex link tokens; P75 changes the delivery to typed codes (D-108).

**Store.** One writer thread, bounded queue 1024 (`store/mod.rs`); schema
`CURRENT_VERSION = 10` with shape-verified features (`schema::ensure`);
`migrate_history` installs v11 only behind `TBG_HISTORY_MIGRATE` and gates
it with `MAX(version) < 11` — **a P75 v12 row would make it skip v11**, so
S1 fixes that gate first. Automatic delivery refuses any schema change
(`deploy/auto/slots.py`), so P75 ships through the attended procedure.

**Server routing.** `lib.rs` builds one public router (auth, health, ws,
public APIs, `/api/me/*`); `/metrics` and `/api/release` are loopback-only
by handler checks. No admin route exists. Caddy routes `/api/me/*` and
`/api/auth/*` with `X-TBG-Slot` to slot a (8080) or b (8082); unknown
`/api/*` goes to slot a only; `admin.tupibg.com` serves a static no-script
placeholder (`web/deploy/Caddyfile.web.template`).

**Match.** `ws/client_msg.rs` stamps the seat from `c.matched` and sends
`MatchCmd`s straight to the match task; per-connection token bucket
(40, 20/s). `match_task.rs` knows each seat's `account: Option<(AccountId,
rating)>` in every mode. `MatchCmd::Surrender` is the forfeit path.

**Protocol.** `PROTOCOL_VERSION = 38`; postcard enums; `ClientMsg` tags
0–26, `ServerMsg` 0–36; `MAX_CLIENT_MSG_BYTES = 1024`. Append only.

**Config and secrets.** `Config` derives `Debug` and `main.rs` logs it in
full at start; `/etc/tbg/server.env` must hold no credentials and is
overwritten by provisioning; `/etc/tbg/slot-%i.env` is rewritten per slot
start; units come only from `provision.sh`. No mail code or crate exists;
`rustls 0.23`, `tokio-rustls`, `webpki-roots` are in `Cargo.lock`.
`image 0.25.10` is in the lockfile (client only; png + webp, no gif).

**Client.** Registration is one name field (`ui/account_ui.rs`,
`ui/textform.rs`, `textform_bevy.rs`); the save-key modal says the key
"cannot be recovered" (`assets/i18n/en.json`). The text widget is a
full-screen single-field modal (`ui/kit/edit.rs` does the editing). Native
auth uses `ureq` with a route allowlist (`auth/transport.rs`); signed-in
calls allow only GET/PATCH on `/api/me`/`/api/players` (`auth/api.rs`, JS
twin `assets/auth/history.js`); `platform/http.rs` (ehttp) is GET-JSON
only. **Keyboard hazard:** in a match Enter activates the focused control
and the implicit first focus is Surrender (`ui/keyboard.rs`), so Enter twice
can surrender today. HUD canvas is 1600×900 logical; free area: right
column x≈1250–1586, y≈65–650, emoji row above the Auxiliaries panel
(x≈1320–1572, y≈661–864); social nodes must carry `Hud::PrivateControls`.
Result screen `ui/result.rs`; P73 directory/profile/history
`ui/history/views.rs` and `ui/social.rs` (name taken: use other module
names). No runtime image-from-bytes, atlas helper, file picker, emoji,
chat or report code exists. UI font is a Latin-1 subset; i18n catalogs are
flat JSON compiled into `Msg` with build-time charset checks.

**Open fronts touching the same files.** `codex/native-settings-updates`
(unmerged: protected account-key backup in the OS vault —
`auth/recovery_key.rs`, `ui/account_backup.rs`, `ui/account*.rs`,
`ui/textform*.rs`). P65 plan branch (unmerged, reuse its security model,
not its content features). Freeze both at S0; never overwrite them.

**Canon this phase reopens:** D-17 (no email/recovery), GAME-DESIGN §7
(emotes deferred), D-39 consequences ("chat out of scope"), P65 BR-NC05 and
WEB-ADMIN-AUTH-CONTRACT ("no game bans", CLI-only operator bootstrap).
The owner's 2026-11-02 request decides them; D-107–D-111 record it.

## 4. Architecture

```
player client ──HTTP /api/auth/* /api/me/*──► slot a|b public router
      │                                       ├─ identity + recovery (store/recovery)
      │                                       ├─ reports, emoji upload/library
      └──WS ChatSend/EmojiSend──► match task ─┴─ relay to the two seats only
                                                 └─ social record at match end
operator browser ─► admin.tupibg.com ─/admin-api/*─► loopback admin listener (each slot)
                                                       └─ store tx: action + audit + feed event
every slot: moderation feed follower (~1 s) ─► caches, lobby (ban forfeit), matches (policy)
one slot (DB lease): mail dispatcher ─SMTP─► provider
```

Module boundaries (one responsibility each; SQL only under `store/`):

| Area | Server modules (new unless noted) |
|---|---|
| Identity | `store/recovery/*` (promoted to production; challenges become codes), `identity/{mod,codes,http}.rs` (account email routes), `auth/recovery.rs` (recover/recovery-request operations), `auth/rotation.rs` (register gains email; changed) |
| Mail | `mail/{mod,dispatch,transport,templates,settings}.rs`; `store/recovery/outbox.rs` |
| Operators | `admin/{mod,listener,session,claim,policy,request}.rs`, `admin/areas/{overview,accounts,reports,emoji,terms,settings,blocks,operators,audit,system}.rs`, `store/operators/*`, `store/audit.rs`, `operator_cli.rs` |
| Moderation | `moderation/{mod,feed,cache,enforce,terms}.rs`, `store/moderation/*`, `lobby/moderation.rs` |
| Reports | `reports.rs` (HTTP), `store/reports.rs` |
| Social | `social/{chat,emoji,limits}.rs`, `match_task/social.rs`, `lobby/social.rs`, `store/social.rs` |
| Custom emoji | `emoji/{upload,normalize,serve,cache}.rs`, `store/emoji.rs` |
| Core | `tbg-core/src/protocol/social.rs` (types), `protocol.rs` (append + bump) |

Client modules: `ui/match_chat/` (log, input, emoji bar, bubbles, mute
menu), `ui/emoji_library/` (Emojis screen), `ui/report.rs` (modal),
`ui/recovery.rs` (lost-key screens), `ui/account_email.rs` (panel),
`ui/suspended.rs`, `auth/recovery.rs` + `auth/account_api.rs` (HTTP),
`emoji/{sheet,atlas,standard}.rs` (fetch, decode limits, frame animation),
`platform/file_pick.rs` (OS adapter), `sync/messages.rs` (new
`SocialInbox`), keyboard gating in `ui/keyboard.rs`, `ui/input.rs`,
`ui/shell/input.rs`, `celebration/mod.rs`.

Web console: `web/admin/app` becomes the real console behind
`api/admin.js`; new site `admin-app` publishable by `web/deploy` once the
owner activates it; `web/admin/public` stays the fallback placeholder.

## 5. Numbering and documents (verify at S0)

| Item | Proposed | Rule |
|---|---|---|
| Protocol | 39 | One bump for all P75 wire changes |
| Schema | 12 (after flag-gated v11) | One shape-verified feature, additive |
| Decisions | D-107 … D-111 | Already added as Proposed by this plan |
| GAME-DESIGN | §22, rules BR-ID, BR-RC, BR-AD, BR-MD, BR-RP, BR-CH, BR-EM, BR-CE | Proposed values; amend BR-P01/P02 in the S3/S4 commits |
| ARCHITECTURE | §28 | Added by this plan |

## 6. Steps

Each step: implement, run its **work-front check** (formatting + at most one
cached narrow `cargo check` under the shared lock, or `git diff --check`
for docs), write its tests, queue their exact commands for S14, commit
`feat(p75): <step title>`. Same error twice → `> blocked:` note in the
report and continue with independent steps.

### S0 — Freeze base, allocations and decisions

- Fetch, branch from current `origin/main`, record the SHA. Re-verify
  protocol/schema/decision/BR numbering, the tag tables in P75-API §1, the
  audit in §3 and every line reference you rely on.
- Inspect open fronts (`git branch -a`, open PRs) touching §4 files,
  especially `codex/native-settings-updates` and the P65 branch; record
  the coordination decision. Never overwrite another front.
- Record owner answers to O-P75-01…13 (or the defaults), the lettre and
  `image` versions/features you will use (checked against their docs:
  rustls with ring, no native-tls/OpenSSL; image with png, gif, webp and
  `default-features = false`), and the native file-picker choice
  (`rfd` with the XDG portal on Linux and the Win32 dialog on Windows, or a
  documented alternative that keeps the visible UI identical; no GTK
  dependency in the AppImage).
- Confirm in game whether Enter-Enter surrenders (CH-11 evidence).
- **Done when:** the report's S0 section lists SHA, allocations, owner
  decisions, coordination and dependency choices.

### S1 — Schema v12 and migration ordering

- `store/schema.rs`: replace `migrate_history`'s `MAX(version) < 11` with
  an explicit check for the v11 row/shape; add the P75 feature from
  `P75-SCHEMA.sql` as one `Feature` installed by `ensure` (all tables or
  none, shape-verified), insert version 12, bump `CURRENT_VERSION` to 12
  and seed `admin_settings` defaults (O-P75-12). Delete the fixture-only
  `store/recovery/schema.rs`; the recovery code now uses the live tables.
- Keep `DATABASE_SCHEMA_VERSION` semantics (deployment metadata).
- Tests (append to `tests/schema_upgrade.rs`): v10 → v12 keeps every row;
  v12 then `TBG_HISTORY_MIGRATE` installs v11; re-run no-op; partial P75
  shape fails closed (IC-13).
- **Done when:** a fresh and a v10 fixture database both open at v12.

### S2 — Mail settings, outbox and dispatcher

- `config.rs`: non-secret fields only (`TBG_MAIL_FROM`, `TBG_SMTP_HOST`,
  `TBG_SMTP_PORT`, `TBG_SMTP_SECURITY`, `TBG_PUBLIC_GAME_URL`,
  `TBG_PUBLIC_ADMIN_URL`, `TBG_SMTP_PASSWORD_CREDENTIAL` name, feed interval
  `TBG_FEED_MS` default 1000). Private values (`TBG_SMTP_USERNAME`,
  `TBG_ADMIN_BOOTSTRAP_EMAIL`) use a `Private<String>` newtype whose Debug
  prints `[private]`. The password is read once at start from
  `$CREDENTIALS_DIRECTORY/<name>` into a struct with **no Debug**; absent →
  mail unconfigured. `--build-info` still needs no environment.
- `mail/transport.rs`: `enum Transport { Smtp(..), Capture(MailCapture) }`
  (lettre, async tokio, rustls/ring, webpki roots, 10 s timeouts).
  `MailCapture` (tests) is an `Arc<Mutex<VecDeque<CapturedMail>>>` capped at
  64 with a redacted Debug; tests read codes from it.
- `mail/templates.rs`: plain-text EN/pt-BR bodies for every purpose in
  P75-API §4; no HTML, no token in any link.
- `store/recovery/outbox.rs`: enqueue (bounded 2,000 pending), lease due
  rows (`status='sending'`, `lease_until`), mark sent/failed with stable
  error codes, backoff 1 min/5 min/30 min/2 h/6 h, max 8 attempts, prune
  sent/cancelled after 30 days.
- `mail/dispatch.rs`: one task per process; holds `mail_dispatch_lease`
  (holder = slot or `single`, 30 s renewals) so only one process sends.
  For code-bearing purposes it creates the challenge **in the lease
  transaction**, sends, then marks sent — no code is ever persisted.
- Metrics: queued, sent, failed, dropped (no addresses).
- Tests: capture flow, lease exclusivity with two servers on one DB, retry
  and give-up, unconfigured mail leaves rows pending, secret canary absent
  from Debug/logs/DB (IC-16, S10).
- **Done when:** a queued `account_verify` row is delivered once through
  the capture transport with a valid code.

### S3 — Required email, verification and email changes

- Promote `store/recovery` to production (`#[cfg(test)]` removed from the
  module, kept on its tests). Codes replace link tokens: 10 Crockford
  characters (`identity/codes.rs`: generate with `getrandom`, format
  `XXXXX-XXXXX`, lenient parse like `parse_key`), digest
  `SHA-256(account_id ‖ 0 ‖ purpose ‖ 0 ‖ code)`, 5 attempts, constant-shape
  compare.
- `auth/http.rs` + `auth/rotation.rs`: `register` requires `email` and
  `locale`; validate email (normalize, block list, ≤ 5 accounts per address,
  `registration_enabled`) **before** consuming the register slot; create
  `recovery_identity` (unverified) and queue `account_verify` in the same
  store transaction as the account (`register_in_transaction`).
- `identity/http.rs`: `GET /api/me/account`, `POST /api/me/email/verify`,
  `/resend`, `POST /api/me/email` (legacy add with key proof; routine change
  with old/new codes; lost-old 7-day hold), `/change/confirm`, `/change/cancel`
  (P75-API §2.2). All Bearer through `history_binding`; DB-backed limits.
- GAME-DESIGN: amend BR-P01/BR-P02 (email required, recoverable key) in
  the **same commit** that updates `tests/contract_registration.rs`'s
  helper to send an email (IC-14); name the change in the commit message.
- Tests: `contract_p75_identity` (BR-ID01–ID11, IC-04 canary).
- **Done when:** registration without email fails, with email succeeds,
  the code verifies it, and the legacy add/change flows pass.

### S4 — Key recovery in place

- `auth/recovery.rs`: `recovery-request` (always `202` for well-formed
  bodies; queues `account_recovery` only for name + **verified** address;
  DB limits) and `recover` (verify code → in one store transaction rotate
  `accounts.key_hash` with a fresh `encode_secret`, bump the epoch, consume
  challenges, write `recovery_revocations`/`recovery_audit`, queue
  `account_key_rotated`) → then `revoke_account(account, "recovered")` →
  then issue a normal grant on this device with `account_key` (D-97 retry
  cache semantics, same `registration_retry_key` rules).
- Recovery never changes moderation state; a banned account rotates but
  stays banned (login refused with `account_suspended`).
- Tests: `contract_p75_recovery` — IC-01 generic FK snapshot (enumerate
  every table referencing `accounts(id)` through `pragma_foreign_key_list`),
  IC-02 on two servers sharing one DB, IC-03 byte-identical answers,
  IC-15, IC-16, IC-20.
- **Done when:** a recovered account keeps every row and only the new key
  signs in.

### S5 — Moderation core and enforcement

- `store/moderation/*`: state projection with `version`, append-only
  actions, feed append/prune, email blocks, leaderboard snapshot rebuild
  excluding active bans.
- `moderation/feed.rs`: per-process follower every `TBG_FEED_MS` (bounded
  batch 256), updating `moderation/cache.rs` (restrictions, settings,
  terms) and notifying the lobby.
- Enforcement: issuance (`login`, `refresh`, `confirm`, `recover` grant)
  refuses active bans with `auth::Error::Suspended` →
  `403 {"error":"account_suspended","until","reason"}`; WS claim refuses;
  `lobby/moderation.rs` on a ban: send `AccountSuspended` to the account's
  sockets, cancel queue/room/invitations/spectating, send the new
  `MatchCmd::ModerationForfeit { seat }` (same effect as Surrender, rating
  per BR-R04), then the acting process calls
  `revoke_account(account, "banned")`; other slots follow both feeds.
  Add `EndDetail::Moderation` (`"moderation"`) in `ranked/mod.rs` and update
  every exhaustive use (11 on the audited base: journal, finalizer, signals,
  stats, history); P60 signals must not count it as a voluntary early
  surrender, and the existing ranked-integrity contract tests stay unchanged.
- Public surfaces hide active bans: leaderboard, `/api/players/*`,
  history public routes, directory, live (same answer as unknown).
- Expiry: the cache treats `ban_until <= now` as not banned; no job needed.
- Tests: `contract_p75_moderation` — MD-01…MD-12, IC-07, IC-18, IC-19,
  S15, two-slot propagation.
- **Done when:** a ban issued through a test helper ends the other slot's
  match by forfeit and refuses sign-in; unban restores everything.

### S6 — Operators, bootstrap, claim and console sessions

- `store/operators/*` + `store/audit.rs` (every privileged write takes an
  `AuditEntry` and inserts it in the same transaction; failure aborts).
- Bootstrap at startup (`admin/claim.rs`): if no Owner exists and
  `TBG_ADMIN_BOOTSTRAP_EMAIL` is set → create Owner `pending_claim`, audit
  `bootstrap`, queue `operator_claim`; while pending, at most one claim mail
  per 24 h. A different configured address never replaces an Owner (warn).
- AdminKey: `TBGADM-` + 32 Crockford characters (160 bits, `getrandom`),
  SHA-256 of the 32 secret characters, lenient parser; claim/recovery
  bumps `key_epoch` and revokes the operator's sessions.
- `admin/listener.rs`: when `TBG_ADMIN=on`, bind loopback only (slot a →
  `TBG_ADMIN_PORT_A` 8090, slot b → `TBG_ADMIN_PORT_B` 8092, single mode
  `TBG_ADMIN_ADDR`); refuse non-loopback addresses at start. Separate
  `Router`; never merged into the public one.
- `admin/request.rs`: request id, client IP through the existing trusted
  proxy rule, optional allow-list, no-store/nosniff headers, Origin +
  JSON + `X-TBG-CSRF` on every mutation, 8 KiB bodies, 5 s deadline.
- `admin/session.rs`: start (AdminKey → login code mail when
  `TBG_ADMIN_LOGIN_CODE` is on), verify, reauth, logout, `GET /session`;
  cookie `__Host-tbg_admin`; idle 30 min, absolute 8 h, ≤ 3 sessions,
  reauth window 5 min; DB-backed attempt budgets; sign-in notice mail.
- `admin/policy.rs`: capability table from P75-REQUIREMENTS §4.4, default
  deny, checked per request against the current DB state.
- `operator_cli.rs`: `tbg-server operator {claim-code --owner|--operator
  <id>, revoke-sessions, disable, enable, set-owner-email}` handled in
  `main.rs` before `Config::from_env` like `--build-info`; TTY confirmation
  for destructive commands; audited as `cli:<user>`.
- Tests: `contract_p75_admin` — IC-08, IC-09, IC-10, AD-01…AD-15, S01–S06,
  S10, S14 (capability matrix with route inventory completeness).
- **Done when:** a test boots with a capture transport, receives the claim
  mail, claims, signs in with key + code and calls `GET /admin-api/v1/session`.

### S7 — Console API areas

- Implement every route in P75-API §3.2 with keyset paging (default 20,
  max 50), optimistic `expected_version`, reasons on every mutation,
  reauth on sensitive ones, masked emails for moderators, and feed events
  for actions that other slots must apply.
- `GET /admin-api/v1/system`: mail configured flag, outbox counts, last
  error code, feed lag, slot/build identity — never secrets or addresses.
- Tests: extend `contract_p75_admin` (matrix rows per route) and
  `contract_p75_moderation` (actions through the API, races S12).
- **Done when:** every route has a matrix row and at least one success and
  one denial test.

### S8 — Player reports

- `match_task/social.rs` writes the `match_social_records` row at match end
  for every online PvP match with an account seat (fire-and-forget like
  `insert_match`), with outcome, both loadouts and the masked transcript.
- `reports.rs`: `POST /api/me/reports` (P75-API §2.3); eligibility from the
  social record (RP-04), evidence frozen into `reports.evidence_json`,
  limits RP-05, `reports_enabled` switch.
- Retention: delete expired social records and closed reports per O-P75-08
  (≤ 256 rows per maintenance turn).
- Tests: `contract_p75_reports` — RP-01…RP-07, S12 duplicate race, S17.
- **Done when:** a finished test match can be reported once with its
  transcript captured, and a stranger cannot report it.

### S9 — Protocol 39: chat and standard emojis

- `tbg-core/src/protocol/social.rs` + `protocol.rs`: append the variants
  of P75-API §1, bump `PROTOCOL_VERSION`, document the bump. Append golden
  tests to `crates/tbg-core/tests/contract_protocol.rs` (IC-05).
- `ws/client_msg.rs`: route `ChatSend`/`EmojiSend` like `Intent` (seat
  from the connection; refused with `SocialRefused{NotAvailable}` without a
  seat; allowed after natural Access expiry on the admitted seat).
- `lobby/social.rs`: load each account's social profile after sign-in
  (verified email, restrictions, loadout) like Arcanas; freeze both seats'
  rights and loadouts in `match_task::Player` at pairing; forward feed
  changes as `MatchCmd::SocialPolicy`.
- `social/chat.rs`: sanitize (CH-04), collapse spaces, whole-word masking
  after case/leetspeak folding with operator terms (seeded from
  `BLOCKED_SUBSTRINGS`), limits CH-05; `social/emoji.rs`: slot resolution,
  limits EM-05. `match_task/social.rs`: relay to both seats only, refusals
  to the sender only, transcript ≤ 200 entries/32 KiB, `MatchSocial` after
  every `MatchStart`/attach and after policy changes. Spectators get none.
- Tests: `contract_p75_chat` — IC-06 (same hash with/without social
  traffic), IC-12, CH-01…CH-10, EM-02…EM-07, guest rules, restriction
  mid-match, reconnect.
- **Done when:** two test clients exchange chat and emojis in a casual
  match and a spectator receives nothing.

### S10 — Custom emoji pipeline

- `emoji/normalize.rs` (`image` with png, gif, webp; `default-features =
  false`; reason comment in `Cargo.toml`): check the declared dimensions
  before decoding, decode with `image::Limits`, at most 120 source frames,
  scale to ≤ 128 px keeping aspect on a transparent square, sample ≤ 24
  frames by time, clamp delays to 50–1000 ms, truncate at 4 s, pack ≤ 6
  columns, encode PNG ≤ 256 KiB, `content_hash = SHA-256(sheet)`. Run in
  `spawn_blocking` behind a 2-permit semaphore with a 2 s budget.
- `emoji/upload.rs`: `POST /api/me/emoji` (1 MiB route limit, CE-01/CE-08,
  verified email, no restriction, `emoji_uploads_enabled`, label charset);
  owner library, private preview, withdraw, `PUT /api/me/emoji/slots`.
- `emoji/serve.rs` + `emoji/cache.rs`: `GET /api/emoji/{public_id}.png`
  for approved assets only, immutable caching, ≤ 32 MiB LRU; revocation
  and approval via feed events update caches and running matches.
- Retention: delete rejected/withdrawn sheets after 30 days.
- Tests: `contract_p75_emoji` — IC-11, IC-17, CE-01…CE-10, S08 corpus of
  hostile files generated in the test (bombs, truncations, polyglots).
- **Done when:** an uploaded GIF is normalized, pending, approved through
  the console API, equipped and delivered in `MatchSocial`.

### S11 — Client

All surfaces: Linux reference, identical on Windows and desktop web,
`DESKTOP-UI-CONTRACT.md` change record + `desktop-ui-contract.json` cases
(P75-CONTRACTS §5), EN/pt-BR keys for every string, Latin-1 only.

1. **Registration and email:** after the name step open an email step in
   the same text form; send `email` + `locale`; new error states. Replace
   "cannot be recovered" copy with recovery guidance. Account panel
   (`ui/account_email.rs`): masked address, state, enter code, resend, add
   (legacy, asks for the key), change, cancel hold.
2. **Lost key:** "Lost your key?" on the sign-in form → name + email →
   generic confirmation → code → new key on the existing save-key modal →
   signed in. After recovery update any stored key backup for that
   account (coordinate with `codex/native-settings-updates`).
3. **Suspension:** `account_suspended` and `AccountSuspended` show a
   suspension screen (until/permanent, reason category).
4. **Battle chat** (`ui/match_chat/`): right-column log (last 50, fades),
   inline input (reuse `kit/edit.rs`, charset filter, 120 limit), emoji bar
   with three slots and hotkeys **1, 2, 3** (free in battle today; never
   Alt/Ctrl+digit or F1/F5/F11/F12, which browsers or the window already
   use; if the owner reserves digits for future Shop shortcuts, record the
   alternative at S0), bubbles at each seat's name plate,
   opponent menu with Mute (text+emojis) and "Report after the match",
   refusal/restriction states, `Hud::PrivateControls`. Gate every game
   keyboard system while the input is focused (keyboard.rs, input.rs,
   shell/input.rs, celebration Result keys).
5. **Keyboard safety (CH-11):** Enter activates a control only after Tab
   moved focus in this match; otherwise it opens chat (or does nothing).
   Unit test: Enter, Enter without Tab never queues Surrender.
6. **Emojis:** standard assets (16 PNGs per O-P75-07, attribution in
   `THIRD_PARTY_NOTICES.md`), `emoji/sheet.rs` (bytes GET ≤ 256 KiB,
   verify PNG header dimensions ≤ 768×512 before `Image::from_buffer`),
   `emoji/atlas.rs` (frame animation with `ImageNode.rect`), Emojis screen
   (standard grid, uploads with status/reason, slot editor, upload via
   `platform/file_pick.rs` with the same visible button/drop zone on all
   targets, size/type errors).
7. **Reports:** modal from Result (finished online matches), history
   entries, profile/directory rows, and the opponent's custom emoji on
   Result; reasons, optional comment, sent/duplicate/limit/error states.
8. **Settings:** Chat All / Emojis only / Off (`config::save_flag` pattern).
9. **Transport:** add `recovery-request`/`recover` to the native allowlist
   (`auth/transport.rs`) and the browser bridge; widen `auth/api.rs` and
   `assets/auth/history.js` to the exact new `/api/me/...` methods (POST,
   PUT, DELETE, raw-bytes upload ≤ 1 MiB); new `SocialInbox` in
   `sync/messages.rs`; block social sends for spectators in `net/mod.rs`.
- **Done when:** the WASM and native clients compile for their targets at
  consolidation and every new surface has a contract entry.

### S12 — Operator web console

- `web/admin/app`: real `api/admin.js` (same-origin, `credentials:
  'same-origin'`, JSON, `X-TBG-CSRF` on every non-GET, 401 → sign-in,
  403 reauth prompt), pages: Claim/recover key, Sign in (key → code),
  Overview, Players (search/detail/actions with reason + confirm dialog),
  Reports (queue/detail with frozen transcript as text), Emoji review
  (frame grid + animated preview from the private sheet), Blocked terms,
  Settings (kill switches), Email blocks, Operators, Audit, System.
  Text-only rendering (`textContent`), EN/pt-BR strings, keyboard usable.
- Keep `web/admin/public` as the placeholder. Add an `admin-app` site to
  `web/scripts/sites.py`, adapt `stage.py`, `check.py` (allow the app's
  JS/JSON only for `admin-app`, forbid innerHTML/eval, require the CSRF
  header) and `deploy/auto/static_policy.py` so the app can be published
  only with the console CSP; activation remains an owner action.
- **Done when:** `python3 web/scripts/check.py` passes and the local
  preview signs in against a local server with the capture transport.

### S13 — Deploy templates and documentation

- `deploy/Caddyfile.domains.template`: `/api/emoji/*` in `@shared`.
  `web/deploy/Caddyfile.web.template`: console block with
  `/admin-api/*` → `127.0.0.1:8090 127.0.0.1:8092` (`lb_policy first`) and
  the console CSP (P75-OPERATIONS §3).
- `deploy/systemd/p75.conf.example` (drop-in) and `deploy/p75.env.example`
  (placeholders only, no real address or secret); optional backup timer
  units replacing the missing cron. No change is applied to any host.
- Update `deploy/README.md` (attended P75 rollout), player patch notes
  EN/pt-BR (BR-UP10), privacy-page draft for owner review, and the
  ROADMAP/GAME-DESIGN/DECISIONS status lines.
- **Done when:** templates parse (`caddy fmt`/`caddy validate` only if the
  binary is already installed locally; otherwise inspect) and docs link.

### S14 — Targeted consolidation, report and Draft PR

- Run the queued targeted tests once (P75-CONTRACTS §4 commands) plus the
  narrow WASM/native client checks, under the shared lock. Run the A01–A12
  scenarios that need real clients once; record unobserved ones as pending.
- Fill §9, push the branch normally, open a **Draft** PR to `main` with
  behavior, contracts, evidence, migration/rollback notes and pending
  gates. Stop for review.

## 7. Validation policy

Work fronts follow `docs/VALIDATION.md` §1 (no `cargo test` by habit; one
cached narrow check; lock + 2 jobs; busy lock = deferred). Consolidation
(S14) runs the union of P75 targets once. No full suite, release build,
balance gate, benchmark, packaging, VPS run or production migration
without an explicit owner request. Native success never proves browser
compatibility; visual parity needs real captures.

## 8. Risks and mitigations

| Risk | Mitigation |
|---|---|
| Owner mailbox compromise = console compromise | Accepted by the owner's design (D-107); login code + notices; CLI recovery over SSH; advise mailbox 2FA |
| Schema bump blocks automatic delivery | Attended procedure (P75-OPERATIONS §4); additive tables; rehearsal on a copy |
| v12 hides P73 v11 | S1 fixes the gate first; IC-13 test |
| Two slots double-send mail | DB lease; tests with two servers on one DB |
| Chat toxicity at launch | Chat off by default, blocked terms, reports, restrictions, kill switch |
| Hostile image uploads | Pre-decode dimension check, decoder limits, normalization, premoderation, never serving originals |
| Enter surrender hazard grows with chat | CH-11 rule and regression test |
| Email in logs (Config Debug) | `Private<String>`, secret struct without Debug, IC-04/IC-16 canaries |
| Large phase for one executor | Ordered steps with commits; stop/continue rules; Draft PR lists pending gates |

## 9. Reviewer checklist

- [ ] Every IC-01…IC-20 test exists, cites its rule and passes at S14.
- [ ] No admin route on the public router; listener loopback-only.
- [ ] No secret/email in `Config` Debug, logs, metrics, DB plaintext.
- [ ] Wire diff is append-only; one protocol bump; golden tests appended.
- [ ] Schema additive; v11 still installs after v12.
- [ ] Simulation untouched (`contract_determinism`, IC-06).
- [ ] Recovery FK snapshot covers every account-referencing table.
- [ ] Ban forfeit settles like a surrender; nothing else edits outcomes.
- [ ] Spectators receive no social messages; HUD hides social controls.
- [ ] Every new surface has a DESKTOP-UI-CONTRACT entry; pending targets listed.
- [ ] EN/pt-BR complete; Latin-1 only; no literals in UI code.
- [ ] Dependencies (lettre, image, file picker) have reason comments.
- [ ] Draft PR lists deferred checks and owner gates honestly.

## Phase report (executor fills in)

### S0 freeze
- GitHub base: `b964875e2219bafe129b19960f7f1369307870d5` fetched on 2026-11-02; planning prerequisite fast-forwarded from `origin/docs/p75-admin-moderation-plan` (`b61764d`). Isolated worktree: `/data/tbg-p75-admin-moderation-social`.
- Allocations confirmed: protocol 39, new ClientMsg tags 27–28, ServerMsg tags 37–41; schema 12 with independently installed history 11; D-107–D-111; GAME-DESIGN section 22.
- Owner explicitly selected all O-P75-01–13 defaults in this execution request. Digits 1/2/3 remain the emoji shortcuts.
- Coordination: GitHub open PR inventory contains planning #58 and native settings #57 (`23e8cb0`); P65 remains planning-only (`b2a12c6`). Neither front is modified, merged or reset. Client integration must preserve the native key-backup seam; conflict resolution with #57 remains a reviewer gate. No other agent edits this worktree.
- Dependencies selected against primary documentation: lettre 0.11.23 (`default-features=false`, `builder,smtp-transport,tokio1-rustls,ring,webpki-roots`), image 0.25.10 (`default-features=false`, `png,gif,webp`); rfd 0.17.2 (`default-features=false`, `xdg-portal`, no GTK). Sources: https://docs.rs/crate/lettre/0.11.23/features, https://docs.rs/image/0.25.10/image/struct.ImageReader.html, https://docs.rs/crate/rfd/0.17.2/features.
- Source audit: `protocol.rs:31` is wire 38; `store/schema.rs:19` is schema 10; `schema.rs:410` uses the unsafe MAX(version) history gate. `ui/keyboard.rs` activates Enter using implicit local focus without a preceding Tab. Runtime Enter-Enter reproduction pending: no P75 client build or running isolated game was available at S0; source inspection is not runtime evidence.
- Validation: `git diff --check` passed. Cargo checks deferred until final scoped consolidation (new worktree has no build cache). Tests will run only at S14.

### Steps
| Step | Commits | Work-front check (command → result) | Tests written | Deferred |
|---|---|---|---|---|
| S11 | This step commit | `cargo fmt -p tbg-client -p tbg-server`; `git diff --check`; PNG signatures and i18n/visual JSON inspected | Actual keyboard system Enter/Enter regression, charset, transport allowlist, PNG predecode dimensions, frame timing | Native/WASM checks, real flows, private preview/library polish and Linux/Windows/web captures queued S14. PR #57 vault integration requires consolidated verification. |
| S10 | This step commit | `cargo fmt -p tbg-core -p tbg-server`; `git diff --check` passed | `contract_p75_emoji`: hostile dimensions/truncation/polyglots/frame count, standard IDs, private upload → approval → equip → revoke, content hash and cache reauthorization | Tests queued S14; 2 s timing and real animated-client presentation pending. |
| S9 | This step commit | `cargo fmt -p tbg-core -p tbg-server`; `git diff --check` passed | Appended wire goldens; real two-seat chat/guest boundary and replay hash test; sanitizer and limit contracts | Tests queued S14; actual spectator attachment/reconnect/mid-match checks pending extension. |
| S8 | This step commit | `cargo fmt -p tbg-server`; `git diff --check` passed | `contract_p75_reports`: participant eligibility, duplicate race, frozen transcript, forged target rejection, privacy and daily bound | S9 populates live transcript/loadouts; tests and real-client filing queued S14. |
| S7 | This step commit | `cargo fmt -p tbg-server`; `git diff --check` passed | Every area route inventoried with success/unauthenticated denial; moderator ceiling, current disabled state, masked-email projection | Runtime tests queued S14; console walkthrough pending S12. |
| S6 | This step commit | `cargo fmt -p tbg-server`; `git diff --check` passed | `contract_p75_admin`: bootstrap, claim, session cookie, separation, CSRF/Origin, idle expiry, atomic audit failure, append-only guards; trusted proxy unit case | API area inventory completes S7; tests queued S14; no real email or host CLI action. |
| S5 | This step commit | `cargo fmt -p tbg-core -p tbg-server`; `git diff --check` passed | Moderation reversibility, two-slot issuance/expiry, optimistic race, exact forfeit state and no voluntary surrender signal | S9 adds suspension wire delivery; S7 extends API and switch tests; full targeted run S14. |
| S4 | This step commit | `cargo fmt -p tbg-server`; `git diff --check` passed | `contract_p75_recovery`: discovered FK snapshot, key format, two-slot revocation, generic replies, code budgets and secret-at-rest canaries | Test run queued S14; real socket closure and full UI recovery remain pending. |
| S3 | This step commit | `cargo fmt -p tbg-server`; `git diff --check` passed | `contract_p75_identity` and preserved P67 offline scenarios; BR-P01/02 helper amended with canon | Compilations/tests queued S14; live verification and desktop captures pending. |
| S2 | This step commit | `cargo fmt -p tbg-server`; `git diff --check` passed | Mail capture, two-writer lease, retry exhaustion, queue bound, code-at-rest and Debug canaries | Cold compilation and tests queued for S14; SMTP never contacted. |
| S1 | This step commit | `cargo fmt -p tbg-server`; `git diff --check` passed | Appended IC-13 history-after-v12 and damaged-shape tests | Compilation cold; tests queued for S14. Legacy recovery fixture API conversion follows in S3. |

### Consolidation (S14)
- Commands and real output:
- Acceptance A01–A12 (observed / pending + reason):
- Visual correspondence (Linux / Windows / web):

### Deviations, blocked items, residual risks
> blocked: S11 initially stopped after two missing-script errors. Auto-review had rejected the unapplied draft because it removed an existing key backup without persisting its replacement. The owner explicitly authorized resuming S11 on 2026-11-02. The replacement implementation never removes backups and uses PR #57's protected vault namespace. No unsafe draft was applied or run.

- S9 freezes social profiles in one store read during pairing setup before the first MatchStart, outside all simulation ticks. The match owns frozen rights/loadouts; live restrictions and switches use the shared cache. No SQL runs during social relay.
- D-110's explicit protocol bump updates only the existing current-version assertion in `contract_protocol`; pre-P75 encoding goldens remain untouched and P75 encodings are appended.

- S6 pre-session mutations use a Secure host-only nonce cookie from unauthenticated `GET /session`, echoed in the CSRF header, plus exact Origin and JSON. Once signed in, CSRF is session-bound and reconstructed from the opaque session token. This defines the necessary pre-authentication CSRF bootstrap absent from the planning API.

- S4 reuses the exact `revoke_account` barrier through `revoke_scope` while holding the existing D-97 coordinator lock; calling `revoke_account` recursively would deadlock. SQL revocation is also committed with key rotation.
- The recovery FK snapshot treats `account_owner` as auth/session ownership (revoked on recovery), alongside `auth_sessions`; all other non-recovery account FK tables are discovered rather than enumerated.
- The P67 offline link fixture remains test-only for its historical contracts; production uses only typed codes and the live P75 tables. No fixture-only schema installer remains.


### PR
- URL, branch, final commit, pending owner gates:
