# Root motion — forward walk translation on `TBG_Root`

Optional **WalkRoot** clip variant: root advances +Z while legs use the same in-place gait.

## Convention

| Axis | Role |
|------|------|
| +Z | Forward (glTF / TBG export) |
| Y | Pelvis vertical bob (from walk sampler) |
| X | Fixed at 0 |

| Parameter | Default | Formula |
|-----------|---------|---------|
| `stride_length_m` | 1.4 m | Displacement per full gait cycle (2 steps) |
| `cadence_spm` | from `WalkStyle` | Steps per minute |
| **velocity** | — | `stride_length_m × cadence_spm / 120` m/s |

Root forward at normalized phase τ:

```text
TBG_Root.z = stride_length_m × τ     where τ = (frame % cycle_frames) / cycle_frames
TBG_Root.y = pelvis bob from walk sampler
TBG_Root.x = 0
```

**In-place default:** `Walk` clip — `TBG_Root.z = 0`.  
**Root motion variant:** `WalkRoot` clip — Z integrated as above.

## Modules

| File | bpy | Purpose |
|------|-----|---------|
| `stride.py` | No | Velocity + root Z from cadence/stride |
| `contacts.py` | No | Heel-strike / toe-off frame indices (stance 62%) |
| `bake.py` | No | Merge forward Z into `GaitPose` |

## CLI

```bash
# In-place (default)
./tools/blender/blender_cmd.sh --background --python tools/blender/scripts/build_walk_clip.py -- HumanoidLight

# Root-motion variant → WalkRoot.glb
./tools/blender/blender_cmd.sh --background --python tools/blender/scripts/build_walk_clip.py -- --root-motion HumanoidLight
```

## Runtime

- **Unity:** `BehaviorProfile.UseRootMotionWalk` + `BehaviorPlayer.SyncWalkRootMotion` (stub — Phase 04 lane sync)
- **Animation Studio:** “Root motion walk” toggle plays `WalkRoot` instead of `Walk`

## Tests

```bash
pytest tests/blender/root_motion/ -q
```
