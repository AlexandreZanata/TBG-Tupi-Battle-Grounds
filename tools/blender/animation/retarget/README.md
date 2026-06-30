# CMU → TBG retarget pipeline

Import CMU cgspeed BVH walk data, extract hip/knee/ankle %cycle curves, and calibrate `biomech_curves.py`.

## Coordinate alignment

| Space | Convention |
|-------|------------|
| CMU cgspeed BVH | Y-up, character faces **+Z**; frame 0 = T-pose |
| TBG FK rig | Y-up, +Z forward (`RIG-SPEC.md`) |
| glTF export | Y-up, +Z forward |

**Frame 0:** skip for gait analysis — cgspeed inserts a synthetic T-pose.

**Leg flexion:** CMU `Xrotation` on `RightUpLeg` / `RightLeg` / `RightFoot` is negated and heel-strike-anchored to Perry reference (+30° hip, 0° knee/ankle at τ=0).

## Bone map

[`BONE_MAP_CMU_TO_TBG.json`](./BONE_MAP_CMU_TO_TBG.json) maps every deformation bone in `RIG-SPEC.md` to CMU joint names.

## Modules

| File | bpy | Purpose |
|------|-----|---------|
| `bone_map.py` | No | Load + validate mapping |
| `bvh_parser.py` | No | Parse BVH hierarchy + motion |
| `curves_from_mocap.py` | No | Gait period detect, curve extract, Perry merge |
| `import_bvh.py` | **Yes** | Sample `GaitPose` from imported armature |

## CLI

```bash
# Extract + print calibrated hip/knee/ankle keyframes (no Blender)
python3 tools/blender/scripts/retarget_mocap_walk.py --calibrate \
  .local/reference/mocap/35_01.bvh .local/reference/mocap/35_04.bvh

# Apply calibration to biomech_curves.py (writes keyframes)
python3 tools/blender/scripts/retarget_mocap_walk.py --write-curves \
  tests/blender/fixtures/mocap/35_01_walk_excerpt.bvh

# Blender: sample frame 60 as GaitPose JSON
./tools/blender/blender_cmd.sh --background --python tools/blender/scripts/retarget_mocap_walk.py -- \
  --sample tests/blender/fixtures/mocap/35_01_walk_excerpt.bvh --frame 60
```

## Reference data

Download CMU walks to `.local/reference/mocap/` (gitignored). Committed test fixture: `tests/blender/fixtures/mocap/35_01_walk_excerpt.bvh`.

**Citation:** Data from [mocap.cs.cmu.edu](http://mocap.cs.cmu.edu/). BVH port: [una-dinosauria/cmu-mocap](https://github.com/una-dinosauria/cmu-mocap).

## Visual comparison (manual)

1. Bake current Walk: `python3 tools/blender/export_game_unit.py demo_scout_01`
2. Run `--sample` on CMU BVH at matching gait phase
3. Compare leg angles in Animation Studio side-by-side
4. Store screenshots in `.local/phases/03b-biomechanical-movement-module/artifacts/`
