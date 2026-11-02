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

## Expanded Faction fixture consolidation

CI 41457513916 caught two remaining seven-Faction assignments in the Augment
Convergence scenario. The fixture now initializes the published FACTION_COUNT
and toggles the same original two counts. The targeted scenario passed (1 test);
its assertions and gameplay behavior are unchanged. The repository search found
no other fixed-size seven-Faction Rust synergy fixtures.

## Protocol audit and attended switch

The next hosted test lane reached the exploratory view_audit example and exposed
constructors that predated lane_z, queued, physical_lanes and barrels. Its
prototype headers now carry those fields, its unit dynamics retain lane_z and
BarrelBurst has a named audit entry. Byte decomposition accounts for all current
fields. Core test and example compilation are checked separately; the full audit
benchmark is deferred. No runtime protocol or gameplay bytes changed.

The explicit operator-only --coordinated-contract path retains all source,
provenance, schema, identity, quiet-lobby, public-health and drain guards. The
systemd timer never supplies that argument. Policy tests cover exact confirmation,
wrong contract/schema rejection, single-process refusal and a busy old match
surviving the switch and later retirement: 54 slot tests and 21 pull-policy tests
passed. No timeout, required CI gate or normal automatic compatibility guard was
relaxed. The first new fixture failed because the fake host still announced its
constant old contract; its start identity now models the intended candidate.

> blocked: the superseded hosted f058d66 test log reached runtime and exposed
> historical TW-01 revision selectors that omitted R13, missing GD-01 rows/C2
> registration, and unit-only fixtures invalidated by tripled tower fire. No
> assertion is removed and no CI gate is relaxed. INT-01 reconciles those specific
> contracts and fixture preconditions, with the exact affected target union queued.

INT-01 targeted consolidation completed: 126 scenarios across the 14 named core
integration targets passed. The initial union passed 125 and exposed one last
literal 4500 HP in the Bombard exclusion scenario; that assertion now compares
against the captured pre-combat HP and its focused rerun passed. Seven source
patch-note entries passed at R13/b66993d9e87a9cfc. The combined slot policy has
57 passing cases. No core source, balance value, timeout, required check or
historical trace row changed; no local full suite or audit benchmark was run.
