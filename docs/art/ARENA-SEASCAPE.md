# Arena seascape visual follow-up

## Scope and base

Owner-requested procedural 3D environment improvement, based on `origin/main` at `6ef155bc9540527c99086aee0d9726908ca6a041`. Branch: `codex/arena-seascape`; isolated worktree `TBG-arena-wt`. No push, PR, merge or VPS deployment is authorized for this task. The supplied image is a visual reference, never a background asset.

The base does not yet include the separate ranking/lobby/Faction PR #14. This branch preserves the UI and cards from its requested main baseline; it changes no UI asset or gameplay rule.

## A — Diagnosis and baseline

Read AGENTS.md, ARCHITECTURE sections 7/10, ART-DIRECTION sections 1–3/9, the relevant P30 measurement guidance, and the existing arena, lighting, camera, mesh helpers and diagnostics. Existing building blocks suffice: flat-shaded vertex-colored Surface/painted meshes, seeded Pcg32 dressing, shared StandardMaterials, one directional light, a 1024 shadow map and one cascade, ambient fill, distance fog and transform-only cosmetic animation. No dependency or alternate rendering architecture is needed.

Current weak points: the sea is one unlit, uniform-color cuboid (12 triangles); coast consists of three continuous color belts; the horizon has no island silhouettes or maritime landmarks. The principal island already has detailed tree/rock/camp geometry and remains the focal point. Existing environment total: 35,868 triangles (props 31,828, meadow 192, BattleLine 444, cliffs 384, sea 12, foam 384, offshore rocks 2,000, life 624).

Selected additions: faceted blue/turquoise depth variation; broken coastal foam and a sparse drift layer; two small background isles with simpler trees/rocks; a compact lighthouse/pier and one sailboat. Prefer merged geometry, shared materials, no distant shadow casters and very few animated transforms. No bird particle system, reflections, physics water, new textures, light sources or post-process pass.

Initial browser baseline uses the existing `?arena&stress=60&measure` path, fixed production camera and 60 animated mixed-roster models. These are synthetic rendering load, not 60 competitive units. Diagnostics instrumentation is identical before/after and opt-in: actual WebGL draw counts, existing Rust rolling frame percentiles, first-frame timing and Chromium JS heap (partial memory only). One foreground IAB page at a time; no browser automation via another API.

Initial warm-cache samples (15 one-second reads, retaining elapsed >=10s):
- 1920px: 5 retained samples; median frame p95 18.60 ms; 60.0 FPS; 422.3 draw calls.
- 1366px: 15 retained samples; median frame p95 7.30 ms; 163.0 FPS; 422.6 draw calls.

A real local Practice at 0:40 had 8 visible units (5 in the player army), 60 FPS, rolling frame p95 18.8 ms and ~294 draw calls. This is functional/visual evidence, not a replacement for the controlled 60-model comparison.

Baseline release build passes. Raw WASM 24,923,835 bytes; gzip-9 7,768,767; Brotli-11 4,855,741; asset folder 14,613,050 bytes. These are existing baseline sizes, not newly raised budgets. [Build measurement](../media/arena-seascape/build-before.json).

Before captures: [empty 1920](../media/arena-seascape/sea-empty-before-1920.jpg), [empty 1366](../media/arena-seascape/sea-empty-before-1366.jpg), [60 models 1920](../media/arena-seascape/sea-stress-before-1920.jpg), [60 models 1366](../media/arena-seascape/sea-stress-before-1366.jpg), [Practice 0:40](../media/arena-seascape/sea-practice-before-1920.jpg).

## B — Ocean

Implemented a seeded 88x64 irregular water grid with subtle static relief, blue depth/turquoise near-shore color variation and a coarse exterior fill, all merged into the existing sea draw. Broken white coastal arcs, shallow transition bands, foam around the ten original offshore rocks and sparse glint ribbons share the existing cosmetic Foam batch and transform drift. Removed the old torus rings from the lit terrain batch; offshore rock positions, scales and rotations remain unchanged. No texture, shader, reflection pass, new material or per-frame vertex upload. The original two sea/foam draw batches and animation entity count remain unchanged.

`cargo check -p tbg-client --target wasm32-unknown-unknown`: pass (2.35 s initial, 0.39 s after non-coplanar foam refinement). `cargo fmt --all`: pass. Final appearance and timing are inspected after assembly; no performance claim yet.

## C — Offshore composition

Added two irregular, low-poly offshore islands, eight small pines, faceted shore rocks and two skerries. The western island carries a compact stone/red-band lighthouse and a wooden pier. One ivory-sailed boat has a bounded transform-only bob/roll. A small eastern cascade and its foam pool share the existing unlit foam batch. All shore foam shares the existing cosmetic drift. Static offshore terrain/landmarks are merged into one mesh, use the existing white terrain material and do not cast shadows; the boat uses the same material and also skips the shadow pass. Two new draw batches, one new animated entity, no material/texture/particle asset added.

`cargo check -p tbg-client --target wasm32-unknown-unknown`: pass (0.89 s). Workspace Clippy passes with `-D warnings` (1.40 s) after correcting a stale helper call during layout separation. Island composition stays in `scenery.rs`; landmark models and the boat's visual motion stay in `maritime.rs`.

## D — Lighting

Retained the original warm upper-left light direction, StandardMaterial path, camera/tonemapping/fog and 1024 shadow map. Reduced cool ambient brightness from 1200 to 950 and raised the key from 12,000 to 13,000 lux to strengthen volume without another light or post-process. The single cascade now focuses its existing map on 42 units instead of 60, covering the original arena while spending fewer texels beyond the play area. Distant terrain and the boat remain excluded from shadow casting.

WASM target check: pass (0.46 s). Composition/readability and the final frame-time target remain browser gates; values may be refined in stage E.

## E — Composition and readability

Release browser inspection led to a brighter azure base, smaller shallow halos, irregular offshore cliff profiles and visible lighthouse bands. The cascade follows the protruding cliff ledge instead of disappearing behind it. The small sailboat now sits in open foreground water at the right edge, with a shared-batch contact wake; its first locations were occluded by the original island/forest, so placement was refined without changing the camera or arena. The boat remains bounded bob/roll only.

Final composition inspected at 1920x1080 and 1366x768, no error-level browser logs. [After 1920](../media/arena-seascape/sea-empty-after-1920.jpg) / [after 1366](../media/arena-seascape/sea-empty-after-1366.jpg). The original meadow, BattleLine, cliffs, props, Stronghold code, character code, UI and camera file remain untouched. Original components still dominate the center. New framing elements remain outside the play surface.

Checks after refinement: workspace Clippy with `-D warnings` passes; all 48 client tests pass; native/WASM checks pass; final release Trunk build passes (Rust release stage 59.19 s, packaging completed). An initial rustfmt check requested a line-wrap change after introducing the shared boat placement constant; rustfmt was applied and the check passes. No final timing claim until paired stage F measurements.

## F — Validation and measured optimization

The initial matched 1920x1080 comparison exceeded the owner's frame-p95 target: baseline median rolling p95 11.2 ms versus 12.9 ms with the first environment build (+15.2%). Both used a 1920x1080 backing canvas, 60 mixed-roster animated models and a visible page. A preceding baseline read without backing-canvas metadata was retained separately rather than treated as the final comparison. [Pre-optimization samples](../media/arena-seascape/pre-optimization/).

The environment was simplified before acceptance: 64x48 water grid instead of 88x64, omission of cells whose four corners are conservatively below the original island footprint, and NotShadowReceiver on the offshore scenery/boat to skip irrelevant shadow-map lookups. StandardMaterial directional/ambient lighting remains on these objects; the original main island and units retain casting/receiving shadows. No extra material or draw batch. Workspace Clippy (0.56 s) and WASM check (0.47 s) pass for this optimization.

### Owner-approved acceptance adjustment (2026-10-30)

The owner explicitly accepted **p95 up to 12.9 ms** and requested retaining the displayed design. For the matched 1920x1080 / 60-model reference scenario, this replaces the initial <10% increase target with an absolute maximum of 12.9 ms, equivalent to a rounded maximum increase of **15.2%** over the measured 11.2 ms baseline. [Versioned acceptance criteria](ARENA-SEASCAPE-PERFORMANCE.json). This is specific to this seascape follow-up; no global frame-rate, download, memory or gameplay budget is raised.

The measured 64x48 version retains the approved composition and PBR lighting. Its median rolling p95 is **12.9 ms**, 132.4 FPS and 423.6 draw calls; the matched baseline has 11.2 ms, 155.6 FPS and 422.5 draw calls. This passes the revised owner-approved limit exactly. The rounded p95 is reported without inventing extra precision. A later, unbuilt experiment to bake offshore illumination was withdrawn at the owner's request to keep the displayed design.

The remaining desktop measurements, release screenshots and final build-size delta are being finalized. No additional visual simplification is planned.
