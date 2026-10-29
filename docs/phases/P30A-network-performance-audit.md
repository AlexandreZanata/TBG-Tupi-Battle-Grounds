# P30-A — Network, view, and scalability audit

**Depends on:** P10, P14, P16-B and P16-C reviewed; starts from `main` at
`fb47717` or later (open army, balance lab with `headless`/`replay`,
Arcanas, protocol v8). P30-B must not start before this phase's report is
reviewed.
**Branch:** `phase/p30a-network-performance-audit`.
**Difficulty:** high. Measurement-first: this phase produces evidence, a
decision, and a handoff. It does not ship a production gameplay or wire
optimization.
**Scope:** `tbg-core` examples, `tbg-server` instrumentation, `tbg-client`
measurement, `scripts/`, and docs. No game-rule, tick-rate, shop, Energy,
targeting, evolution, or outcome change.

## Goal

Answer, with reproducible measurements:

1. What is the real multiplayer bottleneck today?
2. How many bytes per view are redundant?
3. Which fields are sent unnecessarily?
4. Can the server send only changed data without much complexity?
5. Is a delta protocol actually better than a simpler split?
6. Is there a simpler hybrid with most of the gain?
7. How much traffic can be removed?
8. What does that cost in CPU and RAM?
9. Does the client stay exactly synchronized?
10. Is reconnect still safe?
11. Does command latency stay the same or improve?
12. How much capacity per vCPU can be gained?
13. Which optimizations are worth doing now?
14. Which must be deferred?
15. What is the smallest implementation sequence?

The output is a decision, not a framework. A well-supported "keep the
current wire format and fix only the measured hot spots" is a valid and
preferred result when it meets the thresholds below.

## Boundaries

- **P30** owns visual quality, device tiers, load time, and the final
  client device matrix. P30-A measures only the client cost needed to
  decide the view format; it does not tune art or rendering.
- **P30-B** owns the final server capacity table and the kept server
  optimizations. P30-A establishes the limited baseline and the decision.
- **This phase** owns the protocol/view comparison and the integrity
  evidence. It must not become a second P30 or P30-B.

## Deferred by default

Do not add or prototype these in P30-A or P30-B without a measured trigger
and an explicit note in the report:

- Redis, Kafka, NATS, RabbitMQ, a database queue, or an external event bus.
- Microservices, Kubernetes, a separate matchmaker, or a second service.
- A distributed database or cross-server shared state.
- WebRTC, a new transport, or a generic replication/entity framework.
- Spectator-specific replication, prediction, rollback, or lag compensation.
- Client-side damage, Energy, targeting, or outcome calculation.
- Compression or a new WebSocket library solely to save bytes now.
- Delta views if the simpler static/dynamic split clears the threshold.
- Native clients, multi-region routing, or a CDN for game traffic.

The existing single VPS, one server binary, one match task per match, and
one WebSocket per client remain the baseline. The decision must preserve
that shape unless the measurements prove it is the first limit and the
owner approves an architectural change.

## Current data path (verify before measuring)

Verified on `main` `fb47717` (planning review, 2026-10-29); update this map
if the code moved.

| Step | Where | What it does |
|------|-------|--------------|
| Tick | `crates/tbg-server/src/match_task.rs` (`run`) | 20 Hz `tokio::time::interval`; drains `MatchCmd`, calls `state.step()`, then sends views every second tick (10 Hz), plus an ack view for a seat whose intention was processed. |
| Events | `crates/tbg-core/src/state.rs` (`Event`), `match_task.rs` | Each tick's events are cloned into both seats' `pending` buffers, then copied into the view (`events.to_vec()`). |
| View | `crates/tbg-core/src/view.rs` (`view_for`) | Builds a full `MatchView` per seat: header, both Stronghold HPs, all living units, both synergy arrays, own Energy/offers/last_seq, and all pending events. |
| Send | `crates/tbg-server/src/match_task/views.rs` | `send_view` takes the seat's events, builds the view, and `try_send`s `ConnEvent::Send(ServerMsg)`. A full connection queue detaches the seat (slow client). |
| Encode | `crates/tbg-core/src/protocol.rs` (`encode`), `crates/tbg-server/src/ws/client_msg.rs` (`send`) | Every message is encoded with `postcard::to_allocvec` into a new `Vec` immediately before `WebSocket::send`. |
| Write | `crates/tbg-server/src/ws.rs` | Axum WebSocket with 4 KiB write buffer, 64 KiB maximum pending bytes; one task per connection; bounded `CONN_QUEUE` (32). |
| Reconnect | `crates/tbg-server/src/lobby/sessions.rs`, `match_task/command.rs` (`Attach`) | Reattach sends `MatchStart` and a full `View` with all buffered events; this is the authoritative recovery path. |
| Client receive | `crates/tbg-client/src/net/socket.rs`, `sync/messages.rs` | Decode each binary frame; ignored decode errors; copies events into `ViewBuffer.events`; keeps `LatestView` and the last two views. |
| Client render | `crates/tbg-client/src/sync.rs` (`sync_units`) | Clones the whole `MatchView` every frame, linearly searches `prev.units` per unit, and linearly checks every entity in `retain`. |
| Limits | `ws.rs`, `chan.rs`, `lobby.rs` | 1 KiB inbound, 20 msgs/s burst 40, 8 sockets/IP, 64 KiB max WS write, `CONN_QUEUE=32`, `MATCH_QUEUE=256`, `LOBBY_QUEUE=1024`, 15 s idle, 5 s handshake, 30 s reconnect window. |
| Keep-alive | `ws.rs` | A WebSocket `Ping` every 1 s and an app `Ping` every 5 s (answered by `NetStats`): about 1.4 extra server messages per second per connection, on top of 10 views/s and ack views. |
| Replay | `crates/tbg-core/src/replay.rs`, server `Config::replays` | Records every applied intention per tick; `Replay::run` rebuilds the exact state (contract C4). Use it to check that a reconstructed client view equals the server's view on recorded online matches. |
| Production path | `tbg.abastevo.com.br` (local deploy notes) | Cloudflare → nginx (TLS terminates there) → `tbg-server` on loopback, in a k3s namespace with a 2 CPU / 1 GiB quota. TLS cost is nginx's, not `tbg-server`'s. |

The current wire is **full snapshots**. `MatchView` carries both players'
units every view; static unit fields (`seat`, `kind`, `tier`, `branch`)
are repeated even though they change only on spawn/evolution. The client
already receives `Spawn` and `Evolved` events, but uses them only for
presentation, not for state reconstruction.

### Planning measurement (main `fb47717`, reviewer probe, not committed)

300 bot-vs-bot matches (default Arcanas, release), every view encoded with
postcard exactly as the server would, events accumulated per seat between
views. Natural armies only; synthetic large armies are step 2's job.

| Metric | Bot vs bot | Seat A heavy spam |
|---|---|---|
| View size avg / p95 / max | 137 / 205 / 400 B | 150 / 255 / 608 B |
| Payload per player at 10 Hz | 1.37 KB/s | 1.50 KB/s |
| Header (all non-unit, non-event fields) | 22.7 % | 20.8 % |
| Units | 73.5 % (12.5 B per unit, 8.1 units per view) | 75.6 % |
| — of which static `seat, kind, tier, branch` | 23.6 % of all bytes | 24.2 % |
| — of which `max_hp` | 11.8 % of all bytes | 12.1 % |
| Events | 3.7 % (1.2 per view) | 3.6 % |

Header detail (share of all bytes): `synergy` 5.8 % (8 B), `offers` 3.6 %,
`stronghold_hp` 2.9 %, `phase` + `outcome` + two lengths 2.9 %, `tick`
1.4 %, `phase_ticks_left` 1.4 %, `energy` 1.4 %, `last_seq` 1.4 %,
`next_rotation_in` 1.0 %, `you` 0.7 %.

**`max_hp` is derivable.** For 5.9 million live-unit samples,
`state.stats_of(u).max_hp == u.max_hp` held without exception: the client
already has `tbg-core`, the Synergy levels, and both Arcana presets (from
`MatchStart`), so it can compute every unit's max HP exactly.
`phase_ticks_left` and `next_rotation_in` are pure functions of `tick` and
`phase` (`sim::phase_ticks_left`, `shop::next_rotation_in`), and `you` is
fixed by `MatchStart`. Removing these four fields costs no state on either
side and saves about **15 %** of payload.

**Per-message overhead is as large as the savings.** At 137 B of payload a
message also costs roughly 2–4 B of WebSocket framing, ~22 B of TLS record
(at nginx in production), and 40–52 B of TCP/IP headers. With `TCP_NODELAY`
(P9) each view is its own packet, so wire bytes are about 1.5× payload and
CPU per message (syscall, frame, record) does not shrink with the payload.
Payload savings must therefore be reported both as payload and as estimated
wire bytes, and the message count is a first-class metric.

Prior evidence to verify, not to trust blindly: P10 measured about
300 KB RSS per running match, 1.3 KB/s per player at 250 matches, and
`BytesMut::resize` at 42.9% of profiled CPU; P16-B measured 3.0 KB/s per
player at 10 units/side, 5.7 at 20, 11.4 at 40, 17.1 at 60, and 28.6 at
100, with 2 857 B average and 5 492 B maximum view at 100. Do not repeat
P10's rejected changes (mimalloc, `opt-level=3` server profile, encode-once
with `BytesMut`) without a new hypothesis and a fresh measurement.

## Hypotheses

| ID | Hypothesis | Test | Accept if |
|----|------------|------|-----------|
| H1 | Static unit fields are a large share of view bytes; events are small at typical armies (planning probe: static 23.6 % of all bytes ≈ 32 % of unit payload, events 3.7 %). | Field-by-field size breakdown of captured views at 10/20/40/60/100 units/side, synthetic and natural. | Static fields are ≥30% of unit payload at 20+ units/side; events are <20% of the view. Report whether events grow faster than units in large battles. |
| H2 | A static/dynamic split removes that share without per-tick diffing. | Prototype in `view_audit.rs`: static roster/updates plus a complete dynamic list every view; reconstruct every view. | ≥20% fewer bytes at 20 units/side or ≥30% at 100, exact reconstruction, CPU increase ≤10%. |
| H3 | A per-view delta removes more bytes but costs more CPU, memory, and recovery complexity. | Prototype with per-seat previous state, changed-unit list, spawn/death handling, periodic keyframe, reconstruction. | Only if it beats H2 by ≥15 percentage points at 100 units/side *and* stays within the CPU/memory/latency budgets. |
| H4 | 10 Hz is the right view rate; lowering it is not a free win. | Compute bytes at 5/8/10 Hz; temporarily run a prototype at 8 and 5 Hz with bots; compare ack, view gap, and bandwidth. | Keep 10 Hz unless a lower rate shows no ack/gap regression and a clear traffic win; never reduce the 20 Hz simulation. |
| H5 | Ack views are a small share of traffic at human and bot rates. | Count regular vs ack views and bytes per seat. | Compact ack only if ack views are >10% of view bytes at realistic input rates. |
| H6 | Message size drives the WebSocket write/allocation cost, so reducing bytes also reduces server CPU. | Profile full vs split/delta at the same match count; compare `BytesMut::resize`, allocations, and CPU per match. | Keep a wire change only if it reduces CPU per match or raises matches/core by ≥10% at the same late-tick/RTT quality. |
| H7 | The client's per-frame full-view clone and O(n²) lookups are measurable at 100 units/side. | Chrome performance capture with the debug army; compare current vs an `Arc` + sorted-merge POC. | Fix in P30/P30-B only if frame time drops ≥15% at 100 units or ≥5 FPS on a mid-range phone, with no visual change. |
| H8 | "Derive, don't send" removes ~15 % with no state: `max_hp`, `phase_ticks_left`, `next_rotation_in`, `you` (planning probe). | Prototype A (step 3) plus an invariant test over 100 seeds and recorded online replays. | Exact client values on every view; ≥10 % fewer payload bytes at natural armies. |
| H9 | Message count, not payload, dominates per-player server CPU at natural army sizes. | Count server messages per player per second by type (view, ack view, WS ping, app ping, `NetStats`, other) and profile CPU per message vs per byte at a fixed match count. | If ≥50 % of send CPU is per-message, prefer reducing messages (e.g. fold the 1 s WS ping into traffic that already flows, coalesce an ack view with a regular view due the same tick) over shaving bytes. Never delay an ack (P9). |
| H10 | Both seats' views repeat the same units and events; building that part once per tick saves CPU and allocations. | Measure `view_for` build time and allocations per tick; prototype one shared unit list per tick (both seats read it) without changing the wire. | Keep only if build+encode CPU per tick drops ≥15 %. This is not P10's rejected encode-once; it changes what is built, not the encoder. |

## Decision thresholds

Set these after the first baseline and record any change with its reason.
Do not impose a headline percentage; the threshold is the smallest gain
that pays for the added code and risk.

| Area | Keep a change only if |
|------|-----------------------|
| Traffic | ≥20% fewer bytes per player at 20 units/side (typical peak) **or** ≥30% at 100 units/side, with no competitive information lost. |
| Server CPU | Build+encode cost grows ≤10% at 20 units/side; matches/core at the same late-tick and RTT quality does not fall. |
| Server RAM | Per-match RSS grows ≤10% for a delta, or falls for a split; no unbounded per-seat history. |
| Latency | Ack p95 ≤ baseline + 5 ms and ≤ RTT p95 + 60 ms; view-gap p95 ≤150 ms at 40 ± 20 ms added latency; no late-tick regression. |
| Integrity | 100 seeds plus spawn/death/summon/evolution/status edge cases reconstruct exactly; 50 chaos matches with reconnect finish with zero desyncs. |
| Complexity | No new dependency; no generic replication framework; no second simulation in the client. A delta must beat the simpler split by ≥15 percentage points to justify its state machine. |
| Capacity | ≥10% more matches/core, or ≥20% less RSS per match, at the same late-tick/RTT quality. |

If no candidate clears the threshold, the report recommends **no wire
change** and lists the measurements that would reopen the decision.

## Steps

### 1. Baseline and harness

- Record commit, `rustc --version`, `lscpu`, `free -h`, and `uname -a`.
- Build release and reproduce the P16-B `army_load` table on the current
  code. If it differs, record the delta and use the current numbers.
- Add `crates/tbg-core/examples/view_audit.rs` (std + `tbg-core` only):
  - deterministic synthetic armies at 10/20/40/60/100 units/side;
  - a natural corpus of 100 bot-vs-bot matches (same seeds and tick order
    as `headless::play`), recording units, events, encoded view size,
    and per-field contribution;
  - prototypes for (A) trimmed full view, (B) per-view delta,
    (C) static/dynamic split, and (D) events-only as a negative control;
  - reconstruct each prototype into the canonical view every tick and
    assert exact equality for every field the prototype carries;
  - print Markdown and JSON to stdout; the executor redirects raw output
    to `docs/media/p30a/`.
- Add per-match counters to `SeatStats` (`views`, `ack_views`, `events`,
  `units`) and include them in the `match_end` JSON line. Add global
  `view_messages` / `view_bytes` counters in `ws/client_msg.rs::send`
  (encoding already happens there; never encode twice). These are
  observability, not gameplay; no per-unit hot-path atomics.
- Extend `scripts/capacity.sh` with `LIMIT_CPU`, `LIMIT_MEM`, `TBG_WORKERS`,
  and sampling of `bytes_out`, `bytes_in`, `connections`, and
  `matches_running` deltas. Use `systemd-run --user --scope` when available;
  otherwise use `taskset` for CPU and document the less precise memory
  fallback. Bot processes stay outside the server's limit.
- **Done when:** the baseline table and raw files exist, and
  `cargo run --release -p tbg-core --example view_audit` runs from a clean
  checkout.

### 2. Traffic composition

Run the harness for synthetic and natural matches. Report, per unit count:

- average, p95, and maximum encoded view bytes;
- bytes/s per player at 10 Hz;
- bytes from static fields, dynamic fields, header, and events;
- events per view, units per view, and the share of ack views.

Separate synthetic topped-up armies from natural bot matches. Do not
present synthetic numbers as expected player traffic.

### 3. Prototype A — derive, don't send (stateless)

Remove every field the client can compute exactly with the `tbg-core` it
already runs, keeping full snapshots and no per-seat memory:

- `UnitView.max_hp` → a core helper (no `MatchState` needed) that computes a
  unit's max HP from `kind`, `tier`, `branch`, the seat's Synergy level,
  and the seat's frozen Arcana preset (and, once P17-A lands, nothing else:
  Strongholds are not units). Add a core contract test asserting the
  invariant `stats_of(u).max_hp == u.max_hp` for every live unit after
  every step on 100 seeds, with Arcanas.
- `phase_ticks_left`, `next_rotation_in` → the existing core functions.
- `you` → from `MatchStart`.
- Tighter integers only where the range is proven and nothing is lost:
  for example `x` is 0–16 000 milli-tiles, so a `u16` varint takes 2 bytes
  where today's zigzag `i32` takes 3. Measure the gain before proposing.

Expected from the planning probe: about 15 % fewer payload bytes. This is
the preferred first wire change if it clears H8; C builds on it.

### 4. Prototype C — static/dynamic split (preferred candidate)

Prototype:

- `UnitStatic { id, seat, kind, tier, branch }`: sent for all living units
  in a full snapshot (reconnect/first sync) and only for spawned/evolved
  units in normal views.
- `UnitDynamic { id, x, hp, status }`: sent for **every** living unit in
  every view. `max_hp` is derived on the client (Prototype A), not sent.
- Low-frequency state moves to the static section too: both seats'
  Synergy levels (already announced by `SynergyChanged` events) and, when
  P17-A lands, both `StrongholdView`s (kind, tier, branch, max HP; already
  announced by `StrongholdEvolved`). A full view carries them all.
- `MatchView` gains `full: bool` and `static_units: Vec<UnitStatic>`;
  `units` becomes `Vec<UnitDynamic>`.
- On reconnect, `Attach` sends `full = true` with the complete live roster.
- The client keeps a `HashMap<UnitId, UnitStatic>`, replaces it on a full
  view, upserts `static_units`, reconnects if a dynamic id has no static
  entry, then prunes static ids not present in the dynamic list.
- Decode errors must no longer be ignored: they trigger the existing
  reconnect, which yields a full snapshot.

Measure bytes, build+encode CPU, client reconstruction, and the cost of
the extra static lists. This design keeps a complete dynamic state every
view, so a single lost view cannot leave a stale position or HP; only the
static map needs recovery.

### 5. Prototype B — delta views

Implement a per-seat previous-view cache and send only changed units,
spawns, deaths, and events, with a periodic full keyframe. Measure:

- diff CPU and memory (per-seat previous state, tombstones, keyframe);
- bytes at 10/20/40/60/100 units/side;
- reconstruction after spawn, death, summon expiry, evolution, status,
  synergy HP change, and reconnect;
- behavior when a keyframe or a delta is missing (the transport is ordered
  and reliable, but a decode failure must recover, not desync).

Reject it if it does not clear the threshold against C, or if the state
machine and recovery tests are disproportionate to the gain. Do not build
retransmission: WebSocket already orders and delivers, and a full keyframe
plus reconnect is sufficient.

### 6. Events, frequency, compression, send path

- **Events (D):** quantify the share. Confirm that `Spawn`, `AttackStart`,
  `Hit`, `Heal`, `Death`, `Evolved`, `StatusApplied`, and
  `SynergyChanged` remain presentation events; never move damage,
  targeting, Energy, or outcomes into the client.
- **Frequency (E):** compute the bytes at 5/8/10 Hz. Run a temporary
  view-interval prototype with bots at 10, 8, and 5 Hz; record ack p95,
  view gap p95, and completion. Revert the prototype. Keep 10 Hz unless
  the lower rate is clearly indistinguishable and the traffic win matters.
- **Compression (F):** capture real encoded views and compare `gzip -6`
  and `zstd -3` sizes on the files. `ewebsock 0.8` exposes no
  permessage-deflate option; adding a compression layer or replacing the
  transport is a large change. Reject unless compressed payload is ≥30%
  smaller and a transport path exists without a dependency or latency
  regression. Do not add a compression crate in this phase.
- **Send path (G):** profile the current server at the highest clean load
  with `perf` or the P10 pprof method. Do not repeat mimalloc, `opt-level=3`,
  or encode-once without a new hypothesis. Check whether smaller messages
  reduce `BytesMut::resize`, allocations, and write-buffer growth; report
  `BytesMut`/allocation shares before and after the chosen encoding.
- **Client (H):** add a debug-only `?army=N` parameter to local practice
  (inert without the parameter) so a 100-unit match can be rendered without
  the server. Use Chrome DevTools performance capture with CPU throttling
  (desktop and a mid-range phone if available). Measure FPS, frame-time p95,
  JS heap, and the cost of the per-frame `MatchView` clone and linear
  lookups. Prototype an `Arc<MatchView>` plus sorted-merge/HashSet fix and
  measure; keep the prototype only if it clears H7.

### 7. Integrity, reconnect, and security

Run the following against the chosen prototype (and the current code as a
control):

- 100 deterministic seeds; every view reconstructs the canonical state.
- Edge cases: no units; one unit; 200+ units; simultaneous spawn and death
  between views; summon expiry; evolution; status; Synergy max-HP change;
  match end with pending events.
- Reconnect mid-combat: `MatchStart` + full snapshot + buffered events;
  no duplicate or missing units; ack resumes.
- Duplicate/old token, second socket, slow client (full queue), decode
  error, oversized frame, rate limit. Existing tests stay green.
- Hidden data: opponent Energy/offers/RNG never appear in any encoding.
- Confirm that a client with an unknown static id reconnects rather than
  rendering a partial state.

### 8. Capacity under limits

First read the real traffic of the test server, read-only: `/metrics` is
not public, so the owner (or a reviewer with VPS access) runs
`curl -s http://127.0.0.1:18260/metrics` on the VPS twice, 10 minutes
apart, and records `bytes_out`, `connections`, and `matches_running` (no
player data). Then run the limited server at **2 CPU / 1 GiB** (the actual
k3s quota of the test deployment), 1 vCPU / 1 GB, and 2 vCPU / 8 GB. The bots run
outside the server cgroup. Use natural bot matches, then a separate
synthetic large-army microbenchmark; do not mix the two.

Levels:

- 1 vCPU / 1 GB: 100, 250, 500, 750, 1000 matches; stop earlier if late
  ticks appear or the load generator saturates.
- 2 CPU / 1 GiB (the deployed quota): 250, 500, 1000, 1500 matches; same
  stop rule; this is the row the owner's capacity question needs.
- 2 vCPU / 8 GB: 500, 1000, 1500, 2000, 2500 matches; same stop rule.
- Test `TBG_WORKERS=1` and `2` for the 2 vCPU case; record the better
  setting and its late-tick behavior.

For each level record: matches, players, server cores, matches/core, RSS,
KB/match, late ticks, RTT p95, ack p95 (from a full bot run), view gap p95,
KB/s per player, server KB/s, first limit, and `lobby_inbox_max`.

These are local cgroup estimates on the development machine, not a VPS
guarantee. The report must label them as such and list the VPS tests still
pending.

### 9. Decision and handoff

Fill the comparison matrix and choose one outcome:

Prefer, in this order, the first option that clears the thresholds:

- **No wire change:** current full snapshots meet the thresholds; fix only
  measured server/client hot spots (H9, H10, H7).
- **Derive, don't send (A):** stateless; the expected default.
- **A + static/dynamic split (C):** if A alone misses the traffic threshold.
- **Delta (B):** only if it beats A + C by the published margin and passes
  the recovery tests.

Coordinate the protocol number: P39 (quick challenge, protocol v8 on its
branch) and P17-A (Stronghold views) also change the wire. P30-B takes the
next unused `PROTOCOL_VERSION` after whichever of them is on `main`, and
its view design includes `StrongholdView` in the static section if P17-A
is merged.

Then write the exact P30-B handoff: files, protocol version, tests,
rollback, and the metrics to re-measure. Revert every prototype that is
not chosen. Keep only the measurement harness, counters, and debug
parameter that are inert in production. Update `docs/ROADMAP.md` status and
the P30-B dependency/step 0 if the decision requires it.

### 10. Verification and report

Run:

```bash
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
cargo check -p tbg-client -p tbg-core --target wasm32-unknown-unknown
shellcheck scripts/capacity.sh
```

Run `trunk build --release` in `crates/tbg-client/` if the client changed.
Fill the Phase report below with real command output, tables, and raw file
paths. Update `docs/ARCHITECTURE.md` only if a measured budget or the
chosen architecture changes; do not duplicate the report there.

## Starting commands

Adapt paths and levels to the machine; record the exact commands used.

```bash
# Current baseline
cargo run --release -p tbg-core --example army_load
cargo run --release -p tbg-core --example view_audit > docs/media/p30a/composition.txt

# Server tests and capacity (after the harness exists)
cargo test -p tbg-server --test online
PROFILE=release LIMIT_CPU=100% LIMIT_MEM=1G \
  LEVELS="100 250 500 750 1000" WINDOW=40 bash scripts/capacity.sh
PROFILE=release LIMIT_CPU=200% LIMIT_MEM=8G \
  LEVELS="500 1000 1500 2000 2500" WINDOW=40 bash scripts/capacity.sh

# Full-run ack/gap reference at a fixed level (server already running on 8181)
cargo run --release -p tbg-server --bin tbg-bots -- \
  --url ws://127.0.0.1:8181/ws --matches 250 --ramp-ms 5

# Client (100-unit practice; requires the debug army parameter).
# Port 8080 is often taken by another agent's server on the dev PC: pick a
# free port and point the client at your server with ?server=.
cd crates/tbg-client && trunk build --release && trunk serve --release --port 18600
# open http://127.0.0.1:18600/?autoplay=practice&army=100&measure=1
```

## Comparison matrix (fill after measurement)

| Criterion | Current | A: derive, don't send | B: delta | C: A + static/dynamic | D: events only | E: lower Hz | F: compression |
|-----------|---------|-----------------|----------|-------------------|----------------|-------------|----------------|
| Bytes/s per player at 20 units | measured | measured | measured | measured | n/a for state | measured/derived | measured/derived |
| Bytes/s per player at 100 units | measured | measured | measured | measured | n/a | measured/derived | measured/derived |
| Estimated wire bytes/s per player (payload + WS + TLS + TCP/IP) | measured | measured | measured | measured | n/a | measured/derived | measured/derived |
| Server messages/s per player | measured | measured | measured | measured | n/a | measured | measured |
| Build+encode CPU per view | measured | measured | measured | measured | measured | n/a | measured |
| Extra RAM per match/seat | measured | measured | measured | measured | — | — | — |
| Client reconstruction cost | measured | measured | measured | measured | n/a | — | decode cost |
| Complexity (files/state machine) | — | low | high | medium | low | low | medium |
| Desync/recovery risk | low | low | medium | low | high | low | low |
| Reconnect impact | full view | full view | keyframe required | full view | none | full view | full view |
| Debuggability | high | high | low | high | low | high | medium |
| Decision | — | — | — | — | — | — | — |

Do not fill a cell with a guess. Use `not measured` and explain why.

## Acceptance checklist

- [ ] Baseline reproduced on the audit commit; hardware and build recorded
- [ ] `view_audit.rs` runs from a clean checkout; raw output is committed
- [ ] Synthetic and natural matches are reported separately
- [ ] Every alternative that was tested has bytes, CPU, RAM, and integrity data
- [ ] The chosen option clears the published threshold; rejected options say why
- [ ] No game rule, tick rate, or production wire format changed in this phase
- [ ] Reconnect, slow-client, rate-limit, seat-ownership, and hidden-data tests pass
- [ ] The client stays exactly synchronized on 100 seeds and chaos reconnects
- [ ] P30-B has an exact handoff or an explicit "no wire change" decision
- [ ] Every non-chosen prototype is reverted; no dead dependency
- [ ] All verification commands pass
- [ ] Phase report and `docs/media/p30a/` raw files committed

## Risks and mitigations

| Risk | Mitigation |
|------|------------|
| The load generator saturates before the server | Split bots across processes or machines; record the first saturated side; do not report a server limit. |
| `perf`, cgroups, or systemd unavailable | Use `cargo flamegraph`, `/proc` sampling, `taskset`, and `prlimit`; mark the limitation instead of inventing numbers. |
| Development PC is not the VPS | Label all capacity results as local cgroup estimates; list the pending VPS matrix. |
| 100-unit browser test needs art-heavy units | Use the existing placeholder/blockout figures; report that the FPS includes the current renderer, not final art. |
| A delta prototype becomes a framework | Stop; the simpler static/dynamic split is the default. Reject any design with a second simulation, retransmission, or unbounded history. |
| A view grows past the 64 KiB WS write cap | Report the unit count and size; do not add a gameplay cap; note the limit in the decision. |
| Measurement noise hides a small win | Warm up, repeat at least 5 times, report median and p95, and use the same seeds/sequence. |

## Reviewer checklist

- Re-run one baseline level and one capacity level; numbers match the report.
- Confirm every POC that is not chosen is absent from the diff.
- Confirm no private data, no new dependency, and no rule/tick change.
- Confirm the P30-B handoff names exact files, tests, and rollback.
- Confirm the report answers the 15 questions with evidence, not prose alone.
- Confirm the raw outputs in `docs/media/p30a/` support every table.

## Phase report

### A. Current diagnosis

- Data flow: tick → events → per-seat full view → postcard → Axum WS →
  browser decode → view buffer → render.
- Measured bottlenecks, with profile/counter evidence.
- What is already optimized and must not be re-litigated.
- Opportunities, risks, and limits of the measurements.

### B. Alternative comparison

For each tested alternative (A–H): how it works, benefits, costs,
complexity, measured results, risks, and accept/reject reason. Fill the
matrix above; do not leave a measured cell empty.

### C. Benchmarks

| Metric | Current | A: derive, don't send | B: delta | C: A + static/dynamic |
|--------|---------|-----------------|----------|-------------------|
| KB/s per player at 20 units | measured | measured | measured | measured |
| KB/s per player at 100 units | measured | measured | measured | measured |
| Build+encode CPU per view | measured | measured | measured | measured |
| Server CPU per match at fixed load | measured | measured | measured | measured |
| RSS per match / per connection | measured | measured | measured | measured |
| Ack p95 / RTT p95 | measured | measured | measured | measured |
| Integrity on 100 seeds | verified | verified | verified | verified |
| Reconnect mid-combat | verified | verified | verified | verified |

State the hardware, build profile, seeds, repetitions, warm-up, and
whether each number is payload, WebSocket frame, or wire bytes. Label
synthetic and natural matches separately. Do not fill an unmeasured cell.

### D. Chosen architecture

- Which option won and why.
- What is reused unchanged.
- What changes (files, types, protocol version).
- How reconnection gets a full snapshot.
- How events remain presentation-only and ordered.
- How the client detects and recovers from a missing/invalid reference.
- How the design can grow to more units, effects, Strongholds, Arcanas,
  Augments, and spectators without a rewrite.

### E. Minimum implementation plan

- Exact files and type/message changes, in order.
- Tests: determinism, view integrity, reconnect, slow client, rate limit,
  seat ownership, hidden data, size ceiling.
- Acceptance criteria and the commands that prove them.
- Risks and rollback (protocol version, redeploy, no data migration).
- What P30-B implements versus what P30 or a later phase implements.
- VPS validation procedure and the values still unknown.

### F. Capacity estimate

Separate **measured**, **extrapolated**, and **unknown**. Include:

- matches and players per vCPU at the measured late-tick/RTT quality;
- MB RSS per match and per connection;
- KB/s per player and per match at 10/20/100 units/side;
- monthly traffic formula and an occupancy table, with the plan's actual
  traffic allowance marked as a value the owner must confirm;
- the first limit (CPU, RAM, bandwidth, file descriptors, or load
  generator) and the safety margin;
- the difference between local cgroup results and a VPS.

Formula for projections (decimal GB; state the assumed occupancy and
hours per month):

`monthly_GB = matches × 2 players × KB_per_s_player × seconds_per_month / 1_000_000`

### G. Decisions deferred

List explicitly what is not being built now and the measured trigger that
would reopen each item. At minimum: delta if static/dynamic won,
compression, spectator replication, multi-server matchmaking, external
queues, client prediction, and transport changes.

### Answers to the 15 questions

1. Real bottleneck:
2. Wasted bytes per update:
3. Unnecessary fields:
4. Changed-data-only feasibility:
5. Delta versus simpler split:
6. Simpler hybrid:
7. Achievable traffic reduction:
8. CPU/RAM cost:
9. Client synchronization:
10. Reconnect safety:
11. Command latency:
12. Capacity gain per vCPU:
13. Worth doing now:
14. Deferred:
15. Implementation sequence:

### Deviations, blocked items, and pending VPS tests

_Executor fills this in with real command output._
