# Mobile game mesh budgets (MVP)

Single source of numeric limits for `validate_game_glb.py` and `optimize_glb.sh --mode game`.

Constants are mirrored in `tools/mesh_pipeline/budgets.py`.

## Hard limits (CI fail)

| Metric | Limit | Notes |
|--------|-------|-------|
| Triangles | ≤ 8,000 | Skinned or static game mesh |
| Bones | ≤ 64 | Skinned meshes only |
| Materials | ≤ 2 | Per unit |
| Texture max edge | 1,024 px | Hero tier; minion tier may use 512 later |
| GLB file size | ≤ 5 MB | Hard fail |
| Animation clips | 6–12 | Skinned meshes only |

## Soft targets (warning only)

| Metric | Target | Notes |
|--------|--------|-------|
| GLB file size | ≤ 2 MB | Reported when exceeded; does not fail CI |

## Static vs skinned

- **Static game mesh** (Step A vertical slice): no skins — validator checks triangles, materials, textures, and file size only.
- **Skinned game mesh** (Phase 03 pilot): requires armature, bone count, and embedded clip count within limits.

## CLI

```bash
python3 tools/mesh-pipeline/validate_game_glb.py content/characters/meshes/game/demo_sentinel_01.glb
```

## Optimize

```bash
./tools/mesh-pipeline/optimize_glb.sh --mode game --tri 8000 --tex 1024 \
  content/characters/meshes/showcase/demo_sentinel_01.glb \
  content/characters/meshes/game/demo_sentinel_01.glb
```

`--tri` and `--tex` default to `8000` and `1024`.

## Related

- Blender export warnings: `tools/blender/tbg_export/validation_rules.py` (`MAX_TRIS_MVP`)
- Phase 03 plan: `.local/phases/03-game-animation-studio/README.md`
