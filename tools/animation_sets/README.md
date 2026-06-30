# Animation sets (Phase 03)

Manifests under `content/characters/animation_sets/` map gameplay actions to shared clip names per body class.

## Placeholder clip bootstrap

When Blender/Mixamo exports are not yet available, generate dev stub GLBs (named empty animations):

```bash
python3 tools/animation_sets/generate_placeholder_clips.py --set humanoid_heavy_combat_v1
```

Output: `content/characters/animations/clips/{bodyClass}/{ClipName}.glb`

Also export HumanoidLight walk clip (library + scout unit variant). Source: `tools/blender/animation/walk/` — see `.cursor/skills/tbg-walk-animation/SKILL.md`.

```bash
./tools/blender/blender_cmd.sh --background --python tools/blender/scripts/build_walk_clip.py -- HumanoidLight
./tools/blender/blender_cmd.sh --background --python tools/blender/scripts/build_walk_clip.py -- HumanoidLight demo_scout_01
```

Scout uses profile `scout_pilot_v1` (trident arm hold, wider stance). Re-export unit to embed:

```bash
python3 tools/blender/export_game_unit.py demo_scout_01
```

Walk is **in-place** (no root XZ) — lane movement is gameplay-driven (`BUDGETS.md`).

## Mixamo fallback (dev-only)

When TBG proximity/auto weights fail the gate, rig via [Mixamo](https://www.mixamo.com/) and import with `import_mixamo_fbx.py`. See `tools/blender/animation/retarget/BONE_MAP_MIXAMO_TO_TBG.json`.

**Ship policy:** Mixamo **mocap clips must not ship** in production game GLBs. Exported units use the **body-class animation library** (`content/characters/animation_sets/`) baked by `export_game_unit.py`. Mixamo is acceptable only as a temporary rig/weight bootstrap during development.
