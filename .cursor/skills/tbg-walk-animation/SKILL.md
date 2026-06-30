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
    sampler.py        # pure math — NO bpy
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

## Gait method (parametric, not hand-keyed extremes)

1. **Four phases** per stride: contact → recoil → passing → high-point
2. **Legs**: `stride * cos(phase)` on **X axis only** — never Y/Z twist on legs
3. **Knee**: `knee_support + knee_swing * max(0, sin(phase))`
4. **Hip bob**: `hip_bob * sin(2 * phase)` (two bobs per cycle)
5. **Arms**: counter-swing `-arm_swing * cos(phase)` vs legs
6. **In-place**: only `TBG_Root` Y translation; no root XZ (gameplay moves unit)

Refs: [Rusty Animator walk blueprint](https://rustyanimator.com/walk-cycle-animation/), [Little Polygon locomotion](https://blog.littlepolygon.com/posts/loco1/).

## Add a unit profile

1. Add `WalkStyle(...)` preset in `style.py`
2. Register in `profiles.py`:
   - `STYLE_PROFILES["my_profile_v1"] = ...`
   - `UNIT_WALK_PROFILE["demo_unit_id"] = "my_profile_v1"`
3. Run tests: `pytest tests/blender/walk/ -q`
4. Re-export: `python3 tools/blender/export_game_unit.py <unit_id>`
5. Hard-refresh Animation Studio (`Ctrl+Shift+R`)

## Tuning parameters

| Field | Typical range | Effect |
|-------|---------------|--------|
| `stride` | 0.32–0.45 rad | Thigh swing |
| `knee_swing` | 0.40–0.55 | Lift-phase knee bend |
| `hip_bob` | 0.008–0.018 m | Vertical pelvis |
| `arm_swing` | 0.25–0.40 | Free arm swing |
| `arm_swing_r_scale` | 0.0–1.0 | Weapon arm damping |
| `right_arm_hold` | euler offset | Static weapon pose base |

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
- [ ] Visual check in Animation Studio: Walk ~1.33s, legs alternate

See [reference.md](reference.md) for file-size caps and scaling new clip types.
