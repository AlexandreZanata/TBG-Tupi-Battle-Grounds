# P71 — Match continuity under token auth, cross-slot revocation, migration and launch readiness

**Status:** planned and executing (2026-11-01) on `phase/p71-match-continuity`,
stacked on the consolidated candidate (`codex/consolidate-linux-main`,
[PR #34](https://github.com/AlexandreZanata/TBG-Tupi-Battle-Grounds/pull/34)).
This file is the executable plan and the hand-off: any agent continuing P71
starts here, then reads only what each work item names.

## 0. Owner decisions this phase executes (2026-11-01)

- **Keep D-99.** One auth/resume coordinator owns the vault (P66 resume data
  included), schema 10 is shape-verified, and the wire stays at protocol 30
  (contract `w30-r2-3d56156049e626c5` until rules change).
- **Keep the P68 token model** (15-minute Access in memory, 30-day rotating
  Refresh families). It must never be the reason a player loses a
  competitive match: a network drop, a token expiry or a slow vault while
  the player can still come back must end in the same seat. Every original
  feature stays: reconnect, resume after restart, Augments, spectators,
  rematch, ranked settlement.
- **Production data is disposable at this stage.** The current server's
  players may be discarded if something goes wrong. Migration still gets a
  backup and a written procedure, but no data-preservation work beyond that.
- **Windows** is tested on a real Windows 11 VM in Docker with its disk on
  `/data`.
- Every open gate of the PR #34 review (items 7–13) is either resolved here or
  routed to its owner with a recommendation (section 3).
- Working rules for this thread: take the recommended action without asking;
  desktop browsers and desktop native only; English in the repository; no
  force-push or history rewrite; never merge PR #34 or close the superseded
  PRs (owner); no VPS hosts, inventory or credentials in git.

## 1. Findings that drive the plan (code at `907c183`)

### 1.1 How a drop becomes a lost ranked match today

Reconnect window: `reconnect_secs: 30` (`tbg-server/src/config.rs:66`, env
`TBG_RECONNECT_SECS`), timer `Match.detached_at` (`match_task.rs:49`), forfeit
on the 50 ms tick (`match_task.rs:192-201`; both expire on one tick → Seat A
loses). Nothing pauses: combat, countdown and the Augment selection clock keep
running (`sim.rs:124-133`); an absent player's Augment round becomes
`Event::Forgone` after 20 s. The server notices a dead socket only on close,
a failed send, queue overflow or `IDLE = 15 s` without inbound frames
(`ws.rs:30,372`). The opponent gets no presence signal: `ServerMsg` has no
presence/pause variant and `MatchView` no presence field; variant order is
frozen by contract tests, so a new message means a protocol bump.

Paths that can cost a ranked match **even when the network returns within
30 s** (each is a P71 defect, W1):

| # | Path | Where |
|---|---|---|
| 1 | Fast account reconnect while the server still marks the old socket attached (any real drop: detection takes up to 15 s) → resume dropped, new session + `AuthState{Conflict}`, new token overwrites the handle, seat forfeits | `lobby/auth.rs:121-129,186-203`; client `net/socket.rs:128-135` |
| 2 | Match send queue (`CONN_QUEUE = 32`) full → seat detached in the match only; the TCP link recovers, nothing re-attaches, client shows a frozen match and forfeits 30 s later | `match_task/views.rs:78-84` |
| 3 | Access expired during the drop: handshake refuses (`AuthRequired{Invalid}`), one forced renewal; a slow/failed renewal → `ReauthRequired`, no new socket | `auth/access.rs:7-16`, `ws.rs:226-253`; client `auth/mod.rs:93-103,186-197` |
| 4 | Lost refresh response: the client's next attempt comes ≥ 30 s later (60 s retry wait), the server treats it as reuse (`RETRY_SECONDS = 30`) and revokes the whole family, detaching the seat | `auth/rotation.rs:257-306`; client `auth/mod.rs:298-312,645` |
| 5 | Native vault write failed → next refresh reads the stale token from the vault → reuse → family revoked | client `auth/native.rs:219-249` |
| 6 | Native session-only/no vault: a failed refresh leaves a pending operation; the next attempt must read the vault first, fails locally, Access lapses | `auth/native.rs:258-273` |
| 7 | Browser temporary mode: Access-only cannot renew (~14 min); a failed `confirm` turns the next renewal into a logout attempt → `LogoutPending` | `assets/auth/session.js:61-65,87,104-108` |
| 8 | P66 slot mode: a store failure while re-claiming `account_owner` → `SessionTakenOver`, socket left inactive, intents refused | `lobby/ownership.rs:36-40,79-95` |
| 9 | Away > 20 s during an Augment round loses that pick (rule, not a bug) | BR-AG13 |

Between 31 and ~45 s the seat may still resume (detection lag); beyond that
the forfeit is certain and the ranked loss settles. An account returning
after the lobby's 30 s `detached_at` check gets `AuthRequired{Invalid}` and
must sign in again (`lobby/auth.rs:114-119`). BR-AS07 says logout invalidates
the lease; the code refuses an ordinary logout with 409 `in_match` instead
(`auth/access.rs:18-48`) — record as a canon/code mismatch to settle in W1.

Existing tests to keep green and extend: `tbg-server/tests/online.rs`
(`reconnect_resumes_match`, `forfeit_after_window`,
`br_ag_online_paused_round_hidden_pick_reconnect_and_deadline`),
`ranked.rs::br_r04_ranked_forfeit_is_a_loss`, `auth_websocket.rs` (`a09_*`,
`a10_*`), `auth_http.rs` (`a01_a02_*`, `a03_*`),
`accounts.rs::br_p03_second_login_takes_over_live_match`,
`match_task/tests.rs::challenge_initial_detached_forfeits`, client
`auth/tests.rs::br_as07_*`, `platform/resume.rs::br_as06_*`,
`tests/auth-session.test.cjs`. The 30 s value is pinned by
`tbg-server/tests/contract.rs:20-23` (`production_defaults`). Missing today:
an account in-match drop-and-resume test, Conflict during a match,
refresh reuse during a match, reconnect-then-renewal ordering.

### 1.2 Release slots and revocation

Slots are separate processes (`tbg-server@a|b`, 127.0.0.1:8080/8082) on one
SQLite file (WAL). The Access index (`auth/mod.rs:173`), revocation flags
(`auth/mod.rs:139-142`) and the 30 s rotation cache are per process; refresh
families and `revoked_at` live in the database (`store/auth_sessions.rs`).
Every revocation goes through `revoke_scope` (`auth/access.rs:53-118`): flip
local flags → write DB → purge local index → `AuthRevoke` to the local lobby.
Slot B learns of a revocation only when it next reads that family from the
database; its index, flags and live sockets stay valid up to 900 s. There is
no index on `revoked_at`. Periodic hooks available: `auth::Service::start`
1 s timer (`auth/mod.rs:197-214`), `release::run` 1 s tick (slot mode,
`release.rs:20,121-152`), the store worker's `PRAGMA data_version` check
(`store/worker.rs:94-108`).

### 1.3 The P66 two-generation `connection closed` failure

Evidence (owner ops workspace, outside git; protocol 29): isolated pair and
public run both ended with 2 of 4 bots completing (opponent `Forfeit`) and
2 × `connection closed` per group, reconnects 3 attempted / 2 succeeded.
`tbg-bots --chaos-pct 50` drops after 20–60 s and rejoins after **5 s** —
inside the window — so a resume failed or a live socket died. The protocol-29
bot reader mapped any read error, ignored Close and EOF to `connection
closed`; transport-class diagnostics landed later (`botclient.rs:251-268`).
No server logs were kept. Path #1 above (stale attached socket → Conflict) and
path #8 (ownership re-claim) are the leading suspects; reproduce before
concluding (W4).

### 1.4 Migration and deploy facts

`migrate_once` (`store/schema.rs:341-402`) applies v1–v8, then for any version
below 10 verifies the shape of the P66 release set and the P68 auth set,
creates a missing set, refuses a partial one, and records 9 and 10. A failed
open is **not fatal**: the server runs without a store and `/health` still
answers `ok` (`lib.rs:83-91,150`), so a failed migration would pass the deploy
health check (W5 fixes this). `deploy/deploy.sh` is single-slot (Caddy +
systemd); production instead runs the owner's k3s + nginx setup (outside
git): slot `a` runs the protocol-29 P66 image with `TBG_SLOT=a`, slot `b` is
retired, and the live database is schema 9 (P66 variant) — small and, per the
owner, disposable. A v10 binary takes the P66 v9→10 path there. As a new
generation in slot mode it registers warming and admits nothing until
`tbg-release-admin promote` and `ranked`; a w29→w30 contract change cannot be
canaried (`ContractChange`), so production moves by a scheduled switch.
Production nginx has no `X-TBG-Slot` auth routing (fine while one slot runs).

### 1.5 Already done in this phase

- Linux release workflow: linuxdeploy pinned to tag `1-alpha-20251107-1`
  (`907c183`); the `Native Linux release` run on PR #34 now packages and
  uploads both formats.
- Real Caddy 2.10.2 validates `deploy/Caddyfile.template` ("Valid
  configuration").
- HTTPS through that production Caddyfile (internal CA, test-only global
  ports, loopback rc.1 server, Chrome 156): lobby and Practice load, the game
  socket is `wss://…/ws?contract=w30-…` through the proxy,
  `__Host-tbg_refresh` is Secure + HttpOnly + SameSite=Strict + Path=/ and
  host-only, same-family restore works, no credential in `localStorage`,
  logout revokes the family and clears the cookie, `X-TBG-Slot: b` reaches the
  slot-b upstream (502 with no slot b) while `a` behaves like the shared
  route, `/api/release` is 404 and HTTP redirects to HTTPS. Not covered: the
  real domain behind Cloudflare and production nginx.
- PR #34 description no longer claims Draft.

## 2. Work items

Validation follows `docs/VALIDATION.md` (shared nonblocking lock,
`CARGO_BUILD_JOBS=2`, targeted tests once per change set, no full suite
without an explicit owner request). Each item lists the tests to write and
the exact checks to queue.

### W0 — PR #34 CI green (on `codex/consolidate-linux-main`)

`ci` failed on `907c183` only in `spectate::a_guest_watches_a_casual_bot_match_to_the_end`
(`tests/spectate.rs:220`): the bots' 120 s budget ran out before a natural
end. BR-M07 removed the match length and each match draws a random arena, so
natural bot matches have unbounded duration; the same pattern sits in
`online.rs:217,333`, `spectate.rs:379`, `lab_equivalence.rs:37`. Fix without
weakening the contract: give `BotOpts` an optional `surrender_after` (send
`Surrender`, BR-M06, that long after `MatchStart`) and use it where a test
needs an end but not a particular winner; keep natural ends where the outcome
itself is asserted. Queue: `cargo test -p tbg-server --test spectate --test
online -- --test-threads=1`.

### W1 — Reconnect reliability under token auth (no wire change)

Goal: a player whose network or tokens hiccup and who comes back inside the
window always regains the same seat; security properties of D-97 stay
(revocation still ends a seat; another family or a guest never takes an
account seat).

1. **Same-session resume displaces a stale socket** (#1). In
   `lobby/auth.rs:121-129`, when the resume names this session's own token and
   the binding's family and account match, detach the old socket (close it
   with a takeover reason) and attach the new one immediately instead of
   issuing a new session with `Conflict`. Keep the refusal for another family,
   another account or a guest handle (`a09_*`). Mirror for guests is already a
   takeover.
2. **Queue overflow forces a reconnect** (#2). When `Match::send` detaches a
   seat because the per-connection queue is full or closed, also close that
   connection (lobby `Detach` + socket close) so the client's reconnect loop
   runs; never leave a live socket without views.
3. **Seat lease survives Access expiry on reconnect** (#3, BR-AS07). Allow
   `AuthenticatedHello` whose Access is expired but whose family is unrevoked
   (checked in the database, not the in-memory index) to resume a session
   that holds an admitted seat; the socket then gets `AuthRequired{Expired}`
   and renews in the background like a live socket does today. New matches
   and account operations still require valid Access.
4. **Refresh retries stay idempotent** (#4–#6). Client: after a failed or
   unanswered refresh, retry the same operation within the server's retry
   window (never wait 60 s holding a consumed token); keep the newest refresh
   token in memory and never fall back to an older vault copy once rotation
   started. Server: keep reuse detection, but check whether `RETRY_SECONDS`
   (30) should exceed the client's worst-case retry spacing; document the
   final pair. Tests: lost reply during a match → same seat, no revocation;
   true reuse (stolen token) → family revoked and seat ended.
5. **Lobby reattach window** equals the match window (`lobby/auth.rs:114-119`)
   so an account returning late inside the window is not forced to sign in.
6. **Ownership re-claim errors retry** (#8) instead of `SessionTakenOver`.
7. **Browser temporary mode** (#7): keep the match playable until it ends; a
   failed `confirm` must not convert the next renewal into a logout while a
   seat is admitted.
8. **Canon/code mismatch on logout**: decide in DECISIONS (recommended: keep
   the 409 `in_match` refusal for ordinary logout and say so in BR-AS07;
   logout-all and security revocation keep ending the lease).

Tests (new file `tbg-server/tests/match_continuity.rs`, rule IDs in names):
account drop-and-resume inside the window, resume while the old socket is
half-open, resume after Access expiry, lost refresh reply mid-match, queue
overflow → reconnect, revocation still ends the seat. Client unit tests for
the retry schedule and the reconnect-then-renewal order.

### W2 — Connection pause (Proposed rule change, no wire change)

The owner's goal ("never lose a competitive match because the internet
dropped") is not met by a running match with a 30 s window. Recommended rule
(new values marked **Proposed** in GAME-DESIGN in the same commit):

- While a seat is detached during countdown, Running or an Augment round, the
  server freezes the match for both players: combat tick, countdown and the
  Augment selection clock stop; intentions are refused with the existing
  `Rejected`. Each seat has a cumulative pause budget of **120 s per match**
  (Proposed). When the seat returns the match resumes where it stopped
  (BR-AG13's "the time left" stays exact).
- When the budget is spent the match resumes running and the existing 30 s
  window applies; staying away past it forfeits (BR-M03/BR-R04 unchanged).
- If both seats are detached and no one returns, the match ends with no
  result and no rating change (Proposed).
- The client infers the pause from views that keep arriving with a frozen
  `tick` outside an Augment round and shows "Aguardando a conexão do
  oponente…"/"Waiting for the opponent's connection…" with elapsed time;
  spectators the same from `PublicView`. No protocol change, so protocol 30
  and the D-99 contract stay; replays are unaffected (no ticks advance).
- Abuse bound: the budget is per player per match; a seat that exhausts it
  plays under the old rule.

If the owner later wants the remaining budget shown, that needs a presence
message → protocol 31 (a separate, explicit owner decision).

### W3 — Revocation across release slots

Add a revocation feed the other slot applies within ~1 s: an index on
`auth_sessions(revoked_at)` (additive; schema 11 if the table definition
changes, otherwise index-only in the v10 `else` branch) and a poll in the
`auth::Service` 1 s timer that reads families revoked since the last seen
timestamp/rowid and runs the in-memory half of `revoke_scope` (flags, index,
caches, `AuthRevoke` to the local lobby). Test with two `run_with` servers on
one database (pattern in `tests/release_slots.rs`): revoke on A → B's socket
for that family ends within 2 s, B's HTTP refresh refuses, other families
untouched. Two-slot mode stays disabled in production until this passes and
W4 is explained.

### W4 — Reproduce and explain the P66 `connection closed`

Locally: two slots on one database (or one server for a first pass) and
`tbg-bots --ranked --matches 4 --chaos-pct 100 --timeout-s 900`, server
`RUST_LOG=info` kept, bots with the transport diagnostics. Classify every
failure (Conflict on resume, ownership, queue overflow, transport). Re-run
after W1; acceptance: every chaos bot completes and no outcome is a forfeit
caused by a drop shorter than the window. Then the two-slot promote/rollback
sequence of P66 U13/U15 with bots pinned to each slot.

### W5 — Migration and coordinated deploy readiness

1. `/health` reports failure (503 `store`) when a configured database failed
   to open or migrate; in-memory mode (`db: None`) stays `ok`.
2. Migration procedure for the production shape (schema 9 P66 variant →
   10): backup with `sqlite3 .backup` + `integrity_check`, stop slot a, start
   the new generation, `promote` + `ranked` (contract switch), verify
   `/api/version` (protocol 30), auth round trip, one ranked bot match;
   rollback = stop the new generation, restore the backup file, start the
   previous image. Write it in `deploy/README.md` generically (no hosts); the
   owner's k3s/nginx scripts live outside git.
3. Rehearse the migration on a copy of the production-shaped database
   (synthetic P66 v9 fixture already in `tests/schema_upgrade.rs`; a real copy
   only through the owner's ops workspace).
4. Build the compatible pair from one frozen commit: server image + Trunk
   bundle + native Linux `0.1.1-rc.2` (AppImage/tar.gz) and the unofficial
   Windows ZIP; publish rc.2 as a prerelease only after W0–W2 land.
5. The production switch itself runs through the owner's ops workspace; the
   owner said the data is disposable, so it may proceed once rc.2 passes, with
   the backup taken first.

### W6 — Windows 11 VM acceptance (Docker on `/data`)

`dockurr/windows` (digest recorded in the VM report) with KVM, 6 GB RAM, 4
cores, storage/shared/oem under `/data/tbg-windows-vm/`, ports on loopback
only. A command channel (logon scheduled task polling a shared jobs folder)
runs PowerShell in the interactive session. Suite: identity flags,
missing-assets refusal, Credential Manager vault probe and native auth
journey against a host server on the Docker bridge, lobby pt-BR/en, seven
panels, five arena trainings, offline Practice, audio without mute, one
scripted click, and a numeric + visual comparison with the Linux AppImage
captures. Results: `/data/tbg-windows-vm/REPORT.md` (outside git); summarize
in the P71 execution report. Re-run on rc.2. Does not prove real GPU drivers,
a signed installer or physical audio/input.

### W7 — HTTPS on the real domain

Locally done (1.5). Remaining after the production switch: the browser auth
smoke against the public domain (Cloudflare + production nginx): cookie
flags, restore, logout, WSS, and the ranked bot match.

### W8 — Records, decisions and routed gates

- DECISIONS: D-100 (P71 continuity decisions, revocation feed, health
  semantics, logout canon); GAME-DESIGN Proposed rows for W2 and the BR-AS07
  wording; ROADMAP row for P71; ARCHITECTURE §26 update.
- Execution report `docs/phases/P71-execution.md` with commands, results and
  deferred checks.

## 3. Review items 7–13: disposition

| Item | Disposition |
|---|---|
| 7 Balance (18 of 118 audit checks failing) | Kept as known debt of an experimental build; a dedicated balance phase with `tbg-balance` evidence (D-33) — not executed in P71 |
| 8 P66 continuity + O02–O05 | W4 investigates; two-slot continuity stays off until it passes. Recommended defaults recorded as Proposed: O02 offline k-of-n root held by the owner, test keys never published; O03 24 h lead time, running matches never cut; O04 measure first (no limits invented); O05 automatic download **off**, explicit player action |
| 9 Cross-slot revocation | W3; two-slot mode disabled until it passes |
| 10 Windows / visual parity / public HTTPS | W6 (VM evidence; Windows stays experimental), W7; visual parity stays a separate deliverable |
| 11 P67 | Accepted as an offline foundation; nothing activated |
| 12 Look and fun | Owner plays rc.2 and records concrete issues |
| 13 Production protocol 29 vs 30 | W5 coordinated switch with backup and rollback |

## 4. Order and acceptance

W0 → W1 → W2 → W5.1 → W3 → W4 → W5.2–5.4 → rc.2 → W6 re-run → W5.5 → W7 →
W8. P71 is accepted when: PR #34 and the P71 PR have green `check`; the W1
and W2 tests pass; chaos bots (W4) complete with no drop-caused forfeit; the
revocation feed test passes; `/health` fails closed; rc.2 is published with
hashes and download-back; the Windows report exists; the records are
updated. Owner gates that remain after P71: balance, O02 signing custody,
Windows official release, look/fun.

## 5. Status log

- 2026-11-01: plan written; findings 1.1–1.4 mapped; 1.5 done; W6 VM
  installed and suite running; W0–W5 not started.
