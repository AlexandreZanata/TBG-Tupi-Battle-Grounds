# D-104 heartbeat snapshot reliability

The owner explicitly authorized normal merges and observing the official VPS on
2026-11-02, superseding the earlier no-merge instruction. Main merge 53574b6
contains #49, #42 and #46. No public artifact or controller activation is claimed.

> blocked: main CI 41449596346 and the independent landing CI 41450196140 both
> reached MatchEnd/Ranked settlement but timed out waiting for both slots idle.
> Serial and concurrent local slot tests passed. The blind-retry step stops;
> counter snapshot diagnosis is a separate work front. No timeout, retirement
> guard or required check is relaxed, and production remains unchanged.

Inspection found that the heartbeat samples Store jobs before queuing its own
writer operation. A job ahead of it can finish while it waits, leaving its
already-completed count in the retirement row. The regression explicitly holds
a prior job, queues the heartbeat and then releases that job; receipts follow.

## Reproduction and correction

The deterministic held-writer test failed on unchanged main: the heartbeat row
reported results_pending=1 after the prior job had completed. Snapshot load inside
the serialized writer turn, excluding only the observation's own pending count.
Every later queued operation remains counted. Both targeted regressions and all
six concurrent slot cases pass; no assertion or timeout changed. No competitive
number, wire field or schema changes. The compatible native version advances to
0.1.2-rc.3 because rc.2 already has a source tag; nothing is replaced.

Logs live in `/data/tbg-consolidation-evidence/heartbeat-red.log` and
`heartbeat-green.log`. Full local workspace acceptance was not run. A main
retry started before the second CI failure was inspected; no further blind
retry is authorized by this report. Production promotion remains pending.
