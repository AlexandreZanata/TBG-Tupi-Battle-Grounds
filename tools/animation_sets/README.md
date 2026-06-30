# Animation sets (Phase 03)

Manifests under `content/characters/animation_sets/` map gameplay actions to shared clip names per body class.

## Placeholder clip bootstrap

When Blender/Mixamo exports are not yet available, generate dev stub GLBs (named empty animations):

```bash
python3 tools/animation_sets/generate_placeholder_clips.py --set humanoid_heavy_combat_v1
```

Output: `content/characters/animations/clips/{bodyClass}/{ClipName}.glb`

Replace stubs with real retargeted clips from `HumanoidHeavy_template.blend` before shipping pilot units.
