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
operator `--maintenance-window`). Two release slots on this host are the
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

P66 has slot/generation and drain foundations, and since 2026-11-02 the official host runs
them: the single process was switched to slot `a` once, at zero connections
(`deploy/slot-bootstrap.sh`), and `deploy/auto/slots.py` promotes compatible releases into
the idle slot. Cross-slot revocation has been in place since `2e15dd9` (each process polls
`revoked_at` every second); the generation running today (`8be248e`) predates it, so during
the first overlap a session revoked on the new slot stays valid on the old one until that
slot drains. The gaps found on 2026-11-02 are handled as follows: ranked
authority is requested every second by an admitting slot that lacks it and moved by the
coordinator right after the promotion; clients that carry the re-admission change ask
admission again when a draining slot answers `ServerFull`; players waiting in a queue,
room or invitation are counted (`lobby_waiting`), awaited by the coordinator, and refused
with `ServerFull` if one slips in; idle sockets get a bounded grace before a drained slot
stops. Still open: rematch offers and spectators do not follow a promotion, `/api/live`
and `/api/online` are per process during an overlap, older tabs show "server full" until
reloaded, and a Caddy reload would still close every WebSocket (releases never reload it).
Multiple containers or pods do not transfer in-memory matches,
share an Access index safely, or make SQLite a distributed database. Before adding a
host, design explicit match ownership/routing, reconnection, durable settlement,
credential revocation and compatible client admission. A distributed database or
orchestrator is a separate design based on measured need, not an incidental deploy edit.

See [ARCHITECTURE](ARCHITECTURE.md), [RELEASES](RELEASES.md), P66/D-99 and P71/D-100.
