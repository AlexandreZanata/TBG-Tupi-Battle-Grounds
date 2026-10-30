# P30-D — Deterministic server performance investigation

**Owner request:** 2026-10-30, deep server performance investigation.
**Baseline:** `d06279aa5287b67efb24e224c4710db55a8ca739`.
**Branch:** `phase/p30d-server-performance`.
**Scope:** experimental benchmarks, exact-behavior prototypes, research,
and a reviewable implementation plan. No merge, push, deploy, production
configuration changes, gameplay changes, or balance changes. The public
VPS is read-only; stress and capacity experiments are local only.

This supplements P30-B rather than silently expanding its server-only
contract. It covers the current 25-kind roster, support abilities,
Strongholds, Augments, Arcanas, snapshots, and authoritative input paths.

## Read first

- `AGENTS.md`; owner investigation attachment (2026-10-30).
- `docs/ARCHITECTURE.md` sections 4, 5, 10, 11, 12.
- P10 report; P16-B performance report; P30-B spec.
- Current core tick, targeting, support, effects, state, view, and protocol.
- Server match loop, connection routing, channels, metrics, persistence.

## Steps and checks

1. Preserve baseline provenance, existing test failures, environment and
   historical methodology. Re-run `army_load` in release. Check that no
   production artifact changes; record historical limitations explicitly.
2. Add a small reproducible native benchmark using existing dependencies.
   Measure 10/20/50/100/200/300/500/1000 total conventional units, multiple
   seeds/compositions, tick distributions, allocations, views, encoding,
   state sizes, and bytes. Separate instrumented runs from timing runs.
   Synthetic setup and top-ups are outside timed ticks and documented.
3. Audit algorithmic complexity and operation counts, profile available
   tools, and consult primary technical sources. Do not reopen P10's
   rejected allocator/encoding experiments without changed conditions.
4. Test one isolated optimization at a time, only after its baseline is
   preserved. Compare complete serialized state, events, and both views
   at every tick of deterministic adversarial traces; preserve live
   movement/death ordering and all targeting ties. Keep only reproducible
   improvements outside observed noise, with no correctness regression.
5. Investigate sequence-zero/duplicate commands and command-drain work
   bounds locally. Record confirmed defects separately from performance
   wins; never change a contract test or silently change protocol rules.
6. Re-measure local server capacity with load generation outside the
   server CPU affinity and resource constraints, where supported. Report
   unavailable counters/limits honestly. VPS measurements remain light,
   read-only observations of existing services.
7. Complete the report below and a concise, evidence-based follow-up plan.
   Run formatting, clippy, core/server tests, and WASM checks. Client
   builds/browser validation are needed only if client source changes.
   Commit logical steps independently; stop for review without deploying.

## Acceptance

- Historical claims identified by introducing commit, method, and known
  hardware limitations; current measurements do not inherit those claims.
- Reproducible release commands and compact raw benchmark results kept.
- Original baseline and each candidate measured independently; algorithms,
  operation counts, differential traces, memory and bandwidth documented.
- Full serialized-state/event/view equality, including deaths, spawns,
  healing, carries, evolutions, pauses, and Stronghold interactions.
- No rejected speculative code/dependencies left behind.
- Audit distinguishes measurements, external evidence and untested ideas.
- Tests/security limitations and unavailable tools reported exactly.
- No modification to the live VPS, shared services, or other projects.

## Phase report

Executed 2026-10-30. Research and experiments are complete; independent review is pending. Nothing was merged, pushed or deployed. Production stayed on the original baseline. No gameplay values, protocol variants, dependencies or contract tests changed. The `bench-alloc`/`bench-profile` features are opt-in examples/instrumentation, never enabled by the online crates.

**Measured outcome:** 1,000 local matches under 1 CPU / 1 GiB fell from about 294 to 49 MiB RSS and 0.526 to 0.297 busy cores after right-sizing reads, with unchanged 3-ms RTT p95 and zero late ticks. The 2,000-match 60-second window also had zero late ticks after that change. Dense formation CPU and allocation improvements are separate from this normal-workload server gain. Production remains unchanged.

### Provenance and historical audit

Baseline game commit: `d06279aa5287b67efb24e224c4710db55a8ca739`, 25 conventional kinds and protocol v14. Current machine: Pop!_OS 24.04, Intel Core i7-13620H, 16 logical CPUs / 10 physical cores, about 31 GiB RAM; Rust 1.99.0 (`b940084d7`), Cargo 1.99.0, LLVM 23.1.1. Main native samples use logical CPU 2, release `opt-level="s"`, thin LTO, one codegen unit. Dependencies: Tokio 1.54.1, Axum 0.8.9, tungstenite 0.29.0, Postcard 1.1.3.

- P16-B's 45.4 µs mean / 53.5 µs p99 / 28.6 KB/s comes from `38fb0bb` and `examples/army_load.rs`, 1,200 ticks, 100 conventional units per side plus summons. The report identifies only a development PC; CPU model, governor, exact Rust version and raw per-tick samples were not preserved. These are historical observations, not current guarantees.
- P10's 1,500 matches / 1.47 cores / about 300 KB per match comes from report commit `7132b1e`, a 16-logical-CPU / 31 GB development machine and a 40-second steady window. It was not a 1-vCPU cgroup result. Its CPU-model and compiler provenance is incomplete.
- Running the unchanged historical army harness against the baseline today produced 337.1 µs mean / 461.5 µs p99 / 31.7 KB/s at about 203 live units. The current roster, effects, Augment pauses and state size differ, so the ratio is not an isolated algorithm regression claim.
- P10 already rejected `opt-level=3`, encode-once and mimalloc on small-army server throughput. No allocator dependency was reintroduced. The compiler experiment was reopened only for the changed dense-army CPU workload; no production build profile was changed.

### Reproduction and workload limits

```bash
# Reference core/harness: ca5069e in an isolated checkout; final candidate: this branch.
# For 1,400-step reference traces, take fixture.rs from 0a5af9e (pause handling only).
# This workspace already preserves baseline-v2 and buffer binaries under .local/p30d/.
cargo build --release --locked -p tbg-core --examples
cp target/release/examples/performance /tmp/tbg-reference
# Capture this before applying a candidate; each candidate has a separate binary.
python3 scripts/performance.py --binary /tmp/tbg-reference --label baseline --ticks 120
cargo build --release --locked -p tbg-core --example performance --features bench-alloc
# Allocation/operation counts are a SEPARATE run; never compare its time to native time.
python3 scripts/performance.py --binary target/release/examples/performance --label allocation --ticks 20 --seeds 1
python3 scripts/performance.py --binary /tmp/tbg-reference --compare /tmp/tbg-candidate --ticks 1400 --sizes 20,200
```

The eight sizes are **total target conventional units**, split equally between seats. Actual post-step population is logged and can differ through deaths/summons. Seven compositions cover melee/Tanks/Guard, ranged, healing/status, all 25 conventional kinds, adversarial formations, Puppetmaster ranking and cargo/Nester summons. Three seeds (1,7,42), 120 timed steps each, 20 Hz simulation / 10 Hz sampled views. Setup, top-ups, intentional Evolution and view/encoding work are outside tick timing. There is no warmup exclusion: initial cold/setup-sensitive ticks remain in the distributions.

These fixtures are artificial: direct offer selection, replenished Energy, starting formations, wounded allies, existing Augment copies, evolved troops and high Stronghold HP keep populations fighting. They are not normal-game duration, balance, or population forecasts. Long traces use 1,400 steps and normal Forgo/selection resolution to cross Augment pauses. Initial fixture bugs (buying summon kinds and top-ups during a pause) were corrected before accepting any candidate; failed attempts remain in private logs. No canon value was edited.

Full-state traces compare length-framed serialized state, events, every unit's effective stats, both players' full views and derived ArmyComposition **byte for byte at every step**. Each accepted core variant passed 58,800 such comparisons, plus the unchanged frozen full-match hash contract. New tests prove that movement into range and Guard death earlier in the same tick remain visible to later units. Sorting a tick-start snapshot would violate these contracts.

Three additional interleaved baseline/final repetitions on seed 1 at 20/200/1,000 units were preserved locally. Uncontrolled clock/thermal/cache noise remains; large speedups and exact allocation/operation reductions are stronger evidence than small isolated percentage changes. Small timing differences are not capacity claims. The final unpaired 10-unit ranged/healing samples were about 5–8% slower (sub-0.2 µs absolute difference); do not hide those rows or claim a universal tick win. End-to-end ordinary server CPU improved substantially only after the separately tested read-buffer change.

### Complete tick measurements

Times are medians of each seed's statistic. Worst is the largest observed sample across seeds. Speedup is the median **paired seed ratio**, which can differ slightly from the ratio of the displayed medians. Source rows, actual population, bytes/view, view/encoding times and counter modes: [armies.csv](../performance/p30d/armies.csv). Baseline binaries are preserved privately; [provenance.json](../performance/p30d/provenance.json) records hashes and commands. `buffer` is the final core candidate; `final` is the earlier allocation-heavy variant, retained as experimental evidence only.

| Scene | Target units | Before mean µs | After mean µs | After p50 µs | After p95 µs | After p99 µs | Worst observed µs | Paired speedup |
| --- | --- | --- | --- | --- | --- | --- | --- | --- |
| melee | 10 | 2.05 | 1.99 | 1.75 | 3.17 | 5.09 | 647.30 | 1.00× |
| melee | 20 | 4.92 | 4.36 | 3.99 | 7.75 | 8.49 | 15.66 | 1.13× |
| melee | 50 | 19.46 | 14.82 | 13.28 | 24.08 | 28.50 | 47.28 | 1.31× |
| melee | 100 | 61.70 | 38.99 | 35.27 | 66.10 | 81.16 | 128.46 | 1.60× |
| melee | 200 | 221.44 | 118.98 | 104.35 | 200.04 | 237.69 | 288.55 | 1.90× |
| melee | 300 | 515.39 | 232.35 | 216.96 | 340.01 | 385.75 | 505.30 | 2.24× |
| melee | 500 | 1476.15 | 608.05 | 608.82 | 832.46 | 862.20 | 1007.43 | 2.43× |
| melee | 1000 | 6644.01 | 2246.30 | 2297.01 | 2945.78 | 3128.19 | 3292.92 | 2.97× |
| ranged | 10 | 2.13 | 2.25 | 1.90 | 3.62 | 5.55 | 15.92 | 0.95× |
| ranged | 20 | 5.15 | 5.09 | 4.13 | 9.81 | 12.89 | 23.00 | 1.01× |
| ranged | 50 | 19.71 | 15.54 | 12.75 | 35.52 | 48.75 | 69.85 | 1.27× |
| ranged | 100 | 57.50 | 38.84 | 29.35 | 93.09 | 103.89 | 141.42 | 1.49× |
| ranged | 200 | 216.26 | 114.25 | 106.54 | 230.54 | 255.94 | 328.16 | 1.89× |
| ranged | 300 | 509.83 | 223.59 | 213.63 | 398.83 | 459.78 | 540.16 | 2.25× |
| ranged | 500 | 1322.55 | 551.60 | 550.16 | 829.34 | 951.51 | 1057.76 | 2.36× |
| ranged | 1000 | 5521.68 | 1884.04 | 1933.66 | 2935.49 | 3027.59 | 3329.88 | 2.92× |
| healing | 10 | 1.72 | 1.86 | 1.68 | 2.62 | 5.25 | 9.68 | 0.93× |
| healing | 20 | 4.00 | 4.12 | 3.84 | 5.44 | 6.86 | 11.27 | 0.98× |
| healing | 50 | 15.57 | 13.56 | 13.08 | 17.98 | 23.65 | 29.48 | 1.14× |
| healing | 100 | 54.48 | 40.10 | 40.94 | 52.76 | 63.87 | 82.84 | 1.36× |
| healing | 200 | 200.22 | 125.03 | 132.94 | 153.18 | 167.39 | 258.47 | 1.63× |
| healing | 300 | 442.27 | 257.41 | 278.75 | 317.70 | 352.38 | 553.06 | 1.75× |
| healing | 500 | 1198.65 | 637.88 | 683.52 | 822.18 | 865.50 | 953.26 | 1.87× |
| healing | 1000 | 5061.83 | 2476.57 | 2483.34 | 3128.08 | 3181.94 | 3348.18 | 2.09× |
| mixed | 10 | 2.02 | 2.01 | 1.72 | 3.18 | 5.13 | 8.58 | 1.03× |
| mixed | 20 | 5.32 | 4.81 | 4.31 | 8.44 | 10.73 | 17.48 | 1.11× |
| mixed | 50 | 22.28 | 18.02 | 16.05 | 31.68 | 38.33 | 73.84 | 1.24× |
| mixed | 100 | 71.08 | 47.82 | 41.30 | 89.06 | 105.01 | 220.03 | 1.49× |
| mixed | 200 | 225.03 | 130.25 | 84.99 | 310.29 | 345.96 | 405.18 | 1.76× |
| mixed | 300 | 473.52 | 248.07 | 156.80 | 579.79 | 619.17 | 689.71 | 1.98× |
| mixed | 500 | 1257.24 | 597.45 | 417.53 | 1199.05 | 1308.56 | 1572.37 | 2.11× |
| mixed | 1000 | 5473.60 | 2268.67 | 1547.58 | 4023.70 | 4276.82 | 4706.50 | 2.46× |
| formation | 10 | 2.60 | 2.12 | 2.02 | 2.35 | 3.56 | 9.76 | 1.23× |
| formation | 20 | 7.25 | 5.07 | 4.90 | 5.95 | 7.57 | 11.96 | 1.38× |
| formation | 50 | 45.09 | 22.55 | 22.06 | 25.81 | 26.99 | 35.20 | 1.97× |
| formation | 100 | 242.66 | 76.33 | 74.74 | 88.52 | 113.05 | 132.79 | 3.11× |
| formation | 200 | 1358.83 | 276.60 | 266.77 | 328.40 | 366.12 | 385.78 | 4.93× |
| formation | 300 | 3962.11 | 596.65 | 581.68 | 679.44 | 740.81 | 838.54 | 6.53× |
| formation | 500 | 15778.93 | 1642.46 | 1615.39 | 1863.52 | 1997.45 | 2151.33 | 9.52× |
| formation | 1000 | 117372.66 | 6561.45 | 6479.39 | 7529.83 | 7961.64 | 8150.09 | 17.90× |
| puppet | 10 | 2.59 | 2.56 | 2.41 | 3.38 | 4.38 | 8.26 | 1.03× |
| puppet | 20 | 6.67 | 6.07 | 6.04 | 7.33 | 9.22 | 13.45 | 1.10× |
| puppet | 50 | 31.43 | 28.68 | 28.95 | 34.05 | 40.51 | 76.65 | 1.09× |
| puppet | 100 | 127.77 | 100.97 | 104.67 | 113.54 | 115.75 | 205.88 | 1.19× |
| puppet | 200 | 401.10 | 330.94 | 348.22 | 415.75 | 435.56 | 733.09 | 1.24× |
| puppet | 300 | 1134.26 | 777.67 | 824.56 | 896.53 | 920.25 | 1615.59 | 1.40× |
| puppet | 500 | 2764.59 | 2095.99 | 2205.63 | 2509.22 | 2593.24 | 4444.14 | 1.33× |
| puppet | 1000 | 11521.23 | 8168.51 | 8259.21 | 10580.49 | 10937.30 | 17693.71 | 1.40× |
| cargo | 10 | 2.15 | 1.97 | 1.78 | 2.88 | 3.95 | 8.46 | 1.09× |
| cargo | 20 | 5.22 | 4.47 | 4.00 | 7.53 | 8.58 | 12.40 | 1.17× |
| cargo | 50 | 20.79 | 14.19 | 13.34 | 21.12 | 23.84 | 30.91 | 1.47× |
| cargo | 100 | 66.76 | 37.00 | 36.04 | 51.98 | 63.28 | 76.34 | 1.78× |
| cargo | 200 | 247.87 | 110.42 | 105.83 | 164.15 | 188.75 | 263.12 | 2.24× |
| cargo | 300 | 584.20 | 218.55 | 196.89 | 343.52 | 412.19 | 620.29 | 2.71× |
| cargo | 500 | 1558.15 | 530.44 | 501.82 | 821.76 | 1010.60 | 1208.87 | 2.94× |
| cargo | 1000 | 6452.93 | 1914.75 | 1882.17 | 3253.39 | 3922.34 | 4889.72 | 3.40× |


### Scaling and algorithm proof

The baseline nearest-enemy loop did an ID binary search for every already-borrowed candidate: O(N² log N). `unit_index` itself was already O(log N), not linear. Reading the candidate's live carrier field eliminates that redundant lookup and restores this path to O(N²).

The old formation loop was O(M·F·N): every moving unit M checked allied formations F, each scanning all N units for engagement. With proportional formation populations this is O(N³). The 1,000-unit adversarial counter measured 249,500,000 engagement candidates per tick, versus 1,980,000 at 200 (about 126× for 5× units, matching the loop structure). The final code uses O(N) static ability discovery plus current-position sorting and range queries for each mover: worst-case O(N² log N). It rebuilds positions after every preceding unit's movement/death, reusing storage; no stale engagement cache exists.

Drumguard selection collected and sorted all allies merely to take the first. Lexicographic `min()` preserves distance/id ties and hysteresis, reducing O(D·N log N) to O(D·N), with no candidate allocation. None of these changes claims O(N) total combat.

The descriptive normalization below uses target population, not a causal per-unit marginal cost. Different sizes change the composition of a finite fight. Complexity follows the source loops/counters; curve fitting alone is not proof.

| Scene | Target units | µs/target unit | Incremental µs/added target unit | Time / time at N/2 |
| --- | --- | --- | --- | --- |
| mixed | 10 | 0.2008 | — | — |
| mixed | 20 | 0.2405 | 0.2803 | 2.40 |
| mixed | 50 | 0.3604 | 0.4403 | — |
| mixed | 100 | 0.4782 | 0.5960 | 2.65 |
| mixed | 200 | 0.6512 | 0.8243 | 2.72 |
| mixed | 300 | 0.8269 | 1.1782 | — |
| mixed | 500 | 1.1949 | 1.7469 | — |
| mixed | 1000 | 2.2687 | 3.3424 | 3.80 |
| formation | 10 | 0.2119 | — | — |
| formation | 20 | 0.2535 | 0.2950 | 2.39 |
| formation | 50 | 0.4510 | 0.5827 | — |
| formation | 100 | 0.7632 | 1.0755 | 3.38 |
| formation | 200 | 1.3830 | 2.0027 | 3.62 |
| formation | 300 | 1.9888 | 3.2005 | — |
| formation | 500 | 3.2849 | 5.2291 | — |
| formation | 1000 | 6.5614 | 9.8380 | 3.99 |


### Allocation and memory evidence

Measured with opt-in counters in a separate build. Zero counter fields in native timing/commands rows mean instrumentation was disabled, not that those operations allocate nothing. Requested bytes count successful allocation/reallocation requests, not net heap growth. Peak live allocation includes the fixture/driver; it is not per-match server RSS. The counting allocator only forwards System's pointer/layout contract; its small `unsafe` surface is confined to the benchmark example. No unsafe simulation code was added.

| Scene | Units | Before allocations/tick | After allocations/tick | Before requested B/tick | After requested B/tick | Before ID searches/tick | After ID searches/tick |
| --- | --- | --- | --- | --- | --- | --- | --- |
| mixed | 200 | 54.450 | 21.800 | 25628.000 | 14642.400 | 7670.00 | 532.85 |
| mixed | 1000 | 368.800 | 101.050 | 459191.000 | 186618.200 | 163166.45 | 3323.80 |
| formation | 200 | 306.000 | 13.000 | 102992.000 | 7024.000 | 20397.50 | 197.50 |
| formation | 1000 | 2008.000 | 17.000 | 2050064.000 | 30384.000 | 501987.50 | 987.50 |


The first ordered-formation prototype was rejected in its allocation-heavy form: 6,419.2 requests / 3,292,016 bytes per tick at 1,000 units. Reusing one live-position scratch buffer brought that to 17 requests / 30,384 bytes, preserving the same range comparisons. The baseline was 2,008 requests / 2,050,064 bytes. This resolves the CPU-versus-allocation tradeoff rather than hiding it.

Measured native type sizes: Unit 180 B; UnitFx 108 B; UnitView 24 B; Event 16 B; fixed MatchState 568 B (excluding heap). At 200 units, the unit vector alone is about 36 KB, not the historical 12 KB. Fx occupies 60% of Unit; this motivates hot/cold experiments, not an automatic model rewrite.

### Profile and ranked bottlenecks

Valgrind 3.22 was downloaded from Ubuntu's package repository and extracted under `/tmp`; no permanent profiler dependency or host/VPS package installation. Identical 200-unit formation/mixed/puppet, 40-step commands recorded 1,859,196,194 instructions before and 632,907,285 after. These are **Callgrind instruction counts**, not CPU cycles or isolated-tick time; setup/views/encoding are included. Inlined formation work appears within MatchState::step.

Baseline top self-instruction groups: MatchState::step 57.18%; unit_index 9.14%; enemy_in_range 8.21%; augment::unit_bonus 7.10%; Ability::contains 4.84%; stats_with 3.46%; targetable 1.45%; stats_of 1.11%. Global cache simulation: 1,480,470 D1 misses and 3,511,637 branch mispredictions. The model uses 32-KiB D1 / 64-KiB I1 / 24-MiB LL; these are simulated, not hardware performance counters. After optimization the same profile recorded 117,935 simulated D1 misses and 2,168,001 simulated branch mispredictions. Native `perf` was unavailable. No actual hardware cache/branch-miss, timer wakeup, context-switch or command-channel drain-time distribution is claimed. The command experiment measures core validation/application only; mixed 80-command bursts took about 10.8 µs at 200 initial units and 16.1 µs at 1,000 in the recorded run, excluding transport/routing/ack construction.

CPU: redundant target lookups and cubic formation engagement dominated dense armies; repeated effective-stat calculations remain important for Puppetmasters. Memory: socket read buffers dominate normal concurrent matches; temporary formation/support vectors dominated dense-tick allocation. Network: full snapshots still scale with the visible army. Scheduling: ordinary bot capacity did not show a reproducible CPU improvement from the core changes; kernel/runtime fan-out remains significant. A 60-second capacity window cannot prove indefinite stability.

### Subsystem audit

| Subsystem / actual entry point | Algorithm / issue | Evidence and decision |
|---|---|---|
| sim.rs::step | Several linear passes, with selection-only early return | Preserve ordering and pause semantics; no event-bus rewrite |
| targeting.rs::enemy_in_range / targetable | Former O(N² log N), repeated binary searches | Proven redundant lookup removal; live Guard/ties/carrier rules unchanged |
| combat.rs::advance_unit | Stats/cooldowns, id-order movement and windup | New live-movement/death contracts; no static tick snapshot |
| combat/formation.rs::FormationIndex | Former cubic, now worst O(N² log N) | Current positions per query; reused scratch, measured dense wins |
| hits.rs::splash / damage | O(A·V) real hit work; aura can add a further scan | Do not remove mandatory damage work or change hit order |
| augment.rs::damage_reduction_bp | OwnRampart aura scans living units per victim | Potential O(A·V·R) cubic corner with splash; not eliminated by targeting fix |
| state.rs::stats_of / ruleset.rs::stats_with | O(40) constant catalog work, repeated in nested queries | Profile-backed future cache hypothesis; plates/essence/Synergy invalidation must be exact |
| support.rs::eval_bond / bond_candidate | Former sort for minimum | Proven allocation-free minimum; hysteresis/id order unchanged |
| support.rs::puppet_ranked | Sort plus repeated victim stats | Actual 1,000-unit stress remains quadratic/stat-heavy; not replaced speculatively |
| support.rs::transfer_pair / boarding | Linear searches per eligible caster/crab | Overall quadratic in support-heavy armies; correctness traces cover carries/transfers |
| afflictions.rs::tick / brew_target | Linear fixed-stack timers; caster range scans; poison damage path | Keep fixed arrays; tiny cleanse-list sorting is not a priority |
| summon.rs::hatch / cargo paths | Linear population pass; bounded Beetling bookkeeping | Existing summon bounds unchanged; cargo remains uncapped |
| synergy.rs::update / refresh_army | Linear counts; extra linear refresh on level change | Already avoids most stat refreshes when levels do not change |
| stronghold.rs / combat.rs::stronghold_fire | Two linear scans plus current kind/branch effects | Small fixed number of towers; no cache justified |
| shop.rs::draw_offers / intent.rs::apply_intention | Pool-sized draw on purchases; ID search on Evolution | Core-only 80-command burst measured separately; no balance change |
| view.rs::view_for | O(N+events), new vectors and copied per-seat headers | Dense bytes remain costly; do not share private Energy/offers between seats |
| match_task/views.rs::send_view | mem::take drains capacity; view built even for absent seat | Remaining avoidable work; not changed without separate send/reconnect measurement |
| ws.rs / ws/client_msg.rs | Encode allocates, 32-entry bounded queues, bounded frames | Read-buffer experiment below; P10 encode-once/allocator ideas not blindly repeated |
| match_task.rs command drain | try_recv loop has no cooperative await within drain | Bounded channel and existing token bucket remain; no fairness-changing budget added |
| metrics.rs / report.rs | Global RTT sort/lock; per-session RTT vectors grow with match duration | Measurable future long-session memory issue, not solved by a new allocator |
| store/worker.rs | Bounded single-writer queue outside combat | SQLite not in the 20-Hz tick; retain existing architecture |

The added splash/aura stress measured 2.39 ms mean / 6.54 ms p99 / 7.34 ms worst at 1,000 target units in one seeded run. That is a workload observation, not proof that the potential cubic path is harmless. Deaths reduce the number of simultaneous splash victims. Every all-to-all hit may require quadratic *real* work; the extra aura scan is avoidable work worth profiling separately if it dominates a representative long fight.

### Network, layouts and rejected alternatives

Lossless delta prototype: [wire_load.rs](../../crates/tbg-core/examples/wire_load.rs), outside protocol.rs. It sends the complete existing per-seat header/events plus changed units and removals. It verifies exact decoded MatchView equality for both seats, rejects a wrong baseline id, and forces a full baseline every 128 views to exercise recovery. A one-byte envelope allowance is included. The conservative prototype still clones the full view before clearing header units; its CPU cost is measured, not omitted. Payload bytes exclude TCP/TLS/WebSocket framing.

| Scene | Units | Full B/view | Delta B/view | Bytes saved | Full encode µs/view | Delta build+encode µs/view |
| --- | --- | --- | --- | --- | --- | --- |
| melee | 20 | 449 | 235 | 47.8% | 1.13 | 0.88 |
| melee | 200 | 4384 | 2472 | 43.6% | 8.60 | 6.37 |
| melee | 1000 | 17625 | 6547 | 62.9% | 32.15 | 17.52 |
| healing | 20 | 412 | 167 | 59.6% | 0.98 | 0.66 |
| healing | 200 | 3204 | 897 | 72.0% | 6.12 | 2.98 |
| healing | 1000 | 16097 | 4495 | 72.1% | 29.63 | 13.26 |
| mixed | 20 | 444 | 229 | 48.6% | 1.10 | 0.88 |
| mixed | 200 | 4690 | 3092 | 34.1% | 9.87 | 8.49 |
| mixed | 1000 | 32160 | 22995 | 28.5% | 64.86 | 63.03 |
| formation | 20 | 420 | 198 | 52.6% | 0.98 | 0.74 |
| formation | 200 | 3320 | 1479 | 55.5% | 6.23 | 4.29 |
| formation | 1000 | 18063 | 8514 | 52.9% | 33.40 | 21.98 |
| cargo | 20 | 463 | 216 | 53.3% | 1.13 | 0.85 |
| cargo | 200 | 3424 | 1033 | 69.8% | 6.67 | 3.54 |
| cargo | 1000 | 18238 | 5664 | 68.9% | 33.33 | 15.92 |


These are prototype savings, **not deployed network savings**. A production version needs a protocol version, bounded validation, explicit baseline reset on reconnect/takeover, event exactly-once tests, partial-write/backpressure tests and client integration. No generic compression, lossy quantization or lower snapshot/tick rate was added. At 10 Hz, multiply bytes/view by ten; acknowledgement views can raise the actual send rate.

Layout microbenchmark compares identical read-only nearest-enemy kernels: 180-B AoS plus common eligibility flags, 12-B compact records, SoA x/id/flags, and a sorted 1-D index. It deliberately excludes Guard/Stronghold/healing, mutation and index maintenance; exact target equality is checked for that kernel only. At 1,000 units the first run measured 1,020/922/1,112/16 ns per query respectively; index build about 18,103 ns. Three raw repetitions are retained. SoA lost to compact AoS here. At 10 units, sorted lookup offered no meaningful gain after construction. Static sorted data wins large read-only queries but is rejected as a drop-in engine replacement: earlier movement and death must update or invalidate it. No SIMD/ECS/model rewrite is justified by this microbenchmark alone.

`CARGO_PROFILE_RELEASE_OPT_LEVEL=3` improved the dense-kernel median about 16% in the recorded microbenchmark set. It was not adopted: ordinary online throughput was not independently measured under that profile, and P10 had rejected it for that workload. The shared release profile and WASM size policy remain unchanged. Generic frame compression, a different allocator, relevance culling (both players need the single BattleLine), a new ECS and per-match parallelism were not adopted.

### Local capacity and socket read buffers

```bash
ulimit -n 65536
python3 scripts/performance_capacity.py --server /absolute/path/to/server --label candidate --output .local/p30d/new-run --window 60
# Separate generator CPU set (4,5,6,7); server logical CPU 2, hard CPUQuota=100%, MemoryMax=1G.
```

The script is localhost-only, uses uniquely named disposable user scopes, refuses an occupied port/output run, verifies at least 95% requested population and stops increasing load on lateness/population loss/generator saturation. `cpu.max` and `memory.max` were read back; no OOM events occurred. Generator and server share the physical host/kernel/cache but have disjoint CPU affinities. RSS/CPU come from /proc; all windows use wall-clock elapsed time. Random match seeds are not paired across capacity runs: small CPU differences are not causal wins. Probes deliberately abort disposable matches after the sample; their RTT is valid within the window, but no ack/completion summary is invented for aborted clients.

| Version | CPU / RAM cap | Matches requested / minimum observed | CPU cores | RSS MiB | KiB/match | Late counter | RTT p95 ms |
| --- | --- | --- | --- | --- | --- | --- | --- |
| baseline | 1 / 1G | 250 / 250 | 0.166 | 79.2 | 324.3 | 0 | 2 |
| baseline | 1 / 1G | 500 / 500 | 0.277 | 151.9 | 311.2 | 0 | 3 |
| baseline | 1 / 1G | 1000 / 1000 | 0.522 | 294.3 | 301.4 | 0 | 3 |
| baseline | 1 / 1G | 1500 / 1500 | 0.746 | 437.0 | 298.4 | 61 | 3 |
| final | 1 / 1G | 250 / 250 | 0.165 | 79.1 | 324.0 | 0 | 2 |
| final | 1 / 1G | 500 / 500 | 0.284 | 151.9 | 311.2 | 0 | 3 |
| final | 1 / 1G | 1000 / 1000 | 0.514 | 294.6 | 301.6 | 0 | 3 |
| final | 1 / 1G | 1500 / 1499 | 0.731 | 436.8 | 298.4 | 0 | 3 |
| final | 1 / 1G | 2000 / 2000 | 0.980 | 579.6 | 296.8 | 3664 | 4 |
| buffer | 1 / 1G | 1000 / 1000 | 0.526 | 294.4 | 301.4 | 0 | 3 |
| read4k | 1 / 1G | 1000 / 1000 | 0.299 | 48.9 | 50.1 | 0 | 3 |
| read4k | 1 / 1G | 1500 / 1500 | 0.417 | 70.3 | 48.0 | 0 | 3 |
| read4k | 1 / 1G | 2000 / 1999 | 0.529 | 91.8 | 47.0 | 0 | 3 |
| read4k-repeat | 1 / 1G | 1000 / 1000 | 0.297 | 48.9 | 50.0 | 0 | 3 |
| two-cpu-2g | 2 / 2G | 1000 / 1000 | 0.413 | 49.1 | 50.3 | 0 | 2 |
| two-cpu-4g | 2 / 4G | 2000 / 2000 | 0.766 | 92.3 | 47.2 | 0 | 3 |


The final core-only 1,000-match repeat was 0.526 cores / 294.4 MiB / 0 late ticks: essentially the baseline. The 1,500-match 0-versus-61 late-counter difference is not a repeatable capacity claim; 2,000 produced 3,664 late events and hit almost one busy core. The counter marks interval lateness above about half a simulation tick, not a p99 deadline distribution. A missing counter event does not prove a 10-ms p99 target. That core-only version was CPU-limited in the concurrent window. After right-sizing socket reads, 2,000 matches used 0.529 cores and 91.8 MiB with zero late ticks; the highest tested level is not a maximum-capacity claim.

The library's default **128-KiB read allocation per connection** explains most of the historical memory floor. `ws.rs` now requests 4 KiB, retaining the 1-KiB inbound frame/message caps, 4-KiB write buffer, 64-KiB pending-write cap, queues/rates, timeouts and authoritative seat stamping. At 1,000 matches actual RSS fell from 294.4 to 48.9 MiB (about 83%), CPU from 0.526 to 0.299 cores (about 43%), RTT p95 stayed 3 ms and the late counter stayed zero; a separate repeat measured 0.297 cores / 48.9 MiB / zero late events. Two sockets theoretically save 248 KiB of read capacity per match; the table shows actual RSS rather than treating that arithmetic as a measurement. Official crate source was verified locally at the locked Axum/tungstenite versions. FrameCodec::read_in resizes/zero-fills the input BytesMut before reading (tungstenite frame/mod.rs); this identifies avoidable read work. Attributing the entire CPU gain specifically to zero-fill remains an inference from source plus the isolated buffer experiment, not a complete server call-stack profile. The old P10 BytesMut::resize self symbol alone did not establish whether that work belonged to reads or writes. Large outgoing full views can eventually exceed the existing 64-KiB write cap; no safety cap was removed to hide that limit.

### Security, verification and limits

Confirmed `seq=0` defect: `seq != 0 && seq <= last_seq` bypassed replay protection and reset last_seq through the offline core API. This did not grant another seat or free Energy, but it allowed replayed intentions to mutate/spend again and broke monotonic acknowledgement. The server now rejects zero before core application. Positive stale/duplicate requests remain rejected; offline seq=0 remains available to local simulation. New full-state regression was red before / green after; 15 server unit tests passed.

Required formatting and clippy, 121 core tests, all-feature benchmark example clippy and WASM checks passed in the final run. The targeted server suite passed 78 tests with one existing ignored replay test and one filtered 60-second bot test. That unchanged short-deadline test had already failed on the original baseline; the final real-time two-client match completed in 398 seconds with both reconnects. An additional `clippy --workspace --all-targets` found pre-existing unused-import, redundant-guard and collapsible-match warnings in wave tests; unrelated tests were not rewritten. Workspace test attempt: exit 124, 300.0 seconds; see exact command/failure notes below. Local final multiplayer result: exit 0; 2/2 clients completed, 0 failed, 2/2 reconnects, 405.03 seconds. Full JSON and logs are private.

Initial baseline had a 60-second `two_bots_finish_a_match` timeout and an interrupted full-workspace balance run. No assertion, balance threshold or contract golden was relaxed. Full workspace status is reported independently of the exact-state evidence. No client source changed, so a new Trunk build/browser visual sign-off is outside this performance phase; WASM was checked.

Live-VPS observations, deployment identity and shared-service comparisons remain **private in `.local/p30d/`**, following the infrastructure runbook. No live settings, images, files, pods or player data were changed by this investigation. No destructive/stress load was sent there. No maximum production capacity or cheaper-host purchase is recommended from development-PC numbers.

### Primary research and applicability

- **Nicholas Nethercote, [Heap Allocations](https://nnethercote.github.io/perf-book/heap-allocations.html):** profiles allocation sites and shows DHAT evidence. Relevant to temporary vectors; allocator replacement alone does not remove work. Low-complexity experiment: separate counting build and native repeats. TBG evidence supports minimum selection and scratch reuse; no external benchmark number is imported.
- **Nethercote, [Type Sizes](https://nnethercote.github.io/perf-book/type-sizes.html):** measure representation/cache cost rather than assume a model. Relevant to Unit/Fx, but compiler/platform layout differs. Medium-complexity experiment: AoS/compact/SoA plus rebuild costs, then live-state integration. Our kernel rejects SoA as an automatic improvement; a whole-engine rewrite is unvalidated.
- **Tokio project, [Cooperative yielding](https://tokio.rs/blog/2020-04-preemption):** explains scheduler starvation and resource budgets with an external load example. Our locked `mpsc::try_recv` source lacks the asynchronous budget path. Applicable to the command drain; limitations are existing per-connection rate bounds and FIFO/game timing. A bounded-drain change needs measured floods and identical command ordering, not an assumed win. Medium risk; deferred.
- **Glenn Fiedler, [Snapshot Compression](https://gafferongames.com/post/snapshot_compression/):** original packet/roundtrip compression experiments on a different rigid-body workload. Baseline-relative encoding fits reliable WebSockets, but reconnect still loses baseline state. TBG already uses integer coordinates, so lossy float quantization is unnecessary. Medium/high integration complexity; exact delta prototype and byte/CPU results above establish a candidate, not a released protocol.
- **Michele Caini / EnTT, [storage design](https://github.com/skypjack/entt/wiki/Entity-Component-System):** real C++ sparse-set/pool implementation and documented iteration/memory tradeoffs; no comparable TBG benchmark in that reference. Small hot/cold indexes are applicable, but entity lifetime/order differ. A full ECS migration is high complexity and rejected without end-to-end evidence. Our compact-index experiment is the bounded alternative.
- **Valgrind project, [Callgrind manual](https://valgrind.org/docs/manual/cl-manual.html):** instruction/call counting and optional cache/branch simulation; simulation is not hardware PMU evidence. Low setup complexity via a temporary tool extraction. TBG uses it to rank loops and validate instruction reductions; native timing remains the acceptance metric.

- **Axum / tungstenite maintainers, [original WebSocket implementation](https://github.com/snapview/tungstenite-rs):** the locally locked 0.29.0 source defines the 128-KiB default in protocol/mod.rs and the input resize in protocol/frame/mod.rs; Axum 0.8.9 exposes read_buffer_size. No external TBG benchmark is published there. Applicable because legal inbound frames are only 1 KiB. Low complexity, but message-size/rate/backpressure/reconnect regressions must be checked. The isolated cgroup comparison above proves the memory/CPU gain without a new allocator.

### Minimal follow-up plan and rollback

1. **Review this branch:** core targeting/formation/support, socket buffer and seq=0 commits separately. Re-run one seeded matrix, exact traces, protocol/seat/rate/reconnect tests and a local capacity window. Roll back an individual candidate with a new revert commit if results differ; never rewrite history or restore player data. Do not deploy until a later explicit release request.
2. **Lossless snapshots, only after review:** protocol.rs/view.rs, server send path and client decoder/timeline. Acceptance: material measured byte reduction with CPU/RSS documented, exact reconstructed views/events, version mismatch/baseline loss/full reconnect/slow-client tests and bounded malformed payloads. Rollback: retain versioned full snapshots. The current prototype is not integration-ready.
3. **Profile the remaining dense paths before another optimization:** support ranking/effective stats, Rampart aura inside splash and live spatial-index maintenance. Use varied compositions, branch forms, long Augment pauses and 1,000-unit adversaries. Accept only exact traces and reproducible gains outside paired-run noise, including update/build/memory costs. Dirty flags, SIMD, caches, per-match parallelism or scheduler replacement are not approved by this report.

> blocked: `timeout 300 nice -n 10 taskset -c 8,9,10,11,12,13,14,15 cargo test --locked --workspace --no-fail-fast` exited 124 after 300.01 seconds. `c8_a_flat_roster_flags_nothing` passed; `c7_planted_imbalances_are_caught_for_every_unit` was still running. The earlier baseline workspace run also stalled in these balance contracts before interruption. Suspected cause: current-roster adversarial balance workload. No contract test, golden, gameplay value or gate threshold was changed. Full-workspace acceptance remains open; do not label this branch globally green.

### Verification record

| Check actually run | Result |
|---|---|
| `cargo fmt --all -- --check` | exit 0 |
| `cargo clippy --workspace -- -D warnings` | exit 0 |
| `cargo clippy -p tbg-core --all-features --examples -- -D warnings` | exit 0 |
| `cargo test --locked -p tbg-core` | 121 passed, exit 0 |
| `cargo test --locked -p tbg-server --tests --no-fail-fast -- --skip two_bots_finish_a_match` | 78 passed, 1 existing ignored, 1 filtered, exit 0 |
| `cargo check -p tbg-client -p tbg-core --target wasm32-unknown-unknown` | exit 0 |
| Release server/core examples and allocation-instrumented example builds | exit 0 |
| Final real-time two-bot match, 900-second deadline | 2/2 completed, 2/2 reconnects, 398 s, exit 0 |
| Full workspace, 300-second limit | exit 124; C7 unfinished, C8 passed |
| Additional workspace clippy with `--all-targets` | exit 101; pre-existing `Status` unused import, redundant guard, collapsible match in wave tests |

Native per-fixture wall-clock time and allocation counts are not per-fixture process CPU/RSS attribution. Actual process CPU/RSS are measured in the constrained server runs. Scheduler wakeups/context-switch counts and end-to-end command-drain time were not collected; core command application was measured instead. These omissions remain explicit rather than filled with estimates.

### Deviations / unresolved checks

Production deployment and public 50-match load steps from older phase specs were intentionally not run: the owner's attached investigation explicitly prohibits them. Hardware PMU counters, actual minimal-VPS capacity and indefinite match memory are unmeasured. The 60-second local capacity windows are not a soak test. Snapshot deltas, SoA, static targeting caches and a new compiler profile remain unshipped experiments.
