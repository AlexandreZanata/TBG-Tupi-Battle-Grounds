# Attack clips

Procedural combat clips for the TBG FK rig. Shared kinetic chain: `chain.py`.

## Archetypes

| Profile | Archetype | Clip | Behavior |
|---------|-----------|------|----------|
| `scout_trident_thrust_v1` | Trident lunge | `Attack` (embedded) | `ranged_shot_v1` |
| `heavy_melee_swing_v1` | Heavy swing | `Attack` | `tank_melee_v1` |
| `heavy_melee_swing_v2` | Heavy swing alt | `Attack2` | `tank_melee_v1` |

Specs: [THRUST-ATTACK-SPEC.md](THRUST-ATTACK-SPEC.md), [SWING-ATTACK-SPEC.md](SWING-ATTACK-SPEC.md), [DRAW-ATTACK-SPEC.md](DRAW-ATTACK-SPEC.md) (deferred).

```python
from tools.blender.animation.attack import sample_pose_at_ms, frame_count_for_profile

pose = sample_pose_at_ms("heavy_melee_swing_v1", 520.0)  # tank hit frame
```

## Bake commands

```bash
# Library clips (HumanoidHeavy)
./tools/blender/blender_cmd.sh --background --python tools/blender/scripts/build_attack_clip.py -- HumanoidHeavy Attack
./tools/blender/blender_cmd.sh --background --python tools/blender/scripts/build_attack_clip.py -- HumanoidHeavy Attack2

# Scout unit (embedded Attack)
python3 tools/blender/export_game_unit.py demo_scout_01
```

## Contract

Impact visual peak within `behavior.timings.attackHitMs` ± 1 frame — see `tests/blender/attack/`.
