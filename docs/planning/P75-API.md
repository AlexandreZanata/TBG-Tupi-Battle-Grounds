# P75 interfaces — wire protocol, player HTTP, operator API, mail

Companion to [P75](../phases/P75-admin-moderation-and-match-social.md).
Requirement IDs refer to [P75-REQUIREMENTS](P75-REQUIREMENTS.md); IC/S IDs
to [P75-CONTRACTS](P75-CONTRACTS.md). Every shape here is **Proposed**:
freeze the actual discriminants and the next free protocol number at S0.

## 1. WebSocket protocol (one bump: 38 → 39 on the audited base)

Rules: append only (IC-05). On the audited base `ClientMsg` has tags 0–26
(`Hello` … `SetPlayerChallengesEnabled`) and `ServerMsg` tags 0–36
(`Welcome` … `MatchHistoryStatus`); they keep their exact shapes and no
existing variant gains a field. The new variants below therefore take
`ClientMsg` 27–28 and `ServerMsg` 37–41 unless S0 finds a newer base.
New types live in `crates/tbg-core/src/protocol/social.rs` (re-exported
like `protocol/history.rs`), without I/O, clocks or floats.

```rust
pub const CHAT_MAX_CHARS: usize = 120;
pub const EMOJI_SLOTS: usize = 3;
/// Standard emoji ids 0..STANDARD_EMOJIS are permanent (EM-01, IC-11).
pub const STANDARD_EMOJIS: u8 = 16;
pub const CUSTOM_EMOJI_MAX_FRAMES: u8 = 24;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum EmojiRef {
    Standard { id: u8 },
    /// `id` is the approved asset's random public id (CE-05). Sheet layout:
    /// `frames` cells of `frame_px`², row-major, `columns` per row.
    Custom { id: [u8; 16], frame_px: u8, frames: u8, columns: u8, delays_ms: Vec<u16> },
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum ChatPermission {
    Allowed,
    /// Kill switch off, or a mode without chat.
    Disabled,
    Guest,
    Unverified,
    Restricted { until_unix: u64 },
}

/// Sent after `MatchStart` on every (re)attach and again whenever a seat's
/// rights or a loadout change mid-match (restriction, kill switch, revoke).
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct MatchSocial {
    /// The receiving seat's own right to send text.
    pub chat: ChatPermission,
    /// The receiving seat may send emojis.
    pub emojis: bool,
    /// Custom emojis are displayed (else each custom slot shows its default).
    pub custom_emojis: bool,
    /// Frozen at pairing, by `Seat::index()` (EM-03).
    pub loadouts: [[EmojiRef; EMOJI_SLOTS]; 2],
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum SocialRefusal { RateLimited, Duplicate, InvalidText, Restricted, Disabled, NotAvailable }

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum SuspensionReason {
    Cheating, AbusiveChat, OffensiveName, OffensiveEmoji,
    Harassment, BanEvasion, Impersonation, Other,
}
```

Appended variants (after `SetPlayerChallengesEnabled` / `MatchHistoryStatus`):

| Message | Direction | Meaning |
|---|---|---|
| `ClientMsg::ChatSend { text: String }` | client → server | One chat line from the connection's seat (CH-04) |
| `ClientMsg::EmojiSend { slot: u8 }` | client → server | Send the frozen emoji in slot 0–2 (EM-04) |
| `ServerMsg::MatchSocial(Box<MatchSocial>)` | server → seat | Social rights and both loadouts |
| `ServerMsg::Chat { from: Seat, text: String }` | server → both seats | Relayed, sanitized, masked text |
| `ServerMsg::Emoji { from: Seat, slot: u8 }` | server → both seats | Relayed emoji; clients resolve the loadout |
| `ServerMsg::SocialRefused { reason: SocialRefusal }` | server → sender | Nothing was relayed |
| `ServerMsg::AccountSuspended { until_unix: Option<u64>, reason: SuspensionReason }` | server → banned account's sockets | Sent just before the sockets close (MD-03); `None` = permanent |

Server rules:

- The seat comes from the connection (`c.matched`), never from the message.
  Spectators and idle sessions get `SocialRefused { NotAvailable }`.
- Like `Intent`, an admitted seat may chat after natural Access expiry; a
  revoked binding is refused first (existing `client_msg.rs` order).
- The match task validates, rate limits, masks and relays in the same
  turn; it never touches `MatchState`, the replay log or the state hash.
- No social message is ever sent to spectators or written into views.
- Client decoders reject `EmojiRef::Custom` with `frames` outside 1–24,
  `delays_ms.len() != frames`, `columns` outside 1–6 or `frame_px` outside
  16–128, and text over `CHAT_MAX_CHARS`.

## 2. Player HTTP routes (public game router)

All JSON bodies use `#[serde(deny_unknown_fields)]`; responses carry
`Cache-Control: no-store` except where stated; errors are
`{"error":"<code>"}` with the status listed. Signed-in routes use the
P73 `history_binding` path: Bearer Access, channel/origin policy, slot
fence (`X-TBG-Slot`). Rate limits that must hold across slots are stored
in SQLite (`recovery_dispatch_limits`-style buckets), not in `Limits`.

### 2.1 Auth operations (`POST /api/auth/{browser|native}/{operation}`)

| Operation | Body | Success | Errors |
|---|---|---|---|
| `register` (changed) | `{name, email, locale, device_id, persistent, registration_retry_key}` | Existing grant + `account_key` + `"email_state":"unverified"` | existing codes + `email_required` 400, `email_invalid` 400, `email_blocked` 403, `email_limit` 409, `registration_closed` 503 |
| `recovery-request` (new) | `{name, email, locale}` | `202 {"ok":true}` for every well-formed body (RC-01) | `bad_request` 400 (malformed JSON/shape only), `rate_limited` 429 (per-IP only) |
| `recover` (new) | `{name, code, device_id, persistent, registration_retry_key}` | Grant + `account_key` (new key, shown once) | `code_invalid` 401 (unknown name, wrong/expired/used code, too many attempts — one code), `rate_limited` 429 |
| `login`, `refresh`, `confirm` (changed) | unchanged | unchanged | + `account_suspended` 403 with `{"until":<unix or null>,"reason":"<category>"}` |

- `locale` is `"en"` or `"pt-BR"`; it selects email language only.
- `registration_retry_key` keeps D-97 retry semantics for `recover` too:
  a lost reply retried with the same key returns the same grant within
  the existing 300 s cache; a different body conflicts.
- The registration contract (name rules, refund of failed attempts,
  per-IP cap) is preserved; the email is validated **before** the
  register slot is consumed, so an invalid email never counts (IC-14).

### 2.2 Account email (`/api/me/...`, Bearer)

| Route | Body | Success | Notes |
|---|---|---|---|
| `GET /api/me/account` | — | `{name, email_masked, email_state:"none"|"unverified"|"verified", pending_change:{until}|null, chat_until, emoji_until, can_chat, can_upload}` | Never returns the full address to the client after registration |
| `POST /api/me/email/verify` | `{code}` | `{"email_state":"verified"}` | `code_invalid` 401 |
| `POST /api/me/email/resend` | `{}` | `202` | 3/h per account and destination |
| `POST /api/me/email` | `{account_key, email, locale}` | `202 {"step":"verify_new"|"approve_old"|"hold"}` | Legacy add (ID-05), routine change (ID-06) or 7-day hold (ID-07, `"lost_old":true` adds this flag) |
| `POST /api/me/email/change/confirm` | `{old_code, new_code}` | `{"email_state":"verified"}` | Routine change completion |
| `POST /api/me/email/change/cancel` | `{}` | `200` | Also cancellable with the old-address code via `{old_code}` |

### 2.3 Reports (`POST /api/me/reports`, Bearer)

```json
{"context":"match","result_id":"<32 hex>","reason":"abusive_chat","comment":"optional"}
{"context":"profile","target":"PlayerName","reason":"offensive_name"}
{"context":"emoji","result_id":"<32 hex>","emoji":"<32 hex public id>","reason":"offensive_emoji"}
```

`201 {"id":<n>}` · `409 duplicate` · `422 not_eligible` (RP-04) ·
`429 rate_limited` · `403 forbidden` (guest/suspended) · `503 reports_disabled`.
The match target is the **other seat** of the reporter in that result's
social record; the client never names the match target.

### 2.4 Emojis

| Route | Auth | Success | Notes |
|---|---|---|---|
| `GET /api/me/emoji` | Bearer | `{slots:[ref×3], uploads:[{id, label, status, reason, created}], limits:{per_day, total}}` | `ref` is `{"standard":n}` or `{"custom":"<public id>"}` |
| `POST /api/me/emoji?label=<url-encoded>` | Bearer, body = raw bytes, `Content-Type: image/png|image/gif|image/webp`, ≤ 1 MiB | `201 {"id":n,"status":"pending"}` | `413 too_large`, `415 unsupported_type`, `422 image_invalid` with `{"detail":"dimensions|frames|duration|encoded_size|decode"}`, `403 unverified|restricted`, `429`, `503 queue_full|uploads_disabled` |
| `GET /api/me/emoji/{id}/sheet.png` | Bearer, owner only | PNG, `no-store` | Private preview of the owner's own pending/approved upload |
| `DELETE /api/me/emoji/{id}` | Bearer, owner only | `200` | Withdraw; clears slots that use it |
| `PUT /api/me/emoji/slots` | Bearer | `200 {slots}` | Exactly 3 refs; custom refs must be the caller's approved assets |
| `GET /api/emoji/{public_id}.png` | none | PNG, `Cache-Control: public, max-age=31536000, immutable` | Approved only; anything else is `404` (IC-17) |

Upload bodies get their own `DefaultBodyLimit` (1 MiB); every other new
route keeps ≤ 4 KiB. Every signed-in route lives under `/api/me/` on
purpose: Caddy already routes `/api/me/*` with `X-TBG-Slot` to the slot
holding the Access index (`deploy/Caddyfile.domains.template`); a signed-in
route elsewhere would reach slot a only and fail for slot-b sessions.
`/api/emoji/*` joins the `@shared` read-only matcher (both slots). Image work runs in `spawn_blocking` behind a
2-permit semaphore with a 2 s budget (`503 busy` when saturated).

## 3. Operator API (`/admin-api/v1/*`, dedicated loopback listener)

**Listener.** Enabled by `TBG_ADMIN=on`. Slot mode binds
`127.0.0.1:${TBG_ADMIN_PORT_A:-8090}` for slot `a` and
`127.0.0.1:${TBG_ADMIN_PORT_B:-8092}` for slot `b` (mirroring the 8080/8082 game ports); single-server mode binds
`TBG_ADMIN_ADDR` (default `127.0.0.1:8090`). Non-loopback addresses are
refused at startup. Caddy forwards only `admin.tupibg.com/admin-api/*`
(both ports, `lb_policy first`). The public router never mounts these
handlers (IC-10).

**Every request:** client IP from the existing trusted-proxy rule;
optional `TBG_ADMIN_ALLOW_IPS` (comma-separated CIDRs) checked first;
response headers `Cache-Control: no-store`, `X-Content-Type-Options: nosniff`,
`X-Request-Id: <32 hex>`. **Every mutation:** `Origin` equal to
`TBG_ADMIN_ORIGIN` (e.g. `https://admin.tupibg.com`),
`Content-Type: application/json`, header `X-TBG-CSRF` equal to the
session's CSRF token (constant-shape compare), body ≤ 8 KiB, unknown
fields rejected, idempotent replays rejected by `expected_version`.

### 3.1 Claim and session

| Route | Body | Result |
|---|---|---|
| `POST /claim/request` | `{email}` | `202` always; queues `operator_claim` if `email` is a non-disabled operator (AD-04/06) |
| `POST /claim/complete` | `{email, code}` | `200 {"admin_key":"TBGADM-…"}` once; status → active, `key_epoch`+1, all sessions revoked; `401 code_invalid` |
| `POST /session/start` | `{admin_key}` | `202 {"attempt":"<32 hex>"}` and a login code by email (AD-07); with the second factor off: `200` + session |
| `POST /session/verify` | `{attempt, code}` | `200 {operator:{id, role, email_masked}, csrf, idle_expires, absolute_expires}` + `Set-Cookie: __Host-tbg_admin=…; Path=/; Secure; HttpOnly; SameSite=Strict` |
| `POST /session/reauth` | `{admin_key}` | `200` (sets `reauth_unix`) |
| `POST /session/logout` | `{}` | `200`, cookie cleared |
| `GET /session` | — | `200 {operator, csrf, …}` or `401` |

Claim/start/verify errors are uniform (`401 code_invalid` /
`401 auth_required`) so they never reveal whether an email is an operator.

### 3.2 Areas (capability per §4.4 of the requirements)

| Route | Capability | Notes |
|---|---|---|
| `GET /overview` | any | Counts: online, live matches, open reports, pending emojis, active bans, mail queue/failures, slot states |
| `GET /accounts?prefix=&cursor=` | any | Name prefix ≥ 2 chars, page 20 (max 50) |
| `GET /accounts/{name}` | any | Profile, ratings, email (masked; full for Owner), email state, moderation state + version, reports for/against, last 20 moderation actions, last 10 social records |
| `POST /accounts/{name}/ban` | ban (≤30 d) / ban.long | `{duration_hours|permanent:true, reason, note, report_id?, block_email:false, expected_version}`; sensitive (reauth ≤ 5 min) |
| `POST /accounts/{name}/unban` | ban.long | `{reason, note, expected_version}`; sensitive |
| `POST /accounts/{name}/chat` | restrict | `{duration_hours|0, reason, note, expected_version}` (0 lifts) |
| `POST /accounts/{name}/emoji` | restrict | same shape for emoji restriction |
| `POST /accounts/{name}/revoke-sessions` | restrict | `{reason, note}` |
| `POST /accounts/{name}/send-recovery` | owner | `{reason}`; sends a recovery code to the verified address only (MD-09) |
| `GET /reports?status=open&cursor=` / `GET /reports/{id}` | any | Detail includes frozen evidence (transcript, names, outcome, emoji preview link) |
| `POST /reports/{id}/resolve` | any | `{outcome:"actioned"|"dismissed", note, expected_status:"open"}`; actions are separate calls carrying `report_id` |
| `GET /emoji?status=pending&cursor=` / `GET /emoji/{id}` | any | Metadata, uploader, frame list |
| `GET /emoji/{id}/sheet.png` | any | Private normalized sheet (`no-store`), never the original upload |
| `POST /emoji/{id}/approve` · `/reject` · `/revoke` | review | `{reason?}`; approve/revoke emit feed events |
| `GET/POST /chat/terms`, `DELETE /chat/terms/{term}` | review | Terms 2–32 chars after folding |
| `GET /settings`, `PUT /settings/{key}` | owner (write) | `{value:0|1, reason}`; keys from `admin_settings` |
| `GET/POST /email-blocks`, `DELETE /email-blocks/{email}` | owner | Exact normalized addresses |
| `GET /operators`, `POST /operators` `{email, role:"moderator"}`, `POST /operators/{id}/disable` · `/enable` | owner | Owner row cannot be disabled here |
| `GET /audit?target_kind=&target_id=&operator=&cursor=` | owner | Read-only |
| `GET /system` | any | Mail configured yes/no, outbox counts, last error code, feed lag, build/slot identity |

## 4. Mail

Transport enum (two implementations, so no trait is needed):
`Transport::Smtp` (lettre 0.11, tokio, rustls with the ring provider and
webpki roots — verify exact feature names against the crate at S0; never
native-tls/OpenSSL on the server) and `Transport::Capture` (tests).
Plain-text bodies only, compiled EN/pt-BR templates, `From: TBG_MAIL_FROM`,
optional `Reply-To`, `Auto-Submitted: auto-generated`. Bodies contain the
purpose, the code and its expiry, a plain link to `TBG_PUBLIC_GAME_URL` or
`TBG_PUBLIC_ADMIN_URL` **without any token**, and "if this was not you,
ignore this email". Codes: 10 Crockford characters as `XXXXX-XXXXX`
(50 bits) bound to principal + purpose + epoch with 5 attempts.

| Purpose | Recipient | Code TTL |
|---|---|---|
| `account_verify` | new/changed address | 24 h |
| `account_recovery` | verified address | 15 min |
| `account_key_rotated`, `account_email_changed` | verified (and old) address | notice |
| `account_change_old`, `account_change_new` | old / new address | 24 h |
| `operator_claim` | operator address | 24 h |
| `operator_login` | operator address | 10 min |
| `operator_key_rotated`, `operator_login_notice` | operator address | notice |

Configuration (host files, never `Config`'s Debug output — keep secrets
in a separate struct without `Debug`):

| Variable | Example | Secret |
|---|---|---|
| `TBG_MAIL_FROM` | `Tupi Battle Grounds <no-reply@tupibg.com>` | no |
| `TBG_SMTP_HOST` / `TBG_SMTP_PORT` | provider host / `587` | no |
| `TBG_SMTP_SECURITY` | `starttls` (default) or `tls` | no |
| `TBG_SMTP_USERNAME` | provider login | low |
| `TBG_SMTP_PASSWORD_CREDENTIAL` | `smtp_password` → read `$CREDENTIALS_DIRECTORY/smtp_password` | **yes, file only** |
| `TBG_PUBLIC_GAME_URL` / `TBG_PUBLIC_ADMIN_URL` | `https://app.tupibg.com` / `https://admin.tupibg.com` | no |
| `TBG_ADMIN`, `TBG_ADMIN_ORIGIN`, `TBG_ADMIN_PORT_A/B`, `TBG_ADMIN_ADDR`, `TBG_ADMIN_ALLOW_IPS`, `TBG_ADMIN_LOGIN_CODE` (`on` default) | see §3 | no |
| `TBG_ADMIN_BOOTSTRAP_EMAIL` | Owner address (operations runbook) | private, not secret |

Mail is "configured" only when host, username, password credential and
sender are all present; otherwise the dispatcher idles and rows wait.
`--build-info` must keep working with no environment at all.
