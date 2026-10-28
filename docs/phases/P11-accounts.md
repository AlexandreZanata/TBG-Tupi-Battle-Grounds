# P11 — Accounts: unique player name + account key

**Depends on:** P10 reviewed and merged.
**Branch:** `phase/p11-accounts`.
**Difficulty:** medium–high (first persistent player data; security matters).

## Goal

A player picks a unique name and receives an **account key** — their name
plus a random secret — which is the only credential. No email, no
password, no personal data. The key logs the player in on any device.
Guests keep playing practice, casual, and private rooms exactly as today.

## Deliverable

- Rules BR-P01–BR-P05 (GAME-DESIGN §3) implemented server-side.
- SQLite `accounts` table behind a single-writer store task.
- Protocol v2 with `Register`, `Login`, `Logout` and their replies.
- Client: create-account and enter-key screens (HTML overlay for text
  input), "save your key" screen, automatic login on startup, profile chip
  in the menu.
- `tbg-bots --accounts`: bots register and log in (for load tests).

## Read first (only these)

- `AGENTS.md`
- `docs/GAME-DESIGN.md` §2 (glossary rows `Account`, `AccountKey`,
  `PlayerName`) and §3 rules BR-P01–BR-P05
- `docs/DECISIONS.md` D-17 (accounts) and D-20 (HTML text input)
- `docs/ARCHITECTURE.md` §5 (sessions), §8 (persistence), §11 (security)
- Core: `src/protocol.rs`; Server: `src/lobby.rs`, `src/lobby/matchmaking.rs`,
  `src/records.rs`, `src/ws.rs`, `src/ws/client_msg.rs`, `src/chan.rs`
- Client: `src/net.rs`, `src/config.rs`, `src/ui/menu.rs`, `index.html`

## Do NOT

- Store or log IP addresses, keys, or key hashes (logs may contain the
  account id and name only).
- Add email, passwords, OAuth, recovery flows, or name changes.
- Let the client decide anything about accounts; it only displays replies.
- Touch ranked play or ratings beyond reading the default columns (P12).

## Dependencies you may add

| Crate | Where | Why |
|-------|-------|-----|
| `sha2 = "0.10"` | server | hash the key secret before storing it |
| `web-sys` features `Document`, `Element`, `HtmlElement`, `HtmlInputElement`, `HtmlAnchorElement` | client (wasm only) | HTML overlay for text entry and the key download |

## Exact formats

- **PlayerName:** regex `^[A-Za-z][A-Za-z0-9_]{2,15}$` (3–16 ASCII chars).
  Unique case-insensitively (`name_lower`); displayed exactly as typed.
  Rejected when `name_lower` equals a `RESERVED_NAMES` entry or contains any
  `BLOCKED_SUBSTRINGS` entry after mapping `0→o 1→i 3→e 4→a 5→s 7→t`.
- **AccountKey:** `<PlayerName>-XXXX-XXXX-XXXX-XXXX`. The 16 `X` are the
  **secret**: Crockford base32 (`0123456789ABCDEFGHJKMNPQRSTVWXYZ`) from 10
  random bytes (80 bits, `getrandom`). Parsing: split at the first `-`;
  the secret is the rest without `-`, uppercased, with `O→0`, `I→1`, `L→1`;
  it must be exactly 16 alphabet chars. Max key length 40 bytes.
- **Stored:** `key_hash = SHA-256(secret_16_ascii_bytes)` (32 bytes). The
  secret has 80 random bits, so a fast hash is enough; compare hashes in
  constant time (XOR-fold of all 32 bytes, no early return).

## Steps

### 1. Core: names, keys, protocol v2
- New `tbg-core/src/account.rs` (≤ 200 lines): `MAX_NAME_LEN = 16`,
  `MAX_KEY_LEN = 40`, `RESERVED_NAMES` (`admin, administrator, mod,
  moderator, system, server, tbg, tupi, support, staff, official, owner,
  commander, guest, null, root, bot, player`), `BLOCKED_SUBSTRINGS` (20–40
  common English and Brazilian-Portuguese profanity/slur roots, lowercase;
  flag the list for owner review in the report),
  `pub fn check_name(&str) -> Result<(), AccountError>`,
  `pub fn parse_key(&str) -> Option<(String /*name*/, [u8; 16] /*secret*/)>`,
  `pub fn format_key(name: &str, secret: &[u8; 16]) -> String`,
  `pub fn encode_secret(random: &[u8; 10]) -> [u8; 16]`.
- `protocol.rs`: `PROTOCOL_VERSION = 2`; add
  ```rust
  ClientMsg::Register { name: String }
  ClientMsg::Login { key: String }
  ClientMsg::Logout
  ServerMsg::Registered { key: String, profile: Profile }
  ServerMsg::LoggedIn { profile: Profile }
  ServerMsg::LoggedOut
  ServerMsg::AccountError { code: AccountError }
  pub enum AccountError { NameInvalid, NameTaken, BadKey, RateLimited, Unavailable }
  pub struct Profile { pub name: String, pub rating: u16, pub ranked_games: u32,
                       pub wins: u32, pub losses: u32, pub draws: u32 }
  ```
  (append new variants at the end of each enum.)
- **Check:** `cargo test -p tbg-core` with new tests citing BR-P01/BR-P02:
  valid/invalid names (length, first char, symbols, reserved, blocked with
  digit mapping), key round trip, lowercase/`O`/`I`/`L` normalization,
  wrong group count rejected, oversized input rejected.

### 2. Store task (single SQLite writer)
- Replace `records.rs` with `src/store/` (`mod.rs`, `schema.rs`,
  `accounts.rs`, `matches.rs`), still the only code that touches SQL.
- `mod.rs`: `pub struct Store { tx: std::sync::mpsc::SyncSender<StoreCmd> }`
  (capacity 1024) backed by **one** `std::thread` that owns the
  `rusqlite::Connection`. Every command carries a
  `tokio::sync::oneshot::Sender` for its reply. Match records keep working
  through `StoreCmd::InsertMatch` (fire and forget).
- `schema.rs`: table `schema_version(version INTEGER NOT NULL)`; migrations
  run in order at startup inside a transaction; v1 = the existing `matches`
  table, v2 =
  ```sql
  CREATE TABLE accounts (
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
  );
  ```
  Pragmas: `journal_mode=WAL`, `synchronous=NORMAL`, `foreign_keys=ON`.
- Commands: `Register { name, key_hash }` → `Result<(AccountId, Profile), AccountError>`
  (`NameTaken` on the UNIQUE violation); `Login { name_lower, key_hash }` →
  `Result<(AccountId, Profile), AccountError>` (`BadKey` when the name is
  unknown **or** the hash differs — same error, same code path);
  `InsertMatch(Value)`.
- **Check:** unit tests on a temp file DB: migrate twice is a no-op; register,
  duplicate name in other case, login ok, login bad secret, login unknown name.

### 3. Lobby: account sessions
- `Session` gains `account: Option<(AccountId, Profile)>` and `ip: IpAddr`
  (memory only, used for rate limits; ws.rs passes it in `Hello`).
- `Register`/`Login`/`Logout` are accepted only when the session is `Idle`
  (else `AccountError::Unavailable`). The lobby never awaits the store
  inline: it sends the command, spawns a tiny task that awaits the reply,
  and that task posts `LobbyCmd::StoreReply { token, result }` back.
- `Register`: check name in core → rate limit → generate secret
  (`getrandom`) → hash → store → reply `Registered { key, profile }`. The
  plain key exists only in that reply.
- `Login`: parse key (`BadKey` if unparsable) → rate limit → store. On
  success, if another session already holds this `AccountId` (BR-P03), the
  new connection **takes over that session** exactly like a resume (old
  socket gets `Close`; if it was in a match the new connection is attached
  to it), and the now-empty guest session is dropped. Reply `LoggedIn`.
- `Logout`: clear `account`, keep the guest session, reply `LoggedOut`.
- The session `name` becomes the account name while logged in (opponents
  see it in `MatchStart`).
- Rate limits (new `src/limits.rs`, in-memory, per IP, fixed windows,
  cleaned every minute): register 3 per hour (env
  `TBG_REGISTER_PER_IP_HOUR`), login 10 per minute
  (`TBG_LOGIN_PER_IP_MIN`). Exceeding → `RateLimited`.
- Keep every file ≤ 200 lines (split `lobby/accounts.rs`).

### 4. Integration tests (`tests/accounts.rs`)
Cite the rule in each test name/comment:
register → `Registered` with a key that `parse_key` accepts; same name in
another case → `NameTaken`; invalid name → `NameInvalid`; new socket +
`Login{key}` → same profile; tampered secret → `BadKey`; 4th register from
one IP within the hour → `RateLimited`; second login of an account that is
in a match takes the match over (old socket closed, new one receives
`MatchStart` + `View`).

### 5. Client: text entry overlay
- `index.html`: a hidden `<form id="tbg-form">` centered over the canvas with
  one `<input id="tbg-input" maxlength="40" autocomplete="off"
  autocapitalize="off" spellcheck="false">`, a submit and a cancel button,
  and a `<p id="tbg-form-error">`. Minimal inline CSS (dark panel, large
  font, 48 px tall controls). No JS in the page.
- New `src/ui/textform.rs`: `open(kind, prompt)`, `close()`, and a system
  that reads `submit`/`cancel` clicks via web-sys listeners stored in a
  `NonSend` resource (closures kept alive with `Closure::forget` is not
  allowed — store them). Native build: show a toast "Accounts need the web
  build".
- **Done when** typing, pasting, and the phone keyboard all work.

### 6. Client: account flow
- Menu shows a profile chip: guest → `Guest · Create account · I have a key`;
  logged in → `Name · 1000`.
- **Create account:** name form → `Register` → **Save your key** screen: the
  key in large monospace, **Copy** and **Download** (`<Name>-tbg-key.txt`
  via a `data:` URL anchor), text "This key is your account. Anyone with it
  can play as you. It cannot be recovered.", and **I saved it** (enabled
  after Copy or Download). Then store the key in `localStorage`
  (`tbg_account_key`).
- **I have a key:** key form → `Login`.
- Startup: if `tbg_account_key` exists, send `Login` right after `Welcome`.
  On `BadKey` at startup, keep the stored key, show a toast, stay guest.
- Profile panel: **Log out** asks for confirmation ("Did you save your
  key?"), sends `Logout`, deletes the stored key.
- Errors map to short messages: `NameTaken` "That name is taken",
  `NameInvalid` "3–16 letters, digits or _; start with a letter",
  `RateLimited` "Too many tries, wait a bit", `Unavailable` "Finish your
  match first".

### 7. Bots with accounts
`tbg-bots --accounts`: each bot registers `Tb` + 10 random alphanumerics,
then plays logged in. Document in the report that load tests need
`TBG_REGISTER_PER_IP_HOUR` raised on the test server only.

### 8. Docs
ARCHITECTURE §5 (messages, sessions), §8 (store task, `accounts` table),
§11 (key handling, rate limits); README "Run it locally" note about accounts.

## Acceptance checklist

- [ ] All verification commands from `AGENTS.md` pass; `trunk build --release` passes
- [ ] Core + store + integration tests pass, each citing its BR-P rule
- [ ] Browser: create account on desktop, copy key, log in with it on a phone;
      reload keeps you logged in; log out works
- [ ] `grep` of server logs after the test run shows no key text and no IP
- [ ] Guest practice, casual quick match, and private rooms unchanged

## Risks and what to do

| Problem | Action |
|---------|--------|
| HTML overlay steals Bevy keyboard input | Only one input exists and only while open; Bevy ignores keys while `TextForm` is open |
| SQLite busy under load | One writer thread already serializes writes; keep transactions short |
| Name squatting by bots | Register rate limit; owner can delete accounts by SQL (document in deploy README) |

## Reviewer checklist

- `BadKey` is identical for unknown name and wrong secret (no name oracle
  beyond `NameTaken` at registration, which is unavoidable).
- Key appears only in the `Registered` reply and the client's own storage.
- Store is the only SQL user; lobby never blocks on it.
- Constant-time hash comparison has no early return.

## Phase report

Executed 2026-10-28 on the dev PC (branch `phase/p11-accounts`).

### Verification commands (all pass)

```text
$ cargo fmt --all -- --check            # clean
$ cargo clippy --workspace -- -D warnings   # clean
$ cargo test --workspace               # ok: core 6 account + 15 rules, server 10 account + 12 online
$ cargo check -p tbg-client -p tbg-core --target wasm32-unknown-unknown  # ok
$ trunk build --release                # success (dist/ rebuilt with #tbg-form)
```

New tests: `tbg-core/tests/accounts.rs` (6: name shapes, reserved,
blocked+leetspeak, key round trip, `O`/`I`/`L`/lowercase normalization,
bad-key shapes) and `tbg-server/tests/accounts.rs` (4 store-level:
migrate-twice no-op, register→login, duplicate-name `NameTaken`,
`BadKey` shared for bad secret and unknown name; 6 protocol-level:
`Registered` key parses, `NameTaken`/`NameInvalid`, login same profile +
tampered `BadKey`, 4th-register `RateLimited`, BR-P03 takeover with old
socket closed and new socket receiving `MatchStart` + `View`, in-match
register/logout `Unavailable`).

`tbg-bots --matches 2 --accounts` (with `TBG_REGISTER_PER_IP_HOUR`
raised on the test server only): 4/4 bots registered as `Tb…` and
completed matches. Server log `grep` after the run: no key text, no IPs,
no hashes. The 4 rows exist in SQLite with rating 1000, 0 ranked games.

### Browser checks (owner-pending, no devices attached)

Create account on desktop, Copy key, log in with it on a phone; reload
keeps you logged in; Log out works; typing/pasting/phone keyboard in the
overlay. Code paths are in place (probe auto-login, `localStorage` key,
two-tap logout).

### Deviations

- Added `wasm-bindgen = "0.2"` (wasm-only target) next to the allowed
  web-sys features: `Closure`/JS casts live there. One-line reason in the
  crate `Cargo.toml`.
- Rate limits are a sliding log (at most N attempts in any trailing
  window) rather than strict fixed windows: simpler, strictly tighter.
  Same env knobs, same defaults.
- `Register` attaches the account to the session immediately, so the menu
  profile chip and the save-your-key screen work in one step. The plain
  key still exists only in the `Registered` reply and client storage.
- `tbg-bots --accounts` registers then quick-matches on the attached
  session; the `Login{key}` path is covered by integration tests.
- Account traffic reuses the idle probe socket (`NetLink::reuse`); a
  fresh listen-only link opens only when there is no usable socket.
- `BLOCKED_SUBSTRINGS` (30 EN + PT-BR roots) is a starting list flagged
  for owner review (P11 human checkpoint; see the `ponytail:` comment in
  `tbg-core/src/account.rs`).
- `two_bots_finish_a_match` timed out once when the full workspace suite
  ran online + account binaries in parallel (environmental contention;
  green alone and on re-run). Same class as the P10 `rate_limit` flake.

### Blocked items

- Owner checkpoint P11: review the blocked-name list; create an account
  on two devices (login takeover covered by test, needs a human pair).
