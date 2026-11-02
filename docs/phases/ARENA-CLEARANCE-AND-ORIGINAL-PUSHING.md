# RESTORE-02 / ARENA-CLEAR-01 — Original pushing and clear arena dressing

Owner corrections, 2026-11-02: remove the added movement smoothing, retain
exact pre-change pushing/advancement with a wider passage, move the four misplaced
Forest flames and review all eight Arenas for scenery inside the enlarged road.

The crowd solver matches `d4b2c64` (identical to the pre-PK-01 solver) except
`half_width: 2.1` instead of `1.1`; ECS direct display application is restored.
The old seven pushing/overtaking/bounds tests remain. Rejected smoothing-specific
tests are removed with their implementation. Normal Practice/Training keep
RESTORE-01 immediate entry and no physical lane queues. No core rules changed.

The Forest's per-part post-translation is removed. Shared brazier positions now
place both fire and bases outside the paved road; fence/cloth sides share one
constant. Authored tree/scatter origins and near-road rocks, palms, columns and
temple steps clear the enlarged road while keeping compound parts together.
Coastal rock groups move as a group near road corners.

Two new regressions inspect rendered triangle bounds against the full paved
rectangle, including its border: all eight static prop/glow kits; animated
fire/cloth/gears plus Forest offshore scenery, 160 time samples per Arena;
all eight fires must match their pedestal coordinates. Stronghold-attached flags
are intentionally exempt as part of the tower, not surrounding scenery.

Validation and native/install receipts follow below. Windows/desktop web actual
visual comparison remains pending; no full suite or balance gate is requested.

## Targeted consolidation

Source `e8689274404c1cc4f3fcc2b2344125cf5f8e504f`, wire 36 / ruleset 12,
contract `w36-r12-8f5a8ebb3e4f88ec`; core rules and hashes are unchanged.

Commands ran with the nonblocking `/tmp/tbg-validation-1000.lock`,
`CARGO_BUILD_JOBS=2`, existing shared target and existing native dependency sysroot:

```sh
cargo test --offline --locked -p tbg-client --bin tbg-client -- arena_clear01_ crowd::tests --test-threads=1
cargo build --offline --locked -p tbg-client --bin tbg-client
```

The new clearance tests first detected Tundra foreground rock/crystal overlap,
near-road Ruins columns/shrubs, Desert palm overhang and shared coastal rocks
near the corners. Those asset origins were corrected; tower-owned flags were
explicitly excluded from the scenery contract. Final result: nine passed, zero
failed (two arena regressions and seven existing original-pushing scenarios).
The native build passed in 19.18 s. Affected-package formatting and
`git diff --check` passed. The existing unused `Layout::clear` warning remains.

A source comparison against `d4b2c64` verifies only the lateral bound changes in
`crowd.rs`; `sync/crowd_ecs.rs` differs only in its module comment. This removes
the extra RESTORE-01 smoothing/stepping/order/envelope changes completely.
No normal physical-lane queue or authoritative blocking is reintroduced.

Disk pressure initially prevented writing the stripped executable (`No space
left on device`). Recovery used a temporary strip output in shared memory,
validated its exact build information, then retained the stripped executable
and removed only the unused native build executable and its Cargo hard link.
No source, assets, installed versions, active sessions or cache directory tree
was removed. The installed executable links to the immutable staged artifact,
never to a Cargo output that a later build could replace.

Linux installation: `0.1.4-colossus.1-e8689274404c`. Binary SHA-256:
`b06fbfc4cd88ce06cf70c322d77304a9b088a967902a07d87686e43c1d9d43bd`.
Exact source/build information and all 380 installed asset hashes matched.
The existing launcher and desktop entry now select this source; the new catalog
window uses a separate data directory. Earlier matches are not hot-migrated.

Native capture and remaining platform receipts follow below.

## Linux visual receipt

Eight native Practice captures passed and were individually inspected: Forest,
Tundra, Volcanic, Ruins, Storm, Celestial, Clockwork and Desert. Each went through
the Arenas page's own Train action and actual Practice startup, at 1600x1000,
with isolated data and two renderer CPUs/threads. All exited 0 with completed
loading; the road stays wide and surrounding assets stay outside it. Forest's
four near-road flames now sit visibly on their pedestals. Temple steps and
compound decorations remain assembled. These are Linux observations only.

[Per-image source/hash and installed-build receipt](../arena-clearance/native-validation.json).
Original PNGs/logs are retained under `.local/arena-clearance-2026-11-02/` at the
owner checkout. Local review: <http://localhost:8966/arenas.html>. The existing
Colossus gallery links these scenes and no longer describes the rejected
smoothing. No champion portraits or gallery forms were regenerated.

Deferred: actual Windows and desktop game-web observation of the same eight
Arenas and original pushing, plus owner look/fun review of a dense interactive
match. Browser automation remains unavailable (`browser.user is unavailable`);
HTTP checks are not browser appearance evidence. No full suite, additional
platform build, balance gate, release build, merge or public deployment.
