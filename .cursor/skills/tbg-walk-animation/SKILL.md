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
- Root motion (`WalkRoot`), foot IK debug, or velocity scaling

## Architecture (do not collapse into one file)

```
tools/blender/animation/
  MOVEMENT-MODULE.md  # public index (walk/attack/root_motion/retarget)
  walk/
    constants.py      # WALK_FRAMES, WALK_FPS
    style.py          # WalkStyle + presets
    segment_gait.py   # PRIMARY default sampler
    held_arm_gait.py  # demo_scout_01 only
    biomech_curves.py # CMU-calibrated Perry keyframes
    velocity.py       # scale_walk_style_for_speed (runtime contract)
    sampler.py        # profile dispatch — NO bpy
    profiles.py       # STYLE_PROFILES, UNIT_WALK_PROFILE
    api.py            # sample_pose_at_frame (stable API)
    bake.py           # bake_walk_action (bpy only)
    export.py         # export_walk_clip (bpy only)
  root_motion/        # WalkRoot +Z stride on TBG_Root
  attack/             # thrust + swing samplers
  retarget/           # CMU BVH → TBG_*
tools/blender/scripts/
  build_walk_clip.py  # --root-motion for WalkRoot
  build_attack_clip.py
```

**Rules**

| Layer | May import bpy | Purpose |
|-------|----------------|---------|
| `sampler.py`, `api.py`, `profiles.py`, `velocity.py` | **Never** | Unit-testable gait math |
| `bake.py`, `export.py` | Yes | Blender bake + GLB export |
| `scripts/*.py` | Yes | CLI entrypoints only |

`attack/` follows the same layout — see `attack/THRUST-ATTACK-SPEC.md` and `SWING-ATTACK-SPEC.md`.

## Gait method (segment gait — default)

1. **Primary:** `segment_gait.py` + `biomech_curves.py` (Perry/Winter %cycle)
2. **Scout exception:** `held_arm_gait.py` via `gait_sampler="held_arm"` on `scout_pilot_v1`
3. **Pelvis:** vertical bob on `TBG_Root` — **no** spine yaw/roll on FK rig
4. **In-place default:** `Walk` — `TBG_Root.z = 0`
5. **Root motion:** `WalkRoot` — `stride_length_m × τ` on +Z

Ref: `symmetric_gait.py` (legacy). CMU calibration via `retarget/`.

## Tuning parameters

| Field | Typical | Effect |
|-------|---------|--------|
| `cadence_spm` | 95–115 | Cycle length (frames) |
| `stride_length_m` | 1.3–1.5 | Root-motion stride per cycle |
| `angle_scale` | 0.85–1.05 | Global joint amplitude |
| `pelvis_vertical_m` | 0.035–0.045 | Hip bob height (m) |
| `arm_swing` | 0.18–0.26 | Contralateral arm swing |

**Velocity:** `velocity = stride × cadence / 120` — use `scale_walk_style_for_speed()` for lane speed.

## Commands

```bash
# Unit tests (no Blender)
pytest tests/blender/walk/ tests/blender/root_motion/ -q

# In-place Walk GLB
./tools/blender/blender_cmd.sh --background --python tools/blender/scripts/build_walk_clip.py -- HumanoidLight

# Root-motion WalkRoot
./tools/blender/blender_cmd.sh --background --python tools/blender/scripts/build_walk_clip.py -- --root-motion HumanoidHeavy

# Unit-specific Walk + embed in game mesh
python3 tools/blender/export_game_unit.py demo_scout_01
```

## Viewer pitfalls (Animation Studio)

- Use `SkeletonUtils.clone()` for skinned GLB cache — never `skeleton.clone()` alone
- `AnimationMixer` root = armature scene node, not scaled display wrapper
- Mesh URL cache-bust via `mtime` from `/mesh-info`
- **Root motion toggle:** plays `WalkRoot` instead of `Walk`
- **Foot IK debug toggle:** `footIkDebug.js` — preview only, not game build

## Runtime (Unity)

| Flag | Component |
|------|-----------|
| `locomotion.useRootMotionWalk` | `BehaviorPlayer.SyncWalkRootMotion` |
| `locomotion.footIk` | `LocomotionFootIk` (two-bone IK, Walk only) |

## Out of scope (post-MVP)

- Full-body VRIK / Animation Rigging package
- Motion matching / ML locomotion
- Nakama-synced blend trees (Phase 05)

## Verification checklist

- [ ] `pytest tests/blender/walk/ tests/blender/root_motion/ -q` passes
- [ ] Leg bones: Y=0, Z=0 on all frames (contract test)
- [ ] Frame 0 == last frame (loop closure)
- [ ] Visual check in Animation Studio: Walk natural; optional WalkRoot + foot IK debug

See [reference.md](reference.md) for file-size caps and scaling new clip types.
