# P68-A — Persistent authentication for Linux, Windows and web

**Status:** planning only, owner scope extension on 2026-10-31. This is part
of P68, not a separately numbered phase. No credentials, database migration,
server deployment or authentication implementation is delivered here.
Read [P68](../phases/P68-windows-native-release.md), GAME-DESIGN §21 and D-97.
The [source manifest](P68-AUTH-SOURCE-MANIFEST.csv) pins the relevant existing
files by hash/line count; it is source identity evidence, not passing tests.

## 1. Requested policy and audited starting point

The owner requests **15-minute Access Tokens and 30-day Refresh Tokens**
and asks that the attached authentication observations inform this plan.
The attachment is design input; its sample SQL and library links are not
an implementation specification or evidence that code already exists.
Adopt its absolute 30-day family lifetime, rotation, secure platform stores,
manual AccountKey fallback and separation from match reconnect. The TTLs
are product policy, not durations mandated by an authentication standard.

Inspected on the required `55510c5` game source, unchanged by the preceding
P68 documentation commits:

| Current behavior | Source / consequence |
|---|---|
| AccountKey contains an 80-bit random secret encoded into 16 characters | `tbg-core/src/account.rs`, `tbg-server/src/lobby/accounts.rs`; preserve existing key format and validity |
| SHA-256 of the encoded secret is persisted | `tbg-server/src/store/accounts.rs`; reuse the verifier and generic bad-key errors, not a duplicate credential database |
| Automatic login already exists | `tbg-client/src/net/{login,socket}.rs`, `ui/start.rs`; replace its credential source and pending-state behavior |
| AccountKey is stored under `tbg_account_key` | `config.rs`, `platform/storage.rs`; browser localStorage and native preferences JSON are not credential vaults |
| Account UI reads the saved key for copy/download | `ui/account_ui.rs`, `ui/account.rs`; migrate backup/copy UX without inventing a recoverable key from a token |
| 128-bit `Token` resumes a lobby/account and match connection | `protocol.rs`, `lobby/sessions.rs`; currently its possession can resume an account without a new credential check |
| Per-tab reconnect data uses `tbg_token` in sessionStorage; native Session scope is memory-only | `config.rs`, `platform/storage.rs`; preserve the short reconnect purpose, remove standalone account authority |
| Manual login takes over an old account connection, including its match | `lobby/account_done.rs`; reuse seat transfer and ranked settlement ordering, prevent automatic refresh/tabs from repeatedly taking over |
| SQLite uses one bounded writer and migrations through v8 | `store/{mod,worker,schema}.rs`; add transactional auth commands here, not SQL on Tokio/Bevy or combat ticks |
| HTTP currently provides read APIs; `/ws` has no Origin check in its handler | `tbg-server/src/lib.rs`, `ws.rs`; add explicit auth routes, browser origin/CSRF policy and binding before resume |
| Shared HTTP helper is a public GET/JSON helper | `tbg-client/src/platform/http.rs`; add an auth-specific POST transport; do not put secrets into that public GET API |

The roster, simulation and match outcomes remain unchanged. The base wire
version is 29; P68-A requires a coordinated new wire version chosen on the
actual integrated source. Do not label protocol 29 packages compatible with
a changed Hello schema. P67 recovery/commerce is not implemented here;
future AccountKey rotation must call the session-revocation seam.

## 2. Tokens and platform contracts

Use opaque, independent CSPRNG tokens of 32 random bytes, represented as
64 lowercase hex characters in HTTP and fixed-size bytes in the wire.
An Access Token identifies account, family, issued/expiry time and the
expected server audience in a bounded server memory index keyed by its
SHA-256. Refresh Tokens are stored only as SHA-256 in SQLite. Random family,
device and operation IDs are 16 bytes; device/operation IDs are metadata,
never a substitute for a credential or proof of physical device identity.
No JWT, embedded client secret, Redis or separate OAuth server is needed.
Use OAuth security guidance as design input without claiming an OAuth flow.

| Platform | Refresh persistence | Access Token | Required behavior |
|---|---|---|---|
| Native Linux | User-session Secret Service through D-Bus; compatible GNOME Keyring/KWallet provider | Process memory only | Locked/unavailable/denied vault produces an actionable localized status and memory-only session; no plaintext fallback |
| Native Windows | Current user's Windows Credential Manager, explicitly local-machine persistence | Process memory only | Stable credential target across ZIP/installer upgrades; no administrator credential, roaming default or preference-file secret |
| Web, HTML and Bevy UI | Server-set `__Host-tbg_refresh` cookie: Secure, HttpOnly, SameSite=Strict, Path=/, no Domain | Tab/WASM memory only | Refresh never appears in JSON/JS, DOM, storage, URL or messages between tabs; browser storage denial is a supported failure |
| Wine | Test the Windows implementation with synthetic credentials in an owned prefix | Process memory only | API success proves compatibility only, not Windows-at-rest security; memory-only fallback remains valid, real Windows proves vault requirements |

Access expiry is `min(issued_at + 900, family_expires_at)`. The family
expires exactly 2,592,000 seconds after initial login; every rotated refresh
inherits that deadline. Cookie Max-Age is the **remaining** family lifetime,
not another 30 days on every refresh. The server decides time; use monotonic
Access deadlines in-process as well as wall-clock expiry. Tests inject time
without sleeping 15 minutes or changing the host clock.

Use a shared localized Keep me signed in choice. Persistent storage requires
the player's choice; make that choice explicit during legacy migration.
For native session-only login keep the refresh only in process memory.
For browser session-only login use an HttpOnly session cookie without
Max-Age/Expires; do not claim browser close reliably deletes it, since
session restore can preserve cookies. Its server-side absolute cap remains
30 days and logout revokes it. If cookies are blocked entirely, allow
Access-only temporary login and disclose that automatic renewal/reopen
cannot work; never expose a refresh in JavaScript as a workaround.

Namespace native vault entries and process coordination by a canonical
HTTPS server origin, installation user, account and stable random device
ID. Keep only nonsecret account/origin/device metadata in preferences.
An override to another server must never receive production credentials;
disable automatic credential forwarding across redirects/origins. HTTPS/WSS
are required publicly; plaintext development is an explicit loopback-only
configuration with synthetic accounts, never a production cookie downgrade.

## 3. Routes and transport boundaries

All auth mutations use JSON POST, a 4 KiB body cap, explicit token/string
bounds, request deadlines and bounded work queues. Responses use
`Cache-Control: no-store`, no secret-bearing redirects and typed public
error codes. Keep auth responses out of Service Worker/browser caches.

Use separate `/api/auth/browser/` and `/api/auth/native/` route groups so a
caller cannot flip a JSON flag to get a browser cookie echoed as a token.

| Suffix | Request / outcome |
|---|---|
| `login` | AccountKey, device ID and persistence choice; verify with the existing account/store logic, create a family, issue Access plus the platform-specific refresh delivery |
| `register` | Existing PlayerName rules, device ID and persistence choice; preserve account/ranked-baseline transaction, show the newly created AccountKey once, issue the session without saving the key |
| `refresh` | Native sends refresh in JSON; browser sends only its cookie. Both send an operation ID. Atomically consume old refresh and issue one successor plus Access |
| `confirm` | Prove the newly issued refresh is usable without rotating or issuing new Access; native/browser credentials follow their separate channels. Return only the opaque matching family ID for persistence/migration confirmation |
| `logout` | Valid current Access or the channel-specific refresh identifies only its family; revoke its Access/connection bindings and clear cookie/vault plus local in-memory/reconnect state after acknowledgment |
| `logout-all` | Require manual AccountKey verification, revoke every family; any supplied Access must match that account. Permit this rate-limited recovery action without Access so the family cap/expired devices cannot lock out the key holder |

Successful login/refresh JSON contains `access_token`, `access_expires_at`,
`session_expires_at`, `session_id` and existing public profile information.
Only native responses include `refresh_token`. Registration adds
`account_key` once. Never return stored hashes or the old refresh. `confirm`
does not confer account/seat authority or create more tokens.

Use `Authorization: Bearer <Access>` for operations requiring current Access,
such as authenticated account operations; a current refresh can authorize
only its own family's refresh/confirm/logout routes. Logout-all always
requires the manual AccountKey and cannot rely on a refresh alone.
Registration retries need a separate
32-byte secret `registration_retry_key`, generated in client memory, and a
bounded response cache that can repeat the original one-time key/grant after
a lost reply. A public name, device ID or operation ID alone must never
retrieve an AccountKey. Never persist that retry key/cache; a client crash
after account creation but before any credential delivery cannot be made
recoverable by a hash-only server. Report that limitation honestly; P67's
future recovery is separate, and no duplicate account is silently created.

Browser routes require an exact configured Origin allowlist, JSON and the
custom `X-TBG-Auth: 1` header; reject simple form content types, null/foreign/
missing origins and invalid Fetch Metadata. No credentialed wildcard CORS.
Keep frontend/auth same-origin in production; do not relax SameSite to make
an arbitrary `?server=` override receive login cookies. Login and logout
need these checks too. The custom-header plus strict origin/content-type
policy is the selected CSRF defense, not SameSite alone.
Load the allowlist from reviewed server configuration, not an untrusted
Host/forwarded header. Apply existing trusted-proxy client-IP rules to HTTP
auth budgets. HttpOnly does not prevent injected same-origin JS from making
requests: preserve CSP, safe text/link rendering and the existing XSS boundary.

Native routes reject browser Origin/Fetch-Metadata/cookie credential input;
use explicit JSON bearer values and `X-TBG-Client: native`. That header is
not a secret or client authentication. Do not offer CORS access to native
routes. A browser cookie cannot be exchanged at a native route. Validate
Origin on every `/ws` upgrade; absent Origin is allowed for native/bot
clients only through the no-cookie path, which still needs Access for an
account. A native client can forge Origin; never treat Origin as identity.

Reuse existing IP admission and login/API budgets, with separate auth
operation buckets so login abuse does not block refresh globally. Add the
family/token-specific bounds in GAME-DESIGN §21. Return 429 with Retry-After,
503 for store/entropy/capacity failure and generic 401 for unknown/expired/
revoked credentials. A known bounded rotation race may return 409 with no
tokens or Set-Cookie. Never log bodies, bearer headers, cookies, AccountKey,
Access, Refresh, resume handles or secret-bearing errors/Debug values.

## 4. Durable model and transaction invariants

The attachment's one-table sketch does not retain enough rotation history
on its own. Use one family table and one hashed refresh-generation table.
Allocate the next migration after v8 on the actual integration base; do not
reuse a number reserved by P67 or another merged phase. The following is a
proposed schema shape, not a live migration:

```sql
CREATE TABLE auth_sessions (
  id BLOB NOT NULL PRIMARY KEY CHECK(length(id) = 16),
  account_id INTEGER NOT NULL REFERENCES accounts(id),
  device_id BLOB NOT NULL CHECK(length(device_id) = 16),
  client_kind TEXT NOT NULL CHECK(client_kind IN ('browser', 'native')),
  created_at INTEGER NOT NULL,
  expires_at INTEGER NOT NULL CHECK(expires_at > created_at),
  last_used_at INTEGER,
  revoked_at INTEGER,
  revoke_reason TEXT
);
CREATE INDEX auth_sessions_account ON auth_sessions(account_id);
CREATE INDEX auth_sessions_expiry ON auth_sessions(expires_at);

CREATE TABLE auth_refresh_tokens (
  token_hash BLOB NOT NULL PRIMARY KEY CHECK(length(token_hash) = 32),
  session_id BLOB NOT NULL REFERENCES auth_sessions(id) ON DELETE CASCADE,
  generation INTEGER NOT NULL CHECK(generation >= 0),
  issued_at INTEGER NOT NULL,
  consumed_at INTEGER,
  operation_hash BLOB CHECK(operation_hash IS NULL OR length(operation_hash) = 32),
  CHECK((consumed_at IS NULL AND operation_hash IS NULL) OR
        (consumed_at IS NOT NULL AND operation_hash IS NOT NULL)),
  UNIQUE(session_id, generation)
);
CREATE UNIQUE INDEX auth_refresh_current ON auth_refresh_tokens(session_id)
  WHERE consumed_at IS NULL;
```

Validate token format before hashing and account/device/channel association
in the transaction. Login/register, rotation, revocation and cleanup go
through the existing single writer's bounded commands and replies. Refresh
uses one transaction: check nonrevoked/unexpired family and unconsumed hash,
consume exactly one row, insert successor hash, update last-used and commit.
No raw secret is written to SQLite, its journals, an outbox or a retry file.
Keep consumed hashes until family expiry to detect replay; cleanup expired
history/families through their indexes with at most 256 rows total per
maintenance turn, removing token rows before empty families. Do not trigger
an unbounded cascade of a long-lived family's history on the busy writer.
The foreign-key cascade is a safety net, not the normal cleanup batch size.

Use parameterized SQL and existing SQLite/WAL discipline. Check all rows
affected and treat uniqueness/concurrent-consumption errors explicitly.
Generate secrets with fallible OS entropy outside the store; fail closed
without zero/deterministic fallback or panic on a client-triggered path.
Return credentials only after commit and Access-index publication succeed;
revoke any committed orphaned issuance if publication fails. Do not block
the lobby, render thread or simulation on a database/vault/HTTP operation.

## 5. Rotation, races and lost responses

Rotation is one-time, not a second login or account takeover. Establish a
single refresh coordinator per family in the client. Serialize native vault
read/request/write/readback across processes with a user-local OS lock;
the lock file contains no credentials and is distinct from the build lock.
For web use Web Locks across tabs, holding the lock until the HTTP response
and cookie update complete. Use one origin-wide auth-cookie lock for login,
refresh, migration and logout, not separate locks that race across account
switches. An explicit browser account switch revokes the old browser family
and clears all tabs' Access/profile/resume state before attaching the new
account; other devices' families are not erased. Broadcast only nonsecret state/expiry/logout
notifications. Each tab obtains its own in-memory Access; other tabs' still
valid Access Tokens survive ordinary refresh rotation.

Do not implement a localStorage spin lock or broadcast bearer tokens. If a
supported browser lacks Web Locks, use a verified SharedWorker coordinator
or disable persistent multitab operation with a visible safe fallback.
Late callbacks are checked against the coordinator's family/generation and
logout epoch so they cannot restore a logged-out or superseded credential.

Use a **30-second bounded retry window** with a client-generated operation
ID and a short-lived in-memory response cache on the server. Persist only
the operation hash on the consumed row. Reserve/publish issuance under the
auth coordinator so cancellation/lost HTTP responses do not create a second
rotation. A retry with the same old hash, operation, family and channel may
receive the exact existing response within that window only while its
successor is still current. It must never mint another successor or return
an obsolete cached refresh over a newer generation.
Replay the same grant/issued timestamps but recompute cookie lifetime from
the remaining absolute expiry; do not replay a stale Max-Age header.

A different operation using that just-consumed token inside the window
gets 409, no tokens, no cookie overwrite and no immediate family revocation:
this can be a legitimate request already sent by another tab. Coordinate,
reload the current cookie/vault and retry with the current credential.
After the window, reuse revokes the family and all its Access/socket bindings.
Unknown random hashes never revoke an arbitrary supplied family/device ID.
The bounded retry exception is a documented security/availability tradeoff,
not evidence that the old token remains generally valid.

The cache holds secrets briefly in server memory, is bounded and purged;
it is never serialized or logged. If the server restarts after commit but
before delivery and the successor cannot be recovered from the cache,
return an explicit recovery-required result and ask for the AccountKey;
do not resurrect the old refresh, fabricate a successor or claim exactly-once
delivery across a crash. A known retry failure is not proof of theft. Test
both lost replies and concurrent requests, not just sequential happy paths.

## 6. WebSocket authority, expiry and uninterrupted matches

Use a coordinated protocol change, updating client/server/bots and relevant
fixtures together. Preserve the existing `Hello { version, resume }` shape
for guest handshakes and readable rejection of old clients. Append
`AuthenticatedHello { version, resume, access: [u8; 32] }` for accounts,
without reordering old enum variants. Add bounded `Reauthenticate { access }`
and `ClaimAccount { take_over: bool }` control messages, plus typed public
Auth state/conflict/required replies with expiry information. Allocate
actual variants/version after inspecting all consumers; no credential in
the socket URL, subprotocol, command line or logs. Redact secret Debug data.
Check version before any session side effect; a legacy v29 Hello receives
VersionMismatch. A new guest Hello cannot resume an account handle. Reject
legacy WS Register/Login/Logout mutations on the new version: use the
specified HTTP lifecycle so no alternate path can bypass expiry/revocation.
Reserve old wire tags rather than reusing their meanings. An auth-bootstrap
404/incompatible backend is a visible version/deployment error, not a loop
of automatic AccountKey login or silent ranked guest entry.

Verify Access and its family before **any** account resume, profile replay,
socket replacement, match attach or spectator/rematch side effect. For an
account-associated resume handle require the same account and auth family;
the handle alone cannot authenticate, steal a seat or disclose a profile.
Guest resume remains guest-only. A fresh explicit login can use the existing
account takeover flow, with authoritative account/seat from the connection,
never account IDs or seats supplied in a message.

Keep the existing single active account controller, while allowing multiple
durable device families. HTTP refresh and passive startup do not take over
another connection. If a controller already exists, show the same localized
Continue here conflict on all frontends. Only manual login/explicit
ClaimAccount authorizes takeover. A superseded client stops auto-claiming;
refresh/two tabs must not ping-pong the match. Preserve the existing ranked
high-water/profile ordering, Arcanas/Stronghold selections and result state.

Refresh in the background 60 seconds before Access expiry, and reauthenticate the
same socket without reconnecting or pausing combat. Check deadlines in the
connection/lobby control path, never SQL/hash/token validation per combat
tick. At natural Access expiry, deny new account operations, queue entries,
rematches and takeover until refreshed. An already admitted match has a
seat-scoped lease for that match's existing intentions/results only: its
socket can finish while renewal is unavailable. This does not extend the
Access Token's 15-minute lifetime or authorize another match/account action.

Reconnect still uses the existing 30-second window and requires fresh Access
plus the bound resume handle. Refresh valid credentials before reattaching;
never restart/extend the reconnect countdown because auth is pending. A
30-day absolute expiry requires manual login for a new authenticated
connection, including reconnect if no valid Access remains. Keep current
match lease and long-lived account expiry conceptually separate.

Logout/replay revocation/security invalidation is different from natural
expiry: invalidate the family in memory and notify affected sockets through
bounded control channels before acknowledging revocation. Those clients
cannot continue under a pinned match lease. Detach using existing connection
ownership checks and ordinary reconnect/outcome rules; do not add a new
forced-forfeit mechanic. Ordinary user logout while controlling a match
keeps the current idle-only UX and returns an explicit conflict; security
revocation is never deferred until the match ends. Mark the family revoking
in the auth service before the store command; commit DB revocation before
notifying bindings. Reject new grants while live invalidation is pending and return no
success acknowledgment before its barrier completes. A failed control
delivery is a visible fail-closed error, not permission for indefinite access.

After a server restart, Access indexes are intentionally empty; a valid
SQLite refresh can issue a new Access. Existing matches do not become
restart-resilient through auth. Keep release rollout separate: drain new
pairings, preserve established connections/matches and switch only after
they finish, or use an already implemented reviewed P66-compatible route.
Never restart a live production match to activate this phase.

## 7. Credential migration and user experience

The normal end state stores neither AccountKey nor tokens in preferences,
localStorage/sessionStorage, logs, ordinary files or URLs. The exception is
an explicitly requested manual AccountKey backup/copy, never an automatic
export. The 128-bit reconnect handle may remain per-tab sessionStorage only
after the server enforces its account/family binding and match-only purpose.

At first upgraded launch detect the existing `tbg_account_key`; do not
erase it up front or keep using it as the new automatic-login mechanism.
Explain the conversion and let the player save/confirm their manual backup.
Exchange it once over HTTPS, persist the refresh, verify vault write/readback
or perform browser `confirm` with the new cookie, then remove and verify
removal of the legacy key after backup acknowledgment. Clear secret UI/input
buffers when finished and invalidate old account-resume handles after the
new binding is active. Never copy legacy secrets into a migration journal.

If exchange, cookie/vault persistence, confirmation, backup acknowledgment
or legacy removal fails, record migration as incomplete. Preserve the
pre-existing key for recovery, clearly disclose its temporary legacy state,
and offer session-only/manual access; do not create another plaintext copy
or claim that device is fully migrated. A completed conversion must survive
reopening, upgrade/reinstall and a deliberate removal of the old preferences
key. Expired/revoked refresh prompts manual AccountKey; it does not create
a new account or erase purchases/history. P67's email recovery is separate.

Use one Auth coordinator/state machine: guest, restoring, authenticated,
refreshing, persistence unavailable, conflict, reauth required and logout
pending. The existing three-second auto-login guest fallback must not send
a ranked/room request while authentication is still pending. UI, network
and HTML bridge consume the same state; pending callbacks cannot reattach
after logout or a newer login. New text/errors have compiled English/pt-BR
keys. Linux and Windows keep exactly the same frontend appearance, including
the shared auth forms/status layouts. Use common game error text for the
same state; keep provider details in the relevant help/diagnostic boundary.

Offline logout clears Access/active local session immediately and records
a nonsecret logout-pending marker to prevent automatic restoration. If a
refresh remains available, keep it only in the vault/cookie for revocation
retry, not a normal auto-login. On connectivity return, revoke and clear it.
Disclose that remote revocation is pending; local deletion alone is not a
successful server logout. Failure/expiry paths must not erase unrelated
accounts, vault items, Wine prefixes or installed game data.

## 8. Module boundaries and dependencies

- `tbg-server/src/auth/{mod,tokens,http,rotation,access,origin}.rs`: bounded
  auth service, typed routes, secret handling, response-cache/expiry and
  policy. No standalone auth daemon or general identity framework.
- `tbg-server/src/store/auth_sessions.rs` plus existing schema/worker/store:
  transactions/history/revocation/cleanup, reusing the AccountKey verifier.
  `lib.rs`, `limits.rs`, `ws.rs`, `ws/client_msg.rs`, `lobby/accounts.rs`,
  `lobby/account_done.rs`, `lobby/sessions.rs`, `lobby.rs` and `chan.rs` own
  only routing, authority/control events and current game-session binding.
- `tbg-client/src/auth/{mod,session,transport,migration}.rs`: one state
  machine. `platform/credentials/{mod,linux,windows}.rs` owns native vaults;
  use the existing Bevy/native worker facilities for blocking vault calls.
  `config.rs`, `platform/storage.rs`, `net/`, account/start/actions UI and
  i18n consume it, never implement separate refresh loops.
- `assets/auth/session.js` and a narrow WASM bridge own cookie-backed fetch,
  Web Locks and nonsecret cross-tab events for both HTML and Bevy web paths.
  Existing `assets/lobby/account.js` is presentation-only; keep it that way.
- Update `deploy/native/README.txt` and release/user instructions to remove
  obsolete plaintext-key persistence claims; explain secure-store fallback,
  manual-key backup and real sign-out without exposing implementation secrets.
- Core contains bounded wire data only. No I/O/clocks/hash database or
  session server enters deterministic simulation. Balance rules remain intact.

Consulted current keyring 4.2.0 documentation differs from older examples:
for explicit provider control, permit `keyring-core` 1.0.1 plus Windows-only
`windows-native-keyring-store` 1.1.0 and Linux-only
`dbus-secret-service-keyring-store` 1.0.1. Select the Windows Local persistence
modifier explicitly. For Linux use the reviewed Rust crypto backend and
vendored D-Bus option if needed to avoid a new shared-library runtime hole;
record build cost/licenses and never bundle a fake unlocked user vault.
Mocks/sample stores are test-only and never a production fallback.

Revalidate/pin exact versions and API compatibility at implementation.
`serde` 1 derive for server DTOs, `url` 2 for strict origin parsing and native
`getrandom` 0.3 for client operation IDs are permitted when needed, each with
a Cargo.toml reason and lockfile review. Web uses browser cryptographic RNG.
Prefer existing libraries; no new async runtime in the client. Any alternative
dependency is a documented deviation, not an unrelated stack upgrade.

## 9. Ordered execution, rollout and focused verification

Complete P68-A before final P68 packages so Linux, Windows and web share
one coordinated auth protocol:

1. Record base, auth source manifest, wire/schema consumers and revised
   canon. Freeze DTOs, platform persistence and retry/expiry behavior.
2. Add forward-only tables and bounded store commands; test synthetic
   schema upgrades from v8/current schema, existing accounts/ranked history,
   atomic rotation/revocation and failure paths. No production DB mutation.
3. Implement HTTP/token/Origin/CSRF policies and live revocation. Add wire
   binding/reauth/explicit takeover, preserve match state and guest flows.
4. Implement native vaults, shared state machine, web cookie coordinator and
   safe legacy migration. Add compiled translations and exact desktop parity.
5. Consolidate the focused tests once; then build the final same-SHA Linux/
   Windows artifacts and browser client for P68 acceptance. Validate secure
   persistence on real Linux/Windows and functional fallback under Wine.
6. Deliver the PR and deployment/rollback runbook. Deployment itself remains
   outside this planning authorization and follows the owner's rules.

Rollout uses expand-only storage, tested backups including WAL-consistent
state, coordinated protocol/client release and existing drain support.
Do not restore an old database to roll back code or re-enable plaintext
credential persistence. Keep a compatible corrected binary or disable new
auth issuance while preserving data. A new public version/installer cannot
claim old-server compatibility after this wire change. Real production
activation waits for the owner; synthetic test servers are sufficient for
implementation validation. No assumed P66 deployment infrastructure.

Add `tbg-server/tests/auth_sessions.rs` for lifecycle/store/HTTP scenarios,
`auth_websocket.rs` for binding/expiry/reconnect/takeover and
`tbg-client/src/auth/tests.rs` for state/migration/vault results. Use injected
clocks, deterministic synthetic secrets and local endpoints; never wait a
real 30 days. New protocol fixtures follow append-only/canon rules in the
same contract-change commit. Existing account/registration/challenge tests
retain their meaningful scenarios, adapted only with the documented contract.

Future focused consolidation commands, under the shared host lock with two
Cargo jobs and serial test execution:

```text
cargo test --locked -p tbg-server --test auth_sessions -- --test-threads=1
cargo test --locked -p tbg-server --test auth_websocket -- --test-threads=1
cargo test --locked -p tbg-server --test accounts -- --test-threads=1
cargo test --locked -p tbg-client --bin tbg-client auth:: -- --test-threads=1
```

Also select the exact affected protocol/registration/challenge fixture tests
once, and one browser, Linux-vault and Windows-vault journey. These tests
do not exist yet; this is a command queue, not passing evidence. Apply the
existing minimal front/consolidation policy. No workspace/full suite or
balance gate is authorized by this auth extension.

## 10. Auth acceptance additions to P68

| ID | Required evidence |
|---|---|
| A01 | Initial/legacy login and registration preserve identity, stats and manual key; no new plaintext key persistence |
| A02 | Access valid before 900s, rejected at/after deadline for new account operations; next token cannot exceed the family's absolute expiry |
| A03 | Refresh expires at original login plus 30 days; rotation/cookie Max-Age never reset it; family-cap refusal cannot prevent manual-key logout-all/recovery |
| A04 | Transaction creates one successor only; consumed/unknown/expired/revoked/malformed tokens cannot mint grants; family replay revocation works |
| A05 | Concurrent tabs/processes, duplicate/lost/out-of-order responses, crash after commit and coordinator cancellation do not silently overwrite current credentials or cause takeover loops |
| A06 | Logout, logout-all, offline pending state and revocation barriers invalidate Access/resume authority; failure cannot report successful server logout |
| A07 | Linux locked/missing Secret Service, Windows vault denial/local persistence and Wine functional fallback; no mock/plaintext fallback |
| A08 | Browser cookie flags, blocked/session cookies, exact origin/CSRF/CORS checks, native-route isolation and no credentials in JS storage/URLs/logs/caches |
| A09 | Account resume without Access or with wrong account/family fails before attach; guest resume cannot assume an account; explicit takeover alone transfers the seat |
| A10 | Mid-match Access expiry/refresh outage preserves admitted play; refresh updates the same socket; reconnect countdown remains 30s; new matches require valid auth |
| A11 | Server restart loses Access but retains valid refresh; SQLite upgrade/re-run preserves accounts/ranked rows and enforces indexes/history cleanup |
| A12 | Migration removes legacy key only after secure persistence/confirmation and backup acknowledgment; failures preserve recovery without new secret copies |
| A13 | HTTP/body/queue/rate/cache bounds and entropy/store failures fail closed; no SQL or auth crypto enters simulation ticks |
| A14 | Shared English/pt-BR auth UX and exact Linux/Windows frontend visuals; artifact protocol/source/server contracts match |

Required evidence is separate from W01–W18; P68 completion includes both
sets. Deferred auth security/platform acceptance prevents a completed P68
claim. Owner look approval does not waive authentication tests or real
Windows vault evidence. No claim of a measured player capacity follows from
the configured limits; measure actual auth rates/store latency before scaling.

## Primary references checked on 2026-10-31

- [RFC 9700 §4.14](https://www.rfc-editor.org/rfc/rfc9700.html#section-4.14): use rotation/replay protection as guidance for these custom public clients, without installing an OAuth server.
- [OWASP session management](https://cheatsheetseries.owasp.org/cheatsheets/Session_Management_Cheat_Sheet.html), [WebSocket security](https://cheatsheetseries.owasp.org/cheatsheets/WebSocket_Security_Cheat_Sheet.html) and [CSRF prevention](https://cheatsheetseries.owasp.org/cheatsheets/Cross-Site_Request_Forgery_Prevention_Cheat_Sheet.html): transport/storage boundaries, origin validation and independent CSRF controls.
- [MDN Set-Cookie](https://developer.mozilla.org/en-US/docs/Web/HTTP/Reference/Headers/Set-Cookie) and [Web Locks](https://developer.mozilla.org/en-US/docs/Web/API/Web_Locks_API): cookie flags/lifetime behavior and per-origin tab coordination.
- [Secret Service specification](https://specifications.freedesktop.org/secret-service/latest/) and [Microsoft CredWriteW](https://learn.microsoft.com/en-us/windows/win32/api/wincred/nf-wincred-credwritew): real user-session OS stores, not an application-encrypted preferences file.
- [keyring current guidance](https://docs.rs/keyring/4.2.0/keyring/), [keyring-core](https://docs.rs/keyring-core/1.0.1/keyring_core/), [Windows provider](https://docs.rs/windows-native-keyring-store/1.1.0/windows_native_keyring_store/) and [Linux provider](https://docs.rs/dbus-secret-service-keyring-store/1.0.1/dbus_secret_service_keyring_store/): explicit provider selection; Windows persistence defaults and concurrent-store caveats; Linux crypto/runtime requirements.

## Planning validation report

Documentation only. `git diff --check` and standard-library checks validate
the changed links, source manifest (34 hashes/line counts), A01–A14 coverage
and BR-AS01–AS12 registry. The proposed SQL was parsed in a disposable
in-memory SQLite database with a synthetic account: current-token uniqueness,
consumed history, NULL/short-hash/foreign-key rejection and cascade safety
passed. This checks the schema shape, not transactional Rust behavior.
No Cargo/browser build, auth runtime test, installed vault access, credential
creation, Wine installation, production database operation or deploy ran.
