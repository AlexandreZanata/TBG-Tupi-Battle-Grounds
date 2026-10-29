# P30-B — Server capacity on minimal hardware

**Depends on:** P14 merged (already true). Touches only `tbg-server`,
`scripts/`, and docs, so it may run between character phases (P22–P29) on
its own branch — never at the same time as another agent on the same files.
**Branch:** `phase/p30b-server-capacity`.
**Difficulty:** medium–high. Measurement-driven like P10: an optimization
is kept only if the capacity run proves the gain.

## Goal

Serve the most simultaneous competitive matches on the smallest machine
without hurting latency or fairness. P10 tuned throughput on a 16-core dev
PC; this phase measures and optimizes **under hard limits of 1 vCPU and
1 GB RAM**, the cheapest realistic VPS, and closes P10's missed memory
target (~300 KB per running match against ≤ 100 KB).

## Deliverable

- `scripts/capacity.sh` able to run the server inside a CPU/memory limit.
- A before/after table at 1 vCPU / 1 GB: matches, CPU, RSS, KB per match,
  late ticks, RTT p95, ack p95, bytes/s.
- The kept optimizations, one commit each with its measured gain.
- Measured values and the recommended VPS size in ARCHITECTURE §10 and §12.

## Targets (**Proposed**, owner 2026-10-28 — confirm with the first baseline)

On 1 vCPU / 1 GB: ≥ 1 000 concurrent matches (2 000 sockets) with 0 late
ticks over a 60 s window, server-added RTT p95 ≤ 10 ms, and ≤ 150 KB RSS
per running match. If the baseline already beats a target, raise it and
record the new value as Proposed.

## Read first (only these)

- `AGENTS.md`
- `docs/ARCHITECTURE.md` §5, §10, §12
- `docs/phases/P10-server-efficiency.md` (method and profile results)
- Server: `src/lib.rs`, `src/main.rs`, `src/ws.rs`, `src/ws/`, `src/lobby.rs`,
  `src/lobby/`, `src/match_task.rs`, `src/chan.rs`, `src/metrics.rs`
- `scripts/capacity.sh`, root `Cargo.toml` (profiles)

## Do NOT

- Change the simulation, tick rate, protocol, view contents, matchmaking,
  rating, or anything players can observe besides latency.
- Weaken validation, size limits, rate limits, timeouts, or seat ownership
  (AGENTS non-negotiables 2 and 4).
- Add a dependency without a measured reason, or a cache/layer for one use.

## Steps

### 1. Limited baseline
Add `LIMIT_CPU` / `LIMIT_MEM` to `capacity.sh` (run the server with
`systemd-run --user --scope -p CPUQuota=100% -p MemoryMax=1G`, or `taskset`
plus a cgroup when systemd is unavailable; bots stay outside the limit).
Run levels 250/500/1000/1500/2000 and record the table.

### 2. Profile under the limit
CPU: `perf record -g` at the highest level without late ticks. Memory:
per-connection and per-match breakdown (heap profile or counted structs).
Write the top five sinks in the report before changing code.

### 3. Memory per connection and match
Right-size per-connection queues and WebSocket read/write buffers to the
largest legal message, reuse encode buffers, shrink per-match state, free
finished-match memory promptly. Re-measure after each change.

### 4. CPU per tick
Encode each view once per seat per tick into a reused buffer; avoid clones
and reallocations on the hot path (P10's top sink was `BytesMut::resize`);
check the scheduler (worker count, task layout) at 1 vCPU.

### 5. Host tuning (documented, not automated on the VPS)
File-descriptor limit, `somaxconn`, TCP buffer sizes, and `TBG_WORKERS` for
1 and 2 vCPU, in `deploy/README.md`.

### 6. Real-network check
Run 50 bot matches against the real server through the public URL and
compare RTT/ack p95 with the P14 numbers.

## Acceptance checklist

- [ ] All verification commands from `AGENTS.md` pass
- [ ] Before/after table at 1 vCPU / 1 GB in the report; targets met or the gap explained
- [ ] Every kept optimization has its own commit and measured gain
- [ ] Rate-limit, size-limit, reconnect, ranked, and seat-ownership tests unchanged and green
- [ ] ARCHITECTURE §10/§12 updated with measured values and the recommended VPS size

## Phase report

_Executor fills this in: tables, profiles, kept/rejected optimizations,
deviations._
