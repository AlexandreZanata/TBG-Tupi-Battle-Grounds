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

## Remaining stages

B — Ocean; C — background; D — lighting; E — composition; F — final validation and paired measurements. In progress; no final performance claim yet.
