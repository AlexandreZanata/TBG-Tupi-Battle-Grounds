# Arena reference refinement

## Scope and base

Owner request dated 2026-10-31: refine the current main arenas toward four supplied low-poly references, provide empty before/after screenshots, and open a GitHub PR. Base: `9008f117c4289122ddff978c5d25720d2bd79687` (`origin/main` fetched at task start). Branch: `codex/arena-reference-refinement`. This starts from main itself, not the separate arena release or roster expansion branch.

The references guide 3D composition, materials, color and scenery; they are not background images or runtime assets. Frozen Tundra, Volcanic Forge, Ancient Ruins and Storm Valley are refined. Verdant Forest is retained as the approved scale/style reference and is included in the comparison. The fixed camera, island outline, BattleLine footprint, Stronghold positions/kinds, unit art, UI, gameplay, protocol and authority are preserved.

## Implementation

- The four themed surrounding surfaces now use the base arena's approach: a static irregular faceted grid, shallow-water variation, short crests and interrupted coastal foam. Offshore silhouettes are distinct per theme. All water stays in the existing sea/foam batches. The Forest's original procedural functions are preserved.
- Ancient Ruins uses grouped ivory colonnades with capitals, fluting, a turquoise keystone, climbing moss, conifers, flowers, fallen masonry, front stairs and lily pads. Large forms have priority over evenly scattered tiny props.
- Frozen Tundra uses pale snow caps, darker fir skirts, angular snowy ridges, cyan crystal clusters, substantial floating ice and layered blue cliff colors. Foreground props remain low.
- Volcanic Forge uses dark basalt clusters, a jagged crater rim, orange crystals, shore fissures, chained outcrops and plank bridges. Broken gold/orange molten channels replace the concentric lava rings. The BattleLine receives sparse stone cracks without changing its dimensions or height.
- Storm Valley uses four tall conductor spires, blue banners, pale wind-bent conifers, bare branching silhouettes, broken docks, offshore rocks/floes and distant low cloud banks. No strobe, particles or extra lights.
- One existing directional light and ambient fill are tuned per themed kit for better relief. Shadow-map resolution/cascade count, the shared lit/unlit material pipeline and animation topology are unchanged. Ruins activates the existing optional unlit kit batch for its keystone. Static parts merge with the existing terrain/glow meshes; no added per-prop entity, texture, shader, dependency, postprocess or per-frame geometry rebuild.

## Validation policy and execution

Level: focused visual work front with the specific before/after desktop gallery requested by the owner. Follow the owner-provided 2026-10-31 AGENTS.md / VALIDATION.md policy, which supersedes the older broad checklists in this main revision. No native or workspace test suite, workspace Clippy, release or installer-packaging build, benchmark or multiplayer matrix is requested or run.

The first development WASM build was **deferred: shared validation slot busy**, exit 75. The owner explicitly granted an exception and authorized continuation. The before/after development build batch used the same nonblocking `/tmp/tbg-validation-1000.lock`, `CARGO_BUILD_JOBS=2` and the existing shared target cache. The baseline source is an isolated detached worktree at the exact main SHA. The `--dist` folders are build outputs, not fresh Cargo target caches. On another busy response during cache correction, the necessary client rebuilds were queued behind that same host lock under the owner's continuation exception. The lock was never bypassed; two Cargo jobs remained the limit. No other build/session/cache was stopped or removed.

Deferred consolidation command: `cargo test -p tbg-client arena::scene::tests::repeated_switches_keep_exactly_one_scene_and_one_sun -- --test-threads=1`, under the shared lock with two Cargo jobs. It is an existing scene lifecycle check, not a claimed local pass. Release performance, compressed release size, full suite and hardware-wide FPS acceptance remain deferred; development screenshots establish appearance, not release readiness.

## Completed focused checks

Source commit: `687049d`.

- `cargo fmt -p tbg-client -- --check`: exit 0.
- `git diff --check`: exit 0.
- Baseline: `trunk build --offline --skip-version-check --dist /tmp/tbg-arena-reference-before`, development profile, exact main source, exit 0; 123.9 seconds including bundling.
- Final candidate: `trunk build --offline --skip-version-check --dist /tmp/tbg-arena-reference-after`, development profile, exit 0; 29.5 seconds including bundling. Necessary rebuilds followed visual changes, under the same lock and two-job limit. No additional native/WASM check duplicated this compilation.
- Both commands used `CARGO_TARGET_DIR=/home/iiii/PESSOAL-PROJETOS-ALEXANDRE/TBG-deploy-art-wt/target`. The installed wasm-bindgen/wasm-opt directories were added to these commands' PATH; no tool, dependency or package was installed.
- Static comparison verifies core/server, UI/assets, camera, island outline, Strongholds and Cargo manifests/lockfile are unchanged. The Forest water functions are byte-for-byte identical to main; its lighting values, palette and original generation remain unchanged.

Both builds emit the existing main warning `warning: unused variable: lang` at `crates/tbg-client/src/ui/battle_hud.rs:126:52`. This is outside the visual diff; it is not a new warning or an error. No test suite or workspace Clippy pass is claimed.

### Resolved artifact and capture issues

The first candidate bundling reused the baseline WASM from the shared cache: its hash was identical and the new module was absent from the depfile/binary. That artifact was rejected as candidate evidence. Updating the modified sources' mtimes forced the client compilation in the existing cache. The final candidate has a different hash and includes `water/ice.rs`. No `cargo clean`, new target cache or build lock bypass was used. Intermediate visual checks led to the final turquoise Ruins water, supported stairs/chains, angular snowy peaks, irregular ice slabs and darker storm cloud banks. Only the final candidate is used in the committed after gallery.

An initial browser locator wait expired once during a new page load. The page was inspected through its current DOM/screenshot before proceeding. All ten final captures have an actual 1920x1080 canvas, completed scene render, no troops and zero error-level browser logs. The ordinary diagnostic overlays are retained in the screenshots; no arena content was painted or composited into them.

## Desktop gallery

All screenshots are actual local Chromium/WebGL renders at **1920x1080**, using the same fixed arena camera, `?arena=<slug>&measure`, and a development WASM build. Before is main `9008f11`; after is source `687049d`. The five pairs include the preserved Forest reference. No phone layout, busy-match matrix or multiplayer scenario was run. Comparison images only resize/place the two original screenshots side by side and add labels; individual full-size captures are retained.

- [Ancient Ruins comparison](../media/arena-reference-refinement/ruins-comparison.jpg), [before](../media/arena-reference-refinement/ruins-before-1920.jpg), [after](../media/arena-reference-refinement/ruins-after-1920.jpg).
- [Frozen Tundra comparison](../media/arena-reference-refinement/tundra-comparison.jpg), [before](../media/arena-reference-refinement/tundra-before-1920.jpg), [after](../media/arena-reference-refinement/tundra-after-1920.jpg).
- [Volcanic Forge comparison](../media/arena-reference-refinement/volcanic-comparison.jpg), [before](../media/arena-reference-refinement/volcanic-before-1920.jpg), [after](../media/arena-reference-refinement/volcanic-after-1920.jpg).
- [Storm Valley comparison](../media/arena-reference-refinement/storm-comparison.jpg), [before](../media/arena-reference-refinement/storm-before-1920.jpg), [after](../media/arena-reference-refinement/storm-after-1920.jpg).
- [Verdant Forest reference comparison](../media/arena-reference-refinement/forest-comparison.jpg), [before](../media/arena-reference-refinement/forest-before-1920.jpg), [after](../media/arena-reference-refinement/forest-after-1920.jpg).

![Four refined environments](../media/arena-reference-refinement/overview-after.jpg)

## Observed geometry and limits

Logged environment triangles: Ruins **21,194 -> 34,152**; Tundra **17,226 -> 22,062**; Volcanic **17,616 -> 32,284**; Storm **17,158 -> 26,960**; Forest **44,430 -> 44,430**. These are the existing environment logger's counts, not total match geometry. Newly created terrain/props remain merged in the existing batches. The optional Ruins glow layer accounts for its additional batch.

The observed empty-page WebGL draw counts were Ruins **37 -> 39**, Tundra **37 -> 37**, Volcanic **39 -> 39**, Storm **39 -> 39**, and Forest **40 -> 40**. These opportunistic observations include diagnostic text and are not a controlled performance benchmark. Development frame rates varied with startup, browser cadence and concurrent host work; no sustained 60-FPS, release p95, memory or load-time acceptance is claimed. No performance budget is raised.

Development WASM: before **129,380,992 bytes**, after **129,455,656 bytes** (+74,664). These debug artifacts are not release download-size measurements and are not committed. No runtime image asset or dependency is added. The documentation JPEG gallery totals **3,865,706 bytes** and is not copied to the game's runtime assets.

- Before SHA-256: `7c9013f4a8a81158e8d86e27e75b88f53219847add7fcca59c0552cb56a36a3b`.
- Final after SHA-256: `30f1e3f40584485268c1da2462bad6bad32cb4cc850eab9c91f67a7f9ce0b95b`.
- [Structured verification and per-layer counts](../media/arena-reference-refinement/verification.json); [raw browser observations](../media/arena-reference-refinement/observations.json).

Owner visual approval and deferred consolidation/release checks remain open. The requested delivery is a visual review PR, not a main merge or VPS deployment.
