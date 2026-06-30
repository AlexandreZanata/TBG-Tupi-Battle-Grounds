# Walk cycle (in-place)

Parametric FK walk for `TBG_*` armatures. Gameplay drives lane movement; the clip loops with no root XZ translation.

## Module map

| File | Layer | bpy |
|------|-------|-----|
| `constants.py` | Timing (`40` frames @ `30` fps) | No |
| `types.py` | `GaitPose`, `BoneEuler` | No |
| `style.py` | `WalkStyle` dataclass + presets | No |
| `sampler.py` | Sine/cosine gait math | No |
| `profiles.py` | Unit → profile registry | No |
| `api.py` | `sample_pose_at_frame` | No |
| `bake.py` | `bake_walk_action` | **Yes** |
| `export.py` | `export_walk_clip` | **Yes** |

## Add a unit profile

1. Add preset in `style.py` or compose from `WalkStyle(...)`.
2. Register in `profiles.py` → `STYLE_PROFILES` and `UNIT_WALK_PROFILE`.
3. Run contract tests: `pytest tests/blender/walk/ -q`
4. Re-export unit: `python3 tools/blender/export_game_unit.py <unit_id>`

## CLI

```bash
./tools/blender/blender_cmd.sh --background --python tools/blender/scripts/build_walk_clip.py -- HumanoidLight
./tools/blender/blender_cmd.sh --background --python tools/blender/scripts/build_walk_clip.py -- HumanoidLight demo_scout_01
```

## Tuning guide

| Parameter | Effect |
|-----------|--------|
| `stride` | Thigh swing amplitude (~0.35–0.45 rad) |
| `knee_swing` | Knee bend during leg lift |
| `hip_bob` | Vertical pelvis oscillation (meters) |
| `arm_swing` | Free arm counter-swing |
| `arm_swing_r_scale` | Reduce weapon-arm motion (Scout trident) |

## References

- Contact / recoil / passing / high-point — [Rusty Animator walk blueprint](https://rustyanimator.com/walk-cycle-animation/)
- Hip sine + antiphase feet — [Little Polygon locomotion](https://blog.littlepolygon.com/posts/loco1/)

## Future

- Root motion extraction (separate `animation/root_motion/` package)
- IK foot planting for uneven terrain
