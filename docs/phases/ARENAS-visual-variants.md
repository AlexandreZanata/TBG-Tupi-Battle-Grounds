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

Implementation and measurements pending.
