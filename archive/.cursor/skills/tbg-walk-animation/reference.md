# Walk animation reference

## Size limits (harness)

- ≤80 lines per function
- ≤200 lines per file
- Cyclomatic complexity ≤10

Split new logic across `sampler.py`, `style.py`, `profiles.py` before adding files to `scripts/`.

## Scaling to new clip types

Copy the `walk/` layout:

```
animation/<clip>/
  constants.py   # duration, fps
  sampler.py     # pure pose math
  profiles.py    # unit overrides
  api.py         # public sample API
  bake.py        # bpy action bake
  export.py      # optional standalone GLB
```

Register the clip in `retarget_clip_library.py` `_embed_clip_actions` when embedding in game units.

## Import paths

| Context | Import |
|---------|--------|
| pytest | `from tools.blender.animation.walk import ...` |
| bpy script | `ensure_repo_paths()` then `from tools.blender.animation.walk...` |
| legacy | `tools.blender.scripts.walk_gait` shim still works |

## Content paths

| Asset | Path |
|-------|------|
| Library clip | `content/characters/animations/clips/{bodyClass}/Walk.glb` |
| Unit override | `content/characters/animations/clips/units/{unitId}/Walk.glb` |
| Embedded in game mesh | `content/characters/meshes/game/{unitId}.glb` |

## Convention

Walk is **in-place** — documented in `tools/mesh-pipeline/BUDGETS.md`. Lane movement is gameplay-driven (`BehaviorPlayer.EnterWalk`).
