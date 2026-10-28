# P38 — Arcanas human validation, performance and release gate

**Depends on:** P37 reviewed; all P32–P37 reports and approvals complete.
**Branch:** `phase/p38-arcana-release`.
**Status:** planned; final gate for the optional extra after P31.
**Difficulty:** medium–high. Reviewer leads; owner decides fun and release.

## Goal and read first

Demonstrate that the extra creates understandable strategic choices on real
devices without competitive, security or operating-cost regressions.
Read `AGENTS.md`, GAME-DESIGN §6/§9, ARCHITECTURE §10/§11 and Arcanas
addendum, P14/P30/P31 validation contracts and P32–P37 reports. Inspect
implementation only where tests/profiling identify a concrete issue.

## Affected files and scope

Small verified fixes in touched core/server/client modules, examples and
tests; report and approved GAME-DESIGN/DECISIONS/ROADMAP changes; existing
release/deploy documentation and README if the owner approves publication.
No new trees, attributes, supremes, ranking architecture or content expansion.
No paid service, destructive database operation, automatic push, release
tag/public deployment or infrastructure purchase without owner authorization.

## Steps (in order; check and commit each)

### 1. Freeze the candidate and rerun evidence

Record source revision, compiled constants, default, schema/protocol/ruleset
versions, previous release build and migration path. Remove/disable default
access to any experimental tuning API. Release builds use compiled tables;
no client, account record or environment value supplies arbitrary bonuses.
Repeat P34's confirmation matrix with seed-start 1000000 and approved
policies, keeping a fresh block for any later candidate. If a fix affects
outcomes, treat it as a new candidate: increment
ruleset, repeat affected simulations, and retain the rejected results.
**Check:** reproducible manifest, no unresolved universal/mandatory choice,
at least three viable presets, satisfactory default and tested counterplay.

### 2. Controlled CPU, memory, traffic and browser budgets

Use release binaries after warm-up, no competing compiler/load generator on
the measured server. Run the same legal saturated scenarios (currently 16
one-slot units total), with neutral/default/adversarial presets and healing,
splash, Synergy transitions and control. Time stepping separately from
view construction, event observation and harness I/O. Five repetitions;
report p50/p95/p99/max and sample count, not only a mean.

Proposed regression gates: simulation p95 ≤50 us; Arcana p95 increase
≤max(1 us, 10% of controlled neutral p95); extra owned memory <1 KiB per
match; no Arcana-specific allocation during a tick. Measure structure and
owned-heap deltas and profile allocations with existing tools; size_of alone
does not establish heap use. If instrumentation is unavailable, label the
claim unmeasured rather than inventing zero allocations.

Measure MatchStart/reconnect payload and steady-state view bytes, reconnect
frequency, RTT/view-gap/ack latency and the existing per-player bandwidth
budgets (≤6 KB/s down, ≤0.5 KB/s up). Recheck P30's WASM/asset/load/FPS
budgets on desktop, mid/budget Android and iPhone 11-class. Keep P30's
synthetic 30-unit rendering stress separate from legal simulation counts.
Profile before adding a cache; any cache needs buy/Evolution/Synergy
invalidation tests and must still satisfy the memory budget.
**Check:** before/after measurements meet the gates on the actual targets;
resized desktop screenshots do not establish phone performance.

### 3. Security, data safety and multiplayer regression

Repeat manipulated ranks, budget overflow, unknown schemas/rulesets,
oversized/trailing/malformed frames, attempted seat/account spoofing,
mid-queue/room/match edits, reconnect/takeover, late persistence replies,
failure/rollback and historical-row migration tests. Verify the default
constructor is not silently used by an online/practice Arcana path.

On the real validation server repeat P14's reconnect/drop-and-resume matrix
and at least 50 Ranked bot matches with accounts/presets. Finish human
two-client Casual and Ranked games, including a reconnect and forfeit.
Compare packet presets, actual stats, stored result and RatingChanged.
Check the P13 ranking/profile pages still read old/new records correctly.
Rehearse backup/restore and rollback with a copy of SQLite before touching
production; preserve real data and existing P14 safety requirements.
**Check:** no client controls final bonuses/outcomes, no partial ranked
settlement, leaked credential, orphaned preset or changed historical rating.

### 4. Human crossover playtest

At least 8 participants, including newcomers and experienced players, and
24 completed online matches across desktop and actual phones. Use complete
valid presets in both queues; neutral is an offline diagnostic, not a
public selection. Counterbalance default-versus-player-selected presets,
seat and order where possible. Compare prior P31/P32 human metrics with
appropriate caveats instead of equating bot improvements to human fun.

Record comprehension before play (affected units, point budget, freeze),
decision changes while buying/evolving, build variety, chosen counters,
defeat explanations, fun/readability scores and Play again behavior.
Use GAME-DESIGN §9's ≥80% explanation target and §6's existing fun metrics.
Inspect whether players actually adapt or follow an obvious allocation;
counting 141 legal vectors does not prove strategic depth.
Inspect control/Support stalls and offer droughts with players, not only bots.
**Check:** owner records whether choices feel meaningful, defeats fair and
interface readable. If choices are unnoticeable/compulsory, stop release
and simplify/revisit the mutually-exclusive-talent alternative in D-25.

### 5. Tuning decisions and final regression

Record every balance/product decision with its evidence, Proposed status
and owner approval. Existing missed targets (for example baseline Siege
Walker purchase share) need an explicit decision, not an automatic waiver
because they predate Arcanas. A changed target must be documented as a
proposal in the same commit. Never claim the small playtest establishes
precise population win rates; retain statistical uncertainty and bot bias.
Repeat affected P34 confirmation and human checks after meaningful changes.
**Check:** all critical/major risks closed with evidence and no required
check, device, rule decision or subjective approval marked implicitly done.

### 6. Prepare a concrete release, then request the final authorization

Prepare the tested source/build revision, binary/static bundle, additive
migrations, backup/restore/rollback commands, changelog and rule-version
mapping before requesting release approval. Old result rows remain valid;
do not recalculate ratings. Historical presets/versions enable audit, not
full replay without timed intentions and the old implementation.

Use the existing single-server deployment. Drain matches before switching
compiled rules; do not hot-patch running matches or add a multi-version
simulation service. Release client and server coherently; stale clients
must be rejected and offered reload. Do not lower server validation during
rollback; keep the last compatible artifact and database backup available.
Once explicitly authorized, follow the existing deploy process, run public
smoke checks and record actual results. The owner handles pushes/tags;
never push or rewrite history. Do not invent a release version in advance.
**Check:** owner approval and deployed smoke evidence are recorded. Without
authorization, leave a reviewable release candidate and report the pending
owner action; never mark public release complete.

## Verification

```bash
cargo fmt --all -- --check
cargo clippy --workspace -- -D warnings
cargo test --workspace
cargo check -p tbg-client -p tbg-core --target wasm32-unknown-unknown
```

Also run Trunk release, actual browser/device checks, desktop/phone gallery
and editor screenshots, two-client/reconnect matches, and the documented
simulation/security/performance commands with their real outputs.

## Acceptance and reviewer checklist

- [ ] Statistical, human-choice and existing fun targets have evidence/decisions.
- [ ] CPU/memory/traffic/load/FPS budgets and device matrix pass.
- [ ] No pay-to-win/account advantage, untrusted stats or mid-match respec.
- [ ] Versioned presets/results survive migration, reconnect and failure tests.
- [ ] Owner approves fun/look/fairness and separately authorizes release.
- [ ] Real release smoke results recorded before marking the extra complete.

## Phase report

_Not executed. Fill in frozen revision/versions, all commands and datasets,
statistical confirmation, CPU/heap/network and device matrices, security
results, human cohort/matches/survey/choice findings, owner decisions,
release candidate/rollback details, authorized rollout/smoke results,
limitations, deviations and pending actions. Separate local completion
from public release and never manufacture missing participant/device data._
