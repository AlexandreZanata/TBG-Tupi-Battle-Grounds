# P39 — Quick Challenge by link and rematch

**Depends on:** the current guest-session, reconnect, and result-screen code
(P4, P9, P14) and protocol v8 with active Arcanas (D-40) at the integration base. P14 owner sign-off is
not a gate for this lobby feature. Independent of the P16-F strategic
expansion and the visual suite. Do not run it concurrently with P16-F4,
P16-F7, P16-A, or P16-B: they edit the same lobby/menu files.
**Branch:** `phase/p39-quick-challenge-pr`.
**Status:** implemented and validated; PR review and owner device/playtest checks pending.
**Difficulty:** medium. One lobby feature, one protocol bump, three screens.

**Integration follow-up (PR #2):** main already received the original P39
implementation with P16-D through PR #1 (`077abe7`). PR #2 now contains
review fixes and browser evidence; protocol v9 and all existing wire
contracts remain unchanged from main.

## Goal

A guest clicks **Challenge a friend**, shares one link, and a second guest
joins automatically. After the match both players may request a rematch; it
starts only when both accept. Private rooms, casual/ranked queues, reconnect,
and the existing match simulation stay unchanged.

The experience is: open the link and play. No account, no code entry, no
manual room configuration.

## Read first (only these)

- `AGENTS.md`; `docs/ROADMAP.md` P39 row and lane; this file.
- `docs/GAME-DESIGN.md` glossary, BR-M01–M03, BR-P04, BR-R01–R09.
- `docs/ARCHITECTURE.md` §3, §5 (transport, sessions, matchmaking, limits),
  §8 (security).
- `docs/DECISIONS.md` D-06, D-09, D-39.
- Core: `crates/tbg-core/src/protocol.rs`,
  `crates/tbg-core/tests/contract_protocol.rs`,
  `crates/tbg-core/tests/contract_protocol_server.rs`.
- Server: `crates/tbg-server/src/lobby.rs`,
  `lobby/{matchmaking,sessions,ranked}.rs`, `chan.rs`, `ws.rs`,
  `ws/client_msg.rs`, `limits.rs`, `config.rs`, `match_task.rs`,
  `match_task/{command,views}.rs`, `tests/online.rs`, `tests/contract.rs`.
- Client: `crates/tbg-client/src/{config,link,main}.rs`,
  `net/{mod,socket}.rs`, `sync.rs`, `sync/messages.rs`,
  `ui/{mod,start,actions,menu,queue,result,desktop_actions}.rs`,
  `index.html`, `assets/lobby/lobby.css` only if the extra mode button
  needs layout adjustment.
- Never read or edit `archive/`.

## Audit summary (verify at execution time)

Reuse, do not reimplement:

| Need | Existing mechanism |
|------|--------------------|
| Guest without account | `Lobby::hello` creates a session with a 128-bit token; client stores it per tab in `sessionStorage` |
| Pair two sessions | `Lobby::pair` spawns the one `match_task`, sends `EnterMatch`, seats come from the connection |
| Private room | `CreateRoom`/`JoinRoom`/`RoomCreated`/`RoomNotFound`, 5-char code, 10-min TTL, `?room=CODE` |
| Reconnect | `Hello { resume }`, `SessionState::InMatch` → `MatchCmd::Attach`, 30-s window |
| Match lifecycle | `MatchStart` → countdown → `MatchEnd` → result screen → `PlayAgain` |
| Authority | Server owns pairing, seed, simulation, outcome, rating |
| Limits | WebSocket token bucket, per-IP connection cap, `Limits` sliding logs |

Real gaps:

- No challenge/invite record, token, message, or `?challenge=` URL.
- No challenge waiting screen, share action, or automatic guest entry.
- No rematch record, request/accept messages, or result-screen rematch UI.
- `MatchMode` has no value for a direct invitation match.
- `ToMenu`/`Cancel` drops the socket before the `Cancel` frame is flushed;
  a rematch decline needs the socket kept as a probe until the frame leaves.
- A disconnect can race the pairing; the match task must start a detached
  seat inside the reconnect window instead of running a ghost match forever.
- No invite rate limit beyond the general WebSocket bucket.

## Non-goals

Spectators, broadcast, chat, friends, accounts/login changes, ranked/Elo
changes, a new server/process/database/Redis/WebRTC, persistent invitations,
a second matchmaking queue or match task, UI restyle, or spectator fields in
`Match`. Do not add a trait or framework for future spectators.

Conceptual separation preserved: a `Session` is a participant, a `Room` or
`Challenge` is an invitation, and a `Match` is the two-seat simulation. A
future spectator would attach read-only to a `Match`, never become a third
seat or a second pairing. No spectator field is added now.

## Contracts (freeze before coding)

### Wire: `PROTOCOL_VERSION = 9`

Append only; existing variants keep their indices. Old clients are rejected
by the handshake.

```rust
pub type InviteToken = [u8; 16];

pub enum MatchMode { Casual, Ranked, Challenge } // Challenge = index 2

// ClientMsg, appended after SetArcanas (index 13)
CreateChallenge,
JoinChallenge { token: InviteToken },
RequestRematch,

// ServerMsg, appended after ArcanasRejected (index 19)
ChallengeWaiting { token: InviteToken },
ChallengeUnavailable,
RematchOpen { opponent: String },
RematchState { you_requested: bool, opponent_requested: bool },
RematchClosed { declined: bool },
```

Wire indices: `CreateChallenge` 14, `JoinChallenge` 15, `RequestRematch` 16;
`ChallengeWaiting` 20, `ChallengeUnavailable` 21, `RematchOpen` 22,
`RematchState` 23, `RematchClosed` 24. `MatchStart` keeps its fields, including both frozen Arcana presets;
`mode: MatchMode::Challenge` identifies a challenge match to the client.
`Challenge` is a casual match on the wire, never ranked.

`ChallengeUnavailable` is deliberately identical for invalid, expired,
already-used, self-join, and owner-gone invites: no oracle about whether a
token existed. `RematchClosed { declined }` uses `true` only when a player
explicitly declined/left; expiry and session loss use `false`.

### Server state (lobby memory only)

```rust
// config.rs; defaults in parentheses, no environment override required
pub challenge_ttl_secs: u64,   // 600
pub rematch_ttl_secs: u64,     // 120
pub invite_per_ip_min: u64,    // 30

// lobby.rs
enum SessionState {
    Idle,
    Queued,
    InRoom,
    InChallenge { token: InviteToken },
    InMatch { tx: mpsc::Sender<MatchCmd>, seat: Seat },
}

struct Lobby {
    // existing fields ...
    challenges: Vec<(InviteToken, Token, Instant)>, // (invite, owner, created)
    rematches: Vec<RematchTicket>,
}

struct RematchTicket {
    players: [Token; 2],
    requested: [bool; 2],
    /// `None` while the challenge match is live; `Some(deadline)` after
    /// `MatchEnded`. An open ticket is a ticket with `expires.is_some()`.
    expires: Option<Instant>,
}
```

`InviteToken` is generated with `random_bytes::<16>()` (OS RNG), is not
derived from the session token, and is independent of it. Do not reuse
`RoomCode`, `Token`, or the 5-char alphabet for invites. One active
challenge per owner; creating another replaces the previous one.
"Two different players" means two different session tokens, never two
different IPs or proven humans: two tabs of one browser are two sessions and
must not interfere with each other; two players behind one NAT are allowed.

### Lifecycle

**Create challenge**

1. `leave_waiting(token)` (queue, owned room, owned challenge, and any open
   rematch ticket).
2. Generate a unique `InviteToken`; store `(invite, owner, now)`.
3. Set the owner's state to `InChallenge { token }`; send
   `ChallengeWaiting { token }`.

**Join challenge**

1. Check `limits.check_invite(ip, config.invite_per_ip_min, now)`; on
   failure send `Error { code: RateLimited }`.
2. Find an invite with the exact token whose owner is a different session.
   Expiry is enforced lazily: `now - created >= challenge_ttl_secs` removes
   it, resets the owner exactly as cleanup does, and answers
   `ChallengeUnavailable` to the guest.
3. Verify the owner's session exists, is still `InChallenge { token }`, and
   has a live connection (`conn.is_some()`). A reloading host therefore
   does not start a match; the guest gets `ChallengeUnavailable` and may
   retry once the host is back. Do not consume the invite on this failure.
4. Remove the invite atomically; then `leave_waiting(guest)` so the guest's
   own pending state cannot leak.
5. `pair([owner, guest], MatchMode::Challenge, rematchable = true)`.
   The guest is added only after the invite is removed; a second guest sees
   `ChallengeUnavailable`. No simulation runs before this point.

**Cancel / leave to menu**

`Cancel` calls `leave_waiting(token)`. It removes the caller's queue place,
owned room, owned challenge, and any open rematch ticket containing the
caller. No reply is sent.

**Disconnect**

`Disconnected` calls only `leave_queue_room(token)`: queue and owned room
are removed exactly as today; an owned challenge and a rematch ticket are
kept. This is what lets the host reload while waiting and lets either player
reload on the result screen. The 30-s reconnect window and the invite/rematch
TTL bound the orphan state. Do not remove rematch tickets on disconnect.

**Resume (`Hello { resume }`)**

After the existing attach/account handling:

- `InChallenge { token }` and the invite still exists → send
  `ChallengeWaiting { token }` to the new connection.
- `Idle` and an open rematch ticket contains the token → send
  `RematchOpen { opponent }`, then `RematchState` if either side requested;
  call `try_start_rematch` in case the returning player completes a pair
  that was waiting for both connections.
- `InMatch` → existing `MatchCmd::Attach` path, unchanged.

**Match end**

After the existing `Idle`/`LeaveMatch` handling, find the
`RematchTicket` for the same two tokens with `expires == None`. Set
`expires = Some(now + rematch_ttl_secs)` and send
`RematchOpen { opponent }` to each connected player. A disconnected player
receives it on resume. Non-challenge matches have no ticket and no rematch
messages.

**Rematch request**

1. Find an open ticket containing the caller. If none, send
   `RematchClosed { declined: false }` to the caller and stop.
2. Verify both sessions exist and are `Idle`. If not, remove the ticket and
   send `RematchClosed { declined: true }` to the caller (the other player is
   already busy or gone).
3. Mark the caller's `requested` flag (idempotent; a duplicate request only
   resends `RematchState`).
4. Send `RematchState` to both players with each side's perspective.
5. Call `try_start_rematch(players)`: start only when both flags are true
   **and** both sessions have `conn.is_some()`. If one is reconnecting, keep
   the open ticket and wait; a resume calls `try_start_rematch` again. When it
   starts: remove the ticket, then
   `pair(ticket.players, MatchMode::Challenge, rematchable = true)`. The new
   match gets a fresh seed from `pair`; it never enters a queue and never
   touches `Rating`. Do not send a second `RematchState`; `MatchStart` follows.

**Decline / expiry / cleanup**

- `decline_rematch(token)` removes the open ticket containing the caller and
  sends `RematchClosed { declined: true }` to the other player. It is called
  by `leave_waiting` (explicit Cancel/Menu and any new match action), never by
  `Disconnected`.
- Lazy checks on join/request enforce expiry even between sweeps.
- The existing 10-s `cleanup` also removes expired invites and tickets:
  - expired invite: reset the owner to `Idle` if still `InChallenge`, send
    `ChallengeUnavailable`;
  - expired ticket: send `RematchClosed { declined: false }` to both;
  - invite/ticket whose session no longer exists: drop it and notify the
    remaining player if any.
- No permanent task, thread, or timer is added. The lobby sweep is enough.

### Match task: initial detached seat

In `match_task::spawn`, initialize `detached_at[seat]` to
`players[seat].conn.is_none().then(Instant::now)`. If a disconnect races the
pairing, the match must apply the existing 30-s reconnect/forfeit rule
instead of running a ghost seat forever. `Attach` already clears
`detached_at`.

### Client behavior

- `?challenge=<32 hex>` is parsed before `?room=CODE`. A valid token sends
  `JoinChallenge` automatically; an invalid token shows "Challenge link
  invalid" and stays in the menu. A server `ChallengeUnavailable` shows
  "Challenge expired or unavailable". A challenge param suppresses the
  load-time probe.
- `?challenge` is removed from the URL with `history.replaceState` after the
  first `MatchStart`, so a reload cannot re-consume the invite. Preserve
  other query parameters (`?server=`, `?lag=`).
- The host waiting screen shows the invite link (small, wrapped), **Copy
  link**, **Share** when `navigator.canShare`/`share` is available (fallback
  to copy), **Waiting for your friend**, and **Cancel**.
- The guest sees "Joining challenge..." while the socket connects; no
  account or confirmation screen.
- The result screen shows rematch controls only for `MatchMode::Challenge`:
  victory/defeat/draw, opponent name, **Request rematch** (or **Accept
  rematch** when the opponent already requested), the request state, and
  **Menu**. Until `RematchOpen` arrives, show a disabled "Preparing
  rematch..." label and never send a request. The **Menu** action sends
  `Cancel` and keeps the socket as a quiet probe until the frame is flushed,
  so the decline reaches the server. A reload on the result screen restores
  the rematch offer but not the old outcome; show "MATCH OVER" plus the
  opponent and rematch controls, and hide the stale duration/summary/rating
  (persisting the outcome is out of scope).
- The main Bevy menu and the desktop HTML lobby both show **Challenge a
  friend** and **Private room** as separate visible options. Keep the
  existing private-room dialog, code, and `?room=` flow unchanged.

## Steps (in order; check and commit each)

### 1. Protocol v9 and core contract tests

Files: `crates/tbg-core/src/protocol.rs`,
`crates/tbg-core/tests/contract_protocol.rs`,
`crates/tbg-core/tests/contract_protocol_server.rs`.
Rules: ARCHITECTURE §5 append-only wire; D-39; no seat in client messages.

- Add `InviteToken`, the three `ClientMsg` variants, the five `ServerMsg`
  variants, and `MatchMode::Challenge` exactly as above, appended at the end.
- Bump `PROTOCOL_VERSION` to 9 and update the version-test message.
- Extend the frozen variant-order tests with the new variants and their
  indices. Add exact-byte assertions for at least `JoinChallenge`,
  `ChallengeWaiting`, and `RematchClosed`.
- **Check:** `cargo test -p tbg-core --test contract_protocol --test contract_protocol_server`.
- **Done when:** all old indices are unchanged, the new indices are frozen,
  and the version test names v9 and P39.

### 2. Server challenge lifecycle

Files: `crates/tbg-server/src/config.rs`, `limits.rs`, `lobby.rs`,
`lobby/matchmaking.rs`, `lobby/sessions.rs`, `chan.rs`, `ws.rs` or
`ws/client_msg.rs` only as needed, `tests/contract.rs`.
Rules: BR-R08, BR-M01–M03, BR-P04, D-39; authority stays server-side;
private rooms and queues unchanged.

- Add `challenge_ttl_secs`, `rematch_ttl_secs`, and `invite_per_ip_min` to
  `Config` with defaults 600/120/30. Do not add environment variables.
  Update `tests/contract.rs::production_defaults`.
- Add `Limits::check_invite(ip, max_per_min, now)` using the existing
  sliding-log helper, and prune it in `sweep`.
- Add `InChallenge`, `challenges`, and the `create_challenge`,
  `join_challenge`, `cancel`, `leave_waiting`, and `leave_queue_room`
  behavior described above. Keep `rooms` and `JoinRoom` unchanged except
  for the new `pair` signature.
- Change `pair` to take `rematchable: bool`: quick/room/ranked pass
  `false`, challenge passes `true`. Add the `RematchTicket` struct and
  `rematches` vector now; a challenge pairing inserts a ticket with
  `expires: None`. Request handling stays for step 3.
- Route the new `ClientMsg` variants in `lobby.rs`: challenge messages are
  refused while draining with `ErrorCode::ServerFull`; `JoinChallenge`
  checks the invite limit; `RequestRematch` is a no-op until step 3.
- Add the resume behavior in `hello` and keep `Disconnected` on
  `leave_queue_room`.
- **Check:** `cargo test -p tbg-server --test contract` and
  `cargo check -p tbg-server`.
- **Done when:** creating/joining/canceling/expiring a challenge works in
  the lobby, private rooms still pass their existing test, and no challenge
  starts a match task before the second session joins.

### 3. Server rematch lifecycle

Files: `crates/tbg-server/src/lobby.rs`, `lobby/matchmaking.rs`,
`lobby/sessions.rs`, `lobby/account_done.rs`, `match_task.rs`,
`tests/contract.rs`.
Rules: BR-R09, BR-M01–M03, BR-P03, D-39; both players must request and be
connected; no rating change; no public queue.

- Use the ticket inserted by `pair(..., rematchable = true)` in step 2:
  rematch pairing also passes `true`. Implement `MatchEnded` ticket opening
  and `RequestRematch` / `decline_rematch` / cleanup exactly as contracted.
  Refuse `RequestRematch` while draining with `ErrorCode::ServerFull` (it
  starts a new match). The rematch uses `MatchMode::Challenge` and a fresh
  seed through `pair`.
- On an account login takeover that moves a running match seat to a new
  session token (BR-P03), replace the old token in any rematch ticket
  before `leave_waiting(old_token)`, so `MatchEnded` still finds the ticket
  and the old session is not left as an orphan. Add a lobby unit test for
  the token replacement.
- Initialize `match_task::spawn` detached seats from `conn.is_none()`.
- **Check:** `cargo check -p tbg-server` and
  `cargo test -p tbg-server --test contract`.
- **Done when:** a rematch starts only after both requests, duplicate
  requests are idempotent, a decline/expiry closes the ticket, and a
  disconnected player gets the reconnect window instead of a ghost match.

### 4. Server integration and cleanup tests

Files: `crates/tbg-server/tests/online.rs`, `crates/tbg-server/src/lobby/tests.rs`
(new; declare `#[cfg(test)] mod tests;` in `lobby.rs`), `tests/contract.rs`
only if a production default changed.
Rules: the acceptance matrix and BR-R08/BR-R09; tests assert documented
contracts, never weaken existing assertions.

Add integration tests over real WebSockets for:

1. `challenge_pairs_owner_and_guest_without_code`
2. `challenge_only_one_guest_wins`
3. `challenge_rejects_third_guest_and_owner_token`
4. `challenge_cancel_removes_invite`
5. `challenge_owner_reload_resumes_waiting` (guest retries while the host
   is away, gets `ChallengeUnavailable`, then joins after the host resumes)
6. `challenge_guest_reload_resumes_match`
7. `challenge_expires` (set `challenge_ttl_secs: 1`)
8. `challenge_initial_detached_forfeits` (set `reconnect_secs: 1`; safety
   net for a disconnect racing the pairing)
9. `rematch_requires_both`
10. `rematch_duplicate_requests_idempotent`
11. `rematch_decline_closes`
12. `rematch_reload_keeps_offer` (A requests, A disconnects, B requests:
    no `MatchStart`; A reconnects and the rematch starts)
13. `rematch_does_not_enter_public_queue`
14. `two_guests_same_ip_join_different_challenges`
15. `challenge_invite_rate_limit` (set `invite_per_ip_min: 2`)

Add lobby unit tests in `lobby/tests.rs` for:

- expired challenge cleanup resets the owner and sends
  `ChallengeUnavailable`;
- 100 expired challenges are removed in one sweep (bounded memory);
- expired rematch cleanup notifies both players with
  `RematchClosed { declined: false }`;
- an open ticket whose session disappeared is removed and the remaining
  player is notified.

Use a 5-ms tick and short TTLs; never sleep longer than 1.2 s except the
reconnect-window test. `recv` must answer pings and skip `NetStats`/`View`.
End matches with `Surrender` to keep tests fast.
- **Check:** `cargo test -p tbg-server --test online --test contract`.
- **Done when:** every test above passes and the existing
  `private_room_pairs_owner_and_guest`, casual, and ranked tests still pass.

### 5. Client transport, URL, and protocol handling

Files: `crates/tbg-client/src/config.rs`, `net/mod.rs`, `net/socket.rs`,
`link.rs`, `ui/start.rs`, `ui/actions.rs`, `ui/mod.rs`, `sync.rs`,
`sync/messages.rs`, `main.rs` only if a state is needed.
Rules: BR-R08, BR-P04 (no login), BR-A01–A04 (client sends requests only);
the invite token is never a credential.

- Add `Mode::CreateChallenge` and `Mode::JoinChallenge(InviteToken)`;
  map them in `send_request`. Add `UiAction::CreateChallenge`,
  `JoinChallenge(InviteToken)`, `RequestRematch`, `ShareChallenge`, and
  `ChallengeInvalid`.
- Add `config::parse_invite_token`, `config::invite_token_hex`,
  `config::challenge_link`, `config::share_text`, and
  `config::clear_url_param`. Reuse the existing hex token helpers where
  possible. Add the `web-sys` `History` feature and a small `wasm-bindgen` browser
  helper that catches share/clipboard Promise rejections; no new crate.
- Parse `?challenge=` before `?room=`; suppress the probe when either is
  present; show the invalid-link toast for malformed tokens.
- Handle `ChallengeWaiting`, `ChallengeUnavailable`, `RematchOpen`,
  `RematchState`, and `RematchClosed` in `poll_link`. Extend `QueueInfo`
  with `challenge: Option<InviteToken>` and `joining_challenge: bool`; add
  a `RematchInfo` resource and reset it on `MatchStart`. If
  `Error { RateLimited }` arrives while `joining_challenge`, toast and
  return to the menu instead of leaving the player on "Joining...".
- On `MatchStart`, set `MatchInfo.challenge = mode == MatchMode::Challenge`,
  clear queue state, and call `clear_url_param("challenge")` on wasm.
- On `Cancel | ToMenu`, send `Cancel` and try `reuse(Mode::Probe, now)`
  before dropping the link, so the decline/cancel frame is flushed.
- **Check:** `cargo check -p tbg-client` (native) and
  `cargo check -p tbg-client -p tbg-core --target wasm32-unknown-unknown`.
- **Done when:** the client compiles native and WASM and the URL helpers
  have unit tests for valid, invalid, and uppercase hex.

### 6. Client UI

Files: `crates/tbg-client/src/ui/menu.rs`, `ui/queue.rs`, `ui/result.rs`,
`ui/actions.rs`, `ui/desktop_actions.rs`, `crates/tbg-client/index.html`,
`assets/lobby/lobby.css` only if needed.
Rules: BR-R08/BR-R09; keep the current visual language, private-room dialog,
HUD, countdown, and arena untouched.

- Menu: add **Challenge a friend** (`UiAction::CreateChallenge`) and rename
  the existing room button to **Private room**. Keep both visible on the
  Bevy menu and the desktop HTML lobby (`data-action="challenge"` and the
  existing `create` dialog).
- Queue: show the challenge link (small, wrapped), **Copy link**, **Share**
  (if available; fallback to copy), **Waiting for your friend**, and
  **Cancel**; show "Joining challenge..." for the guest. Keep the existing
  room code/link and the 30-s practice offer behavior.
- Result: for challenge matches show the rematch section described above
  and hide **Play again**; for other matches keep the current result screen.
  The rematch label/button update when `RematchInfo` changes; hide the
  button once the player requested, both requested, or the ticket closed.
- Do not change the HUD, countdown, arena, or visual style.
- **Check:** `trunk build --release` in `crates/tbg-client/`; open the page
  and verify the menu, waiting screen, and result screen at desktop and
  phone sizes. Do not commit generated `crates/tbg-client/dist/` changes.
- **Done when:** a human can create a challenge, copy/share the link, see
  the waiting state, and see the rematch state without a layout overlap.

### 7. Canon and architecture updates

Files: `docs/GAME-DESIGN.md`, `docs/ARCHITECTURE.md`, `docs/DECISIONS.md`,
`docs/ROADMAP.md` only if the status/row needs correction.
Rules: new rules stay **Proposed**; no existing rule, number, or phase is
renumbered or silently changed.

- Confirm the planning commit's Proposed glossary rows `Quick Challenge`
  and `Rematch`, the extended `MatchMode` row, BR-R08, BR-R09, D-39, and
  the removal of "rematch" from the deferred list. Edit them only if the
  implementation proves a contract change; keep them **Proposed**.
- Update `ARCHITECTURE.md` protocol to v9 and the new messages/mode,
  sessions/matchmaking/lifecycle, and the invite per-IP limit.
- Do not rewrite existing decisions or phase files.
- **Check:** `rg -n "PROTOCOL_VERSION|Quick Challenge|BR-R08|BR-R09" docs`
  shows one consistent version and the new rules.
- **Done when:** docs and code agree; no unrelated rule/number changed.

### 8. Verification and phase report

Run, in this order:

```bash
cargo fmt --all -- --check
cargo clippy --workspace -- -D warnings
cargo test --workspace
cargo check -p tbg-client -p tbg-core --target wasm32-unknown-unknown
cd crates/tbg-client && trunk build --release
```

Then the manual acceptance matrix (desktop + Android phone + PWA; two
tabs; private room; casual/ranked smoke). Save desktop and phone
screenshots under `docs/media/p39/` and reference them in the report.
Fill the **Phase report** below with real commands and results, list
deviations, and commit. Never claim a check you did not run.

## Acceptance matrix

| # | Case | Proof |
|---|------|-------|
| 1 | Two guests play without login | `challenge_pairs_owner_and_guest_without_code` |
| 2 | Guest opens the link on a phone | manual: Android Chrome + PWA |
| 3 | Host reloads while waiting | `challenge_owner_reload_resumes_waiting` |
| 4 | Guest reloads during combat | `challenge_guest_reload_resumes_match` + existing reconnect test |
| 5 | Third user opens a used invite | `challenge_rejects_third_guest_and_owner_token` |
| 6 | Two guests join simultaneously | `challenge_only_one_guest_wins` |
| 7 | Same session token tries both seats | `challenge_rejects_third_guest_and_owner_token` |
| 8 | Invalid link | client hex unit test + manual toast |
| 9 | Expired link | `challenge_expires` |
| 10 | Host cancels | `challenge_cancel_removes_invite` |
| 11 | Participant disconnects | `challenge_initial_detached_forfeits` + existing reconnect |
| 12 | Participant surrenders | `rematch_requires_both` (ends by `Surrender`) |
| 13 | Only one accepts the rematch | `rematch_requires_both` |
| 14 | Both accept the rematch | `rematch_requires_both` |
| 15 | Duplicate rematch requests | `rematch_duplicate_requests_idempotent` |
| 16 | User leaves during rematch wait | `rematch_decline_closes` |
| 17 | Two tabs of the same browser | manual: distinct sessions, no interference |
| 18 | Two players on the same IP | `two_guests_same_ip_join_different_challenges` |
| 19 | PWA and normal browser | manual: both open the same link |
| 20 | Reload/navigation with a consumed invite | URL cleanup after `MatchStart` + `challenge_rejects_third_guest_and_owner_token` |
| 21 | Match starts only with both sessions connected | `challenge_owner_reload_resumes_waiting` + `rematch_reload_keeps_offer` |

Regression gates: `private_room_pairs_owner_and_guest`, all casual/ranked
tests, reconnect tests, and the protocol contract tests. No test may delete
or weaken an existing assertion.

## Reviewer checklist

- [x] Existing `CreateRoom`/`JoinRoom`/`?room=` behavior and TTL unchanged.
- [x] Challenge token is 128-bit OS-random, single-use, independent of the
      session token, and never logged.
- [x] Only two distinct session tokens can occupy a challenge match; the
      owner cannot join its own invite; a third guest cannot interfere.
- [x] No match task is spawned before the second session joins.
- [x] Challenge/rematch state is bounded and swept; no permanent task.
- [x] Rematch requires both requests, uses a fresh seed, is not ranked, and
      never enters the public queue.
- [x] Reload during waiting/combat/result recovers the expected state;
      explicit Menu declines the rematch.
- [x] Same-IP players are not blocked by the invite limit.
- [x] `PROTOCOL_VERSION` is 9; old indices are unchanged; docs agree.
- [x] All verification commands above were actually run.

## Phase report

- Date: 2026-10-29. Initial review implementation HEAD: `c239dee`, recorded
  by `5569b11`. Branch: `phase/p39-quick-challenge-pr`, originally created
  directly from `fb47717`. Original `phase/p39-quick-challenge` remains at
  `22571c5`. The follow-up integrates main `077abe7` after PR #1 brought
  the original P39 and P16-D into main. P16-D/P16-F are preserved as that
  baseline; this PR changes none of their source, specs, or evidence.
- Protocol: **v9**. P39 follows v8 Arcanas: client tags 14–16, server tags
  20–24, Challenge mode tag 2. The existing Arcana tags and all `MatchStart`
  fields/bytes stay intact. BR-AR05 explicitly includes Quick Challenge.

### Commands and real results

All commands ran on the new branch. WebSocket tests used loopback sockets.

```text
cargo fmt --all -- --check
  exit 0
cargo clippy --workspace -- -D warnings
  Finished `dev` profile [unoptimized + debuginfo] target(s) in 2.99s
cargo test --workspace
  166 passed; 0 failed; 1 ignored (sum of the individual suite results)
cargo check -p tbg-client -p tbg-core --target wasm32-unknown-unknown
  Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.73s
trunk build --release  (crates/tbg-client; RUST_LOG=info, persistent terminal)
  Finished `release` profile [optimized] target(s) in 0.41s
  2026-10-29T15:46:27.141208Z  INFO ✅ success
  exit 0
git diff --check
  exit 0
node scripts/check-p39-share.mjs
  7 share/clipboard scenarios passed; no unhandled rejections
```

The full workspace run includes 29 online tests, 11 server unit tests,
3 server production/FIFO contract tests, 2 BR-R08/BR-R09 contract tests,
3 existing online Arcana tests, and 8 protocol contract tests. The client
has 26 passing tests, re-run after the final URL wrapping change, along
with WASM, fmt and clippy. The one existing ignored test is
`c4b_realtime_server_match_equals_headless_replay`: a 1x match, explicitly
reserved for balance phases/releases; the normal replay-equivalence test
passed. No assertions were deleted or weakened.

Additional regression evidence:

- `match_task::tests::challenge_initial_detached_forfeits` starts seat A
  detached, observes the full one-second test reconnect window, then gets
  `MatchEnd { winner: B, reason: Forfeit }` and match cleanup.
- `challenge_freezes_arcanas_through_reconnect_and_rematch` checks two
  distinct presets, rejection of waiting/in-match edits, guest reconnect,
  both rematch requests, unchanged presets and different replay seeds.
- Lobby tests cover lazy rematch expiry on resume, drain protection, a
  delayed account preset arriving after challenge freeze, expired invite
  cleanup (including 100 records), ticket cleanup and takeover tokens.
- Client tests run the real waiting/result Bevy systems, including all
  rematch states, so the conflicting-query B0001 regression fails directly.
  Transport tests ensure reconnect never recreates or reconsumes an invite.

### Browser observations

Used the Codex in-app Chromium browser, two independent tabs, desktop
1440×900 and 1280×720, and a **390×844 viewport**. This is viewport testing,
not a physical Android device. A local authoritative server ran at 50 ms
on `127.0.0.1:8081`, two workers, a disposable SQLite file in `/tmp`, and
`?lag=100` on both tabs. The existing service on port 8080 was left alone.

Observed working:

- Desktop and Bevy menus expose Challenge and Private Room separately.
- Create/copy invite; Share falls back to copy in this browser; reload the
  host waiting screen and keep the same link; join automatically in the
  other tab with a different guest name/seat.
- After `MatchStart`, only `challenge` disappears from the guest URL;
  `server`, `lag` and `#guest-p39` remain. Reload the guest during combat
  and resume the same names, seat and running clock.
- Both clients finish the match by Surrender. A single rematch request
  shows Waiting/Accept without starting a match. Host result reload shows
  MATCH OVER and the restored request, with no stale duration/summary.
  Accept starts a fresh countdown between the same guests. Both clients
  finish that rematch; Menu on one side delivers Opponent declined on the
  other, hides the request button, and returns the leaver to the menu.
- Cancel leaves the public queue; the other client stays searching rather
  than pairing with the canceled client. Private Room still opens its
  create/join dialog, creates a code/link, and Cancel removes the room.
  Opening the canceled code returns to the menu. The existing real-socket
  private-room pairing/FIFO/ranked tests also pass.
- Cancel a challenge, then open its copied link in the other client:
  Joining returns to the menu rather than starting a match. Copy was also
  verified on the phone viewport. Final waiting URLs wrap inside the panel
  on both desktop and phone, including a long `server` override.
- No browser console errors in either tab during the completed match and
  rematch; no B0001 panic. Ranked remains locked for guests; authenticated
  ranked regression was tested automatically, not played manually.

Screenshots (actual browser captures):

- [Desktop menu](../media/p39/desktop-menu.jpg),
  [waiting](../media/p39/desktop-waiting.jpg),
  [match](../media/p39/desktop-match.jpg),
  [result](../media/p39/desktop-result.jpg).
- [Phone menu](../media/p39/phone-menu.jpg),
  [waiting with wrapped URL](../media/p39/phone-waiting.jpg),
  [match](../media/p39/phone-match.jpg),
  [result](../media/p39/phone-result.jpg),
  [one request](../media/p39/phone-rematch-wait.jpg),
  [result reload](../media/p39/phone-rematch-reload.jpg),
  [decline](../media/p39/phone-rematch-declined.jpg).

### Deviations and limits

- Initially ported the three original P39 commits onto `fb47717`, resolving
  Arcanas/D-40 without the old P16-F ancestry. v9 replaced the planning
  branch's colliding v8. After PR #1 landed, integrated `077abe7` without
  rewriting history. Protocol declarations and frozen-wire tests now match
  main exactly, including its additional Arcana byte assertions. No balance
  value or P16-D/P16-F file differs from the new base.
- The initial-detach proof is a deterministic match-task unit test rather
  than a nondeterministic WebSocket pairing race, as allowed by the
  handoff. It exercises the actual spawn/forfeit path.
- `contract_challenge.rs` only gains the mandatory Arcana field binding
  and an extra frozen-preset assertion; all original BR-R08/R09 assertions
  remain. This integration was committed with the BR-AR05 canon update.
- Challenge and rematch code have separate lobby modules. Reconnect now
  preserves acknowledged invite/result modes; resume enforces expiry and
  draining; late database replies cannot change frozen selections; late
  waiting/offer messages cannot reopen an explicitly dismissed screen.
- Sharing uses the existing wasm-bindgen dependency for feature detection,
  AbortError handling and clipboard fallback. Native share-sheet success,
  physical Android, installed PWA and owner look/fun sign-off remain
  unverified. This client has no installable PWA manifest/service worker;
  adding that is outside P39. No claim of PWA/device acceptance is made.
- No dedicated performance/heap benchmark was run. Final release WASM:
  **24,712,473 bytes**, uncompressed. State uses the existing bounded
  session lifetime and cleanup sweep; the 100-expired-invite test passes.

### Execution interruptions

> blocked (resolved): initial loopback tests failed in the sandbox with
> `Os { code: 1, kind: PermissionDenied, message: "Operation not permitted" }`.
> Running the same tests with local-socket permissions passed.

> blocked (resolved): build sessions were interrupted with signal 15
> (`exit 143`), including one after Trunk had printed `✅ success`.
> Rust compilation had succeeded; the suspected cause was execution-session
> lifetime, not confirmed. Stopped the interrupted invocation, inspected
> logs/processes, and changed to a persistent terminal with full logs.
> The final invocation printed `✅ success` and returned **exit 0**; the
> final package was reloaded and the wrapped link verified in the browser.

No implementation blocker remains. PR review and the physical-device/PWA
and subjective owner acceptance checks remain pending; no merge or public
deployment is included in this work.


### Follow-up after PR #1 merged into main

The checks on `5569b11` both passed (GitHub Actions runs `40906366558`
and `40906267887`; no review comments or unresolved review threads). The
subsequent main merge `077abe7` made PR #2 conflict because it included the
original P39 separately. Integrated that main into this branch with a
merge commit, retaining the reviewed reconnect/expiry/Arcana fixes and
all contract assertions. Auto-merge duplicates of `MatchInfo.challenge`,
initial detach initialization, and decision D-39 were removed before checks.
The P16-D engine, its contract tests/evidence, and the P16-F plans match
`077abe7` exactly; they are not additional changes in PR #2.

Re-ran on the integrated tree:

```text
cargo fmt --all -- --check
  exit 0
cargo clippy --workspace -- -D warnings
  Finished `dev` profile [unoptimized + debuginfo] target(s) in 1.33s
cargo test --workspace
  178 passed; 0 failed; 1 ignored (sum of the individual suite results)
cargo check -p tbg-client -p tbg-core --target wasm32-unknown-unknown
  Finished `dev` profile [unoptimized + debuginfo] target(s) in 1.03s
node scripts/check-p39-share.mjs
  7 share/clipboard scenarios passed; no unhandled rejections
```

```text
trunk build --release  (crates/tbg-client; RUST_LOG=info, persistent terminal)
  Finished `release` profile [optimized] target(s) in 1m 09s
  2026-10-29T16:08:01.583042Z  INFO ✅ success
  exit 0
git diff --check
  exit 0
```

Rechecked the integrated package in two new browser sessions against the
local 50 ms authoritative server, with `?lag=100`. Created an invite and
reloaded the waiting host: the same displayed invite returned. The second
session joined by that displayed link; `challenge` disappeared while
`server`, `lag` and the fragment remained. Guest combat reload resumed the
same opponents and clock. Both players received the result after Surrender.
One request showed Accept on the other side; host result reload retained
the offer, Accept started a fresh countdown, and both clients finished the
rematch. No console errors occurred. Captured the refreshed desktop result,
390×844 result/accept view and rematch view in the existing screenshot files.
Clipboard/share success remains covered by the initial browser run and the
seven Promise scenarios; it was not re-established in this follow-up run.
Physical Android, installed PWA and subjective owner sign-off remain open.

The original two successful CI runs belong to `5569b11`; CI must run again
on this follow-up merge commit. PR #2 remains open, with no PR merge or
public deployment authorized by this continuation.
