# Mobile game mesh budgets (MVP)

Single source of numeric limits for `validate_game_glb.py` and `optimize_glb.sh --mode game`.

Constants are mirrored in `tools/mesh_pipeline/budgets.py`.

## Hard limits (CI fail)

| Metric | Limit | Notes |
|--------|-------|-------|
| Triangles | ≤ 9,000 (hard) / 8,000 (target) | Skinned or static game mesh |
| Bones | ≤ 64 | Skinned meshes only |
| Materials | ≤ 2 | Per unit |
| Texture max edge | 1,024 px | Hero tier; minion tier may use 512 later |
| GLB file size | ≤ 5 MB | Hard fail |
| Animation clips | 6–12 | Skinned meshes only |

## Soft targets (warning only)

| Metric | Target | Notes |
|--------|--------|-------|
| GLB file size | ≤ 2 MB | Reported when exceeded; does not fail CI |

## Runtime VRAM (mobile target)

| Assumption | Value |
|------------|-------|
| Units visible on screen | 6 (same team + enemies in lane) |
| Target VRAM per unit (mesh + textures + clips) | ≤ 12 MB |
| Scene budget (6 units) | ≤ 72 MB character VRAM |

LOD1 static meshes live in `meshes/game/lod1/` at ~50% triangle budget and half texture resolution.

## Animation clip memory

- **Shared per body class:** `HumanoidLight.controller` / `HumanoidHeavy.controller` and clip library under `content/characters/animations/clips/{bodyClass}/`.
- **Per unit:** game GLB embeds retargeted clips for glTFast; `BodyClassClipRegistry` tracks one animation set id per body class.
- Future: external clip-only GLBs to deduplicate clip bytes across units.

## Texture strategy (game tier)

- Post-export: `pack_orm_textures()` merges metallic/roughness + occlusion into one ORM map (R=AO, G=roughness, B=metallic).
- KTX2 / BasisU transcoding: deferred until Unity glTFast path supports it.

## Static vs skinned

- **Static game mesh** (Step A vertical slice): no skins — validator checks triangles, materials, textures, and file size only.
- **Skinned game mesh** (Phase 03 pilot): requires armature, bone count, and embedded clip count within limits.

## CLI

```bash
python3 tools/mesh-pipeline/validate_game_glb.py content/characters/meshes/game/demo_sentinel_01.glb
./tools/mesh-pipeline/batch_seed_to_game.sh
```

## Optimize

```bash
./tools/mesh-pipeline/optimize_glb.sh --mode game --tri 8000 --tex 1024 \
  content/characters/meshes/showcase/demo_sentinel_01.glb \
  content/characters/meshes/game/demo_sentinel_01.glb
```

`--tri` and `--tex` default to `8000` and `1024`.

## Walk clip convention

**Default:** lane movement uses an **in-place** `Walk` clip (`TBG_Root.z = 0`). Units advance via gameplay position.

**Optional:** `WalkRoot` clip bakes forward root motion (+Z) — `stride_length_m × τ` per cycle (see `tools/blender/animation/root_motion/README.md`). Bake with:

```bash
./tools/blender/blender_cmd.sh --background --python tools/blender/scripts/build_walk_clip.py -- --root-motion HumanoidLight
```

Unity stub: `BehaviorProfile.UseRootMotionWalk` + `BehaviorPlayer.SyncWalkRootMotion` (Phase 04 lane sync).

## Related

- Blender export warnings: `tools/blender/tbg_export/validation_rules.py` (`MAX_TRIS_MVP`)
- Phase 03 plan: `.local/phases/03-game-animation-studio/README.md`
