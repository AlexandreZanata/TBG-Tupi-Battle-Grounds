# R01 — Bulwark delivery

Implementation: own stone walker/dome, visible face/tool, five forms, heavy
compact gait and two-arm downward blow. Living Wall uses the shared independent
timer, fixes current x during the capped protection and leaves basics working.
A changes HP/duration; B changes basic damage/Slow, exactly §13 (D-72).
Public pools remain unchanged. The actual desktop lineup is now observed below.

Work-front checks: affected-package formatting and `git diff --check` passed.
A scoped cached WASM check is the only requested compilation on this front;
result follows. No per-front cargo test, full suite or balance gate.

Deferred to the next combined consolidation (shared lock, CARGO_BUILD_JOBS=2):
`cargo test -p tbg-core --lib r01 -- --test-threads=1`, the shared specials timing
scenarios affected by the new self-protection channel, and the updated roster
contract. Cases cover ready waiting, anchoring/release, basics during protection,
Root without duration extension, global reduction cap and all branch values.
Actual desktop five-form screenshot, full-clip/ability-VFX observations, live
reconnect, performance and balance acceptance remain pending, not passing.

Scoped work-front result: the shared-lock, two-job
`cargo check -p tbg-client --bin tbg-client --target wasm32-unknown-unknown`
passed without warnings in 2.00s, using CARGO_TARGET_DIR=/tmp/tbg-pr20-target.
No native/client duplicate target or per-front tests were run. Preview/screenshots
stay queued for the R01–R03 combined build; this check alone is no browser proof.

D-72 also corrects inherited offline counter capacity: LabStats covers registered
kinds, while the active roster CSV reads only its existing legend prefix. This
prevents staged prototype damage from indexing beyond 27 slots without activating
any Shop entry. The new `lab::tests::staged_kinds_collect_lab_damage_without_public_pool_activation`
scenario is queued with --features lab at consolidation. Native balance-binary
compilation is a scoped compatibility check, not a balance experiment or gate.

Compatibility result: shared-lock/two-job
`cargo check -p tbg-balance --bin tbg-balance` passed without warnings in 1.98s
using the existing /tmp/tbg-pr20-target cache. This compiled the affected lab feature
and active CSV projection; it did not run simulations, benchmarks or a balance gate.

Consolidation (R01–R03 integrated working tree): 58 distinct focused tests passed
across `--features lab --lib r0`, `specials::tests`, `roster_slots::tests`,
`staged_kinds`, and the named contract_roster/contract_protocol/contract_determinism
binaries. Commands held the shared host lock, used two Cargo jobs and one test
thread. No workspace suite or balance gate ran. Logs are local under
/tmp/p59-r01-r03-{consolidation,preview}.log. The initial real-spawn fixture reused
a handcrafted UnitId; it now uses a fresh MatchState. The legacy faction count
now explicitly reads the frozen ALL_KINDS prefix, preserving every old count;
prototypes remain covered by separate exact §13 rows (D-72). After these changes,
the affected tests passed, including all three frozen legacy match outcomes/hashes.

Requested desktop preview: shared-lock/two-job Trunk release build passed
(compilation 2m52s, cached offline tools, BINARYEN_CORES=2). The real 1280×720
lineup shows Base/A1/A2/B1/B2 across idle/walk/attack rows. Observed all five own
stone silhouettes and branch attachments; triangle counts 2012/2060/2108/2308/2320.
Evidence: ../../media/p59/bulwark/lineup.jpg. Full-clip/ability/live play and owner
look/fun acceptance remain deferred; the lineup is not proof of those scenarios.
