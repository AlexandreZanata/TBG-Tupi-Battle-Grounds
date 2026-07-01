# Shared animator controllers (Phase 02 §7)

One `.controller` per **body class** (see [RIG-SPEC.md](../../tools/blender/RIG-SPEC.md)). Animation clips are embedded in unit `.glb` files (glTF names: `Idle`, `Spawn`, `Attack`, `Death`) or loaded from separate animation `.glb` assets in Phase 03.

## Clip library (Phase 03)

Shared clips per body class live under `clips/{bodyClass}/{ClipName}.glb`. Manifests in `../animation_sets/` map action keys → clip names.

| Set id | Body class | Clip source |
|--------|------------|-------------|
| `humanoid_heavy_combat_v1` | HumanoidHeavy | `clips/HumanoidHeavy/*.glb` |

Regenerate dev placeholder clips (empty animation stubs):

```bash
python3 tools/animation_sets/generate_placeholder_clips.py --set humanoid_heavy_combat_v1
```

## Controllers

| File | Body class | Attack state |
|------|------------|--------------|
| `HumanoidHeavy.controller` | HumanoidHeavy | `AttackHeavy` |
| `HumanoidLight.controller` | HumanoidLight | `AttackRanged` |
| `Mechanical.controller` | Mechanical | `AttackCast` |
| `Organic.controller` | Organic | `AttackFast` |

Triggers (all controllers): `Idle`, `Spawn`, `Attack`, `Death`, `Walk`, `Ultimate`.

Regenerate YAML assets (writes to `content/characters/animations/` and `Packages/com.tbg.character-library/Animations/`):

```bash
python3 tools/unity/generate_animator_controllers.py
```

## Behavior profiles

Archetype → profile mapping lives in `content/characters/behaviors/*_v1.json`. `BehaviorPlayer` reads triggers and state names from the profile linked on each unit definition (`behaviorProfile` field).
