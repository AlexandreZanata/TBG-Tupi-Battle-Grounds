# Arena background refinement V2

## Request and private references

Owner request, 2026-10-31: keep the five supplied arena reference PNGs locally outside Git, refine all five backgrounds toward them, and compare against the previous update's final screenshots. This round was developed on `codex/arena-background-refinement-v2`, based on `2e4cbb981a0b277db6cc6dc87f0783621bb3b4c7`. Rendering source: `8bfe677`.

The permanent private folder is `.local/art-references/arena-backgrounds-v2-20261031/` in the owner's primary `TBG-Tupi-Battle-Grounds` checkout. A working copy also exists in `TBG-ranking-wt`. Each contains five unchanged reference PNGs, five immutable baseline JPEGs, `manifest.json` with SHA-256 hashes, and a README. All twelve files in each location are ignored by the existing `.local/` rule; none is tracked. The source PNGs are not embedded in runtime assets or the comparison gallery.

**Before means the previous final after**, not a fresh main screenshot: the five `docs/media/arena-reference-refinement/*-after-1920.jpg` files, rendering revision `687049d`. Their bytes and hashes match the private baseline copies. The previous gallery remains untouched.

## Implemented appearance

- Forest: taller offshore lighthouse island, larger waterfall cliff and pool, three additional islets, more trees and foreground plant clusters, visible sailboat beside the pier, and irregular broken foam. The warm lighting and BattleLine framing remain intact.
- Ruins: beveled masonry and temple edges, irregular stone silhouettes, more greenery/flowers around architecture and fallen columns, and turquoise water detail.
- Tundra: seeded conifer crowns, angular mineral ridges and fitted snow shells sharing the underlying rock silhouette, plus small snow-covered coastal groups.
- Volcanic: a generated turbulent molten surface with broad yellow heat channels and red cooling patches replaces the previous straight ribbon accents. The existing basalt, chains, crater and lane cracks remain.
- Storm: irregular wind-bent crowns and coastal rocks, stronger distant haze, and soft vertex-alpha squalls replace the hard cloud solids. The cloud layer is confined to the offshore background.
- Shared terrain: four uneven cliff strata and a finer meadow color breakup; the island's top outline, lane mesh, Stronghold positions and camera framing are unchanged.

New scenery shapes are isolated in `art/mesh/scenery.rs`; existing troop shapes are unchanged. A single 256x256 RGBA8 tile is generated in memory for the selected water/lava material: 262,144 bytes before mip/storage overhead, no asset download. It uses the existing stock unlit `StandardMaterial`, repeat sampling and planar UVs. Storm adds one stock alpha-blended material/draw layer for the distant squalls. No dependency, custom shader, post-processing pass, extra light or shadow map was added. Geometry remains merged into the existing environment batches except that distant cloud layer. These choices intentionally replace the older texture-free sea implementation for this refinement.

## Validation actually completed

Level: **work front, with the owner-requested empty desktop comparisons**. The revised owner AGENTS/VALIDATION policy dated 2026-10-31 governs this front; broader historical phase checklists were not repeated.

- `cargo fmt -p tbg-client -- --check`: exit 0.
- `git diff --check`: exit 0.
- Development browser build, exit 0:

```bash
flock -n -E 75 /tmp/tbg-validation-1000.lock \
  env CARGO_BUILD_JOBS=2 \
  CARGO_TARGET_DIR=/home/iiii/PESSOAL-PROJETOS-ALEXANDRE/TBG-deploy-art-wt/target \
  python3 .local/arena-background-v2/build.py
```

The local wrapper runs `trunk build --offline --skip-version-check --dist /tmp/tbg-arena-background-v2` from `crates/tbg-client`, adding the already installed wasm-bindgen/wasm-opt paths. The final build took **37.4 seconds**, with Rust compilation **5.39 seconds**. Relevant source edits justified intermediate development rebuilds: the first lava pattern was visually too regular, then the final Ruins vegetation and Storm clouds changed. All used the same host lock, existing target cache, and at most two Cargo jobs. The final round acquired the nonblocking lock successfully. No new cache, `cargo clean`, release build or duplicate native check was used.

The only compiler warning is the existing `unused variable: lang` at `ui/battle_hud.rs:126:52`, also present in the baseline.

Captured development WASM: **129,558,258 bytes**, SHA-256 `aeb69ef8fb5c11291528ca9ee5de3abb3fd06aa9746ab59d2782e44678402afd`. This is a debug artifact, not a release download-size measurement, and is not committed.

All five captures are actual local desktop Chromium/WebGL renders, with a **1920x1080 canvas**, an empty arena and **zero error-level console logs**. Forest, Tundra and Volcanic captures reuse the preceding successful build (SHA-256 `d44aac988ff2d74a2190bcd4d655f7a0b21cf12f1810748ae6eebe96630e45fc`); the final source changes affect only Ruins vegetation and the Storm-only squall branch, so their relevant visual evidence remains valid. Ruins and Storm were recaptured on the final build. The per-capture artifact hash and raw observations are recorded below. Diagnostic overlays remain visible; arena content was not painted or composited into screenshots.

Static comparison confirms no changes to core/server logic, UI, runtime asset files, Cargo manifests/lockfile, Stronghold/tower implementation or island outline. The complete BattleLine generation function is byte-for-byte identical to the previous round. Camera changes only select the theme's fog falloff, without changing FOV, target, projection or framing logic. Both private reference copies passed hash/ignore checks.

### Publication CI follow-up

The first publication's [remote CI run](https://github.com/AlexandreZanata/TBG-Tupi-Battle-Grounds/actions/runs/41207367636) passed formatting but failed Clippy on `arena/scene.rs`: `this function has too many arguments (8/7)`. Grouping the mesh/material pair into a Bevy system-parameter tuple, while keeping the image resource separate, brings the arena-switch signature down to seven parameters. An intermediate three-resource tuple removed the argument warning but hit `type_complexity` in [the next CI run](https://github.com/AlexandreZanata/TBG-Tupi-Battle-Grounds/actions/runs/41207567313); the final pair avoids that complexity warning. It preserves the resources and build calls; no geometry, palette or material generation changed, so the existing visual evidence is reused.

Focused validation of that correction: `cargo fmt -p tbg-client -- --check` and `git diff --check` exit 0. The following command also exited 0 in **4.63 seconds**, with only the existing `unused lang` warning:

```bash
flock -n -E 75 /tmp/tbg-validation-1000.lock \
  env CARGO_BUILD_JOBS=2 \
  CARGO_TARGET_DIR=/home/iiii/PESSOAL-PROJETOS-ALEXANDRE/TBG-deploy-art-wt/target \
  cargo check -p tbg-client --bin tbg-client --target wasm32-unknown-unknown --offline
```

A final focused lint command checked the two reported lints on the browser target:

```bash
flock -n -E 75 /tmp/tbg-validation-1000.lock \
  env CARGO_BUILD_JOBS=2 \
  CARGO_TARGET_DIR=/home/iiii/PESSOAL-PROJETOS-ALEXANDRE/TBG-deploy-art-wt/target \
  cargo clippy -p tbg-client --bin tbg-client --target wasm32-unknown-unknown --offline \
  -- -D clippy::too_many_arguments -D clippy::type_complexity
```

**Exit 101**, not a pass: no arena diagnostic remained, but Clippy reported the existing five-resource HUD tuple at `ui/battle_hud.rs:88` as `type_complexity` on WASM. That UI file is byte-for-byte unchanged from main; it was not changed to broaden this art publication. The existing `unused lang` warning also remained. This local lint step stopped at the unrelated error. The configured remote CI uses its native Clippy target and reruns after the corrective push; its outcome is separate from this failed local WASM lint check.

The corrective push schedules remote CI again. Its result is reported in PR #23; this local check is not a workspace Clippy/test pass. The focused lifecycle scenario and performance acceptance remain deferred.

## Comparison gallery

Left: previous update. Right: V2. Comparisons only resize/place the two genuine game screenshots and add labels. The ten new JPEG files total **2,323,246 bytes**; the largest is **268,452 bytes**. Full-size after captures and structured evidence are retained alongside them.

- [Forest comparison](../media/arena-background-refinement-v2/forest-comparison.jpg), [full after](../media/arena-background-refinement-v2/forest-after-1920.jpg).
- [Ruins comparison](../media/arena-background-refinement-v2/ruins-comparison.jpg), [full after](../media/arena-background-refinement-v2/ruins-after-1920.jpg).
- [Tundra comparison](../media/arena-background-refinement-v2/tundra-comparison.jpg), [full after](../media/arena-background-refinement-v2/tundra-after-1920.jpg).
- [Volcanic comparison](../media/arena-background-refinement-v2/volcanic-comparison.jpg), [full after](../media/arena-background-refinement-v2/volcanic-after-1920.jpg).
- [Storm comparison](../media/arena-background-refinement-v2/storm-comparison.jpg), [full after](../media/arena-background-refinement-v2/storm-after-1920.jpg).
- [Structured verification](../media/arena-background-refinement-v2/verification.json), [raw browser observations](../media/arena-background-refinement-v2/observations.json).

Logged environment triangles, previous -> V2: Forest **44,430 -> 44,060**; Ruins **34,152 -> 41,600**; Tundra **22,062 -> 28,357**; Volcanic **32,284 -> 32,716**; Storm **26,960 -> 33,228**. These are the environment logger's totals, not a complete match geometry count.

Observed empty-page draw calls, previous -> V2: Forest **40 -> 40**, Ruins **39 -> 39**, Tundra **37 -> 37**, Volcanic **39 -> 39**, Storm **39 -> 40**. These observations are diagnostic, not a controlled FPS benchmark. The old owner-approved 12.9 ms bound applies to its historical Forest reference case; V2 does not claim that acceptance or raise any budget.

## Deferred checks and review limits

The existing scene lifecycle test setup now registers `Image` because scene construction creates a generated image. Its exact focused consolidation command is deferred, not passed:

```bash
flock -n -E 75 /tmp/tbg-validation-1000.lock \
  env CARGO_BUILD_JOBS=2 \
  CARGO_TARGET_DIR=/home/iiii/PESSOAL-PROJETOS-ALEXANDRE/TBG-deploy-art-wt/target \
  cargo test -p tbg-client arena::scene::tests::repeated_switches_keep_exactly_one_scene_and_one_sun \
  -- --test-threads=1
```

Consolidation should also observe Practice -> alternate arena -> Forest in one browser session to check image/material teardown and fog restoration. The isolated empty previews verify rendering, not that full runtime transition scenario. No additional competitive regression test is introduced because simulation rules are untouched.

Release size/performance, controlled reference-case p95, loaded-match/multiplayer checks and hardware-wide acceptance are deferred; request the relevant heavy scope explicitly before running it. No workspace suite, Clippy campaign or full suite ran. The images are an incremental procedural 3D interpretation: exact parity with the references' dense scenery, rich lighting and surface detail is not claimed. After the comparison delivery, the owner approved all five visual references and authorized GitHub publication on 2026-10-31; the deferred technical checks remain pending.

The owner-authorized publication includes the V2 source, reports and game screenshot comparisons in existing PR #23, with `codex/arena-background-refinement-v2` retained as the round branch. Original reference PNGs remain exclusively in the ignored local folders. Publication does not merge main or deploy the VPS.
