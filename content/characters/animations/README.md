# Shared animator controllers (Phase 02 §7)

One `.controller` per **body class** (see [RIG-SPEC.md](../../tools/blender/RIG-SPEC.md)). Animation clips are embedded in unit `.glb` files (glTF names: `Idle`, `Spawn`, `Attack`, `Death`) or loaded from separate animation `.glb` assets in Phase 02 §8.

## Controllers

| File | Body class | Attack state |
|------|------------|--------------|
| `HumanoidHeavy.controller` | HumanoidHeavy | `AttackHeavy` |
| `HumanoidLight.controller` | HumanoidLight | `AttackRanged` |
| `Mechanical.controller` | Mechanical | `AttackCast` |
| `Organic.controller` | Organic | `AttackFast` |

Triggers (all controllers): `Idle`, `Spawn`, `Attack`, `Death`.

Regenerate YAML assets:

```bash
python3 tools/unity/generate_animator_controllers.py
```

## Behavior profiles

Archetype → profile mapping lives in `content/characters/behaviors/*_v1.json`. `BehaviorPlayer` reads triggers and state names from the profile linked on each unit definition (`behaviorProfile` field).
