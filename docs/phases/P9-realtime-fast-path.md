# P9 — Real-time fast path and reconnection

**Depends on:** P8 merged; the temporary real-network run (P5 report,
"Temporary environment run") read.
**Branch:** `phase/p9-realtime-fast-path`.
**Difficulty:** medium. Measure before and after every change; keep a
change only when the numbers say it helps.

## Goal

Make every tap feel instant and make a dropped player come back to the
same match without thinking about it — on desktop browsers, phone
browsers, and after a page reload. Nothing in the rules changes.

## Deliverable

- Server: `TCP_NODELAY` on every socket; an immediate "acknowledgement
  view" to the seat whose intention was applied; protocol-level WebSocket
  pings that keep a hidden tab's socket alive; `queue_wait` metrics.
- Client: automatic resume on page load; reconnect with backoff and a
  visible countdown; tap-to-acknowledgement latency on the overlay.
- `tbg-bots`: reports `ack_ms` and `queue_wait_ms` percentiles.
- A before/after table in the report.

## Read first (only these)

- `AGENTS.md`
- `docs/ARCHITECTURE.md` §5 (networking) and §10 (budgets)
- `docs/phases/P5-real-environment.md` → "Temporary environment run" (findings)
- Server: `src/lib.rs`, `src/ws.rs`, `src/match_task.rs`, `src/metrics.rs`, `src/botclient.rs`
- Client: `src/net.rs`, `src/link.rs`, `src/sync/messages.rs`, `src/overlay.rs`, `src/config.rs`

## Do NOT

- Change `tbg-core` rules, `GAME-DESIGN.md` values, or the 20 Hz tick / 10 Hz
  regular view rate.
- Add lockstep, rollback, client-side simulation of online matches, or a
  second transport (WebRTC/WebTransport).
- Raise the reconnect window (30 s, BR-M03).

## Dependencies you may add

None. `web-sys` may gain the feature `Document` (client, wasm only).

## Definitions (use exactly)

- **ack latency** (`ack_ms`): time from the client sending `Intent{seq}` to
  receiving the first `View` whose `last_seq >= seq` (or `Rejected{seq}`).
- **queue wait** (`queue_wait_ms`): time from the lobby receiving
  `QuickMatch` to it sending `MatchStart` for that session.

## Steps

### 1. Baseline
- In `botclient.rs`, record `ack_ms` per intention (store send time by
  `seq`; resolve on the first matching `View`/`Rejected`) and
  `queue_wait_ms` (from sending `QuickMatch` to `MatchStart`). Add both
  percentiles (p50/p95/max) to the human summary and the JSON line of
  `tbg-bots`.
- Run against a local release server and save the output:
  ```bash
  cargo build --release -p tbg-server --bins
  ./target/release/tbg-server &   # TBG_ADDR default
  ./target/release/tbg-bots --matches 50 --delay-ms 40 --jitter-ms 20
  ./target/release/tbg-bots --matches 50
  ```
- **Done when** the report has a "Before" table: RTT p50/p95, ack p50/p95,
  view gap p95, queue wait p95 — for both runs.

### 2. `TCP_NODELAY` everywhere
- Server (`lib.rs`): wrap the listener so every accepted stream gets
  `set_nodelay(true)` — axum 0.8 provides `axum::serve::ListenerExt::tap_io`
  (see `axum-0.8.9/src/serve/listener.rs` in the cargo registry for the
  exact example). Log a warning if `set_nodelay` fails; never panic.
- Bot client: connect with `tokio_tungstenite::connect_async_with_config(url,
  None, true)` (the third argument disables Nagle).
- **Check:** re-run both step 1 commands. **Done when** the numbers are in
  the report (expect ack p95 to drop when there is no added delay).

### 3. Acknowledgement view
In `match_task.rs`:
- Replace the single `events: Vec<Event>` with `pending: [Vec<Event>; 2]`
  (index = `Seat::index()`): every tick appends the step's events to both.
- Keep `applied: [bool; 2]`: set when a tick applied (or rejected) at least
  one intention of that seat.
- After a tick: for each seat, send a view when `tick % 2 == 0`, or the
  match ended, **or** `applied[seat]`. Sending a view to a seat uses
  `view_for(seat, &pending[seat])` and then clears only `pending[seat]`
  (events are never lost or duplicated per seat). Reset `applied`.
- Update ARCHITECTURE §5 "Rates" with one sentence about the extra view.
- `match_task.rs` is 226 lines today: split it below 200 (e.g. `match_task/views.rs`).
- **Check:** new integration test in `tests/online.rs`
  `intent_is_acknowledged_next_tick`: with `TBG_TICK_MS=50`, a client
  sends a valid `Purchase` and receives a `View` with `last_seq >= seq`
  within 2 ticks + 20 ms of local slack. Re-run step 1 commands.
- **Done when** ack p95 (no added delay) ≤ 60 ms locally and the test passes.

### 4. Keep hidden tabs connected
Browsers pause the frame loop of a hidden tab, so the app-level `Pong`
stops and the server idles the socket out after 15 s (seen in P5).
Browsers still answer **protocol-level** WebSocket pings automatically.
- `ws.rs` run loop: every 5 s also send `Message::Ping` (empty payload).
  Treat any received frame — `Binary`, `Ping`, or `Pong` — as activity that
  resets the idle timer. App-level `Ping{id}`/`Pong{id}` stays unchanged and
  is still the only RTT source.
- **Check:** integration test `protocol_pong_keeps_socket_alive`: a raw
  tokio-tungstenite client that answers protocol pings (default behavior)
  but sends no app messages for 20 s is still connected.
- **Done when** a desktop Chrome tab left hidden for 60 s mid-match is still
  in the match when shown again (server log: no `idle_timeout` for it).

### 5. Automatic resume on page load
- `config.rs` already keeps the session token per tab (`sessionStorage`).
- On startup, if a token exists and the URL has no `?room=`, create the
  `NetLink` immediately in a new `Mode::Probe` (send `Hello{resume}` and
  nothing else). The menu stays visible and usable.
- In `sync/messages.rs`, `MatchStart` already moves to `InMatch` from any
  state — keep that; a probe that receives `MatchStart` + `View` resumes
  the match with no tap.
- If nothing but `Welcome` arrives within 3 s, keep the socket (it is reused
  by the next Quick Match / room action) — never open a second socket.
- **Done when** reloading the page mid-match returns to the same match in
  ≤ 3 s without any tap (desktop and phone), and reloading from the menu
  does not start anything.

### 6. Reconnect with backoff and countdown
In `net.rs`:
- After a drop, retry at 0.25 s, 0.5 s, 1 s, then every 2 s, until 30 s
  after the drop (`GIVE_UP` stays 30 s). A frame that runs after the tab
  becomes visible again retries immediately if a retry is due.
- `NetStatus::Reconnecting { since: f64 }`: the HUD shows a centered banner
  `Reconnecting… 23 s` (seconds left in the window), updated every frame;
  on success the banner disappears and a toast says `Reconnected`.
- After `GIVE_UP`, show `Connection lost` with a **Back to menu** button.
- **Check:** devtools → Network → Offline for 10 s mid-match → banner counts
  down, match resumes; Offline for 40 s → forfeit result screen.
- **Done when** both behave as described on desktop Chrome and one phone.

### 7. Metrics and overlay
- `metrics.rs`: `queue_wait_ms` p50/p95 over the last 60 s in `/metrics`.
- `overlay.rs`: add `ack 48 ms` (p50 of the last 20 intentions) next to RTT.
- **Done when** both show sensible values in a local online match.

### 8. After table and docs
Re-run step 1 exactly. Add an "After" table next to "Before". Update
ARCHITECTURE §5 (Limits: protocol pings; Sessions: auto-resume) and §10 (new
row "ack latency p95 ≤ RTT p95 + 60 ms").

## Acceptance checklist

- [ ] All verification commands from `AGENTS.md` pass; `trunk build --release` passes
- [ ] `cargo test --workspace` includes the two new integration tests
- [ ] ack p95 ≤ RTT p95 + 60 ms in both step-1 runs
- [ ] Reload mid-match resumes with no tap; hidden tab 60 s survives; offline
      10 s resumes, 40 s forfeits
- [ ] Before/after table in the report

## Risks and what to do

| Problem | Action |
|---------|--------|
| Ack views raise bandwidth noticeably | They only go to the acting seat; if down-bandwidth per player rises > 20 %, send the ack view only for `Purchase`/`Evolve` that succeeded |
| Event duplication after step 3 | The per-seat `pending` buffers are the only event source; add a test that counts `Spawn` events per unit = 1 |
| A proxy strips protocol pings | Keep the app-level idle rule; record it under Deviations |

## Reviewer checklist

- Seat is still stamped by the connection; no client field decides anything.
- No unbounded buffer: `pending` is cleared on every view to that seat.
- Before/after numbers come from the same commands on the same machine.

## Phase report

_Executor fills this in: commands run with real output, before/after
tables, deviations, blocked items._
