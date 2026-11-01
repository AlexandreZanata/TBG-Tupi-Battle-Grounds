# R04 — Trigger delivery

Own feline/twin-pistol/hat/tail kit, five forms, alternating real-joint muzzles,
heel-toe gait, seated fall and independent offhand fourth-hit cue. Crossfire
spends every fourth completed enemy-unit primary, including kills/summons;
nearest distinct secondary by distance/id, no carried/allied/tower victim or
recursive hooks. Absolute branch values and static ratio follow §13 (D-75).
Protocol 25 appends the chosen secondary at Event tag26; public pools stay closed.

Work-front validation: package formatting/diff and one cached scoped WASM check
are recorded below. Native scenarios and real desktop screenshot remain pending.
Deferred to the next combined consolidation under the shared lock/two Cargo jobs:
`cargo test -p tbg-core --features lab --lib r04 -- --test-threads=1`, shared
Acidmaw nearest-selection scenarios and contract_roster/contract_protocol. Cases
cover fourth-hit/killing primary, distance/id, no candidate, Stronghold exclusion,
carried/allied targets, exact branch values and reconnect progress.
Full-clip and ability-VFX playback, live reconnect, performance, balance and full
suite remain deferred, not passing. Screenshots will use actual rendered geometry.

Scoped work-front result: the cached shared-lock/two-job WASM check passed in
6.33s after explicitly classifying Crossfire as public in the exhaustive spectator
event filter. It emitted one unused-parentheses warning in the own pose, now
removed with package formatting/diff inspection. No duplicate target build or
native tests ran on this front; warning-free recompilation is queued with the next
combined browser build. The first check's exact failure was E0004 in
view/visibility.rs: `&Event::Crossfire { .. }` not covered; classification fixed it.

Consolidation initially found E0425 in acidmaw.rs:172: its selector regression
still called `secondary_ids` after extraction. The call now uses the same shared
`secondary_targets::nearest` function as production, with all old assertions intact.
No scenario ran before that compile failure; the focused union is rerun below.

Consolidation R04–R07: 75 distinct focused tests passed on the integrated tree:
--features lab --lib r0 (34), r10 (4), specials::tests (6, one overlap),
roster_slots::tests (12), staged_kinds (1), and named roster/protocol/determinism
contracts (6/11/2). Shared host lock, CARGO_BUILD_JOBS=2, --test-threads=1. The
initial test-only selector-call E0425 was corrected with every old assertion kept.
Shared-lock offline Trunk release preview passed (2m49s Rust compilation,
BINARYEN_CORES=2); actual desktop lineup observations follow below. No full suite,
balance gate, cold native client build, CI waiting or production operation.

Actual desktop lineup observed at1280×720: Base/A1/A2/B1/B2 in idle/walk/attack,
with the own silhouette and branch attachments, using the combined R04–R07 WASM
binary. Triangles: 1704/1728/1900/1760/1784. Screenshot: ../../media/p59/trigger/lineup.jpg.
These observations and the consolidation above resolve earlier pending compile,
test and lineup entries. Full-clip/ability/HUD/live reconnect/performance/balance,
full suite and owner look/fun acceptance remain open. No pool activation/deploy.
