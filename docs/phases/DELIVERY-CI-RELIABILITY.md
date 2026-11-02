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

INT-04 avoids rebuilding unrelated mutant duel rows: the full reference matrix
stays, and every consumed target/opponent/budget/seat result is measured through
the same pair builder. A new exact equivalence test protects row scores,
averages and weak/oppressive classification for all three mutation cases.
No samples, duel duration, prices, gate thresholds or canary assertions change.
The test-only helper is not exported by the balance library or used in reports.
Local compile/run is deferred to hosted CI while the shared host validation
front remains contended; no lock polling, extra target or parallelism bypass.

## Hosted result at 3fb894d (2026-11-02 16:02 UTC)

Native Linux/Windows, quality, clippy, WASM and desktop UI passed. The
complete required test lane passed all targets except the stalled-spectator
fixture: its natural match exceeded the existing 120-second bot budget.
BR-M07 imposes no match deadline; use the existing spectator replay's
20-second surrender, preserving both completion and no-detachment assertions.
Production clocks, bots and spectator handling do not change.

INT-03 cadence and INT-04 exact-row equivalence passed in the balance lane.
C7 still failed Bloomkeeper: shared 0.88 -> 0.80; isolated 0.93 -> 1.00.

> blocked: Two completed healing-canary runs miss the unchanged +0.10
> requirement. The isolated baseline leaves less than 0.10 headroom. Do not
> lower a threshold, change live balance or keep adding convenient policies.
> Owner confirmation was requested for a controlled wounded-army healing
> fixture at the same 256 pairs/+0.10 criterion. The protected merge remains
> blocked; the current healthy VPS generation is preserved.

Scoped validation: `cargo test -p tbg-server --test spectate
a_spectator_that_never_reads_is_dropped_and_the_seats_finish -- --exact
--nocapture` passed (1 scenario, 20.02 seconds) under the shared lock with two
build jobs. The shared spectator helper and production behavior are unchanged.

New front discovered during monitoring: PR #51 at 5357045 adds P74 economic
auxiliaries and builds on the earlier champion head. It conflicts with main.
Read-only review found appended wire view fields without a protocol bump,
a second incompatible R13 identity, and auxiliary state skipped by serde.
These must be reconciled and explicitly validated before that front can be
released; do not merge it solely because it includes the champion ancestry.

## Exact CROWD-EXIT-01 integration requested by the owner

Merge 73c7a07c28833d6663a11dda65332a578a8c4a3a normally, preserving
its complete overhead, live-body and regression changes. Resolve the
earlier 41ebc27 formation fix in favor of HOME_HALF_WIDTH, retaining
its two useful regressions. TUNDRA-01 9233cd021e5b9af04976bba1f6422f3e2ab3bc21,
SIDE-01 11cf5af323975f738992147dc76b71f0b2475fbd and 683477aeb3f5
were already ancestors. Two supplied abbreviated hashes differed from
the actual objects; exact requested terminal SHA and its ancestry were used.
The later GD-03 and P74 commits remain in PR #51 and are not silently
included in this explicitly bounded integration.

PR heads #41 bf6da86, #43 19f4cfb and #44 9fb7be0 are exact ancestors of
this consolidated branch. Do not discard or close them as merged into main
before main actually contains those histories. No branch is deleted.

Local focused union (client crowd, side01_, envelopes, gd02_, Shatterback
clearance; core practice_bot side01_; server ranked side01_) used
`flock -n -E 75 /tmp/tbg-validation-1000.lock` and CARGO_BUILD_JOBS=2.
It returned 75 before any command ran: deferred, shared validation slot busy.
Do not poll or bypass the lock. Rustfmt on the affected files, source routing
and seven patch-note entries passed; git diff --check passed.
The main checkout's nine dirty/untracked files were hashed before integration;
no checkout, stash or reset was performed there.

## INT-05 controlled-fixture result (owner approved)

The owner approved the controlled wounded-army scenario without reducing
the 256 paired seeds, +0.10 gain or any live values. The dedicated regression
passed under the shared lock with CARGO_BUILD_JOBS=2:
`cargo test -p tbg-balance --test wounded_army_control -- --nocapture
--test-threads=1` (1 passed, 11.04 s). Its predeclared grid covers all nine
original damaging kinds, budgets 20/40/60 and 25/50/75 percent starting HP,
with equal Energy and both physical seat orders. Baseline score 0.273438;
x2 HP/heal score 0.519531 (+24.609375 percentage points). No parameter sweep,
live rule adjustment or acceptance claim is involved. Full hosted C7 is
still required; the earlier shared/isolated observations remain in its
assertion and diagnostics. The new helper is test-only.

## INT-06 — P74 and GD-04 integrated, 2026-11-02

The owner extended consolidation to PR #51 and both new Gods/Colossus from
PR #52. Exact published heads 5357045 and 74473cc were merged normally;
#52's later evidence-only head is reconciled next. No worktree owned by
another front was edited. The main checkout's original nine dirty/untracked
files and HEAD were rechecked unchanged.

Contract: w38-r15-cc459351581b7763, native 0.1.4-colossus.1, database schema 10
(history schema 11 remains disabled). Preview roster 84, online FULL_POOL 46.
The landing derives its actual 84 champions and ten Factions from core/i18n.
Volthar, Astreon, the refined five-form Colossus, and GD-03 are retained.
HOME_HALF_WIDTH and complete 73c7a07 overhead/live_body changes remain.

P74 repairs discovered during integration: auxiliary phase/free currency now
serialize and affect deterministic hashes; owning reroll quotes survive the
wire; successful auxiliary intentions are handled by server telemetry;
construction follows the authoritative countdown and freezes with Augment
selection. Decorative animation respects low motion. No economic or combat
value was changed.

Scoped checks passed under the shared nonblocking lock, CARGO_BUILD_JOBS=2:

- Core auxiliary state/wire/privacy/pause regressions 3; protocol 21; identity 2.
- Auxiliary rules 3; GD-04 combat 12.
- R15 determinism/lab/legacy/constant/Stronghold union 26.
- All five observed reference games keep R14 end ticks and winners; new hashes
  reflect serialized auxiliary fields. Explicit R15 rows were appended.
- Real WebSocket P74 case 1 (0.55 s): Red seat, slot rejection, paid hiring,
  three purchases/free reroll, reconnect, one-hire rejection, Miner construction,
  opposite-seat ownership/privacy and equal end outcomes.
- Client presentation 35: auxiliary HUD and all-eight-Arena precinct geometry 2;
  Gods/Colossus finite forms, grips and clearance 8; crowd 11; sides 4; envelopes 2;
  widened Arena clearance 2; bilingual catalog coverage 6.
- Strict core/client/server scoped clippy passed (one P74 lantern vec-init lint
  corrected without changing any vertices); server tests compile.
- Web source/catalog checks and patch notes passed (8 entries).

The prior 267593e CI is fully green, including complete hosted C7, Linux and
Windows packaging. That evidence does not validate this extended head; its
protected CI and native factories are still required before merge/publication.
No local full suite, complete balance gate or cross-platform rendered parity
claim is made. Official artifacts/application/timer-only proof remain pending.

Packaged ui-acceptance client compilation also passed. The view-audit example
now carries auxiliary state, private free rerolls and quotes through every
prototype and exact header accounting; its focused compilation passed.
The audit benchmark itself was not executed.

The first extended CI at 8fc4a04 exposed two missing balance-lab integration
hooks: exhaustive Ruleset classification and successful-intention accounting.
P74's owner-fixed auxiliary profile is explicitly untunable by the generic
search (whose reference Bot never hires auxiliaries); dedicated hiring policies
are required for useful valuation. Validated hires now count their actual
Energy cost in runner spending. No gameplay value, search bound, sample or
gate threshold changed. These scoped fixes are checked before republication.

Balance integration fix validation passed: strict balance-library clippy,
C5 exact preflight (1), wound/cadence/Favor control union (4), and the
R15 Ranked SIDE-01 command/reconnect/account-settlement case (1).
These use the shared lock and two jobs; the hosted complete lane remains
mandatory and no unrelated local suite was run.

CI a557c6a found the legacy core balance example's exhaustive intention
match. Propagate HireAuxiliary there without treating it as a champion
purchase. Its ten-entry purchase counters are also sized to UNIT_KIND_COUNT
so canonical purchases cannot index outside them. All core examples now
compile with lab enabled; no benchmark or example simulation was executed.

The next full hosted test lane at 4721870 passed every other target and
found a training-only count fixture: 190 forms were exercised successfully
but the final literal still required 180. GD-04 adds two preview champions
with five forms each. Keep every scenario and assert five forms for every
preview-only kind, deriving the active/preview boundary from FULL_POOL.
This test is enabled by training, so a lab-only core run did not cover it.
The same head passed the complete hosted balance lane, including C7.

The 190-form scenario passed with the CI-equivalent lab+training feature
combination and its fully qualified exact name. An earlier training-only
command selected zero tests and is not counted as evidence. Source hash
remains cc459351581b7763 because completion/tests.rs is classified test-only.

The 5e1bee4 test lane exposed two intermittent client-fixture races. Army
sync sent buys during the existing Augment selection and assumed every
rejection was Energy. Preserve the no-cap assertion, handle the legitimate
Paused race, send only after acknowledgement, and end by controlled surrender
after twelve purchases and a real synchronized death (BR-U07/BR-M07).

The slot test left its finished sockets unread while waiting for directory
idle; they could expire and create new reconnect receipts. Keep all four
clients answering pings through the unchanged retirement assertions. Add
exact load diagnostics on timeout; no production gate, grace or timeout is
relaxed. The first local diagnostic attempt returned lock-busy (75) and
did not execute; no lock polling or bypass was used.

The revised local fixture check also deferred with lock-busy (75). Formatting
and diff inspection passed; execution is delegated to the protected hosted
lane. Keep the exact load diagnostics if the slot wait recurs rather than
relaxing a retirement guard. The prior failure log did not include the
blocking counter, so unread-client expiry is a fixture-hardening hypothesis,
not a claimed production root cause. All production source and R15 values
remain unchanged by this fixture-only commit.


At a1fceb5 the army and guest-slot scenarios passed hosted CI; the equivalent
Ranked fixture still left its four sockets unread during the idle wait.
Reuse the responsive client loop there, preserving the two exact-once match
rows, all account game counts and the unchanged zero-pending retirement gate.
If the wait recurs, print the ranked, reconnect-receipt, history and Store job
components directly from the test's Shared handles. No production code or
retirement threshold changes. Local Rust execution remains deferred because
the shared slot is owned by another work front; protected CI executes it.
