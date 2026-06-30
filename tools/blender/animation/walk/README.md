# Walk cycle (in-place)

## Sampler roles (audit 2026-06-30)

| Module | Status | Used by |
|--------|--------|---------|
| `held_arm_gait.py` | **Primary** (`sampler.py`) | All profiles today; `scout_pilot_v1` locks trident arm |
| `segment_gait.py` | Ready — Perry trunk + arms | Promote in §2 after scout held-arm validation |
| `symmetric_gait.py` | Legacy reference only | Tests |
| `biomech_curves.py` | **Calibrated** | CMU mocap via `retarget/` (subjects 35, 39) |

Mocap calibration: `python3 tools/blender/scripts/retarget_mocap_walk.py --write-curves <bvh...>`

> Legacy note: README previously listed `held_arm_gait` as primary for weapon units only — `sampler.py` still delegates entirely to `held_arm_gait` until §2.

## Specs

| Document | Purpose |
|----------|---------|
| [BIOMECHANICAL-WALK-SPEC.md](BIOMECHANICAL-WALK-SPEC.md) | TBG implementation reference |
| [AI-WALK-AUTHORING-SPEC.md](AI-WALK-AUTHORING-SPEC.md) | **Agent-facing** biomechanical control-point format (Perry phases, keyframe tables, physics rules) |
| [../retarget/README.md](../retarget/README.md) | CMU BVH → TBG curve calibration |

## Module map

| File | Layer | bpy |
|------|-------|-----|
| `constants.py` | Cadence → frame count | No |
| `curves.py` | Cyclic Catmull-Rom | No |
| `biomech_curves.py` | Hip/knee/ankle ° vs %cycle | No |
| `types.py` | `GaitPose`, `BoneEuler` | No |
| `style.py` | `WalkStyle` + presets | No |
| `sampler.py` | **held_arm_gait** delegate | No |
| `held_arm_gait.py` | Legs-only walk, weapon arm locked | No |
| `segment_gait.py` | Perry segment gait (next default) | No |
| `symmetric_gait.py` | Legacy cosine gait | No |
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
python3 tools/blender/scripts/retarget_mocap_walk.py --calibrate tests/blender/fixtures/mocap/35_01_walk_excerpt.bvh
```

## Default timing

110 spm → 33 frames @ 30 fps (~1.1 s per cycle). Adjust via `WalkStyle.cadence_spm`.
