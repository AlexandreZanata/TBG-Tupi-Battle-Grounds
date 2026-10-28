# P10 — Server efficiency: most players per CPU core

**Depends on:** P9 reviewed and merged.
**Branch:** `phase/p10-server-efficiency`.
**Difficulty:** medium–high. This phase is **measurement-driven**: every
optimization is kept only if the capacity run shows it helps.

## Goal

Serve as many simultaneous matches as possible on one small VPS (2 vCPU /
2–4 GB) without hurting latency or fairness. Find the real bottleneck with
a profiler, fix it, and prove the gain with the same load test.

## Deliverable

- A repeatable capacity script and a before/after table.
- The kept optimizations, each in its own commit with its measured gain.
- New measured values in ARCHITECTURE §10.

## Read first (only these)

- `AGENTS.md`
- `docs/ARCHITECTURE.md` §5, §10, §12
- Server: `src/lib.rs`, `src/main.rs`, `src/ws.rs`, `src/ws/client_msg.rs`,
  `src/lobby.rs`, `src/lobby/matchmaking.rs`, `src/match_task.rs`,
  `src/chan.rs`, `src/metrics.rs`
- Root `Cargo.toml` (profiles)

## Do NOT

- Change simulation rules, tick rate, view contents, or `tbg-core` behavior
  (the determinism hash for a fixed seed + script must stay identical).
- Add Redis, a message queue, a second process, or unsafe code.
- Drop validation, rate limits, or size limits to gain speed.
- Keep any change without a measured improvement ≥ 5 % on its target metric.

## Dependencies you may add

| Crate | Where | Why |
|-------|-------|-----|
| `mimalloc = { version = "0.1", default-features = false }` | server binary only | candidate in step 6; remove if it does not win |

Tools (not dependencies): `perf` + `inferno` or `cargo flamegraph`
(`cargo install --locked flamegraph`), `/usr/bin/time -v`, `pidstat`.

## Targets (start values; record what you reach)

| Metric | P4 baseline (local) | Target |
|--------|---------------------|--------|
| Matches per busy core, 0 late ticks | ≈ 280 | ≥ 600 |
| RSS per running match (incl. 2 sockets) | ≈ 300 KB | ≤ 150 KB |
| Down-bandwidth per player | 1.46 KB/s | ≤ 1.5 KB/s (no regression) |
| p99 tick lateness at 1000 matches | — | ≤ 10 ms |

## Steps

### 1. Capacity script
New `scripts/capacity.sh` (bash, shellcheck-clean): builds release, starts
`tbg-server` with `TBG_MAX_CONNECTIONS=20000 TBG_MAX_PER_IP=20000`, runs
`tbg-bots` at 250, 500, 1000, 1500 concurrent matches (`--ramp-ms 5`), and
for each level samples every 5 s: server CPU % (`pidstat -p PID 5`), RSS
(`/proc/PID/status` VmRSS), `/metrics` late ticks, RTT p95, ack p95, and the
`tbg-bots` summary. Prints one table row per level. Raise `ulimit -n` to
65536 inside the script.
- **Done when** a "Before" table (four levels) is in the report. Stop
  increasing levels when late ticks > 0 or the bots machine saturates
  (record which one happened).

### 2. Profile
At the highest clean level, record 30 s with `perf record -g -p PID` (or
`cargo flamegraph --pid`). Save the top 15 symbols by self time in the
report, grouped as: simulation, view building, encoding, WebSocket
write/read, channels/scheduling, allocation, logging, other.
- **Done when** the report names the top two groups. Do the steps below in
  order of that ranking; skip steps whose group is under 5 % of CPU.

### 3. Faster server build profile
Root `Cargo.toml`: add
```toml
[profile.server]
inherits = "release"
opt-level = 3
```
(the shared `release` profile uses `opt-level = "s"` for WASM size). Build the
server with `cargo build --profile server -p tbg-server --bins`; update
`deploy/deploy.sh`, `deploy/README.md`, and `scripts/capacity.sh`.
- **Done when** measured; keep it if CPU per match improves ≥ 5 %.

### 4. Encode once, send bytes
- `chan.rs`: `ConnEvent::Send(ServerMsg)` → add `ConnEvent::Bytes(bytes::Bytes)`
  (`bytes` is already in the tree through axum; use `axum::body::Bytes`).
- Match task encodes each per-seat `View` once into `Bytes` and sends those;
  `ws.rs` writes them as `Message::Binary` with no re-encode. Other messages
  may stay `Send(ServerMsg)`.
- Reuse one `Vec<u8>` per match for encoding (`postcard::to_extend`).
- **Done when** measured; keep if the "encoding" group shrinks.

### 5. Lobby lookups in O(1)
`lobby.rs` scans `Vec<Session>` for every message. Replace with
`HashMap<Token, Session>` (the lobby is not simulation; `HashMap` is
allowed here) and `VecDeque<Token>` for the queue. Split `lobby.rs` below
200 lines while there (it is 201 now). Behavior and tests unchanged.
- **Done when** all online tests pass and the 1500-level run shows no lobby
  task backlog (`/metrics` gains `lobby_inbox_len`, max over the run).

### 6. Allocator
Add `#[global_allocator] static GLOBAL: mimalloc::MiMalloc = mimalloc::MiMalloc;`
in `src/main.rs` only (not in tests, not in `tbg-bots`). Measure CPU and RSS.
- **Done when** measured; remove the dependency again if neither improves ≥ 5 %.

### 7. Socket buffers and queues
- Set Axum `WebSocketUpgrade::write_buffer_size(4096)` and
  `max_write_buffer_size(64 * 1024)`; keep frame/message limits from P4.
- `CONN_QUEUE`: measure RSS at 64 vs 32. Keep 32 only if no
  `server_closed` (slow-client) drops appear in a 50-match run with
  `--delay-ms 150 --jitter-ms 50`.
- **Done when** measured and the slow-client rule still works (existing test).

### 8. Tokio runtime
Make worker threads configurable: env `TBG_WORKERS` (default = available
cores). Run the 1000 level with 1 and 2 workers on a 2-core cgroup
(`systemd-run --scope -p CPUQuota=200% ...`) and record late ticks.
- **Done when** ARCHITECTURE §9/§10 state the recommended worker count for a
  2 vCPU VPS with measurements.

### 9. Optional: staggered ticks
Only if step 2 ranked "channels/scheduling" first: start each match's tick
interval at an offset `(match_id % 5) * 10 ms` so 1000 matches do not wake on
the same millisecond. Keep only with a measured p99 lateness gain.

### 10. After table and docs
Re-run `scripts/capacity.sh`; "After" table next to "Before". Update
ARCHITECTURE §10 measured column and the "Capacity reasoning" paragraph with
the new matches-per-core number.

## Acceptance checklist

- [ ] All verification commands from `AGENTS.md` pass
- [ ] Determinism: `cargo test -p tbg-core` hash tests unchanged
- [ ] Before/after capacity tables from the same machine and script
- [ ] Every kept optimization has its own commit with a measured gain
- [ ] No regression in RTT p95, ack p95, or bandwidth per player at 250 matches

## Risks and what to do

| Problem | Action |
|---------|--------|
| The bots machine saturates before the server | Run bots with `--bots` split across two processes, or record the server limit as "not reached" with CPU % |
| An optimization changes behavior | Revert it; correctness beats capacity |
| `perf` unavailable | Use `cargo flamegraph`; if both fail, time sections with `Instant` counters exposed in `/metrics` |

## Reviewer checklist

- The ranking in step 2 matches the order of work actually done.
- Removed candidates are really removed (no dead dependency).
- Numbers are reproducible: re-run one level of `scripts/capacity.sh`.

## Phase report

_Executor fills this in: before/after tables, profile summary, per-change
measurements, deviations, blocked items._
