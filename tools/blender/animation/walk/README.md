# Walk cycle (in-place)

**Level 2 biomechanical sampler** — Perry/Winter joint curves + Saunders pelvis determinants.  
See [BIOMECHANICAL-WALK-SPEC.md](BIOMECHANICAL-WALK-SPEC.md) for theory and validation checklist.

## Module map

| File | Layer | bpy |
|------|-------|-----|
| `constants.py` | Cadence → frame count | No |
| `curves.py` | Cyclic Catmull-Rom | No |
| `biomech_curves.py` | Hip/knee/ankle ° vs %cycle | No |
| `types.py` | `GaitPose`, `BoneEuler` | No |
| `style.py` | `WalkStyle` + presets | No |
| `sampler.py` | Pelvis + arms + leg sampling | No |
| `profiles.py` | Unit → profile registry | No |
| `api.py` | `sample_pose_at_frame` | No |
| `bake.py` | `bake_walk_action` | **Yes** |
| `export.py` | `export_walk_clip` | **Yes** |

## Add a unit profile

1. Add `WalkStyle(...)` in `style.py`
2. Register in `profiles.py` → `STYLE_PROFILES` + `UNIT_WALK_PROFILE`
3. `pytest tests/blender/walk/ -q`
4. `python3 tools/blender/export_game_unit.py <unit_id>`

## CLI

```bash
./tools/blender/blender_cmd.sh --background --python tools/blender/scripts/build_walk_clip.py -- HumanoidLight
python3 tools/blender/export_game_unit.py demo_scout_01
```

## Default timing

110 spm → 33 frames @ 30 fps (~1.1 s per cycle). Adjust via `WalkStyle.cadence_spm`.
