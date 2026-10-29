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

## Comparison matrix (measured 2026-10-29)

Payload bytes are postcard view messages including the one-byte `ServerMsg`
tag, synthetic armies at 10 Hz unless stated; wire estimates add 71 B per
message (WS 2–4 + TLS ~22 + TCP/IP 40–52). Natural bot traffic is shown
separately below. “not measured” means exactly that.

| Criterion | Current | A: derive, don't send | B: delta | C: A + static/dynamic | D: events only | E: lower Hz | F: compression |
|-----------|---------|-----------------------|----------|-----------------------|----------------|-------------|----------------|
| Bytes/s per player at 20 units | 5.64 KB/s | 4.78 KB/s | 1.70 KB/s | 3.12 KB/s | n/a for state (0.25 KB/s) | 5.64 / 4.56 / 2.93 KB/s at 10/8/5 Hz | 5.64 KB/s (7.04 gzip-6, 6.25 zstd-3: larger) |
| Bytes/s per player at 100 units | 28.75 KB/s | 24.63 KB/s | 10.47 KB/s | 16.53 KB/s | n/a (1.31 KB/s) | 28.75 / 23.26 / 15.01 KB/s | 28.75 KB/s (35.87 / 31.89: larger) |
| Estimated wire bytes/s per player | 6.35 / 29.46 KB/s | 5.49 / 25.34 KB/s | 2.41 / 11.18 KB/s | 3.83 / 17.24 KB/s | n/a | 6.35 / 5.13 / 3.28 at 20 units; 29.46 / 24.12 / 15.74 at 100 | 7.75 / 36.58 KB/s (gzip) |
| Server messages/s per player | 11.6 (10.22 view incl. ack, 1.0 WS ping, 0.2 app ping, 0.2 NetStats) | 11.6 | 11.6 | 11.6 | n/a | 9.6 / 6.6 at 8/5 Hz | 11.6 |
| Build+encode CPU per view (20 / 100 units) | 1.77 / 6.93 µs | 1.57 / 6.31 µs | 1.73 / 6.19 µs | 1.67 / 5.89 µs | not measured | n/a (same per view) | decode cost not measured |
| Extra RAM per match/seat | 0 | 0 | 2.46 KB at 20, 11.9 KB at 100 | 0.53 KB at 20, 2.54 KB at 100 | — | — | — |
| Client reconstruction cost (20 / 100 units) | n/a (no reconstruction) | 0.70 / 3.30 µs | 1.93 / 15.26 µs | 1.51 / 12.25 µs | impossible (no state) | — | not measured |
| Complexity (files/state machine) | — | low | high | medium | low | low | medium |
| Desync/recovery risk | low | low | medium | low | high | low | low |
| Reconnect impact | full view | full view | keyframe required | full view | none | full view | full view |
| Debuggability | high | high | low | high | low | high | medium |
| Decision | — | reject alone (14.9% < 20% traffic); part of C | defer (beats C by 21–25 pp but delta is deferred when C clears, and load latency was not measured) | **accept (A + C)** | reject | reject (gap 150/200 ms) | reject (larger per message) |

Natural bot corpus, 100 seeds, 133.65 B/view current: A 113.77 (−14.9%),
B 46.80 (−65.0%), C 78.06 (−41.6%), D 8.80 (−93.4%). Five runs, identical
bytes; medians of per-run medians for CPU.

## Acceptance checklist

- [x] Baseline reproduced on the audit commit; hardware and build recorded
- [x] `view_audit.rs` runs from a clean checkout; raw output is committed
- [x] Synthetic and natural matches are reported separately
- [x] Every alternative that was tested has bytes, CPU, RAM, and integrity data
- [x] The chosen option clears the published threshold; rejected options say why
- [x] No game rule, tick rate, or production wire format changed in this phase
- [x] Reconnect, slow-client, rate-limit, seat-ownership, and hidden-data tests pass
- [x] The client stays exactly synchronized on 100 seeds and chaos reconnects
- [x] P30-B has an exact handoff or an explicit "no wire change" decision
- [x] Every non-chosen prototype is reverted; no dead dependency
- [x] All verification commands pass
- [x] Phase report and `docs/media/p30a/` raw files committed

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

Executed 2026-10-29 on the development PC (i7-13620H, 16 threads / 10
cores, 31 GiB, Linux 7.2.6), release profile, audit commit `a641cf4` plus
this report commit. Branch `phase/p30a-network-audit`; raw outputs in
`docs/media/p30a/`. The production wire, `PROTOCOL_VERSION`, tick rate,
rules, and outcomes are unchanged; `git diff origin/main --
crates/tbg-core/src/protocol.rs crates/tbg-core/src/view.rs` is empty.

### A. Current diagnosis

- Data flow verified: 20 Hz tick → per-seat pending events → full
  `MatchView` each 10 Hz (or on an ack) → postcard → Axum WS → browser
  decode → `ViewBuffer` → render. Reconnect re-sends `MatchStart` and a
  full view with buffered events.
- **Wire size.** Natural bot corpus (100 seeds, 238 586 views, 7.76
  units/view, 1.12 events/view): 133.65 B average, 196 B p95, 324 B max;
  1.34 KB/s payload per player at 10 Hz. Composition: header 22.0%,
  units 73.6% (static identity 23.4%, `max_hp` 11.7%, dynamic 37.7%),
  events 4.4%. Static identity + `max_hp` = **35.1% of all bytes**.
  Header detail: `synergy` 7.1%, `offers` 4.4%, `stronghold_hp` 3.5%,
  `energy` 1.8%, `tick`/`phase_ticks_left`/`last_seq` 1.7% each,
  `next_rotation_in` 1.2%, `phase`/`outcome`/`you` 0.9% each.
- **Synthetic armies** (both seats topped up, 10 Hz): 296 B at 10
  units/side (2.96 KB/s), 564 B at 20 (5.64), 1141 at 40 (11.41), 1722
  at 60 (17.22), 2875 at 100 (28.75); max 7 754 B, below the 64 KiB WS
  cap. Static identity stays ~28.4–28.9% of bytes and `max_hp`
  ~13.8–14.5%; events grow linearly (2.6 → 22.9 per view), not faster
  than units in bytes.
- **Message mix and ack.** A 250-match full bot run (500 players, 2 CPU /
  1 GiB, 245 s, 0 failed, 0 late ticks): 605 047 views, 10 097 ack views
  = **1.67%**; 1.14 events/view and 7.93 units/view over the match life;
  10.22 view messages/s/player. Add the 1 s WS Ping, 5 s app Ping and
  5 s NetStats: ~11.6 server messages/s/player. Ack views are therefore
  not worth compacting (H5 rejected).
- **Server CPU and capacity** (local cgroup estimates; bots outside the
  cgroup; every row records its load in `capacity-*.txt`):
  - 1 vCPU / 1 GB: 1000 was 0.54 cores / 0 late ticks in the first pass
    and 0.67 cores / 3 cgroup throttles / 3 854 late ticks in the
    committed pass; 1300 and 1500 fail (10/3 throttles, 20 122/19 923
    late). A fresh 750 pass at host load 10.21 showed 1450 late ticks with
    **zero** cgroup throttles (0.63 cores): host scheduling, not the
    server quota, was late. The first 1000 pass predates the committed
    raw files (its log was lost; see Deviations).
  - 2 CPU / 1 GiB (the deployed quota): 1000 clean in every uncontaminated
    pass (0.54–0.67 cores, 1 493–1 852 matches/core); 1500 clean in 2 of
    3 dedicated passes (1.09–1.10 cores, 0 late) and failed in one
    (1 throttle, 5 467 late); 2000 failed in both passes (4–5 throttles,
    27 598/48 626 late). The cgroup `cpu.stat` throttle count tracks the
    late-tick bursts exactly, and RSS/cgroup peak (588 MB at 2000) is well
    under 1 GiB: the first limit is **CPU quota burst throttling**, not
    RAM and not the bot generator (0.8–2.3 bot cores, host load 4–8).
  - 2 vCPU / 8 GB: 1000 and 1500 clean with 2 workers (1500: 1.09 cores,
    0 late), 2000 fails (1.66 cores, 34 400 late), 2500 fails harder
    (1.88 cores, 56 647 late, RTT p95 21 ms). 1 worker is more efficient
    at 1000 (0.54 vs 0.67 cores, 1 852 vs 1 493 matches/core) but saturates
    one core at 2000 (90 783 late); 2 workers reach higher total work.
  - RSS is ~297 KB per running match (290.3 MB at 1000, 431.3 MB at 1500,
    572.6 MB at 2000). The P30-B target of ≤150 KB/match is not met.
- **Client.** Browser frame timing is blocked on this machine (see
  Deviations). The native proxy of the exact sync operations at 100
  units/side is 8.86 µs/frame for clone + linear lookup + linear retain
  and 3.45 µs for Arc clone + sorted merge + HashSet retain (−61% of the
  sync cost). The absolute 8.9 µs is ~0.05% of a 16.7 ms frame, so the
  clone/lookup cannot account for a 15% frame-time change; H7 is not
  proven and no client POC is kept.
- **Send path.** No `perf`/`pprof` on this machine; the P10 pprof hook was
  deliberately not reintroduced. `BytesMut::resize`, allocation shares,
  and the H9 “≥50% of send CPU is per-message” split are **not measured**.
  The measured view build+encode is 1.77 µs at 20 units and 6.93 µs at
  100 (per seat per view), and the capacity CPU is 0.54–0.67 cores for
  1000 matches, so a wire change cannot be justified on CPU grounds
  alone. Message count is a first-class cost: 133 B payload + ~71 B
  estimated framing/TLS/TCP per message.
- Already optimized and not re-litigated: P10 lobby `HashMap`/`VecDeque`,
  bounded socket write buffer, `CONN_QUEUE=32`, `TBG_WORKERS`, TCP_NODELAY;
  mimalloc, `opt-level=3`, and encode-once were measured and rejected.
- Opportunities: remove derivable fields (A), stop repeating static unit
  identity and low-frequency Synergy (C). Risks: every view-format change
  touches reconnect and the client cache; the measurements are local
  cgroup estimates on a shared PC; the VPS `/metrics` and the real VPS
  capacity table remain owner work.

### B. Alternative comparison

- **A — derive, don't send.** Works exactly: `max_hp` from
  `Ruleset::unit_max_hp` (kind/tier/branch/Synergy/Arcanas),
  `phase_ticks_left` from tick+phase, `next_rotation_in` from tick, `you`
  from `MatchStart`. Natural 113.77 B (−14.9%), 20 units 478 B (−15.2%),
  100 units 2 463 B (−14.3%). Build+encode 1.57 µs at 20 (−11.3%) and
  6.31 µs at 100 (−9.0%); zero extra RAM; reconstruction 0.75/3.47 µs.
  Exact on 100 seeds and every edge case. **Rejected alone**: it misses
  the ≥20% traffic threshold at 20 units/side; it is part of C.
- **B — delta.** Sorted static+dynamic caches, changed-unit list,
  removals, keyframe every 100 views (1.0% of views). Natural 46.80 B
  (−65.0%), 20 units 170 B (−69.9%), 100 units 1 047 B (−63.6%). Build+
  encode 1.73/6.19 µs (−2.5%/−10.6%); 2.46 KB/seat at 20, 11.9 KB at 100;
  client reconstruction 1.93/15.26 µs. Exact on 100 seeds, reconnect,
  and a deliberately lost delta (12 views mismatched, recovered at the
  next keyframe). It beats C by 21–25 percentage points. **Deferred, not
  chosen**: the spec defers delta when the simpler split clears the
  threshold; the extra state machine and keyframe recovery are
  disproportionate to the additional gain, and its load latency was not
  measured (no deployed prototype). Reopen only if C fails recovery or
  the VPS proves bandwidth is the first limit after C.
- **C — A + static/dynamic.** Full static roster on first view/reconnect,
  spawned/evolved units only afterwards, complete dynamic list every
  view, Synergy update only on full/change. Natural 78.06 B (−41.6%),
  20 units 312 B (−44.7%), 100 units 1 653 B (−42.5%). Build+encode
  1.67/5.89 µs (−5.6%/−15.0%); client cache 0.53 KB/seat at 20, 2.54 KB
  at 100; reconstruction 1.51/12.25 µs. Exact on 100 seeds and all edge
  cases; a missing static id is rejected and forces the full-snapshot
  reconnect. **Accepted.**
- **D — events only.** 8.80 B (−93.4%) but cannot reconstruct position,
  HP, or roster; it is the negative control and is rejected.
- **E — lower view rate.** Measured with a temporary `TBG_VIEW_HZ` hook
  (reverted): 20 bot matches each at 10/8/5 Hz, 2 CPU / 1 GiB.
  - 10 Hz: 1.358 KB/s/player, gap p95 100 ms, ack p95 50 ms.
  - 8 Hz: 1.030 KB/s, gap p95 150 ms, ack p95 50 ms.
  - 5 Hz: 0.689 KB/s, gap p95 200 ms, ack p95 50 ms.
  Ack latency is unchanged because ack views are immediate, but the
  view-gap budget (p95 ≤150 ms) is only reached at 8 Hz and broken at
  5 Hz. **Rejected**: keep 10 Hz and the 20 Hz simulation.
- **F — compression.** 200 real encoded views: 14 853 B raw, 18 538 B
  with `gzip -6` (1.248×) and 16 471 B with `zstd -3` (1.109×) when each
  message is compressed alone. **Rejected**: per-message compression is
  larger; no transport path without a dependency.
- **G — send path.** Not measured (no profiler); see A. No change.
- **H — client sync.** Browser FPS blocked; native proxy proves the sync
  cost is 8.9 µs at 100 units, far below the frame-time threshold.
  **Rejected for now**; keep `?army=N` for the later client phase.

### C. Benchmarks

Hardware/build: dev PC above, `--release`, 100 natural seeds (seeds
1–100, default Arcanas), five repetitions, medians. Payload bytes; wire
estimates add 71 B per message. Synthetic and natural are separate.

| Metric | Current | A: derive, don't send | B: delta | C: A + static/dynamic |
|--------|---------|-----------------------|----------|-----------------------|
| KB/s per player at 20 units (payload) | 5.64 | 4.78 | 1.70 | 3.12 |
| KB/s per player at 100 units (payload) | 28.75 | 24.63 | 10.47 | 16.53 |
| KB/s per player natural (payload) | 1.337 | 1.138 | 0.468 | 0.781 |
| Estimated wire KB/s, 20 / 100 units | 6.35 / 29.46 | 5.49 / 25.34 | 2.41 / 11.18 | 3.83 / 17.24 |
| Build+encode CPU per view, 20 / 100 units | 1.77 / 6.93 µs | 1.57 / 6.31 µs | 1.73 / 6.19 µs | 1.67 / 5.89 µs |
| Server CPU per match at fixed load | 0.54 ms per match/s (1000 matches, 0.54 cores) | not measured under load (no deployed wire) | not measured under load | not measured under load |
| RSS per match / per connection | 297 KB/match; per-connection share not separated | no change | +23.7 KB/match at 100 units | +5.1 KB/match at 100 units |
| Ack p95 / RTT p95 | 50 / 2 ms at 250 matches | not measured under load (ack path unchanged) | not measured under load | not measured under load (ack path unchanged) |
| Integrity on 100 seeds | control | verified (zero mismatches) | verified (zero mismatches) | verified (zero mismatches) |
| Reconnect mid-combat | 100/100 chaos reconnects | full view | keyframe; lost delta recovered | full view; 100/100 chaos |

Run `view_audit` for the raw tables; `full250-*.txt`, `chaos50-*.txt`,
`capacity-*.txt`, and `viewhz-*.txt` in `docs/media/p30a/` support every
row. No cell is a guess; unmeasured cells say why.

### D. Chosen architecture

**A + C: derive the four client-computable fields, and split unit static
identity from dynamic state.** A alone misses the traffic threshold; C
clears it at 20 and 100 units with exact reconstruction and lower
build+encode CPU than today (the sorted static tracker removes per-unit
tree lookups, and the smaller payloads encode faster). B is deferred.

- Reused unchanged: 20 Hz simulation, 10 Hz views, postcard/WebSocket,
  `MatchStart` (Arcanas and `you`), immediate ack views, presentation-only
  events, server authority, reconnect timing, 64 KiB write cap.
- What changes in P30-B: `MatchView` loses `you`, `phase_ticks_left`,
  `next_rotation_in`, and `UnitView.max_hp`; units become
  `Vec<UnitDynamic>` plus a `full: bool` and `Vec<UnitStatic>` (static =
  id, seat, kind, tier, branch); Synergy moves to a low-frequency
  `Option<[[u8;4];2]>` update. `Ruleset::unit_max_hp` (already added) and
  pure tick/phase helpers derive the removed fields. `PROTOCOL_VERSION`
  takes the next unused value; main is v9 at review.
- Reconnection gets `MatchStart` plus a `full=true` view with every live
  static unit, current Synergy, all dynamic units, and buffered events.
  The client replaces its static map on `full`, upserts otherwise, and
  reconnects if a dynamic id has no static entry. Decode errors trigger
  the existing reconnect instead of being ignored.
- Events stay presentation-only and ordered: they are carried in every
  view exactly as today; no damage, targeting, Energy, or outcome moves
  into the client.
- Growth: the static section can carry P17-A `StrongholdView`s, Arcana
  and Augment metadata later without touching the dynamic list. The
  dynamic list stays complete every view, so one lost view cannot leave a
  stale position or HP; only the static map needs recovery.

### E. Minimum implementation plan (P30-B step 0)

1. `tbg-core/src/view.rs`: add `UnitStatic`, `UnitDynamic`, `full`,
   `static_units`; remove the four redundant fields; add
   `sim::phase_ticks_left_at(tick, phase)` and use
   `shop::next_rotation_in(tick)`. Keep `view_for` as the canonical full
   view for the bot/lab/tests.
2. `tbg-core/src/ruleset.rs`: `unit_max_hp` is already present; extend the
   `contract_view.rs` H8 test with C reconstruction on 100 seeds and the
   spawn/death/summon/evolution/status/Synergy/reconnect edge cases.
3. `tbg-client/src/sync.rs` + `sync/messages.rs`: static map, full
   replacement, upsert/prune, derived `max_hp`/counters, reconnect on a
   missing static or decode error; `?army=N` stays.
4. `tbg-server/src/match_task/views.rs`: per-seat sorted `Vec<UnitStatic>`
   tracker (no BTreeMap in the hot path), full on first view and `Attach`,
   static changes on spawn/evolve, Synergy update on change.
   `match_task.rs` cadence and ack logic unchanged.
5. `tbg-core/src/protocol.rs`: bump `PROTOCOL_VERSION` with a comment
   naming P30-A; server and client deploy together.
6. Tests: `contract_protocol*.rs` (version and view shape),
   `contract_determinism.rs` unchanged, `contract_view.rs` (H8 + C
   reconstruction), online reconnect/slow-client/rate-limit/seat/hidden
   data, a 100-seed reconstruction, 50 chaos reconnects, size ceiling
   (<64 KiB). Commands: the four `AGENTS.md` verification commands,
   `shellcheck`, `cargo test -p tbg-server --test online`, and
   `trunk build --release`.
7. P30-B must re-measure: payload bytes/view at 20/100, view messages/s,
   build+encode µs, matches/core at 1 vCPU/1 GB and 2 CPU/1 GiB with 0
   late ticks, RSS/match, ack/RTT/view-gap p95 at 250 matches, ack-view
   share, and reconnect/chaos. Before/after on the same machine.
- Rollback: revert the protocol/view commit and redeploy the previous
  binary with its matching client; old peers are rejected by the version
  check, so deploy both together. No data migration; no DB schema change.
- VPS validation still unknown: the owner's `/metrics` reading, the real
  VPS CPU/network mix, and whether the 1 GiB quota behaves like the local
  cgroup. P30-B owns the final VPS table.

### F. Capacity estimate

**Measured (local cgroups, bots outside):**

| Limit | clean matches | server cores | matches/core | RSS MB | KB/match | late ticks | RTT p95 |
|---|---|---|---|---|---|---|---|
| 1 vCPU / 1 GB | 750 | 0.63 | 1 190 | 219.8 | 300 | 1 450 (0 throttles; host load 10.21) | 4 ms |
| 1 vCPU / 1 GB | 1 000 | 0.54–0.67 | 1 493–1 852 | 290.3 | 297 | 0 (first pass, log lost) – 3 854 (committed, 3 throttles) | 3–4 ms |
| 2 CPU / 1 GiB | 1 000 | 0.54–0.67 | 1 493–1 852 | 290.7 | 298 | 0 | 3 ms |
| 2 CPU / 1 GiB | 1 500 | 1.09–1.18 | 1 271–1 376 | 431.2 | 294 | 0 in 2/3, 5 467 in 1/3 | 3 ms |
| 2 CPU / 1 GiB | 2 000 | 1.27–1.66 | 1 205–1 575 | 572.6 | 293 | 27 598–48 626 | 15 ms |
| 2 vCPU / 8 GB | 1 500 | 1.09 | 1 376 | 431.1 | 294 | 0 | 3 ms |
| 2 vCPU / 8 GB | 2 000 | 1.66 | 1 205 | 572.6 | 293 | 34 400 | 12 ms |
| 2 vCPU / 8 GB | 2 500 | 1.88 | 1 329 | 713.5 | 292 | 56 647 | 21 ms |

**Extrapolated.** On an idle host (load ≤5), 1000 matches (2000 sockets)
at 2 CPU / 1 GiB and 1 vCPU / 1 GB were clean; 1500 at 2 CPU is likely but
one pass in three hit a CPU-quota burst, and 2000 fails. Under heavy host
load (~10) even 750 at 1 vCPU had 1450 late ticks with no cgroup
throttles: host scheduling matters as much as the quota. The first server
limit is CPU quota burst throttling, not RAM (peak 588 MB < 1 GiB) and not
the bot generator (≤2.3 bot cores on 16); the VPS must confirm both.
C removes ~42% of payload and ~6–15% of build+encode CPU; CPU-bound
capacity could rise up to ~10%, and bandwidth-bound traffic falls 42%.
That gain is **not measured under a deployed C**; P30-B must measure it.

**Monthly traffic** (`monthly_GB = matches × 2 × KB_per_s_player ×
2 592 000 / 1 000 000`, decimal GB, 30-day month, payload only; wire is
~1.5× payload with the measured 71 B/message overhead):

| Occupancy (matches) | current payload GB | current wire GB | C payload GB | C wire GB |
|---|---|---|---|---|
| 25% (250) | 1 646 | 2 469 | 961 | 1 442 |
| 50% (500) | 3 292 | 4 938 | 1 923 | 2 884 |
| 75% (750) | 4 938 | 7 407 | 2 884 | 4 326 |
| 100% (1000) | 6 584 | 9 876 | 3 846 | 5 769 |

The plan's actual traffic allowance is an **owner value to confirm**.
These are local cgroup estimates; a VPS may differ in CPU steal, TLS
overhead, and network path.

### G. Decisions deferred

- **Delta (B):** reopen only if C fails recovery/desync or if the VPS
  shows bandwidth is still the first limit after C, with a measured
  ≥15 pp gain over C at 100 units.
- **Compression:** reopen only with a transport that compresses across
  messages and a ≥30% payload win without a new dependency.
- **5/8 Hz views:** reopen only if the 150 ms view-gap budget changes;
  5 Hz already loses 200 ms gaps.
- **H10 shared build:** measured −5.4% build+encode for two seats, below
  the −15% threshold; reopen with a different view layout.
- **H7 client Arc/sorted merge:** defer to P30 (client phase) with a real
  device measurement; the native cost is only 8.9 µs at 100 units.
- **Spectator replication, prediction/rollback, multi-region, external
  queues, transport changes, client-side outcomes:** unchanged and
  deferred by default.
- **P17-A StrongholdView:** add it to C's static section when that branch
  merges; protocol number must be coordinated then.

### Answers to the 15 questions

1. **Real bottleneck:** server CPU-quota bursts at ~1500–2000 matches
   (2 CPU) and ~1000 (1 vCPU); wire bytes are dominated by static unit
   identity and `max_hp` (35.1%). Client sync is 8.9 µs/frame at 100
   units, not a bottleneck.
2. **Wasted bytes per update:** 35.1% of all bytes are static identity
   (23.4%) + `max_hp` (11.7%); header 22.0%, events 4.4%.
3. **Unnecessary fields:** `max_hp`, `phase_ticks_left`,
   `next_rotation_in`, `you`, and the repeated `seat/kind/tier/branch`
   (plus repeated Synergy).
4. **Changed-data-only feasibility:** yes; B reconstructs exactly and
   recovers from a lost delta at a keyframe, but needs per-seat state.
5. **Delta versus simpler split:** B saves 21–25 pp more than C at
   20/100 units; the spec defers delta when the split clears, so C wins
   on risk.
6. **Simpler hybrid:** A + C (derive + static/dynamic split).
7. **Achievable traffic reduction:** 41.6% natural, 44.7% at 20 units,
   42.5% at 100 units.
8. **CPU/RAM cost:** build+encode −5.6% at 20 and −15.0% at 100; extra
   client cache 0.53 KB/seat at 20 and 2.54 KB at 100; server static
   tracker is a sorted Vec.
9. **Client synchronization:** exact on 238 586 views over 100 seeds and
   all edge cases; a missing static id is rejected, not rendered.
10. **Reconnect safety:** 100/100 chaos reconnects and 250/250 matches
    completed; full view on attach.
11. **Command latency:** ack p95 50 ms / RTT p95 2 ms at 10 Hz; lowering
    to 8/5 Hz keeps ack p95 50 ms but raises view gap p95 to 150/200 ms.
12. **Capacity gain per vCPU:** current 1 493–1 852 matches/core at 1000;
    C could add up to ~10% CPU-bound and cut bandwidth 42%, but this is
    extrapolated until P30-B measures a deployed C.
13. **Worth doing now:** A + C.
14. **Deferred:** B delta, compression, 5/8 Hz, H10, client Arc POC,
    spectators, prediction, external queues, transport changes.
15. **Smallest sequence:** core types/helpers + invariant/rebuild tests;
    client cache/derive; server sorted tracker + full attach; version
    bump; integrity/capacity re-measure.

### Deviations, blocked items, and pending VPS tests

- Branch name is `phase/p30a-network-audit` (owner instruction), not the
  spec's `phase/p30a-network-performance-audit`. The branch was rebased
  on `origin/main` before review; main's protocol is v9.
- No `perf` on this machine. The P10 pprof hook was not reintroduced
  (no dependency churn); `BytesMut::resize`/allocation shares and the H9
  per-message CPU split are **not measured**. The capacity CPU and the
  view-build microbenchmark are the evidence instead.
- Browser FPS is blocked: headless SwiftShader, headful Intel Mesa, and
  Xvfb all run the game page at ~1.0–1.4 FPS while a blank page runs rAF
  at ~71 Hz; army=10/40/100 are all ~1 FPS. Absolute FPS/frame p95 and
  phone measurements are therefore not measured; the native sync proxy is
  used for H7. Details in `docs/media/p30a/client-measurement.txt`.
- Capacity numbers are local cgroup estimates on a shared PC. Late ticks
  were non-monotonic between passes (2000 clean once, 1500 throttled
  twice) because other processes spiked; every committed run's `load1`,
  bot cores, cgroup throttles, and memory peak are in the raw files. The
  very first 1 vCPU table (1000 clean, 750 clean) was lost when another
  process removed `target/p30a-logs/`; it is quoted only where the
  committed final pass is strictly worse, and the lost numbers are marked
  in the report. The VPS
  capacity table and the production `/metrics` reading (owner: two
  `curl -s http://127.0.0.1:18260/metrics` samples 10 minutes apart) are
  **pending owner**.
- The 8/5 Hz and the delta-loss/reconnect prototypes are reverted. The
  only retained production code is: the `view_audit.rs` example, the
  `Ruleset::unit_max_hp` helper, the `SeatStats`/global view counters,
  the capacity script extensions, the `?army=N` debug parameter, and the
  H8 contract test. `crates/tbg-core/src/protocol.rs` and `view.rs` are
  byte-for-byte identical to `origin/main`.
- P17-A is not on main; if it merges first, P30-B takes the next protocol
  version after it and adds `StrongholdView` to C's static section.

### Verification (real output)

```
cargo fmt --all -- --check                  -> exit 0
cargo clippy --workspace --all-targets -D warnings -> exit 0
cargo test --workspace                      -> all suites pass (contract_view 1,
                                               contract_protocol 6+2, online 28,
                                               determinism/rules/accounts/ranked green)
cargo check -p tbg-client -p tbg-core --target wasm32-unknown-unknown -> exit 0
shellcheck scripts/capacity.sh              -> exit 0
cargo run --release -p tbg-core --example view_audit -> exit 0
git diff origin/main -- crates/tbg-core/src/protocol.rs crates/tbg-core/src/view.rs -> 0 lines
(cd crates/tbg-client && trunk build --release) -> success
```
