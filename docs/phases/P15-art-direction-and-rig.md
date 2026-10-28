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
- Rig v2 in `src/art/` (17 body joints + up to 3 spring joints + sockets),
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

### Step 4 evidence

- Eight gray v2 blockouts rendered in the release browser build.
- [Silhouette row](../media/p15/blockout-silhouette-desktop.jpg): small
  body height is approximately 64 px at 1280 × 800; Large bodies retain 1.5× scale.
- [Gray desktop](../media/p15/blockout-desktop.jpg) and
  [gray phone](../media/p15/blockout-phone.jpg).
- Reviewer judgment and owner glance on identification remain pending.
- Corrected the saw's viewing plane and hid arena/tower geometry for the
  silhouette board after inspecting the first capture.

### Step 5 evidence

- Native check and release build passed. Browser verified named `archer`,
  slow motion, turntable, silhouette, and frozen quarter-frame contact sheet.
- `&clip=attackranged` optionally narrows a contact sheet to one clip.
  Unfiltered `&frames` shows all ten clips of the selected base form;
  ordinary named-unit gallery shows all five forms cycling all clips.
- Gallery modules are each below 200 lines. Responsive layout reflows
  after Bevy receives the browser's initial resize.
- Palette phone capture was replaced with the correctly framed 375 × 812 view.
- `&measure` counts actual WebGL draw calls (including shadow passes) and
  rendered RAF frames over five seconds. Tool smoke check: 60.0 FPS,
  55 draws/frame for the old Archer quarter-frame sheet on this desktop.

### Step 6 evidence

- Archer base: 1,042 cached triangles on 19 mesh-bearing nodes, plus 74
  shared bow-string/arrow triangles = **1,116 triangles** (limit 1,500).
- Five client tests pass, including the actual wrist/bowstring contact at
  the 0.18 s Release anchor, strike sampling, event counts, and foot locking.
- [Archer desktop](../media/p15/archer-desktop.jpg),
  [Archer phone](../media/p15/archer-phone.jpg), and
  [slow-motion attack contact sheet](../media/p15/archer-attack-contact-desktop.jpg).
- Browser logged `Archer Release: 0.18 s, arrow leaves bowstring`; no
  browser errors in the reviewed gallery or 30-Archer stress views.
- Team tabard/fletching use cached ally/enemy vertex colors with the same
  material. Only Archer base uses final art; old evolutions remain adapted.
- Gallery studio floors keep foreground arena props from hiding review
  figures; palette and live-match checks still use the arena and game light.
- Physical mid-range-phone performance and subjective approval remain pending.

### Step 7 — owner sign-off pending

No owner answers were supplied. The executor does not approve the look,
display names, or D-22. No rejection count is inferred from implementation
iterations, and no fallback asset spike has been started.

The owner must record these three answers after reviewing the captures
and the running gallery:

- **(a) Look: owner sign-off pending.** Approve or list changes to the
  palette, body proportions, 64 px silhouette identification, hood/linen/
  bracer/longbow/quiver, team identity, planted feet, secondary lag, and
  draw → anchor → Release at 0.18 s → follow-through. Review
  `?gallery=palette`, `?gallery=blockout&silhouette`, `?gallery=archer`,
  and `?gallery=archer&slow=0.25&clip=attackranged&turn`.
- **(b) Display names: owner sign-off pending.** Approve or change
  Shieldbearer, Archer, Bone Brute, Hexer, Sawknight (`Sawbot`), Ogre
  Archer (`SiegeWalker`), Bloomkeeper, and Rootcaller. Code ids stay unchanged.
- **(c) Pipeline / D-22: owner sign-off pending.** Confirm keeping the
  code-built pipeline, or request changes. Only two actual owner
  rejections trigger the reviewer-run Blender-scripted glTF comparison.

Physical mid-range-phone Low-tier validation remains pending as well.
The desktop browser at a phone viewport is not a phone benchmark.

### Step 8 evidence

- ARCHITECTURE §7 now describes the implemented rig, adapter, shared
  assets/material, curves/events, studio review tools, and measurement limits.
- ART-DIRECTION records candidate proportions, inherited skin, spring
  settings, and Archer timings. *Start* marks and Proposed names/colors
  remain because no owner approval was supplied; locking is pending.
- D-22 and ROADMAP status are unchanged. The reviewer owns integration
  and status updates; no phase completion or approval is claimed.

### Review (reviewer, 2026-10-28)

Inherited the executor's seven commits plus its uncommitted report and
captures (committed as 42e6093), merged `main` (P10–P14 and the protocol v6
surrender/names/login fixes) into the branch with no conflicts, and re-ran
everything on the merged tip.

- **Scope:** the branch changes only `crates/tbg-client` and `docs`; no
  gameplay, server, protocol, or `tbg-core` change (the determinism goldens in
  `contract_determinism.rs` are unchanged and pass).
- **Verification:** `cargo fmt --check`; `clippy --workspace --all-targets -D
  warnings` and wasm32 clippy: 0 findings; wasm32 `cargo check` ok;
  `cargo test --workspace`: 93 passed, 0 failed (88 from `main` + 5 art tests:
  strike sampling, two footsteps per walk cycle, easing endpoints/Hold, planted
  foot holds world X, and the Archer budget + Release anchor);
  `trunk build --release` ok, WASM 24 492 794 bytes (+119 KB over `main`).
- **Archer budget:** 1 042 figure + 74 bow/arrow = **1 116 triangles** (limit
  1 500), asserted by `archer_budget_and_release_anchor_obey_p15`.
- **Standards:** every P15 file ≤ 200 lines with a `//!` header; `rig.rs`,
  `springs.rs`, `curves.rs`, `feet.rs`, and `mesh.rs` name no unit (rig v2 is
  data-driven). Observation: `anim.rs` routes the Archer base through an
  `archer` flag to `archer_clips` — acceptable for the style lock; P16
  (per-Archetype clip sets) should replace the flag with data.
- **Joint count:** the Deliverable said 18 body joints while the spec's exact
  enum (and the code) has 17; resolved in the spec text as 17.
- **Browser (release build, merged tip, 1280×800, in-app browser):**
  `?gallery=archer` (base in final art + four adapted evolutions),
  `&frames&clip=attackranged` contact sheet (identical frames to the executor's
  capture: rest, anchor, follow-through, recovery), and
  `?gallery=blockout&silhouette`. A live practice match (`?autoplay=practice`)
  shows the new Archer next to adapted enemy units (Siege Walker, Bone Brute)
  with the v6 HUD (names beside the bars, Surrender): 41–81 FPS, no console
  errors — [live match](../media/p15/live-match-adapter-desktop.jpg).
- **Silhouette row (reviewer judgment):** all eight are identifiable at the
  64 px row by their hooks; the weakest is Rootcaller, whose root claws read
  as a small spike — revisit in P29. Archer and Siege Walker both show a bow;
  body size (Humanoid vs Large) separates them.
- **Performance:** `?gallery=archer&stress=30&measure` reproduced **169 draws
  per frame** (executor: 168.9). FPS cannot be judged here: this browser pane
  renders WebGL in software (it logs no GPU compute/clustering) and reported
  0.7 FPS for 60 shadowed figures. The executor's GPU desktop measurement
  (165 FPS) stands; the ≥ 30 FPS mid-range **phone** criterion stays open
  until a physical phone is measured.

**Status: implementation complete and reviewed; owner sign-off (a), (b),
(c) of step 7 pending**, together with the step-8 locking it unlocks (remove
*start* marks in ART-DIRECTION for approved values) and the physical-phone
FPS check. P16 starts only after (a) is approved.
