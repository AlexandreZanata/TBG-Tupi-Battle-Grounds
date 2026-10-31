# R02 — Demolisher delivery

Implementation: own hydraulic fists/cuffs, pressure tank and visor, five forms,
piston idle, heavy punch, kneeling collapse and independent two-fist pressure
slam. Area Crush is caster-centered, includes enemy summons/carried units and
never displaces or interrupts. A changes registry special damage/radius, B period
then basic Slow. Values and the static-scaling reading are recorded in D-73.
Public pools stay unchanged; screenshot/character commit pending R01–R03 preview.

Work front: affected-package formatting and `git diff --check` passed. A cached
WASM check is queued below; no Rust/browser result is assumed. Native regression
runs belong to the combined consolidation, not this individual front.

Deferred commands under the shared host lock with CARGO_BUILD_JOBS=2:
`cargo test -p tbg-core --lib r02 -- --test-threads=1`, the modified shared timed
special scenarios, and contract_roster. Cases cover caster versus primary center,
area transport/allied/radius exclusions, no displacement/status, A2 damage/radius,
B2 basic-only Slow, period replacement and a dead caster's canceled special.
Desktop screenshot, ability-VFX/live reconnect, full-clip playback, performance,
balance gates and full suite remain unverified/pending, not passing.

Scoped cached WASM result: the shared-lock, two-job
`cargo check -p tbg-client --bin tbg-client --target wasm32-unknown-unknown`
passed without warnings in 2.37s using CARGO_TARGET_DIR=/tmp/tbg-pr20-target.
No per-front tests or duplicate native client build. Screenshot remains deferred
to the combined preview; this scoped check is not browser evidence.

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

Requested real desktop preview: the same shared-lock/two-job Trunk release passed
(R01–R03 combined binary, 2m52s compilation). Observed at 1280×720: all five own
forms, idle/walk/attack, two visible piston fists, tank/visor and branch hardware.
Triangles Base/A1/A2/B1/B2: 1348/1372/1396/1636/2224. Screenshot:
../../media/p59/demolisher/lineup.jpg. Full-clip/ability/live scenarios remain open.
