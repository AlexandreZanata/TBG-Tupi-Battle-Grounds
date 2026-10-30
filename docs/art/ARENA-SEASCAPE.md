# Arena seascape visual follow-up

## Scope and base

Owner-requested procedural 3D environment improvement, based on `origin/main` at `6ef155bc9540527c99086aee0d9726908ca6a041`. Branch: `codex/arena-seascape`; isolated worktree `TBG-arena-wt`. The initial handoff was local only. The owner subsequently requested publishing all changes from this chat to the existing art PR #14; the seascape history is now integrated into `codex/ranking-ui`. No main merge or VPS deployment is part of this publication. The supplied image is a visual reference, never a background asset.

The isolated measurement base does not include the separate ranking/lobby/Faction PR #14. The seascape itself preserves that main baseline UI and cards and changes no UI asset or gameplay rule. Publication combines the two histories in PR #14; the measurements below remain those of the recorded isolated seascape build, not a new benchmark of the combined presentation.

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

Initially implemented a seeded 88x64 irregular water grid (final resolution 64x48, with hidden cells omitted; see stage F) with subtle static relief, blue depth/turquoise near-shore color variation and a coarse exterior fill, all merged into the existing sea draw. Broken white coastal arcs, shallow transition bands, foam around the ten original offshore rocks and sparse glint ribbons share the existing cosmetic Foam batch and transform drift. Removed the old torus rings from the lit terrain batch; offshore rock positions, scales and rotations remain unchanged. No texture, shader, reflection pass, new material or per-frame vertex upload. The original two sea/foam draw batches and animation entity count remain unchanged.

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

### Final measurements and build

Final 1366x768 comparison: median rolling p95 **17.0 -> 17.1 ms** (+0.59%), **60 -> 60 FPS**, and **422.5 -> 423.5** WebGL draws/frame. Both used a 1366x768 backing canvas. This pair ran at a 60-FPS cadence, whereas the 1920 reference pair was uncapped; compare each resolution against its own baseline, not across those runs. [Raw samples and acceptance summary](../media/arena-seascape/acceptance/summary.json). Every retained sample had the correct viewport, backing canvas, 60 models and visible-page state. Warm-up >=15 seconds, then 12 one-second reads of the existing rolling percentiles. No compilation ran during these final captures.

The 1920 reference pair's JS heap was 10,598,907 -> 10,655,697 bytes; first drawn frame 720.5 -> 653.6 ms on these warm local navigations. At 1366, JS heap was 10,689,206 -> 10,761,338 bytes; first drawn frame 684.0 -> 716.2 ms. These are individual local observations, not a claim of faster loading. The Chromium heap does not include total WASM, GPU or browser memory; GPU timing/RSS were not measured. Reported renderer: WebKit WebGL. Browser cadence and hardware affect results; no hardware-wide guarantee is claimed.

Final environment: **44,430 triangles** versus 35,868. Sea 5,516; foam 2,090; original offshore rocks 800; offshore scenery 2,466; boat 86. Original meadow 192, BattleLine 444, cliffs 384, props 31,828 and life 624 are unchanged. Two added render entities share existing materials; only the boat adds a new animated transform. The measured net WebGL draw increase in the stress pair is about one draw/frame. The added non-indexed POSITION/NORMAL/COLOR payload is 1,027,440 bytes per copy, derived from the observed triangle delta (8,562 x 3 vertices x 40 bytes); this is not a total process/GPU memory measurement. No per-frame mesh regeneration or vertex upload. [Actual release geometry logs](../media/arena-seascape/sea-arena-log-after.json).

Final WASM: **24,949,503 raw bytes** (+25,668), **7,779,439 gzip-9** (+10,672), **4,862,886 Brotli-11** (+7,145, **+0.147%**). Game asset folder remains **14,613,050 bytes** (+0); no generated image, texture, audio or external asset was added. Opt-in diagnostic instrumentation adds 480 raw JS bytes / 162 Brotli bytes over main. The full built tree, using identical measurement instrumentation in both versions, grew by 25,666 raw bytes. [Full size/hash evidence](../media/arena-seascape/build-after.json).

The final release WASM SHA-256 is `50426c57cb645409d6284ae6eb86472d96192915d07469eaeccd0de8d8157748`, exactly matching the binary used for the approved measurements. The withdrawn baked-light experiment was never included in that binary. All final screenshots use the approved PBR design.

### Functional verification and limits

- Format, workspace Clippy, native/WASM compilation and release Trunk build pass.
- Native workspace run: **266 passed, 0 failed**, one legacy ignored test and one explicitly filtered persistence case. All **48 client tests** passed again after the final rendering changes. **16 JavaScript tests** passed; diagnostic JS syntax and `git diff --check` pass. [Commands/results](../media/arena-seascape/verification.json), [native output](../media/arena-seascape/workspace-tests.txt).
- Final real local Practice showed live combat, automatic purchases and the existing HUD in the 1:00 battle state: Army 6, six visible units, 60 FPS, rolling p95 17.0 ms, zero error-level browser logs. This is functional evidence with a different clock/army than the baseline 0:40 capture, not another paired performance comparison. [Practice](../media/arena-seascape/sea-practice-approved-1920.jpg), [observations](../media/arena-seascape/sea-practice-approved.json).
- Clean before/after screenshots at both desktop sizes retain the original camera. The render stress path is 60 walking/attacking models, not a claim of 60 competitive simulated units. Main geometry, camera, units, Stronghold logic, UI/assets, core/server, dependencies and protocol are unchanged against the main base; the protected paths are recorded in verification.json.

> blocked (pre-existing online test, outside this visual change): earlier workspace and serial-online attempts repeated `assertion failed: a.completed && b.completed` in `records_insert_and_count`. Suspected cause: debug real-time match completion/deadline under host load. AGENTS.md stopped that case; this follow-up skips it and runs all independent cases. No third attempt, deletion or alteration of the test is claimed. The owner's p95 approval does not change this separate test limitation.

### Reused systems and modified files

Reused Surface, painted/merge, Shape primitives, seeded Pcg32, the existing pine builder and palette, existing white lit/unlit materials, one directional light/cascade and the existing Foam drift. No dependency, custom water shader, reflection pass, physical water, extra light or particle system.

Production changes are limited to `crates/tbg-client/src/arena/water.rs`, `scenery.rs`, `maritime.rs`, `mod.rs`, `lighting.rs` and opt-in `crates/tbg-client/diagnostics/measure.js`. The versioned owner-approved threshold is `docs/art/ARENA-SEASCAPE-PERFORMANCE.json`; reports/evidence are under `docs/art/` and `docs/media/arena-seascape/`.

Local commits: `14a1795` diagnosis/baseline; `b593890` ocean; `a07fe38` background; `627f5dc` lighting; `4a51de7` composition; `9b55932` approved p95 limit and retained design. Final evidence is committed separately at the branch tip. At the initial local handoff, no push, PR, main merge or VPS change had been performed. Owner approved the displayed design and revised reference-case p95 limit on 2026-10-30.

The work follows the owner's attached executable brief rather than a numbered roadmap phase. The dedicated `codex/arena-seascape` branch follows the host's branch convention and starts from the refreshed main base. This follow-up does not mark the broader P30 device/performance program complete.

## Publication integration (2026-10-30)

At the owner's request to publish all work from this chat, merge commit `39c5a47` combines all seven seascape commits through `5e8a7ba` with the existing art PR #14 (`codex/ranking-ui`, previously `120fa72`). The sole merge conflict was in ROADMAP.md; both follow-up records are retained. Rendering/diagnostic source matches `5e8a7ba` exactly, and presentation/assets/tests match `120fa72` exactly. Core/server/dependencies remain unchanged against the fetched main base `6ef155b`.

Integrated validation: `cargo fmt --all -- --check`, `cargo clippy --workspace -- -D warnings`, `cargo check -p tbg-client -p tbg-core --target wasm32-unknown-unknown`, `node --check crates/tbg-client/diagnostics/measure.js`, and `trunk build --release` all pass. `cargo test --workspace -- --skip records_insert_and_count --test-threads=1` passes **266 tests**, with one existing ignored case and the documented excluded persistence case; the excluded case was not retried. `node --test --test-isolation=none --test-reporter=tap crates/tbg-client/tests/*.test.js` passes **27 tests**. Native suite: 221.5 s; release packaging: 79.9 s.

The combined release WASM SHA-256 is `50426c57cb645409d6284ae6eb86472d96192915d07469eaeccd0de8d8157748` (24949503 bytes). It matches the approved measured WASM. On the integrated release, a local 1280x720 desktop browser check showed the illustrated lobby and the new 3D environment together, then real Practice with a successful Archer purchase, Army changing from 0 to 1, offer refresh, live combat and all four colored Faction frames. No error-level browser logs were observed. [Combined lobby](../media/art-publication/lobby-integrated.jpg), [combined Practice at 0:20](../media/art-publication/practice-integrated.jpg). This is an integration check, not a new paired performance measurement.

Publication targets the existing PR #14; no main merge or VPS deployment is included.

> blocked (preview tab retention tooling only): two attempts to mark the pre-existing arena preview as a retained browser deliverable returned `browser.user is unavailable`. Suspected cause: the selected IAB adapter does not expose that retention API. No further retention attempt was made. The documented tab API completed the integrated browser check and closed its temporary test tabs; the error is separate from the game, its error-free browser logs, and GitHub publication.
