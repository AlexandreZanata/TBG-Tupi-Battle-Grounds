---
name: tbg-walk-animation
description: >-
  Author and extend TBG in-place walk cycles for FK TBG_* armatures. Use when
  adding walk profiles, tuning gait parameters, baking Walk clips, or organizing
  animation code under tools/blender/animation/.
---

# TBG Walk Animation

## When to load

- User asks to improve, add, or fix **Walk** / walk cycle / gait animation
- Tuning scout or humanoid locomotion in Animation Studio
- Refactoring or extending `tools/blender/animation/walk/`

## Architecture (do not collapse into one file)

```
tools/blender/animation/
  README.md           # index of clip types
  walk/
    constants.py      # WALK_FRAMES, WALK_FPS
    types.py          # GaitPose, BoneEuler
    style.py          # WalkStyle + presets
    curves.py         # Catmull-Rom cyclic interpolation
    biomech_curves.py # Perry/Winter ° vs %cycle keyframes
    sampler.py        # biomech pose assembly — NO bpy
    profiles.py       # STYLE_PROFILES, UNIT_WALK_PROFILE
    api.py            # sample_pose_at_frame (stable API)
    bake.py           # bake_walk_action (bpy only)
    export.py         # export_walk_clip (bpy only)
    README.md
tools/blender/scripts/
  build_walk_clip.py  # thin CLI entry only
  walk_*.py           # backward-compatible shims
```

**Rules**

| Layer | May import bpy | Purpose |
|-------|----------------|---------|
| `sampler.py`, `api.py`, `profiles.py` | **Never** | Unit-testable gait math |
| `bake.py`, `export.py` | Yes | Blender bake + GLB export |
| `scripts/*.py` | Yes | CLI entrypoints only |

Future clip types (`idle/`, `attack/`) follow the same layout under `animation/`.

## Gait method (symmetric mirrored — FK TBG rig)

1. **Hips:** `R = A·cos(2πτ)`, `L = -A·cos(2πτ)` — same instant, opposite sagittal (no diagonal)
2. **Knee/foot:** phase-offset `τ+0.5` per leg; stance 62% / swing 38%
3. **Pelvis:** vertical bob only on `TBG_Root` — **no** spine yaw/roll
4. **Arms:** contralateral vs opposing leg phase
5. **In-place:** no root XZ

Ref: `symmetric_gait.py`. Perry curves in `biomech_curves.py` for future retarget only.

## Tuning parameters

| Field | Typical | Effect |
|-------|---------|--------|
| `cadence_spm` | 100–120 | Cycle length (frames) |
| `angle_scale` | 0.85–1.0 | Global joint amplitude |
| `pelvis_vertical_m` | 0.035–0.045 | Hip bob height (m) |
| `pelvis_yaw_deg` / `pelvis_roll_deg` | 3–5° | Saunders rotation/obliquity |
| `arm_swing_deg` | 18–28° | Contralateral arm swing |
| `arm_swing_r_scale` | 0–1 | Weapon arm damping |

## Commands

```bash
# Unit tests (no Blender)
pytest tests/blender/walk/ -q

# Library Walk GLB
./tools/blender/blender_cmd.sh --background --python tools/blender/scripts/build_walk_clip.py -- HumanoidLight

# Unit-specific Walk + embed in game mesh
./tools/blender/blender_cmd.sh --background --python tools/blender/scripts/build_walk_clip.py -- HumanoidLight demo_scout_01
python3 tools/blender/export_game_unit.py demo_scout_01
```

## Viewer pitfalls (Animation Studio)

- Use `SkeletonUtils.clone()` for skinned GLB cache — never `skeleton.clone()` alone
- `AnimationMixer` root = armature scene node, not scaled display wrapper
- Mesh URL cache-bust via `mtime` from `/mesh-info`

## Out of scope (separate future packages)

- Root motion / foot sliding fix → `animation/root_motion/` (planned)
- IK foot planting → requires IK rig targets
- Mixamo retarget → one-time import pipeline, not procedural walk

## Verification checklist

- [ ] `pytest tests/blender/walk/ -q` passes
- [ ] Leg bones: Y=0, Z=0 on all frames (contract test)
- [ ] Frame 0 == frame 40 (loop closure)
- [ ] Visual check in Animation Studio: Walk ~1.1s @ 110 spm, legs alternate

See [reference.md](reference.md) for file-size caps and scaling new clip types.
