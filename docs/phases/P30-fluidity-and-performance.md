# P30 — Fluidity and performance for every player

**Depends on:** P29 reviewed (all eight units final).
**Branch:** `phase/p30-fluidity`.
**Difficulty:** medium–high (profiling on real devices).

## Goal

The game runs smoothly on any reasonable device — from a budget Android
phone to a desktop — with no hitches, fast loading, and instant-feeling
input. Quality adapts automatically; information never disappears.

## Deliverable

- Automatic quality selection with a manual override (Low/Medium/High/Auto).
- Hitch-free first match (pipelines and meshes warmed during loading).
- Input that responds on press, not release.
- A device matrix with measured FPS, frame-time p95, load time, memory.
- WASM and asset sizes at or under budget.
- Pooled effects and projectiles with zero per-frame allocation (moved
  here from the archived P21, D-31).
- Triangle counts per unit form and arena layer reported (no fixed cap,
  D-30); detail is reduced only where a measured FPS or memory target fails.

## Read first (only these)

- `AGENTS.md`
- `docs/ART-DIRECTION.md` §9 (budgets)
- `docs/ARCHITECTURE.md` §6, §10
- Client: `src/main.rs`, `src/overlay.rs`, `src/ui/input.rs`, `src/ui/kit/`,
  `src/arena/`, `src/art/`, `src/vfx/`, `crates/tbg-client/Cargo.toml`, `index.html`

## Do NOT

- Remove information to gain speed (team rings, HP bars, statuses, splash
  rings stay on every tier).
- Change simulation, network rates, or protocol.
- Raise any budget.

## Steps

### 1. Device matrix baseline
Measure on: desktop Chrome, desktop Firefox, desktop Safari (if
available), a mid-range Android (Chrome), a budget Android (≤ 3 GB RAM),
an iPhone 11-class (Safari). For each: time to menu (cold cache, 4G
throttling for phones), FPS and frame-time p95 in a busy match (30 units)
at each tier, browser memory. Use the overlay plus `?stress=30`.

### 2. Automatic quality
`quality.rs`: start at Medium (Low when `navigator.hardwareConcurrency
<= 4` or `deviceMemory <= 3`); during the first 10 s of every match
measure frame-time p90; step down if > 33 ms, step up if < 14 ms and the
next tier is allowed; at most one change per match, never mid-fight more
than once; remember the last tier in `localStorage`. Manual choice in
Settings disables Auto. Tiers also scale render resolution: Low 0.75,
Medium 0.9 (phones) / 1.0 (desktop), High 1.0 — via the window scale
factor override.

### 3. Warm-up during loading
During the loading screen/lobby: spawn every unit form, Stronghold style,
and VFX mesh once off-camera for 2 frames (forces pipeline and buffer
creation), render portraits, then despawn. Measure the first-match
frame-time max before and after.

### 4. Frame pacing and battery
Cap at the display refresh (vsync). In Lobby/Result with no animation
changes, lower the update rate to 30 FPS on phones (`WinitSettings`
reactive mode where supported); full rate in matches.

### 5. Input responsiveness
Shop cards and evolution choices act on **press** (touch start / mouse
down) with immediate pending visuals; scrolling gestures cancel. Unit
tap volume stays enlarged. Measure tap-to-pending-visual (target ≤ 1 frame).

### 6. Size and load
- Re-check Bevy features (drop anything unused since P19), `wasm-opt -Oz`,
  `opt-level = "z"` vs `"s"` (measure both: size and FPS).
- Fonts/icons/audio within the first-load budget; audio loaded lazily after
  the first frame.
- Precompressed brotli and gzip in the deploy kit; long cache headers on
  hashed files.

### 7. After matrix
Repeat step 1. Table before/after per device and tier.

## Acceptance checklist

- [ ] All verification commands from `AGENTS.md` pass; `trunk build --release` passes
- [ ] Every device in the matrix ≥ 30 FPS in a busy match on its auto tier;
      desktops 60 FPS
- [ ] First-match frame-time max ≤ 50 ms after warm-up
- [ ] WASM ≤ 6 MB brotli (target ≤ 4.5 MB); first-load other assets ≤ 1.5 MB
- [ ] Time to menu within ART-DIRECTION §9 targets

## Reviewer checklist

- Auto quality never toggles repeatedly (check the log over 3 matches).
- Low tier still shows all gameplay information (screenshot comparison).

## Phase report

_Executor fills this in: device matrix before/after, sizes, deviations._
