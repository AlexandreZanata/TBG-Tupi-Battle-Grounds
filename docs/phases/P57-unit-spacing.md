# P57 — Units take up space on screen (client display layout)

**Status:** implemented on `phase/p57-unit-spacing` (client only, D-57). Owner look pending.
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

## Phase report

### What was built

Commits on `phase/p57-unit-spacing` (from `docs/p57-unit-spacing`):

- `a9582d1` `visuals/layout.rs` (display slots, greedy placement, easing,
  unit tests) wired into `sync.rs`: `Scene::place` draws at the slot,
  `lane_z` is gone, and the next view is read with one cursor walk.
- `0c12ebb` melee impacts land on the real target; the lunge is skipped
  when the target is out of reach (`aim.rs`).
- `64f871a` one-and-a-tenth-tile buckets, the fastest-of-50 timing in the
  300-unit test, and the ignored `probe_bot_matches` measurement.
- this commit: the phase report, the ROADMAP line, D-57 and `docs/media/p57/`.

`git diff --stat docs/p57-unit-spacing..HEAD` touches only
`crates/tbg-client`, `docs/DECISIONS.md`, `docs/ROADMAP.md`,
`docs/phases/P57-unit-spacing.md` and `docs/media/p57/`. Nothing in
`tbg-core`, `tbg-server`, the protocol, GAME-DESIGN or a contract test.

### Deviations

- **Cargo Crab passenger is not tied to its carrier.** The view does not
  carry the carried bit (`tbg-core/src/view.rs`; P55 already records it), so
  the client cannot tell who rides. A passenger has the carrier's `x` in
  the simulation and is laid out as an ordinary unit, so it takes a free
  slot next to its crab. Fixing it needs a view change (protocol), which
  this phase forbids.
- **Dying units do not reserve space.** They leave the view when they die
  and keep their last transform (nothing moves them), but they are not
  layout obstacles. For the death clip and the 0.9 s corpse a neighbour can
  slide over a corpse.
- **Melee impacts stay on the real target at any distance.** The 2.6-tile
  guard used to send an impact to the attacker's blade when the target was
  far. The layout can put a back-rank attacker 3–5 tiles from its target, so
  the impact now lands on the target and the lunge is what stops at
  `REACH` (2.6 tiles). Visible look: owner sign-off.
- **Opposing seats are laid out independently**, as the plan says. Units
  of different seats can still overlap where both seats stand on their
  rank-0 row in the same place (see the numbers). Removing that needs one
  joint layout, which would move one seat's front line off its true `x`;
  the plan forbids that. Recorded as a limitation, not fixed.
- **Ignored probe.** `probe_bot_matches` is measurement code, kept so the
  numbers can be reproduced: `cargo test --release -p tbg-client
  probe_bot_matches -- --ignored --nocapture`.
- **D-57 on a gap.** D-56 is taken by parallel Cargo Crab work on another
  branch (`bc2942b`); this branch goes from D-55 to D-57, as the plan asked.

### Verification (real output)

Shared machine: other sessions kept the load average at 14–18 on 16 cores
during the browser runs, so only the native numbers and the build results
below are clean.

| Check | Result |
|-------|--------|
| `cargo fmt --all -- --check` | clean |
| `cargo clippy --workspace --all-targets -- -D warnings` | `Finished`, exit 0 (final code) |
| `cargo test --release -p tbg-client layout -- --include-ignored` | `test result: ok. 6 passed; 0 failed` |
| `cargo test -p tbg-client` (debug, final code) | `test result: ok. 101 passed; 0 failed; 1 ignored` (the ignored one is the probe) |
| `cargo check -p tbg-client -p tbg-core --target wasm32-unknown-unknown` | exit 0 |
| `node --test tests/*.test.js` | `pass 30`, `fail 0` |
| `trunk build --release` | `✅ success` (final code) |

### Layout unit tests (`visuals/layout.rs`)

| Test | Contract |
|------|----------|
| `no_overlap_while_the_lane_has_room` | 20 units at one `x` per seat, plus a spread army with large bodies: zero overlapping pairs |
| `front_unit_stays_at_its_true_x_and_nothing_is_ahead` | each seat's front unit is drawn at its `x`; no unit is drawn ahead of it |
| `same_input_gives_the_same_slots_in_any_order` | reversed input, identical slots |
| `surviving_units_keep_their_file_when_one_dies` | the front unit of seat A dies; every other unit keeps its file |
| `three_hundred_units_within_budget` | 300 units, fastest of 50 frames, must be under 2 ms in debug |

### Layout cost

| Case | Result |
|------|--------|
| 300 units, release, fastest of 50 frames (stress: 150 per seat packed into 14 tiles) | 295 µs; the budget is 200 µs, so **over budget in this stress case** |
| 300 units, debug | under the 2 ms test bound |
| bot matches, release, mean over 22,590 samples (about 11 units each) | 1.1 µs per frame |

A real army is at most about 28 units (the bot's peak), so the stress case
is about ten times larger than a real match needs.

### Overlap before and after (40 bot × bot matches, sampled every 0.5 s)

Probe: `probe_bot_matches`, offline `tbg-core` matches (no network). Radii
from the figure plan (0.36 small, 0.54 large). Both columns use the same
drawn-disc test (centres closer than `r₁ + r₂ − 0.02`).

| Metric | Old lane rule (`lane_z`) | New layout |
|--------|--------------------------|------------|
| Samples (22,590) with at least one overlap | 91.4 % | 19.6 % |
| Overlapping pairs in total | 225,933 | 4,663 |
| …of which same seat | 221,847 | 86 |
| …of which opposing seats | 4,086 | 4,577 |
| Units (249,321 unit-samples) overlapping at least one other | 63.6 % | 3.7 % |
| Units overlapping two or more others | 41.0 % | 0.014 % (36 units) |
| Worst pile: most overlaps on one unit | 16 | 3 |
| Opposing-seat pairs with both units on rank 0 | not counted | 3,850 (1,563 with two small bodies) |

The target from the plan, zero overlapping pairs, is not reached: the
remaining pairs are almost all between opposing seats, which the plan
lays out independently (see *Deviations*). Same-seat overlaps went from
221,847 to 86.

### Browser (desktop Chrome headless, software GL, ?measure, 1920×1080)

`?autoplay=practice&army=N&measure`, 45 s after the hook appeared, the
baseline build (`main` of this phase, `8587f9a`) against this branch.
Units are the sampled figures from `window.__getTbgMetrics()`.

| Run | Army | Build | FPS | p50 ms | p95 ms | p99 ms | Load 1 min |
|-----|------|-------|-----|--------|--------|--------|------------|
| 1 | 40 | baseline | 16.5 | 50.1 | 83.4 | 116.7 | ≈17 |
| 1 | 40 | new | 7.6 | 133.3 | 216.7 | 316.7 | ≈17 |
| 1 | 150 | baseline | 7.0 | 133.3 | 266.6 | 383.4 | ≈17 |
| 1 | 150 | new | 13.8 | 66.7 | 116.6 | 166.7 | ≈17 |
| 2 | 40 | new | 14.2 | 66.6 | 133.4 | 183.4 | 17.0 |
| 2 | 40 | baseline | 18.5 | 50.0 | 100.0 | 133.3 | 18.6 |
| 2 | 150 | new | 19.0 | 50.0 | 83.4 | 116.7 | 18.0 |
| 2 | 150 | baseline | 19.6 | 50.0 | 83.3 | 133.3 | 17.1 |
| 3 | 40 | new | 16.9 | 50.0 | 100.0 | 133.3 | 16.3 |
| 3 | 40 | baseline | 19.8 | 49.9 | 99.9 | 166.7 | 15.3 |
| 3 | 150 | new | 21.8 | 33.3 | 149.9 | 183.3 | 14.1 |
| 3 | 150 | baseline | 23.2 | 33.3 | 133.3 | 166.7 | 13.9 |

Reading: at army 40 the new build was slower in all three pairs (FPS 2–9
lower, p95 up to 130 ms higher). At army 150 the pairs are mixed. The
layout itself costs about 1 µs per frame at real army sizes, so this is
not the layout's arithmetic. The likely cause is the animator: eased
positions keep units moving, so more units walk and fewer idle. That is a
hypothesis, not measured. **The FPS and p95 criterion is not met on this
machine**, and the software GL plus the shared load make it impossible
to settle here. Re-measure on the owner's desktop when the machine is idle.

Screenshots (960×540 JPG, mid-match, not a comparison of the same frame):
`docs/media/p57/base-40.jpg`, `docs/media/p57/new-40.jpg`,
`docs/media/p57/base-150.jpg`, `docs/media/p57/new-150.jpg`.

### Not done

- The two-minute Practice match looked at for jitter and lane swaps, and a
  selection click followed to its ring and tag. Not run end to end. The
  selection code is unchanged: the tag and ring follow the unit's root, and
  the root now carries the slot.
- The overlap target of zero across opposing seats (see *Deviations*).
- The FPS and p95 criterion at `army=40` (see *Browser*).

### Owner sign-off (look and fun)

- The five-file, eight-rank look, and whether back ranks read well.
- Melee impacts on displaced back-rank attackers (*Deviations*).
- Whether the unit-to-unit overlap that remains between opposing seats is
  acceptable for the first version.
