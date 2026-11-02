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
