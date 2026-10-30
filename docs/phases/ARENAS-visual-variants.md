# Arena visual variants — owner-directed art follow-up

**Base:** `main` at `077abe7`. **Branch:** `codex/themed-arenas`.
**Scope:** four client-only arena environments and their procedural assets.
This independent visual follow-up does not reserve a numbered gameplay phase.

## Read first

- `AGENTS.md`; owner desktop-only instruction dated 2026-10-29.
- `docs/ART-DIRECTION.md` §1–3, §9.
- `docs/ARCHITECTURE.md` §7; `docs/DECISIONS.md` D-22, D-26, D-27.
- `docs/phases/P16A-desktop-lobby-and-arena.md` steps 2–5 and its report.
- `crates/tbg-client/src/arena/`, `src/art/mesh.rs`, `src/camera.rs`.

## Contract

Keep Verdant Forest (arena 1) as the default and visual scale reference.
Preserve the fixed island outline (center x=8, approximate radii 12 and
5.7), BattleLine x=0..16 and width 3, Strongholds at x=-0.9 and 16.9,
and the existing desktop camera. Tall props stay behind the BattleLine;
front-edge props stay low. Team blue/red, rings and combat HUD remain readable.

Build Frozen Tundra, Volcanic Forge, Ancient Ruins and Storm Valley using
the same code-built low-poly pipeline as P16-A: `Shape`, `mesh::paint`,
vertex colors, baked contact shading, fixed-seed placement, merged static
meshes and shared animated meshes. No texture downloads or new dependencies.
Change the ground, lane finish, cliffs, surrounding sea/substitute,
shoreline, offshore props, foliage, camp fittings, Stronghold appearance,
light, fog and ambient accents for every theme. Recoloring the forest alone
does not satisfy the task.

Only the selected theme's meshes are built. Theme selection is a local
visual review option, never an authoritative match arena. No `tbg-core`,
server, protocol, matchmaking, combat, Energy or balance changes.
The attached gameplay modifiers remain a future, separately approved scope.

## Steps and acceptance

1. Document the art contract and add the small client-only theme palette.
   Check formatting before committing the specification and palette.
2. Integrate theme palettes into shared terrain, surroundings and lighting;
   create four distinct prop kits and Stronghold treatments. Preserve the
   original forest generation path. Expose `?arena=tundra|volcanic|ruins|storm`
   for inspection and `?theme=...` for the live lobby/practice scene.
   Check native compilation, formatting and WASM compilation.
3. Build with Trunk, observe all five scenes in a desktop browser, capture
   empty and 30-unit views at 1920×1080 and the four new arenas at 1280×800.
   Record layer triangles, WebGL draws, rendered FPS, browser errors and
   compressed WASM size. Inspect a live practice scene for integration.
4. Run the workspace checks in AGENTS.md, fill the report below, and
   commit the evidence. Stop for owner visual review; never push or merge.

## Phase report

Implemented on 2026-10-30 in `/tmp/tbg-themed-arenas`, based on local `main`
at `077abe7e4cdd0ac06b8bddc64d849bdb69d053c0`. No push or merge.
The original worktree's unrelated performance edits were preserved.

### Implementation

- Added Frozen Tundra, Volcanic Forge, Ancient Ruins and Storm Valley,
  each with its own generated prop kit, terrain palette, surrounding
  surface, shoreline, offshore assets, fittings, Strongholds and light.
- Kept the original island outline, desktop framing, BattleLine and
  Stronghold positions. The forest uses its original prop/tower builders,
  colors and light; its logged total remains exactly 37,212 triangles.
- Kept the existing `Shape`/`mesh::paint`/merged-mesh methodology. No new
  dependency, downloaded model, texture, shader or service was introduced.
- Local review uses `?arena=tundra|volcanic|ruins|storm`; `?theme=...`
  also dresses the live lobby and practice match. Unknown values fall back
  to the forest. There is no authoritative arena selection or modifier.
- The desktop-only owner instruction takes precedence over the historical
  phone acceptance lines in the main branch's P16-A/AGENTS documents.

The asset inventory and exact production procedure are in
[`docs/ARENAS.md`](../ARENAS.md). `tbg-core`, `tbg-server`, the protocol,
balance contracts and `GAME-DESIGN.md` were not changed.

### Browser observations and evidence

All five scenes were inspected at 1920×1080, empty and with the existing
mixed 30-unit stress scene. All four variants were also inspected at
1280×800. Geometry, team rings, health bars and names remain visible; no
new scenery crosses the BattleLine. Browser warning/error queries returned
empty arrays. No phone or touch validation was attempted.

A separate live Practice vs Bot match in Ancient Ruins recruited units,
rendered combat and the existing HUD, and reached a result: **Defeat,
1:33, 15 units deployed**. The own Stronghold's upper damage pieces were
absent in the result capture, demonstrating integration with the existing
HP-driven visibility system. This was local practice, not a multiplayer test.

Evidence is in [`docs/media/arenas/`](../media/arenas/):

- `overview.jpg`: four-panel contact sheet of the actual rendered scenes.
- `<theme>-empty-1920x1080.jpg` and `<theme>-busy-1920x1080.jpg`, including
  forest reference captures.
- `<theme>-empty-1280x800.jpg` for each of the four new arenas.
- `ruins-practice-1920x1080.jpg` and `ruins-practice-result-1920x1080.jpg`.
- `metrics.json`: layer triangles, diagnostic samples, image dimensions
  and byte sizes. Each JPEG is below 300 KB; the gallery totals about 2.8 MB
  and is documentation only, not part of the browser download.
- `workspace-tests.txt`: complete output of the final 168-test workspace run.

### Measurements

Logged environment totals, including both Strongholds and ambient meshes:

| Environment | Triangles | Animated entities |
|---|---:|---:|
| Verdant Forest (reference) | 37,212 | 27 |
| Frozen Tundra | 17,650 | 26 |
| Volcanic Forge | 18,096 | 27 |
| Ancient Ruins | 21,746 | 27 |
| Storm Valley | 18,250 | 27 |

The new empty scenes sampled **29–31 actual WebGL draws/frame**, including
the shadow pass. The mixed 30-unit scenes sampled **376.8–377.0 draws/frame**;
the forest reference sampled 376.9. Draw calls remain diagnostic under D-28.

Five-second busy-scene FPS windows were forest 40.5, tundra 127.8,
volcanic 43.7, ruins 127.4 and storm 98.0. These were interactive inspection
windows, with builds/another practice tab active during parts of the run;
they are **not a controlled comparative benchmark**. Some windows fall
below 60 FPS, including the unchanged forest. Sustained 60 FPS on the
owner's desktop and modest desktop hardware remains unestablished; the
triangle/draw measurements alone do not prove it. Browser/GPU memory and
cold time-to-menu were not measured.

Final WASM: **24,757,226 bytes raw; 5,600,489 bytes Brotli quality 9**,
below the 6 MB compressed ceiling. No external asset bytes were added to
the production bundle; all theme meshes are built locally, only for the
selected theme. No server-performance or balance claims are made.

### Verification

- `cargo fmt --all -- --check`: exit 0.
- `cargo clippy --workspace -- -D warnings`: exit 0, no warnings.
- `cargo test --workspace`: 168 passed, 0 failed (31 result summaries).
- `cargo test -p tbg-client` after the last visual refinement: 21 passed.
- `cargo check -p tbg-client -p tbg-core --target wasm32-unknown-unknown`:
  exit 0 after the final source changes.
- `trunk build --release`: the initial build passed. Final packaging
  passed with the installed tool directories on `PATH` and
  `--offline --skip-version-check`: `2026-10-30T14:46:40.121523Z INFO ✅ success`.
- `git diff --check`: exit 0.

Checks use the existing shared Cargo target directory to avoid duplicating
the Bevy toolchain artifacts on a nearly full disk. Build outputs are ignored.

### Resolved execution blockers

> resolved: the first sandboxed workspace run could not bind test sockets:
> `Os { code: 1, kind: PermissionDenied, message: "Operation not permitted" }`.
> Rerunning with localhost sockets enabled passed; no test was weakened.

> resolved: two final packaging attempts exited `143` (SIGTERM) after
> Cargo, the logged attempt ending at `Finished release profile [optimized]
> target(s) in 1m 19s`. The step was paused and recorded. Offline diagnostics
> then reported `couldn't find application wasm-bindgen (version: 0.2.129),
> unable to download in offline mode`. Adding the already installed
> `~/.cache/trunk/wasm-bindgen-0.2.129` and
> `~/.cache/trunk/wasm-opt-version_131/bin` directories to this build's
> `PATH` let offline packaging finish. The original SIGTERM source was not
> established; no code workaround or dependency change was needed.

### Deviations and remaining review

- The attachment was a broader planning brief with gameplay modifiers;
  the owner's direct request narrowed this delivery to implemented visuals
  and documentation. The owner explicitly chose the attachment's four
  themes (snow, lava, ruins, storm), excluding desert.
- This is an independently named visual follow-up instead of reserving a
  numbered gameplay phase; later phase numbers already exist on other
  working branches.
- Frost lanterns, jade ritual bowls and electric lanterns reuse the existing
  shared animation topology. Storm atmosphere uses static cloud banks and
  lightning inlays, without particles or flashing lights.
- Owner visual approval is pending. Controlled desktop FPS/memory
  validation remains pending as described above. No online arena system,
  gameplay modifier, ranked rule or balance validation is included.
