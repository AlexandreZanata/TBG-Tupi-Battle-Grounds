# R13 — Luminous Oracle delivery

Own empty-hood/robe/cape/sun-staff kit, five forms and closing-offhand weakening
pose. Shared authoritative single-target Harmonic Pulse uses exact period/duration
replacements and the bounded weakening slot; Ward/Brew and control-only immunity
retain their distinct contracts (D-80/§13). Basic SunSpark has correct Control
release/flight. Separate lazy weakness/mark badges remain visible under a control
ring. Seven-Faction native accent lookup now matches the existing D-64 palette.
Public pools/protocol25/simulation layout remain unchanged.

Work-front result: package formatting/diff passed. Shared-lock/two-job cached
`cargo check -p tbg-client --bin tbg-client --target wasm32-unknown-unknown`
passed without warnings in3.81s. No per-front Rust tests or native client build.

Queued next consolidation: `cargo test -p tbg-core --features lab --lib r13 --
--test-threads=1`, affected special/roster/protocol/determinism cases as needed.
Scenarios cover basic-at-resolution weakening versus unchanged auxiliary damage,
Ward versus control immunity, absolute duration/cadence, public expiry and Brew
cleansing. Actual desktop screenshot/character commit pending the next combined
preview. Full-clip/ability/HUD/badge/live reconnect/performance, balance gates,
full suite and owner look/fun sign-off remain pending, not passing. No deployment.

Combined R13–R15 consolidation passed40 focused scenarios: --features lab --lib
r1 (21; includes affected R10/shared special/R12 and all nine new R13–R15 cases),
contract_roster6, contract_protocol11 and contract_determinism2. Shared host lock,
two Cargo jobs, one test thread. No failures or hash migration. The same cached
offline Trunk release preview passed (3m44s Rust compilation, BINARYEN_CORES=2).
Actual desktop lineup observations follow below; other deferred acceptance stays.

Observed visual failure: the custom ring's parent was later than it in the node
array, so one-pass spawn used the figure root and hid the ring inside the robe.
The fixed-slot spawner now allocates every entity first, then attaches parents.
Regression: art::figure::tests::custom_ring_follows_its_later_weapon_parent.
Initial image rejected; corrected real preview pending. No gameplay change.

WASM test-target type checking found E0063 in archer/tests.rs:26 and kits/tests.rs:123:
older Moment fixtures omitted reduction/counter/armor_flash/special/special_ready.
Added their inactive defaults without changing any existing assertion. This checks
those fixtures and the hierarchy regression without a cold native Bevy build;
it is compilation evidence, not test execution. Affected check rerun below.

Corrected integrated type check: the shared-lock/two-job cached
`cargo check -p tbg-client --bin tbg-client --tests --target wasm32-unknown-unknown`
compiles the production hierarchy fix and all client test targets, including the
new reproduction, without a cold native platform build. Result recorded below;
this command never executes native tests. Corrected screenshot still pending.

The integrated production/client-test WASM type check passed without warnings
in2.90s. It also compiles Jawguard and the prior renderer refactor. Native execution
of the hierarchy test is deferred: `cargo test -p tbg-client --bin tbg-client
art::figure::tests::custom_ring_follows_its_later_weapon_parent -- --test-threads=1`
under the shared host lock/two jobs when the native client cache is warm or explicitly
scheduled. No native test pass is claimed from a WASM type check.

Corrected hierarchy/R16 preview passed under the shared lock/two jobs (2m15s Rust
release compilation, offline cached tools, BINARYEN_CORES=2). Twelve focused
R16/R14/roster scenarios passed in that combined build, in addition to the unchanged
R13–R15 forty-case evidence. Actual desktop review now shows the ring above and
attached to the staff in every form/motion row. The earlier rejected capture is
superseded; native hierarchy-test execution and live badge/ability playback remain
deferred as recorded, never inferred from compilation or lineup screenshots.

Actual desktop lineup observed at1280×720: Base/A1/A2/B1/B2 in idle/walk/attack,
with the own silhouette and branch attachments, using the combined P59 WASM
preview. Triangles: 750/750/764/778/802. Screenshot: ../../media/p59/luminous_oracle/lineup.jpg.
These observations and the consolidation above resolve earlier pending compile,
test and lineup entries. Full-clip/ability/HUD/live reconnect/performance/balance,
full suite and owner look/fun acceptance remain open. No pool activation/deploy.
