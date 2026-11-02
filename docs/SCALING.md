# Scaling without sacrificing running matches

Current production is one official Debian VPS, Caddy and a Rust/systemd backend,
with SQLite and static client releases. This is an indie beta. There is no measured
capacity claim in this consolidation and no reason to move it to Kubernetes merely
to deploy static pages. The retired `tbg.abastevo.com.br` environment is not a target.

## Use the existing hardware first

`TBG_WORKERS` configures Tokio worker threads; 0 uses available cores. Matches are
separate authoritative tasks, while a single match's deterministic simulation stays
ordered. Extra workers can help independent matches but cannot fix an expensive
single tick, database contention, memory pressure or client/network RTT. Do not
change the 50 ms production tick, game rules, admission caps or ranked writer as a
performance workaround. Offline `tbg-balance` simulation throughput is not a live
multiplayer capacity measurement.

Observe the existing loopback `/metrics` and `/api/release` alongside system CPU,
RSS, disk availability/latency and network. Relevant counters include connections,
`matches_running`, `late_ticks`, `step_us_p99`, RTT, queue waits, ranked inflight/jobs,
result receipts, journal failures and settlement errors. Separate simulation time,
network time, queueing and persistence before changing a limit. Keep the operator
endpoint private; public `/health` alone cannot prove match or database capacity.

Use an explicitly authorized focused load scenario to establish the baseline and
repeat after one relevant change. Preserve enough CPU, memory and disk headroom
for the game plus result/reconnect persistence, backups and recovery. Agree numeric
headroom/latency goals with the owner from actual beta measurements. Do not promise
that a configured maximum connection count is a supported player capacity.

## Deployment is part of capacity

Build in GitHub, not on the playing VPS. Delivery workers run with low CPU/I/O
priority, bounded package sizes and a common host lock. Static site and browser
promotions do not restart the Rust server or reload Caddy. Keep old immutable assets
available so a tab already in a match can finish using its original release.

The compatible single-process backend path drains admission and waits without a
forced timeout for matches/results/reconnect windows. New matchmaking pauses;
there is no claim of uninterrupted admission, so the timer never runs it (only an
operator `--maintenance-window`). Two release slots on this host are the planned
uninterrupted path; their overlap needs measured headroom (2026-11-02: 4 vCPU,
16 GB RAM, about 0.5 GB in use while idle) and a preflight that defers a release
rather than starving the generation that still owns matches. Never kill a previous generation to
meet a release deadline, restore an old database over player progress or automatically
remove old release directories. Audit disk usage and retention with an operator;
an unbounded release history is an explicit current limit, not safe automatic GC.

## When one host is insufficient

First locate the bottleneck. Possible measured next steps are allocation/tick work,
reducing unnecessary network payloads, caching read-only ranking, or moving non-match
jobs away from simulation workers. Keep ranked settlement durable and single-writer.
Raising task count beyond memory/CPU capacity only increases queueing.

P66 has slot/generation and drain foundations. Cross-slot revocation is implemented
since `2e15dd9` (each process polls `revoked_at` every second); the server deployed
on 2026-11-01 (`8be248e`) predates it. Two-slot rollout stays disabled until the
two-generation continuity acceptance passes and the gaps found on 2026-11-02 are
closed: ranked authority moving inside the admission switch, client re-admission
instead of `ServerFull` on a draining slot, notifying queued/room/rematch waiters,
connection counts in the retire gate and a directory-backed first admission answer.
The single process cannot hand over to a slot while it runs (`tbg-server@` conflicts
with it and it never reads the directory), so the switch to slots needed one restart (done 2026-11-02 10:31Z, `deploy/slot-bootstrap.sh`),
scheduled when it holds no connection, match or pending result (D-99). Multiple containers or pods do not transfer in-memory matches,
share an Access index safely, or make SQLite a distributed database. Before adding a
host, design explicit match ownership/routing, reconnection, durable settlement,
credential revocation and compatible client admission. A distributed database or
orchestrator is a separate design based on measured need, not an incidental deploy edit.

See [ARCHITECTURE](ARCHITECTURE.md), [RELEASES](RELEASES.md), P66/D-99 and P71/D-100.
