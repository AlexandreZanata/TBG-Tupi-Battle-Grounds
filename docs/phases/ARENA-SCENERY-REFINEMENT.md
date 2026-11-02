# ARENA-REFINE-01 — Readable independent scenery silhouettes

Owner request, 2026-11-02: refine rocks entering trees and other scenery
interpenetrations after widening, with no gameplay changes.

Remove redundant boulders within the Forest rear tree strip and common rear
coastal groups crossing authored trees/temples. Separate foreground garden
assets and Tundra fir origins, remove two cramped Celestial statue planters,
and move rear Clockwork boilers behind their fence runs. Build optional trees
and scatter after authored props/glow/fittings, rejecting occupied sites using
actual transformed mesh bounds and conservative whole-asset site footprints.
No per-frame collision work, road/camera changes or simulation effects.

A new regression checks rendered rock/canopy bounds across every Arena and
retains a minimum Forest silhouette. Existing full-road/static/animated tests
remain. Consolidation also includes the two pending Colossus arm-clearance
regressions from GD-02. Validation receipts follow below.

## Consolidated evidence and current pending check

The owner added this scenery front after GD-02 had been deferred for a busy
validation lock. A new, deduplicated consolidation of the changed scenery plus
the pending Colossus cases acquired the same nonblocking lock with two Cargo
jobs, the existing target/native sysroot, offline/locked and one test thread:

```sh
cargo test --offline --locked -p tbg-client --bin tbg-client -- colossus arena_clear01_ arena_refine01_ --test-threads=1
```

Compilation passed (11.03 s). Five tests passed: both existing eight-Arena road
clearance/animated attachment regressions and all three Colossus art/arm/neighbor
cases. One new scenery test failed its minimum Forest silhouette requirement:
the initial optional-placement filter removed too many trees. No rock/canopy
non-intersection pass is claimed from this run. The original warning about
unused `Layout::clear` remains.

Corrective source: add nine curated, staggered rear tree anchors before optional
fill candidates; retain the same minimum-density assertion and aggregate all
Arena diagnostics. This preserves the forest instead of weakening the test.
The exact targeted rerun below returned **exit 75**, shared slot busy, before
compilation; no automatic retry, polling or bypass followed:

```sh
cargo test --offline --locked -p tbg-client --bin tbg-client -- arena_clear01_ arena_refine01_ --test-threads=1
```

`cargo fmt -p tbg-client -- --check` and `git diff --check` passed. Current
forest density, final rock/canopy separation and native appearance are **pending**,
not a passed result. Logs are retained in `.local/arena-refinement-2026-11-02/`.
GD-02's three passing cases need not be repeated unless their relevant code
changes. At the next consolidation, run the three Arena cases once; then build
the committed native source once under the same lock/two-job policy and install
with `scripts/install-champion-preview.py`. Verify the executable and all 380
assets and inspect the eight affected scenery compositions, including foliage
and monument attachments. Windows/desktop web remain unobserved.

Installed Linux/source and current gallery images remain `e868927`; neither
GD-02 nor ARENA-REFINE-01 is installed or visually verified. No new captures,
full suite, balance gate, extra target, release build, merge or public deployment.

Disk capacity: 760 duplicate immutable asset files across the three already
installed preview slots now share identical file contents/inodes, with every
path/content preserved after SHA comparison. The previous client test executable
was replaced under the validation lock. Source, installed executables, sessions
and cache directories were preserved.

## Final geometry consolidation during SIDE-01

All three Arena regressions passed under the shared nonblocking host lock,
two Cargo jobs, existing native sysroot/cache, offline/locked and one test
thread. The retained Forest anchors pass the density assertion. The bounds
check additionally found a foreground Tundra fir/coastal-rock overlap; moving
its complete fir/crystal group to z=3.4 fixes that overlap. No assertion, road
width, movement, stats or solver was changed. The final command was:

```sh
cargo test --offline --locked -p tbg-client --bin tbg-client -- arena_clear01_ arena_refine01_ --test-threads=1
```

Result: 3 passed, 0 failed (1.03 s), after the relevant Tundra correction.
Logs: `.local/blue-red-sides-2026-11-02/client-arena-final.log`. Native
installation/captures are recorded by [SIDE-01](BLUE-RED-SIDES.md); preceding
installed-source notes are historical. Windows/desktop web remain pending.
