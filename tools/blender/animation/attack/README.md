# Attack clips

Procedural combat clips for the TBG FK rig.

## Trident lunge thrust (`scout_trident_thrust_v1`)

- **Unit:** `demo_scout_01`
- **Clip:** `Attack` (embedded in game GLB)
- **Timings:** 300 ms windup / 450 ms hit / 300 ms recovery
- **Spec:** [THRUST-ATTACK-SPEC.md](THRUST-ATTACK-SPEC.md)

```python
from tools.blender.animation.attack import sample_pose_at_ms, frame_count_for_profile

pose = sample_pose_at_ms("scout_trident_thrust_v1", 400.0)  # mid-hit
```

Re-export Scout after pose tuning:

```bash
python3 tools/blender/export_game_unit.py demo_scout_01
```
