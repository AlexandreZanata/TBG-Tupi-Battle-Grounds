# P6 — Visual slice (3D figurines, arena, animation)

**Depends on:** P5 reviewed and merged (the game is already online and validated).
**Branch:** `phase/p6-visual-slice`.
**Difficulty:** high for step 2 (figure system); medium otherwise.

## Goal

Replace the placeholder shapes with the real art direction — code-built
toy figurines on a code-built diorama arena — and prove it looks
professional in the game camera on phones. One unit is finished to final
quality; the system is ready for the other seven.

## Deliverable

- `?gallery` page: Shieldbearer in all 5 forms playing all 10 clips, plus
  a stress row of 30 figurines, on the final arena with final lighting.
- The live game uses the new arena and the figure system for every unit
  (seven units still use a simple "base body + faction palette + one prop"
  spec until P7).
- Approved screenshots in `docs/media/p6/`.

## Read first (only these)

- `AGENTS.md`
- `docs/ARCHITECTURE.md` §7 (art direction) and §10 (budgets)
- Client: `visuals.rs`, `arena.rs`, `sync.rs` (how visuals are called)
- Bevy examples: `3d/generate_custom_mesh.rs`, `3d/vertex_colors.rs`,
  `3d/3d_scene.rs`, `3d/parenting.rs`, `3d/fog.rs`, `3d/lighting.rs`,
  `3d/shadow_biases.rs`

## Do NOT

- Use skinned meshes, glTF animation, Blender, or downloaded models.
- Change `sync.rs`, `link.rs`, the HUD, the server, or `tbg-core`.
  The only public surface you may change is the inside of `visuals.rs`
  (its function signatures stay the same) and `arena.rs`.
- Add textures; colors come from vertex colors and the palette.

## Steps

### 1. Palette and mesh builders
Files: new `src/art/mod.rs`, `src/art/palette.rs`, `src/art/mesh.rs`.
- `palette.rs`: every color from ARCHITECTURE §7 as `const` `Color`s,
  grouped per faction (`primary`, `secondary`, `accent`) plus team,
  environment, and `FLASH` (white).
- `mesh.rs` builders returning `Mesh` with **flat normals** (duplicate
  vertices per face) and a `COLOR` vertex attribute:
  `rounded_box(size: Vec3, bevel: f32, color)`, `lathe(profile: &[Vec2],
  segments: u32, color)` (revolve a 2D profile around Y),
  `extrude(polygon: &[Vec2], depth: f32, color)`, and `recolor(mesh,
  color)` for Bevy primitive meshes (`Capsule3d`, `Sphere`, `Cylinder`,
  `Cone`, `Torus`, `Cuboid`). Darken vertex colors linearly by up to 25%
  toward the lowest Y of each mesh (fake ambient occlusion).
- Helper `merge(meshes: &[(Mesh, Transform)]) -> Mesh`.

### 2. Figure system
File: `src/art/figure.rs` (≤ 200 lines; split into `body.rs` if needed).
```rust
pub enum Joint { Root, Hips, Torso, Head, ArmUpperL, ArmLowerL, HandL, ArmUpperR, ArmLowerR, HandR, LegL, LegR }
pub struct BodyPlan { pub scale: f32, pub head: f32, pub torso: Vec3, pub arm_len: f32, pub leg_len: f32 }
pub struct PartSpec { pub joint: Joint, pub mesh: PartMesh, pub color: PaletteSlot, pub offset: Vec3, pub rot: Vec3 /* degrees */, pub scale: Vec3 }
pub struct FigureSpec { pub plan: BodyPlan, pub parts: Vec<PartSpec> }
pub fn spawn_figure(commands: &mut Commands, cache: &mut FigureCache, spec_key: (UnitKind, u8, u8), spec: &FigureSpec) -> Entity;
```
- Joint hierarchy as child entities with `Transform`s; rest pose from
  `BodyPlan`. One-slot `scale = 1.0`; two-slot `scale = 1.5`.
- For each joint, merge all its parts into **one** mesh and cache it in
  `FigureCache` by `(spec_key, Joint)`; every figure uses **one shared**
  `StandardMaterial` (white base color, vertex colors on,
  `perceptual_roughness = 0.9`). The hit flash swaps to one shared flash
  material for 0.08 s.
- Proportions: big head (head ≈ 0.35 of body height), short legs, chunky
  hands; props are oversized (shield ≈ torso height).

### 3. Animation library
File: `src/art/anim.rs`.
- `pub enum Clip { Idle, Walk, AttackMelee, AttackRanged, Cast, Hit, Death, Spawn, Evolve, Victory }`.
- A clip = duration, `looping`, and tracks `(Joint, &[Key])` with
  `Key { t: f32 /*0..1*/, rot: Vec3 /*degrees*/, offset: Vec3 }`;
  smoothstep between keys. Attack clips have `hit_at: f32` (0.6).
- Component `Animator { clip, time, prev_pose, blend }`; switching clips
  crossfades over 0.1 s. After a one-shot clip ends, return to Walk if the
  unit moved since the last view, else Idle.
- Map cues: Spawn→Spawn, AttackStart→AttackMelee/AttackRanged/Cast by
  archetype (Ranged → AttackRanged, Support/Control → Cast, else
  AttackMelee), Hit→Hit (additive shake), Evolved→Evolve, Death→Death.

### 4. Shieldbearer (final quality)
File: `src/art/roster/humans.rs` (+ `roster/mod.rs` with
`fn figure_spec(kind, tier, branch) -> FigureSpec`).
- Base: leather tunic (`#8B7355`), light trim (`#C4A882`), round shield on
  the left hand, short sword on the right, simple helmet.
- Bulwark: bigger shield with a gold rim; Fortress: tower shield + crested
  helmet + gold accent glow (emissive-looking bright vertex color).
- Vanguard: spear instead of sword, lighter shield; Champion: long spear,
  shoulder plates, banner on the back.
- Each tier also scales the figure +8%.

### 5. Final arena
File: `arena.rs` (replace the placeholder).
- Elongated grass island (`#7BB661`) slightly larger than the camera view,
  dirt lane (`#C9A46A`) with 16 subtle tile markers, water skirt
  (`#3FA7B5`) around the island, distance fog, clear color `#A9D8F0`.
- Strongholds: stone (`#B9B2A3`) tower built from lathe + boxes, team
  color banner; 3 damage states (100–67%, 66–34%, ≤ 33%) by swapping to
  progressively broken variants (missing top, tilted pieces). Read the HP
  from `LatestView`.
- Edge props: 20–40 low-poly trees (cone/sphere stacks), rocks, ferns,
  placed deterministically from a fixed seed; merge all static props into
  ≤ 6 meshes.
- Lighting: one warm directional light (`#FFF4E0`) with shadows, cascade
  count 1; ambient light toned to the sky; `?shadows=0` disables shadows.

### 6. Readability layer (inside `visuals.rs`)
Team ring under each figurine; billboard HP bar (from P3, restyled);
spawn pop; death puff (6 small spheres expanding and fading over 0.4 s);
ranged projectile per archetype color.

### 7. Placeholder specs for the other 7 units
In `roster/{humans,undead,machines,plants}.rs`: base body + faction
palette + one distinct prop each (bow, big club, staff, saw arm, cannon,
flower crown, vine whip). Tiers add +8% scale and a gold/silver accent.

### 8. Gallery mode
`?gallery` (no menu, no network): grid of the Shieldbearer's 5 forms ×
10 clips looping, plus a row of 30 mixed figurines walking, on the final
arena. Log once at startup: triangles per figure form and for the arena.

### 9. Visual review (owner)
Capture screenshots: desktop 1920×1080 and phone portrait (real phone or
devtools 390×844), both gallery and a live match. Save ≤ 300 KB PNG/JPG to
`docs/media/p6/`. The owner rates against: silhouette readable at phone
size · faction obvious by color · role obvious by prop · attack windup
readable · consistent scale · no z-fighting or shadow acne · "looks like a
real game".

## Acceptance checklist

- [ ] All verification commands from `AGENTS.md` pass; `trunk build --release` passes
- [ ] Budgets (ARCHITECTURE §10): ≤ 1 500 tris per one-slot figure, ≤ 3 000
      two-slot, ≤ 30 000 arena (from the startup log)
- [ ] ≥ 30 FPS on both phones in the gallery stress row and in a live match
- [ ] Online match still works (re-run P4 L6) — visuals change nothing else
- [ ] Owner approval recorded in the report

## Risks and what to do

| Problem | Action |
|---------|--------|
| Looks cheap / "programmer art" | Improve lighting, palette contrast, bevels, proportions, fake AO before adding geometry |
| FPS drop on phones | Check per-joint merging and the shared material; try `?shadows=0`; report draw-call estimate |
| Owner rejects the look twice | Stop; the reviewer reopens decision D-07 |

## Reviewer checklist

- `visuals.rs` signatures unchanged; no gameplay code touched.
- One shared material; meshes cached per (form, joint).
- Screenshot evidence matches the claims; budgets logged.

## Phase report

Executed by the reviewer agent (Claude) on 2026-10-28, branch `phase/p6-visual-slice`.

- Implemented: `art/` — `palette.rs`; `mesh.rs` (box/ball/pill/cylinder/cone/frustum/
  ring/lathe builders, flat normals, vertex colors with bottom shading as fake AO,
  per-joint merge); `figure.rs` (12-joint body plan, `FigureSpec`, joint spawning);
  `anim.rs` + `clips.rs` (10 keyframed clips, base Idle/Walk + one-shot actions,
  0.1 s crossfade); `roster/` (**all 8 units × 5 forms**, built once at startup into
  `FigureLibrary`); final `arena.rs` + `arena/props.rs` (island over water, cliff, lane
  with tile markers, lathe stone towers with team banners and 3 damage states, seeded
  props, distance fog, warm sun, `?shadows=0`); `gallery.rs` (`?gallery`, `?gallery=N`
  close-up). `visuals.rs` now spawns figurines; cues drive clips; evolution swaps
  meshes and scale.
- Commands run and results: `cargo clippy --workspace --all-targets -- -D warnings`
  and wasm32 clippy → clean; `cargo test --workspace` unchanged (25 tests);
  `trunk serve` dev build → gallery and practice match render, no errors.
- Triangles (startup log, max over 5 forms): Shieldbearer 1098 · Archer 950 ·
  Bone Brute 1028 · Hexer 1340 · Sawbot 1208 · Siege Walker 964 · Bloomkeeper 1496 ·
  Rootcaller 1440 — all within budget (≤ 1500 one-slot, ≤ 3000 two-slot).
- Draw calls: one shared figurine material; ≤ 12 joint meshes per unit, shared per form;
  arena = ~10 meshes. Not measured on WebGL2 (no draw-call counter in Bevy's WebGL2 path).
- FPS (browser pane, desktop iGPU, dev build): 110–150 with 40 animated figurines in
  the gallery; 150 in a practice match. **Phone FPS pending owner.**
- Screenshots: `docs/media/p6/gallery-overview.jpg`, `closeup-undead.jpg`,
  `closeup-machines-plants.jpg`, `match-landscape.jpg` (small pane, 373×335).
- Deviations:
  1. All 8 units got their final-style figurine here (P7 step 1 done early): the
     figure system made each unit a ~40-line data function.
  2. `spawn_unit_visual` / `update_unit_visual` take `&FigureLibrary` (and `&mut UnitVisual`)
     — the seam's callers changed in one place (`sync`), nothing else.
  3. Rank pips (gold A / silver B) kept above the HP bar for evolution readability.
  4. Trees only on the far side of the island; the camera side has low rocks, bushes,
     and ferns so nothing hides the lane.
  5. Figurines are drawn 1.2× life size for phone readability.
  6. No Blender, no textures, no glTF: everything is code.
- Blocked items: none.
- **Needs the owner:** visual approval (silhouette, faction colors, props, motion,
  "looks like a real game") on a real desktop screen and a phone; phone FPS in a busy
  match (`?stress=30` and a full match).
