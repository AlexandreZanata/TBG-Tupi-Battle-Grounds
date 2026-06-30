# Walk cycle (in-place)

## Sampler roles (2026-06-30)

| Module | Status | Used by |
|--------|--------|---------|
| `segment_gait.py` | **Primary** (`sampler.py` default) | `humanoid_light_v1`, `humanoid_heavy_v1` |
| `held_arm_gait.py` | Scout weapon hold | `scout_pilot_v1` → `demo_scout_01` only |
| `symmetric_gait.py` | Legacy reference | Tests |
| `biomech_curves.py` | Calibrated Perry curves | CMU mocap via `retarget/` |

## Body-class presets (`style.py`)

| Profile | Cadence | `angle_scale` | Notes |
|---------|---------|---------------|-------|
| `humanoid_light_v1` | 115 spm | 0.90 | Agile, smaller joint amplitude |
| `humanoid_heavy_v1` | 95 spm | 1.05 | Slower, reduced arm swing |
| `scout_pilot_v1` | 110 spm | held_arm | Trident guard on right arm |

## Specs

| Document | Purpose |
|----------|---------|
| [BIOMECHANICAL-WALK-SPEC.md](BIOMECHANICAL-WALK-SPEC.md) | TBG implementation reference |
| [AI-WALK-AUTHORING-SPEC.md](AI-WALK-AUTHORING-SPEC.md) | Agent-facing control-point format |
| [../retarget/README.md](../retarget/README.md) | CMU BVH calibration |

## Module map

| File | Layer | bpy |
|------|-------|-----|
| `sampler.py` | Routes segment vs held_arm | No |
| `segment_gait.py` | Perry trunk + legs + arms | No |
| `held_arm_gait.py` | Legs + trident lock | No |
| `biomech_curves.py` | Hip/knee/ankle ° vs %cycle | No |
| `profiles.py` | Unit/body-class registry | No |
| `bake.py` / `export.py` | GLB bake | **Yes** |

## CLI

```bash
./tools/blender/blender_cmd.sh --background --python tools/blender/scripts/build_walk_clip.py -- HumanoidLight
python3 tools/blender/export_game_unit.py demo_scout_01
# Root-motion library clip:
./tools/blender/blender_cmd.sh --background --python tools/blender/scripts/build_walk_clip.py -- --root-motion HumanoidLight
```

## Default timing

115 spm (light) → 31 frames @ 30 fps. Adjust via `WalkStyle.cadence_spm`.
