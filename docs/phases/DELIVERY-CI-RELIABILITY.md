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

## Timer startup contention observed on the official host

After the #50 controller installation all four timers shared a past two-minute
boot deadline. Default systemd coalescing scheduled backend and web together at
14:21:46 and 14:32:09 UTC; web deferred to the shared lock both times. This did not
affect existing games, but automatic frontend advancement was delayed. Stagger
activation by a minute per family and use AccuracySec=1s; keep the nonblocking
lock, random jitter and ten-minute cadence. A policy regression requires starts
farther apart than the 30-second jitter plus accuracy and no shared boot deadline.

## C7 control and transferred Faction fixtures

Hosted balance 125338013106 exposed Tuned Gears applied to SiegeWalker after
its explicit OG-01 transfer to Ogres, plus the unchanged C7 cost canary failing.
The attack-cap fixture now uses its Ranged Machine replacement RivetBallista
and still asserts the +100% cap on real hit intervals. Six Arcana cases pass.

A direct policy regression reproduced the price control discarding an already
offered expensive target for a cheap redraw. INT-02 fixes both scheduled and
fallback reaction decisions; two focused regressions pass. No C7 assertion,
threshold, sample, mutation or budget changed. A 32-pair original Bot-control
screen reports 1.000000 normal and 0.796875 tripled cost, above its unchanged
10-point sensitivity. Full C7 is deferred to the required hosted balance lane.
Counterfactual diagnostics retained: BotAvoid and ranged-spam rows were too
insensitive, and the ten-kind duel subgroup missed the 60-Energy criterion;
none was added to the test as an escape hatch.

The successful native PR run 41463262341 produced Linux and Windows packages.
Its synthetic merge source is b07cb2e4242db5df53aefc95667e09d8231372c7, not a
production release. The Linux tarball checksum and all 391 package-file hashes
passed, and --build-info reported release/profile ui-acceptance, protocol 36 /
R13 contract. It was not installed or published as an official main package.

Linux package runtime also passed one scoped five-form Colossus capture (idle,
walk, attack), 1600x1000: boot_ready=true, scene_waiting=false, zero pending
images/texts and zero failed images, normal process exit in 9.55 seconds. The
native image was inspected. This is Linux package evidence only; Windows and
browser rendered correspondence remain unobserved.

## Full-match fixture assumptions after TW-01 and SIDE-01

The canceled 05c42d9 test log exposed three independent fixture assumptions.
C4a and reconnect bots could exceed their unchanged wall deadline under the
stronger towers and random server seeds. Their synthetic fast clock is now
1 ms (already used by other integration fixtures), retaining the same complete
natural match, replay hash/outcome and wrong-seed checks; production remains
50 ms and no deadline increases. C4b still exercises 50 ms when explicitly run.

The Fortification fixture spent its initial 5 Energy on the now-6-Energy first
tier, then waited forever for an impossible accepted view. Wait for the actual
affordable server view before sending the intention; view waits now have the
existing 10-second total bound instead of restarting on every ping. No rejection
or authority check is removed.

The rematch reload fixture could read P71's frozen previous-result MatchStart
as the new game. SIDE-01 gives each game independent seats, exposing that stale
comparison. Track the previous result id and require both new seats to name the
same distinct rematch id; keep the challenge-mode and opposite-seat assertions.
The original scenario passed once locally, consistent with a seat-dependent
flake; a previous CI run failed. No runtime player behavior changes here.

Scoped online validation passed on the corrected fixtures: rematch reload 0.21 s,
real Fortification/reconnect/replay 0.47 s, bot reconnect 4.51 s, and C4a complete
natural-match replay equivalence 28.41 s. Every original outcome/hash/wrong-seed
assertion remains. No full local server/workspace suite or ignored C4b was run.

## Isolated healing control (INT-03)

The completed 747e2ad balance job 125354555002 passed the earlier SiegeWalker
canaries and failed on the pure healer: shared-roster scores 0.88 -> 0.80.
The prior test only measured a buff applied to both armies. Add the value
engine's existing BotAvoid control, retaining the old observation and identical
256-pair/+0.10 requirement. Fix its double reaction clock before interpreting
that control. No live stat or gate threshold changes.

The new absent-kind cadence regression was written, but its first local command
returned lock-busy (75); it did not run. Respect the shared lock and defer the
new controller check/full C7 to hosted CI. The earlier 32-pair screens remain
screening evidence only. Do not claim the isolated healing criterion passed.
