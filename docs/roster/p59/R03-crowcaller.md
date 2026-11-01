# R03 — Crowcaller delivery

Implemented: own plague-mask/coat/crossbow/raven kit and five forms, with latch
reload, stalking gait, opposing raven glance, marked-shot wing and kneeling fall.
Ready-on-next-hit Mark is post-damage, caster-owned and cannot become a team damage
amplifier. Ward/Brew behavior is explicitly Proposed in §13/D-74. Protocol 24
provides bounded public effect/expiry facts; a persistent target badge and optional
inspection effects row reconstruct after reconnect. Public pools remain unchanged.

Work-front checks: formatting/diff and one cached WASM check are queued below;
no compile, screenshot or native-test pass is assumed. Screenshot/character commit
is pending the R01–R03 combined preview. Native tests belong to consolidation.

Deferred focused commands (shared host lock, two Cargo jobs):
`cargo test -p tbg-core --features lab --lib r03 -- --test-threads=1`, updated
contract_roster/contract_protocol, relevant roster-slot/Brew scenarios and the
client inspection bridge test when a warm client-test build is available.
Cases cover trigger/own bonus order, two casters, cut non-addition, waiting/Root/
Stronghold/ward/killing behavior, control immunity, Brew versus Ravager records,
real spawn cooldown, all forms and full seat/public/wire reconnect snapshots.

Known baseline limit: Brew's inherited minimum-score ordering differs from its
most-afflicted comment. This change adds eligibility for new harmful families
without changing that old ranking. Full-clip/ability VFX, live reconnect, HUD visual
review, payload/performance measurements, balance acceptance and full suite remain
pending, not passing. The requested screenshot will be real rendered geometry.

Scoped work-front result: formatting, `git diff --check` and
`node --check crates/tbg-client/assets/battle/hud.js` passed. Under the shared host
lock with CARGO_BUILD_JOBS=2 and existing /tmp/tbg-pr20-target cache,
`cargo check -p tbg-client --bin tbg-client --target wasm32-unknown-unknown`
passed without warnings in 5.27s. Native scenarios and browser appearance remain
queued for the combined consolidation; this check does not prove either.

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

Requested desktop preview: shared-lock/two-job offline Trunk release passed in
the combined R01–R03 preview (2m52s Rust compilation, BINARYEN_CORES=2). Observed
all five own forms at 1280×720 in idle/walk/attack: plague mask/beak, shoulder raven,
ritual crossbow, torn coat and branch attachments. Triangle counts:
1528/1828/1950/1656/1788. Screenshot: ../../media/p59/crowcaller/lineup.jpg.
The seven R03 scenarios passed with lab enabled in the combined consolidation.
These results resolve the earlier pending compile/test/lineup entries; full-clip,
ability playback, HUD/live reconnect, performance/balance and owner approval stay
open. No public roster activation, merge or deployment was performed.
