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

### P30-C — desktop motion over the VPS (2026-10-29)

Desktop only (owner rule). Branch `phase/p30c-desktop-fluidity`, cut from
`phase/p16f-infinite-economy` (PR #6) because the timeline reads
`MatchView::clock`; client-only, no protocol or server change.

**Diagnosis.** The client drew units between the last two views, timed by
their *arrival* (`alpha = (now − last_t) / (last_t − prev_t)`, clamped to
1). Any view later than its predecessor's gap froze every unit until it
came (alpha stuck at 1); any earlier view restarted the lerp from the
previous view and moved units by a jump. On localhost views arrive in
phase with frames, so it looked fluid; over Cloudflare → sslh → nginx →
the k3s pod they do not.

Evidence: two `tbg-bots` (built from `main` 9a813d7, protocol v9, a local
probe patch logging arrival time and tick per view, not committed) played
a full casual match through `wss://tbg.abastevo.com.br/ws`: 2 464 views
per seat, gap p50 100 / p95 117–124 / p99 ~190 / max ~410 ms, 5 % of gaps
under 70 ms, 3.5 % over 130 ms; RTT p50 49 / p95 95 ms. The server clock
fell 2.2 s behind wall time in one burst of late ticks about 100 s in
(VM CPU steal; `/metrics` showed 42 late of 9 376 ticks). The trace is in
`crates/tbg-client/src/sync/timeline/vps-trace.txt`.

**Fix.** `sync/timeline.rs`: views keyed by server clock; a lower-envelope
transit estimate (drops at once on a faster view, rises 10 % per view so a
late view barely moves it but a server slowdown is followed in ~1 s);
delay = 100 ms + p95 jitter of the last 32 views + 10 ms, clamped
100–350 ms; the render clock never runs backwards, eases at ±10 % speed
toward its target, and jumps only beyond 0.5 s (reconnect); when nothing
newer exists it holds the newest view (counted as starved). Events are
released when the render time reaches their view, so Spawn, Death,
attacks and Evolutions stay in step with positions. A clock that restarts
(new match) clears the buffer. Augment pauses need no special case: the
clock runs while positions are frozen. The per-frame `MatchView` clone of
the old path is gone (frames borrow the buffer). `?measure` adds frame
p50/p95/p99 and frames over 33 ms, view-gap p99, delay/buffer/starve/snap/
stale and a motion probe (stalled and jumping unit-frames), mirrored on
`<html data-tbg-measure>`.

**Harness** (`sync/timeline/tests.rs`, 60 FPS replay, one unit at
1 tile/s; legacy = the old formula reimplemented in the test):

| Arrival schedule | Legacy stall / jumps | Timeline stall / jumps | Mean visual delay legacy → timeline |
|---|---|---|---|
| Stable 100 ms | — | 0 % / 0 | — → < 160 ms (asserted) |
| Jitter 60–140 ms | 16.1 % / 225 | 0 % / 0 | 55 → 84 ms |
| Real VPS trace (4 min) | 6.80 % / 319 | 1.12 % / 0 | 55 → 92 ms |

On the VPS trace 89 of the remaining 165 stalled frames fall in the ten
seconds where the server itself fell behind (no newer state existed);
elsewhere it is ~0.5 %. No backward step in any schedule. Also covered:
250 ms holds every 20th view (bounded buffer, no backward step), a late
view (counted stale, its events still play), twenty views in one frame,
spawn/death timing, a 10-s reconnect (one snap), a new match, and a 2-s
Augment pause (frozen, resumes without a jump).

**Not measured (honest gaps).** Browser FPS and frame-time p95/p99: the
in-app browser pane was hidden while measuring, so the page ran as a
background page (~2 FPS, the same throttle P30-A hit); the new overlay
reports them with `?measure` once someone watches the page. CPU, memory
and bytes per player in the browser: not measured; the change adds no
traffic and removes one view clone per frame. The public-domain A/B
(Scenario C) needs this branch deployed to `tbg-temp`.

**Costs.** About 30–40 ms more visual delay on average (the price of never
freezing); HUD values (Energy, Shop, War Council) still update on arrival.

