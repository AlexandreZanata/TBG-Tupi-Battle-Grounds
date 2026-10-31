# P57 — Units take up space on screen (client display layout)

**Status:** planned (owner request 2026-10-30). Not started.
**Goal:** large armies stop drawing units inside each other, with no
server change, no extra bandwidth and no visible FPS cost.

## Read first

- `AGENTS.md`; ARCHITECTURE §3 (crate boundaries) and the client
  sections; ART-DIRECTION (silhouettes, readability at match distance).
- `crates/tbg-client/src/sync.rs` (`world_x`, `lane_z`, `Scene::place`),
  `visuals.rs` (`UnitVisual`, ring), `visuals/aim.rs`,
  `visuals/attacks.rs` (`REACH`), `sync/events.rs` (`pos_of`),
  `sync/actions.rs`, `visuals/names.rs`.
- `crates/tbg-core/src/combat/formation.rs` (only to see that the
  simulation is one-dimensional; do not change it).

## The problem, measured

The simulation is one-dimensional: a `Unit` has only `x` (milli-tiles
along the 16-tile lane). Units walk until an enemy is in range and then
stop, so every melee unit facing the same enemy stops at the same `x`.
Nothing in the rules reserves room between allies.

The client draws a unit at `world_x(x)` and at one of only **three
fixed depths**: `lane_z(id) = [-0.8, 0, 0.8][id % 3]`. A unit's footprint
(its ring) has a radius of about 0.36 tiles (0.54 for large bodies), so
two units of the same row closer than ~0.7 tiles are drawn inside each
other.

Probe on 40 shipped-bot matches (sampled every 0.5 s, today's rule):

| Metric | Value |
|---|---|
| Alive units, mean / peak | 11.9 / 28 |
| Samples with at least one overlap | 83 % |
| Units overlapping at least one other | ≈ 50 % |
| Units in a stack of 3 or more | 25 % |
| Worst stack (same spot) | 8 |

Human armies are larger than the bot's and there is no army cap
(BR-U07), so real matches are worse.

## Options considered

| Option | Where | Effect on rules | Server cost | Verdict |
|---|---|---|---|---|
| A. 2D bodies with collision in the simulation | `tbg-core` | Big: units block each other, fewer melee units reach the front, DPS and every balance number move; protocol and replays change | Per tick, per match, on the VPS (spatial hash, O(n) to O(n log n)) | Rejected now: a game-rule change that needs DECISIONS, the lab and re-balancing |
| B. 1D spacing (allies queue along `x`) | `tbg-core` | Same as A (back units never reach) | Small but per tick | Rejected for the same reason |
| C. **Client display layout**: the server keeps the true `x`; the client chooses where to *draw* each unit so bodies do not overlap | `tbg-client` only | **None** (pure presentation) | **Zero** (no server code, no bytes) | **Chosen** |
| D. Cosmetic tricks only (smaller models, transparency, outlines) | client | None | Zero | Not enough alone; may complement C |

C keeps the AGENTS.md authority rule: the server decides combat and
outcomes; the client only decides how they look.

## Design (option C)

### Contract

- The authoritative `x` is never changed; the layout returns a display
  offset per unit: `back` (tiles toward the unit's own Stronghold, ≥ 0)
  and `z` (across the lane).
- **The front line stays truthful**: the frontmost units of a seat are
  drawn at their true `x`; only crowded units behind them step back.
- A unit is never drawn ahead of its true `x`.
- Pure function of the current frame plus each unit's previous slot
  (for stability); no randomness, no clocks except the smoothing `dt`.

### Algorithm (per seat, every frame)

1. Collect the living, not-carried units of the seat with their
   interpolated true `x` and footprint radius `r = 0.36 × plan.scale`
   (large bodies get their own radius). Dying units keep their last slot
   until despawn; carried units follow their carrier (Cargo Crab).
2. Sort front-first (`x × dir` descending), ties by id, so the result
   does not depend on view order.
3. Place each unit greedily:
   - candidate depths: **five files** `z ∈ {0, −0.6, +0.6, −1.2, +1.2}`
     (centre out), all inside the lane's visible width (±1.5);
   - try the unit's previous file first (stability), then centre-out;
   - accept the first spot whose disc does not overlap an already placed
     disc (`distance ≥ r₁ + r₂ − 0.02`);
   - if no file is free at this `x`, step one rank back
     (`0.62 × max(r)/0.36` tiles toward its own side) and try again;
     cap at 8 ranks.
4. Neighbour search uses a uniform grid along `x` (1-tile buckets):
   each unit checks at most 3 buckets, so the pass is O(n). All buffers
   live in a `Resource` and are reused: **no allocation per frame**.
5. Smoothing: the entity eases toward its slot with exponential
   smoothing (rate about 10/s) on `back` and `z` only; the true `x`
   component is applied without delay, so movement speed and timing do
   not lag. On spawn the unit appears directly at its slot.
6. Replace `lane_z(id % 3)`; `Scene::place` writes
   `(world_x(x) − dir × back, 0, z)`.

The two seats are laid out independently. Opposing front units cannot
overlap: melee range (0.8 tiles) is larger than two radii (0.72).

### What must keep working (already reads entity transforms)

Shots, impacts, synergy bursts (`sync/events.rs::pos_of`,
`sync/actions.rs`), aiming (`visuals/aim.rs`), health bars, the
selection ring and name tag all take the unit entity's `Transform`, so
they move with the drawn unit. Review only:

- `visuals/attacks.rs`: a melee impact looks for a target within
  `REACH = 2.6` tiles; with at most 8 ranks a back-rank melee unit can be
  further away. Keep the impact on the real target; if it is out of
  reach, skip the lunge rather than draw a long jump.
- `visuals/aim.rs` owns the figure child's translation (lunge/recoil);
  the layout moves the **root** entity only, never the figure.

### Performance budget

- Layout of 300 units (the `?army` debug cap) under **0.2 ms** per frame
  native, measured by a unit test with a timer (generous bound) and in
  the browser with `diagnostics/measure.js`.
- Also fix `Scene::place`, which looks up each unit in the next view with
  a linear `find` (O(n²) per frame): views are sorted by id, so a merge
  walk or a reused id→index map makes it O(n).
- The server, protocol, view size and VPS CPU are unchanged by
  construction; prove it with `git diff --stat` (no `tbg-core` or
  `tbg-server` file touched).

## Steps

1. New module `crates/tbg-client/src/visuals/layout.rs` (one
   responsibility: display slots) with the algorithm above and its
   reusable buffers. Unit tests: no overlap for armies up to the lane's
   capacity; front unit at true `x`; never ahead of true `x`;
   deterministic for the same input in any order; stable slots when one
   unit dies; 300 units within budget.
2. Wire it into `sync.rs` (`Scene::place`), remove `lane_z`, add the
   smoothing; make `Scene::place` O(n).
3. Review `attacks.rs` and the Cargo Crab carrier (passenger rides the
   carrier, not a slot).
4. Measure: rerun the overlap probe (bot matches) against the new layout
   — target **0 overlapping pairs** while the lane has room; record the
   browser FPS/p95 frame time before and after with
   `?autoplay=practice&army=40` and `army=150` at 1920×1080.
5. Report, ROADMAP status, D-57 in DECISIONS ("display layout is
   presentation only; the simulation stays one-dimensional").

## Acceptance

- No unit drawn inside another in normal battles; in extreme crowds
  overlap only after the 8-rank cap, never stacked on one spot.
- Front line drawn at the true position; no unit drawn ahead of it.
- No jitter or slot swapping visible in a 2-minute Practice match.
- FPS and p95 frame time unchanged within noise at `army=40`; layout
  under 0.2 ms at 300 units.
- `tbg-core`, `tbg-server` and the protocol untouched; AGENTS.md checks
  pass; `trunk build --release`; desktop browser check.
- Owner sign-off on the look (ART-DIRECTION readability).

## Later, only if the owner wants it as a rule

Real body blocking (options A/B) would make space a gameplay mechanic:
tanks physically screening the back line, flanking room, crowding
penalties. That is a GAME-DESIGN change with balance consequences and a
per-tick server cost; it needs its own decision and lab evidence. P57
does not prevent it: the display layout would then follow the 2D
positions the server sends.
