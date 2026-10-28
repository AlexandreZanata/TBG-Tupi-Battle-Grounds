# P15 — Medieval art direction, rig v2, and style lock

**Depends on:** P14 signed off by the owner.
**Branch:** `phase/p15-art-direction`.
**Difficulty:** high (rig v2 replaces the 12-joint body plan). The
reviewer may do step 6 (style lock) personally.

## Goal

Lock the medieval look before producing anything at scale: palette,
proportions, rig v2 with knees, feet and secondary motion, gray-box
blockouts of all eight units, and **one style-lock character (Archer base
form)** that the owner approves. Everything after this phase copies it.

## Deliverable

- `docs/ART-DIRECTION.md` values locked (palette, proportions, names).
- Rig v2 in `src/art/` (18 body joints + up to 3 spring joints + sockets),
  with the old 12-joint figures still rendering through an adapter until
  each unit is redone (P22–P29).
- `?gallery=blockout`: all 8 units as gray-box blockouts on rig v2.
- `?gallery=archer`: Archer base form at final quality with idle, walk,
  and attack clips.
- Owner sign-off on look and display names; D-22 confirmed or switched.

## Read first (only these)

- `AGENTS.md`
- `docs/ART-DIRECTION.md` (all)
- `docs/DECISIONS.md` D-07 and D-22
- `docs/ARCHITECTURE.md` §7 and §10
- Client: `src/art/` (all), `src/visuals.rs`, `src/visuals/*`, `src/gallery.rs`
- Bevy examples: `3d/generate_custom_mesh.rs`, `3d/parenting.rs`,
  `animation/eased_motion.rs` (easing curves)

## Do NOT

- Change gameplay, the server, the protocol, or `tbg-core`.
- Add textures, skinned meshes, glTF, or downloaded models (see step 7 for
  the only exception path).
- Redo more than the Archer base form at final quality (the rest is P22+).

## Steps

### 1. Palette and lighting lock
- `art/palette.rs`: every ART-DIRECTION §3 color as a named `const`, plus a
  `PaletteSlot` enum (Base, Shade, Accent, Linen, Team, Skin, Metal, Glow…).
- Lighting per §2 (key, two-tone ambient, fog). A `?gallery=palette` board:
  one swatch sphere per slot and per faction under the game light.
- **Done when** the palette board screenshot is in the report.

### 2. Rig v2
Files: `art/rig.rs` (joints, hierarchy, rest poses, ≤ 200 lines),
`art/springs.rs` (secondary joints), update `art/figure.rs`.
```rust
pub enum Joint { Root, Hips, Spine, Chest, Head,
                 UpperArmL, ForearmL, HandL, UpperArmR, ForearmR, HandR,
                 ThighL, ShinL, FootL, ThighR, ShinR, FootR }
pub enum Spring { Back, HeadGear, Tabard }      // optional per figure
pub enum Socket { WeaponR, OffhandL, Back }
pub enum BodyPlan { Humanoid, Large }           // proportions per ART-DIRECTION §2
```
- Spring joints: damped spring on rotation (stiffness and damping per
  spring, defaults 60 and 8), driven by the parent's angular and linear
  velocity; clamped to ±35°.
- Keep the merged-mesh-per-joint cache and one shared material (D-07 rules).
- Adapter: old `FigureSpec`s map their 12 joints onto rig v2 (legs → thigh,
  shin and foot share the old leg part) so every unit keeps rendering.
- **Check:** `?gallery` still shows all 8 old units animating; startup log
  prints joints and triangles per figure.

### 3. Animation curves upgrade
`art/anim.rs`: keys gain an easing (`Linear`, `CubicIn`, `CubicOut`,
`CubicInOut`, `Hold`); clips gain `events: &[(f32, ClipEvent)]` with
`ClipEvent::{Strike, Release, Footstep}`; foot locking: during `Walk`, the
planted foot's world X is held while in contact (cycle length derived from
the unit's speed from `tbg-core::content`). Per-unit random phase offset.
- **Check:** unit test: sampling a clip at its `Strike` time returns the key
  pose; footstep events fire twice per walk cycle.

### 4. Gray-box blockouts (all 8)
`art/roster/blockout.rs`: each unit as simple gray shapes on rig v2 with
the correct body plan, proportions, and the silhouette hook from
ART-DIRECTION §5 (shield, longbow, cleaver, staff, saw arm, great-bow,
flower staff, root claws). `?gallery=blockout&silhouette` renders them
as black silhouettes in a row.
- **Done when** each unit is identifiable in the silhouette row at 64 px
  height (reviewer judgment + owner glance), screenshot in the report.

### 5. Gallery tools
`?gallery=<unit>` (one unit, all forms, all clips), `&slow=0.25` (slow
motion), `&silhouette`, `&turn` (turntable), `&frames` (pauses each clip at
0/25/50/75 % for contact sheets). Keep `gallery.rs` ≤ 200 lines (split).

### 6. Style lock: Archer base form
`art/roster/humans/archer.rs`: final-quality base form (hood, linen tunic,
leather bracer, longbow taller than the figure, quiver on the `Back`
spring, team-color tabard and fletching), ≤ 1 500 triangles. Clips at
final quality: `Idle`, `Walk` (foot-locked), `AttackRanged` with draw →
anchor → release at 0.18 s → follow-through, and an arrow that leaves the
bowstring on `Release`.
- **Done when** desktop + phone screenshots and a slow-motion contact
  sheet are in `docs/media/p15/`.

### 7. Owner sign-off and pipeline decision
The owner reviews the palette board, silhouette row, and the Archer, and
answers in the report: (a) look approved / changes; (b) display names in
ART-DIRECTION §5 approved / changed; (c) pipeline: keep code-built
(D-22). If the owner rejects the Archer twice, the reviewer runs the
fallback spike from D-22 (Blender-scripted skinned glTF Archer) and the
owner chooses; record the outcome in DECISIONS.

### 8. Docs
Lock values in ART-DIRECTION (remove *start* marks that were approved);
ARCHITECTURE §7 points to ART-DIRECTION and describes rig v2.

## Acceptance checklist

- [ ] All verification commands from `AGENTS.md` pass; `trunk build --release` passes
- [ ] Old units still render and animate through the adapter (live match OK)
- [ ] Archer base form ≤ 1 500 triangles; ≥ 30 FPS on a mid-range phone in
      `?stress=30` (Archer only)
- [ ] Owner answers (a), (b), (c) recorded

## Risks and what to do

| Problem | Action |
|---------|--------|
| Rig v2 breaks the old figures | Keep the adapter until P29; never delete an old spec before its unit phase |
| Springs jitter at low FPS | Sub-step springs at fixed 120 Hz inside the frame |
| Draw calls rise with more joints | Merge non-animated joints (e.g. Spine+Chest when a unit never bends) per figure; measure |

## Reviewer checklist

- Rig v2 code is data-driven (no unit-specific code in `rig.rs`).
- Contact sheets show anticipation and follow-through on the Archer attack.
- Screenshots are real captures from the build, not mockups.

## Phase report

Status: implementation in progress; **owner sign-off pending**.

### Step 1 evidence

- Release WASM palette board observed in the browser, no console warnings/errors.
- Screenshot: [palette desktop](../media/p15/palette-desktop.jpg) (1280 × 800).
- `trunk build --release`: `Finished release profile [optimized] target(s) in 48.31s`.
- Two-tone ambient uses the sky fill plus baked ground-facing vertex color;
  the key light and fog are shared with the game.

### Deviations (in progress)

- The phase says 18 body joints, but its exact enum contains 17. Use the
  17 specified names, plus three secondary joints and three sockets.
- P14 sign-off was not supplied with this request; execution starts from
  the requested P13 main. P14 owner validation remains an external prerequisite.
- The specified server command needs `--bin tbg-server`: Cargo reports
  `error: cargo run could not determine which binary to run. Use the --bin option to specify a binary, or the default-run manifest key.`

### Step 2 evidence

- Native check passed (temporary unused v2 builders will be consumed by step 4).
- Release WASM gallery observed: all eight old units, all five forms,
  changing poses through the adapter; no browser errors.
- [Adapter gallery](../media/p15/adapter-gallery-desktop.jpg), 1280 × 800.
- Startup reports 17 body joints + 3 springs + 3 sockets; legacy forms
  use 15 mesh-bearing joints. Full geometry measurements are recorded below.
- The first Trunk process ended with exit 143 and no Rust diagnostic.
  Retry succeeded; Rust release compilation took 53.54 s.

### Step 3 evidence

- `cargo test -p tbg-client`: 4 passed; 0 failed.
- Tests cover sampling the 0.30 s Strike key, two Footstep events per
  cycle without duplicate boundaries, all easing endpoints/Hold, and a
  planted foot retaining world X as the figure moves.
- Walk cadence uses content speed and stride length; instance phase is
  decorrelated by a stable hash of the visual unit id.
- Bevy's example uses optional `bevy_curve` symbols absent from this
  feature set. Equivalent cubic formulas keep the existing dependency set.
