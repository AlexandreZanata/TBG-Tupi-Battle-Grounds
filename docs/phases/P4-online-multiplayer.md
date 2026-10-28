# P4 — Online 1v1, latency metrics, local bot validation

**Depends on:** P3 reviewed and merged.
**Branch:** `phase/p4-online-multiplayer`.
**Difficulty:** high (steps 2–5). Follow the task and channel design
exactly; do not invent a different concurrency model.

## Goal

Two real players on separate devices play a server-authoritative match
with reconnects. Every connection and match is **measured** (RTT, view
timing, tick lateness, bandwidth). A bot-client tool plays hundreds of
matches against the local server, with artificial latency and random
disconnects, and proves the server holds up — all before any real VPS.

## Deliverable

- `tbg-server` with lobby (quick match + private rooms), match tasks,
  sessions with resume, limits, and `/metrics`.
- Client menu: **Quick Match**, **Play with a friend** (create room + link),
  auto-join from `?room=CODE`; reconnect overlay; `?lag=MS` simulator.
- `tbg-bots` binary: N bot clients over real WebSockets with
  `--delay-ms`, `--jitter-ms`, `--chaos-pct`.
- Integration tests and a filled-in local validation report.

## Read first (only these)

- `AGENTS.md`
- `docs/ARCHITECTURE.md` §5 (networking) and §11 (security)
- `crates/tbg-core/src/protocol.rs`, `view.rs`, `bot.rs` (public API)
- Client: `link.rs`, `net.rs`, `ui/menu.rs`, `sync.rs`
- Server: `src/main.rs` from P1
- Bevy example: `ui/text/text_input.rs` (only if you build the code entry)

## Dependencies you may add

| Crate | Where | Why |
|-------|-------|-----|
| `serde_json = "1"` | server | `/metrics` JSON |
| `getrandom = "0.3"` | server | session tokens, match seeds |
| `tokio-tungstenite = { version = "0.29", features = ["rustls-tls-webpki-roots"] }` (same version axum 0.8.9 uses) | server | bot client and tests |
| `futures-util = "0.3"` | server | split WebSocket streams |
| `tower-http = { version = "0.6", features = ["fs"] }` | server | optional static file serving for LAN tests |
| `tracing = "0.1"`, `tracing-subscriber = "0.3"` | server | logs |
| `web-sys` features `Storage`, `Clipboard`, `Navigator` | client (wasm only) | token storage, copy room link |

Nothing else.

## Server design (implement exactly)

`crates/tbg-server` becomes a library plus two binaries:

```
src/lib.rs          pub struct Config; pub async fn run(config: Config, listener: TcpListener)
src/main.rs         reads env into Config, binds, calls run()
src/bin/tbg-bots.rs the bot load tool (step 7)
src/config.rs       Config { addr, max_connections: usize (env TBG_MAX_CONNECTIONS, default 2000),
                    max_per_ip: usize (env TBG_MAX_PER_IP, default 8), reconnect_secs: u64 (env TBG_RECONNECT_SECS, default 30),
                    static_dir: Option<PathBuf> (env TBG_STATIC_DIR) }
src/ws.rs           one task per connection
src/lobby.rs        one task: sessions, queue, rooms
src/match_task.rs   one task per match
src/metrics.rs      shared counters + RTT samples
src/names.rs        generated display names
src/botclient.rs    reusable bot client (used by tbg-bots and tests)
```

Channels (all `tokio::sync::mpsc`, all **bounded**):

```rust
// lobby inbox (capacity 1024)
enum LobbyCmd {
    Hello { resume: Option<Token>, conn: ConnHandle, reply: oneshot::Sender<Token> },
    Client { token: Token, msg: ClientMsg },          // QuickMatch, CreateRoom, JoinRoom, Cancel
    Disconnected { token: Token, conn_id: u64 },
    MatchEnded { tokens: [Token; 2] },
}
// connection inbox (capacity 64) — the only way to talk to a client
struct ConnHandle { id: u64, tx: mpsc::Sender<ConnEvent> }
enum ConnEvent { Send(ServerMsg), EnterMatch { seat: Seat, tx: mpsc::Sender<MatchCmd> }, LeaveMatch, Close }
// match inbox (capacity 256)
enum MatchCmd { Intent { seat: Seat, seq: u32, intent: PlayerIntention },
                Detach { seat: Seat, conn_id: u64 }, Attach { seat: Seat, conn: ConnHandle } }
```

Rules:
- **Seat ownership:** `ws.rs` stamps the seat it received in `EnterMatch`
  on every `Intent`. Client messages never carry a seat.
- **Never block a match on a client:** the match task uses
  `conn.tx.try_send(..)`; on `Full` it sends `ConnEvent::Close` best-effort
  and treats the seat as detached; on `Closed` it treats it as detached.
- **Stale connections:** `Disconnected` and `Detach` carry the
  connection id; the lobby and the match task ignore them when that id is
  not the session's/seat's *current* connection (an old socket closing
  after a resume must not detach the new one).
- The lobby owns `sessions: Vec<Session>` (`Session { token, name, conn:
  Option<ConnHandle>, state: Idle | Queued | InRoom(code) | InMatch {
  tx, seat }, detached_at: Option<Instant> }`). `Vec` + linear search is
  fine at MVP scale.

## Steps

### 1. Server skeleton and config
Split into `lib.rs`/`main.rs`, add `config.rs`, `tracing` logs, routes
`/health`, `/ws`, `/metrics` (step 6), and when `static_dir` is set a
`ServeDir` fallback so `http://<LAN IP>:8080/` serves the client `dist/`.
`axum::serve(listener, app.into_make_service_with_connect_info::<SocketAddr>())`.

### 2. Connection task (`ws.rs`)
For each upgraded socket (`max_message_size(1024)`, `max_frame_size(1024)`):
1. Reject with `Error{TooManyConnections}` + close if the global count ≥
   `max_connections` or this IP has ≥ `max_per_ip` sockets. IP = peer
   address, except when the peer is `127.0.0.1`/`::1` and an
   `X-Forwarded-For` header exists (Caddy in P5): then the first address in it.
2. The first message must arrive within 5 s and be `Hello`. If `version !=
   PROTOCOL_VERSION` → send `VersionMismatch{server}` and close. Else send
   `LobbyCmd::Hello`, await the token, send `Welcome{token, name}`.
3. Main loop with `tokio::select!` over: socket messages, the connection
   inbox, a 1 s ping interval, and an idle deadline.
   - Binary frame → `decode_client`; error → `Error{BadMessage}` + close.
   - Token bucket: capacity 40, refill 20 per second; empty →
     `Error{RateLimited}` + close.
   - `Intent` → if in a match, `MatchCmd::Intent` with the stamped seat,
     else `Error{NotInMatch}`. `Pong{id}` → RTT sample (below). Others →
     `LobbyCmd::Client`.
   - Every 1 s send `Ping{id}` (remember `id → Instant`); on `Pong` compute
     RTT ms, record it in `metrics`, keep it for this connection, and send
     `NetStats{rtt_ms}`.
   - No message from the client for 15 s → close.
4. On exit send `LobbyCmd::Disconnected` and, if in a match,
   `MatchCmd::Detach`. Decrement counters. Log one line:
   `conn closed id=.. secs=.. bytes_in=.. bytes_out=.. rtt_p50=.. rtt_p95=..`.

### 3. Lobby task (`lobby.rs`)
- `Hello` with a resume token of a known session whose `conn` is `None`:
  attach the new connection; if `InMatch`, send `MatchCmd::Attach` and
  `ConnEvent::EnterMatch`; reply the same token. Otherwise create a new
  session (16 random bytes from `getrandom`, generated name).
- `QuickMatch`: if another session is `Queued`, pair them; else mark
  `Queued` and send `Queued`. `Cancel` returns to `Idle`.
- `CreateRoom`: 5-character code from `ABCDEFGHJKMNPQRSTUVWXYZ23456789`
  (unique among open rooms), state `InRoom`, send `RoomCreated`. Rooms
  expire after 10 minutes. `JoinRoom{code}`: found → pair; else `RoomNotFound`.
- Pairing: random `u64` seed from `getrandom`, spawn `match_task`, set both
  sessions `InMatch`, send `EnterMatch` to both connections.
- `Disconnected`: if `Queued`/`InRoom` → remove from queue/room; set
  `conn = None`, `detached_at = now`. Every 10 s, drop sessions detached
  longer than `reconnect_secs` that are not in a match.
- `MatchEnded`: both sessions back to `Idle`; send `LeaveMatch`.

### 4. Match task (`match_task.rs`)
```
let mut interval = tokio::time::interval(Duration::from_millis(50));
interval.set_missed_tick_behavior(MissedTickBehavior::Delay);
send MatchStart{you, opponent} to both seats
loop {
    interval.tick().await;
    record lateness = now - expected tick time (metrics);
    while let Ok(cmd) = rx.try_recv() { handle Intent / Detach / Attach }
    let t0 = Instant::now(); events.extend(state.step()); record step duration;
    if state.tick % 2 == 0 { send View(view_for(seat, &events)) to each attached seat; events.clear(); }
    for each detached seat: if detached longer than reconnect_secs → state.forfeit(seat)
    if let Some(outcome) = state.outcome { send final View + MatchEnd to both; break }
}
send LobbyCmd::MatchEnded; log one JSON line with: seed, duration_s, outcome,
reason, per seat {rtt_p50, rtt_p95, intents, rejections, bytes_out}, step_us_mean,
step_us_max, late_ticks (lateness > 25 ms), max_lateness_ms.
```
- `Intent` → `apply_intention`; on `Err(reason)` send `Rejected{seq, reason}`.
- `Attach` → store the connection, send `MatchStart` and an immediate full `View`.
- The task must not panic on any client input. Wrap nothing in `unwrap`
  except channel creation.

### 5. Client network link (`net.rs`, `link.rs`)
- Add `MatchLink::Net(NetLink)`. `NetLink` owns the `ewebsock` sender and
  receiver, the session token, and a reconnect state machine:
  `Connecting → Hello sent → Ready → (lost) → Reconnecting (every 1 s, up to
  reconnect_secs + 5) → Failed`.
- Token storage: wasm → `localStorage` key `tbg_token` (hex); native → memory.
- `?lag=MS` (default 0): hold every outgoing and incoming message in a
  queue for `MS/2` milliseconds each way (use `Time::elapsed_secs_f64`).
- On `VersionMismatch` show "A new version is available — reload the page".
- While `Reconnecting`, show a centered "Reconnecting…" overlay; the match
  view keeps the last frame.
- The overlay from P1 now shows: `RTT` from `NetStats`, `view p95` (95th
  percentile of the last 50 view inter-arrival times in ms), `lag sim` when
  `?lag` is set. Remove the P1 echo ping.

### 6. Metrics (`metrics.rs`, `/metrics`)
`Arc<Metrics>` with atomics: `connections`, `matches_running`,
`matches_finished`, `queue_len`, `bytes_in`, `bytes_out`, `late_ticks`,
`rejections`, and a `Mutex<VecDeque<(Instant, u16)>>` of RTT samples kept
for 60 s. `GET /metrics` → JSON with those values plus `rtt_p50_ms`,
`rtt_p95_ms`, `rtt_max_ms` over the last 60 s and `uptime_s`. It answers
only when the peer IP is loopback **and** there is no `X-Forwarded-For`
header; otherwise 404.

### 7. Bot client and `tbg-bots`
`botclient.rs`: `pub async fn play_one(url, opts, bot_seed) -> BotResult`
using `tokio-tungstenite`: connect, `Hello`, `QuickMatch`, answer `Ping`
with `Pong`, on each `View` ask `tbg_core::bot::Bot` and send its intent;
apply `delay_ms ± jitter_ms` (uniform, from the bot's own `Pcg32`) before
every send and after every receive. With probability `chaos_pct`, close the
socket once at a random time 20–60 s into the match and reconnect with the
token 5 s later. Ends on `MatchEnd` or a 6-minute timeout.
`BotResult { completed: bool, outcome, rtt_samples, view_gaps_ms, rejections, reconnected: bool, error: Option<String> }`.

`src/bin/tbg-bots.rs`: args `--url ws://127.0.0.1:8080/ws --matches N
--delay-ms 0 --jitter-ms 0 --chaos-pct 0 --ramp-ms 20` (manual parsing of
`std::env::args`). Starts `2N` bots `ramp-ms` apart, waits for all, prints a
table and one JSON summary line: matches completed/failed, outcomes,
RTT p50/p95/max, view-gap p95/max, reconnects attempted/succeeded,
errors. Exit code 1 if any bot failed.

### 8. Online menus
`ui/menu.rs`: enable **Quick Match** (shows "Searching… m:ss" and Cancel;
after 30 s also offers "Practice while you wait"), **Play with a friend**
(creates a room, shows the code big plus **Copy link** →
`https://<host>/?room=CODE`), and auto-join when the page has `?room=`.
If joining fails (`RoomNotFound`), show a message and return to the menu.
Practice vs Bot stays.

### 9. Integration tests (`crates/tbg-server/tests/online.rs`)
Each test starts `run(config, listener_on_127.0.0.1:0)` in a task and uses
`botclient`. Write exactly these:
- `two_bots_finish_a_match` — both `completed`, same outcome seen by both.
- `cannot_evolve_opponent_unit` — after the first View that contains an
  enemy unit, send `Evolve{unit: enemy_id, branch: 0}` → `Rejected{NotYourUnit}`.
- `malformed_frame_closes_connection` — send 3 random bytes after `Hello` →
  `Error{BadMessage}` then close.
- `version_mismatch_rejected` — `Hello{version: 999}` → `VersionMismatch`.
- `rate_limit_closes_flooder` — send 200 `Pong` messages at once → `Error{RateLimited}`.
- `reconnect_resumes_match` — disconnect a bot for 2 s, resume with the token, match completes.
- `forfeit_after_window` — config `reconnect_secs = 1`; disconnect a bot
  for good → the other bot receives `MatchEnd` with reason `Forfeit`.

### 10. Local validation runbook (fill in the report table)
Run the server in release mode: `TBG_MAX_PER_IP=10000 TBG_STATIC_DIR=crates/tbg-client/dist cargo run --release -p tbg-server`
(after `trunk build --release`). In a second terminal:

| # | Command | Pass when |
|---|---------|-----------|
| L1 | `cargo test -p tbg-server` | all pass |
| L2 | `cargo run --release -p tbg-server --bin tbg-bots -- --matches 1` | 1 completed |
| L3 | `... --matches 50 --delay-ms 40 --jitter-ms 20` | 50 completed, view-gap p95 ≤ 150 ms |
| L4 | `... --matches 50 --delay-ms 150 --jitter-ms 50 --chaos-pct 20` | all completed, reconnects succeeded = attempted |
| L5 | `... --matches 300 --ramp-ms 10` | all completed; record server CPU % and RSS (`ps -o %cpu,rss -p <pid>` every 10 s), `curl localhost:8080/metrics` at peak |
| L6 | Two browser windows on the PC → Quick Match | both play to the end, same result |
| L7 | PC browser + phone on Wi-Fi (`http://<LAN IP>:8080/`) → Play with a friend link | match completes |
| L8 | During L7: phone Wi-Fi off 10 s, then on | "Reconnecting…", then the match continues |
| L9 | Repeat L7, phone Wi-Fi off 40 s | both sides see the result; phone side lost by Forfeit |
| L10 | `?lag=300` on one browser, play a match | playable; record subjective notes |

## Acceptance checklist

- [ ] All verification commands from `AGENTS.md` pass
- [ ] L1–L10 pass (paste outputs and measurements into the report)
- [ ] Bandwidth from L3: mean `bytes_out` per player per second ≤ 6 000
- [ ] L5: no failed matches; `late_ticks` / total ticks < 1%
- [ ] Server log has no panic and no `ERROR` lines during L2–L9

## Risks and what to do

| Problem | Action |
|---------|--------|
| Deadlock / matches stall | A task awaited a full channel. Use `try_send` from match and lobby tasks; only `ws.rs` may `await` sends to its own socket |
| Bots fail with `TooManyConnections` | All local bots come from 127.0.0.1: start the server with `TBG_MAX_PER_IP=10000` for L2–L5 |
| Reconnect creates a new session | The client must send the stored token in `Hello.resume` |

## Reviewer checklist

- Seat is never read from client input; `NotYourUnit` test passes.
- No `.await` on another task's channel from lobby or match tasks.
- Limits: 1 KiB frames, 5 s handshake, 15 s idle, token bucket, per-IP and global caps.
- `/metrics` unreachable through a proxy header.
- Report numbers are real and reproducible (re-run L3 and L5).

## Phase report

Executed by the reviewer agent (Claude) on 2026-10-28, branch `phase/p4-online-multiplayer`,
on the owner's Linux PC (16 threads). Server and bots on the same machine.

- Commands run and results:
  - `cargo test --workspace` → tbg-core 15 passed; tbg-server **8 passed** (the 7
    listed in step 9 + `private_room_pairs_owner_and_guest`), ~9 s at a 5 ms tick.
  - `cargo clippy --workspace --all-targets -- -D warnings` and the wasm32 clippy → clean.
  - `cargo build --release -p tbg-server` → ok (41 s); `trunk build --release` → ok.
  - WASM (release, opt "s"): raw 23,598,033 B · gzip -9 7,343,161 B · brotli q11 4,593,445 B.
- Validation table (release server, `TBG_MAX_PER_IP=100000`, real 50 ms tick):

  | # | Result |
  |---|--------|
  | L1 | 8/8 integration tests pass |
  | L2 | 1 match, 2/2 bots completed, 108 s, RTT p50 2 ms, view gap p95 100 ms |
  | L3 | 50 matches, +40±20 ms each way: 100/100 completed, RTT p50 82 / p95 111 ms, **view gap p95 128 ms** (≤150 ✓) |
  | L4 | 50 matches, +150±50 ms, 20% chaos: 100/100 completed, **29 drops → 29 resumed**, RTT p95 373 ms |
  | L5 | **300 concurrent matches** (600 sockets, 10 ms ramp): 600/600 completed, view gap p95 101 ms, **0 late ticks** (max lateness 7 ms over 401 matches) |
  | L6 | Browser (`?autoplay=quick`) vs 1 socket bot through the release server: full match, VICTORY screen matches the server log (seat A, 74 s) |
  | L7 | "Play with a friend" → room ACXMR + Copy link; second tab via `?room=ACXMR` joined, match ran to VICTORY (47 s) |
  | L8 | Browser socket force-closed mid-match (JS) → client reconnected with its token, match continued; server log `"reconnects":1` for that seat |
  | L9 | Covered by `forfeit_after_window` (real WebSocket, 1 s window). Browser run ended by Stronghold before the window. **Real-device Wi-Fi-off test pending owner.** |
  | L10 | `?lag=300`: overlay `RTT 308 ms · view p95 110 ms · lag sim 300 ms`; match fully playable |

- Server cost during L5 (300 matches): **peak 1.06 CPU cores, 89.5 MB RSS**; bandwidth over
  all runs **1.46 KB/s down, 3 B/s up per player** (81,238 player-seconds, budget ≤6 KB/s).
- Deviations:
  1. Added `TBG_TICK_MS` (default 50) so tests run whole matches 10× faster.
  2. `tbg-bots --bots N` to pair a single bot with a browser.
  3. `tbg-server` modules split (`ws/`, `lobby/`, `match_task/`, `botclient/`) to stay under 200 lines.
  4. RTT samples also flow to the match task (`MatchCmd::Rtt`) so each match log carries per-seat RTT.
  5. Client: `?autoplay=quick|practice` starts a mode automatically; reconnecting while queued
     re-sends the mode (the server drops queued players on disconnect).
  6. Two tabs of the same browser origin share one session (token in `localStorage`); a second
     tab takes over the first. Correct for one player; testers must use different devices or origins.
  7. `wss://` through the bot tool is untested until P5 (no TLS endpoint locally).
- Blocked items: none.
- Needs the owner: L7/L9 on two real devices (desktop + phone on the same Wi-Fi), phone Wi-Fi off 10 s and 40 s.
