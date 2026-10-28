# P17 — Medieval battlefield arena and backgrounds for every screen

**Depends on:** P16 reviewed.
**Branch:** `phase/p17-arena`.
**Difficulty:** medium–high (art quality + performance).

## Goal

A beautiful, living medieval battlefield that fills **every** screen shape
— phone portrait, phone landscape, tablet, 16:9 desktop, 21:9 ultrawide —
with no empty color bands, and stays cheap. The same diorama is the
animated background of the menu, lobby, queue, and result screens.

## Deliverable

- New arena: a dirt road (the `BattleLine`) across a meadow between the two
  Strongholds, framed by near props, a mid layer, and a far layer (hills,
  forest, river, fields, distant castle silhouettes), and a gradient sky
  dome with drifting clouds.
- Cheap ambient animation: banners, water, clouds, birds, chimney smoke,
  grass tufts, cloud shadows.
- Responsive framing for all aspect ratios; menu "establishing shot"
  camera with a slow orbit.
- Quality tiers wired into the arena (Low/Medium/High).

## Read first (only these)

- `AGENTS.md`
- `docs/ART-DIRECTION.md` §1–§3, §9
- Client: `src/arena.rs`, `src/arena/props.rs`, `src/camera.rs`, `src/main.rs`
- Bevy examples: `3d/fog.rs`, `3d/lighting.rs`, `3d/shadow_biases.rs`,
  `3d/vertex_colors.rs`, `3d/generate_custom_mesh.rs`

## Do NOT

- Change lane length, Stronghold positions, or anything in `tbg-core`.
- Add textures or post-processing passes; colors are vertex colors.
- Put props inside the lane or between the camera and the lane at any
  aspect ratio (P6 lesson: tall props on the camera side hide units).

## Steps

### 1. Layout and layers
`arena/` folder: `mod.rs` (spawn, tiers), `ground.rs` (meadow, road with
wheel ruts and stones, river bank), `near.rs` (fences, carts, barrels,
banners on poles, low bushes — **only low props on the camera side**),
`mid.rs` (forest edges, wheat fields, windmill, tents of both camps),
`far.rs` (hills ring, distant castles, mountains), `sky.rs` (gradient
dome mesh `#7FB8E6` → `#E8D9B5`, sun disc, 6–10 cloud meshes). All static
geometry merged per layer into ≤ 8 meshes total; deterministic placement
from a fixed seed.

### 2. Responsive framing
`camera.rs`: compute the visible world rectangle for the current aspect
and make sure ground/mid/far layers extend ≥ 15 % beyond it at 21:9 and at
9:21. The lane framing rules from P6 stay (lane along the long axis). Add
`?aspect=W:H` dev param that letterboxes the canvas to test ratios.
- **Done when** screenshots at 390×844, 844×390, 768×1024, 1920×1080,
  2560×1080 show no empty bands and the full lane.

### 3. Ambient animation (cheap)
- Banners and flags: 2–3 rigid segments each, rotated by a sine per
  banner (phase from position).
- Water: two layered strips sliding slowly; foam dots moving.
- Clouds: drift along X, wrap around; their shadows: dark translucent
  ellipses on the ground moving with them (Medium/High only).
- Birds: one flock of 5–8 tiny V shapes on a looping path (High only).
- Chimney/campfire smoke: 3 puffs per source recycled (Medium/High).
- Grass tufts near the road sway (High only).
- Total animated entities ≤ 80; zero allocation per frame.

### 4. Lighting and mood
Warm key light per ART-DIRECTION §2, cascade count 1, shadow map 1024
(High) / 512 (Medium) / off (Low); ambient two-tone; fog color = horizon.
Golden-hour variant for the menu only (`?mood=dusk` to preview).

### 5. Menu establishing shot
In `Menu`, `Lobby`, `Queue`, and `Result` states, the camera shows the
whole diorama from a higher angle and orbits ±8° over 40 s; UI panels sit
on top. Entering a match eases into the game framing over 0.8 s.

### 6. Quality tiers (arena part)
`Quality { Low, Medium, High }` resource (full auto-detection is P30):
Low = no shadows, far layer simplified, no birds/grass/cloud shadows;
Medium = 512 shadows, smoke, clouds; High = everything. `?quality=low|medium|high`.

### 7. Measure
Startup log: triangles per layer and total; draw calls in a busy match
(Bevy diagnostic or renderdoc count) at each tier.

## Acceptance checklist

- [ ] All verification commands from `AGENTS.md` pass; `trunk build --release` passes
- [ ] Arena total ≤ 45 000 triangles; busy-match draw calls within budget per tier
- [ ] Five aspect-ratio screenshots with no empty bands, in `docs/media/p17/`
- [ ] ≥ 30 FPS on a mid-range phone on Low and Medium with 30 units
- [ ] Owner approves the look (report)

## Risks and what to do

| Problem | Action |
|---------|--------|
| Far layers cost too much on phones | Merge further; drop far details on Low first |
| Busy scenery hurts unit readability | Desaturate and lighten mid/far layers 15 %; the lane stays the highest-contrast area |

## Reviewer checklist

- No prop occludes the lane at any ratio (check the five screenshots).
- Ambient animation allocates nothing per frame.

## Phase report

_Executor fills this in: triangle/draw-call tables per tier, FPS, screenshots,
owner approval, deviations._
