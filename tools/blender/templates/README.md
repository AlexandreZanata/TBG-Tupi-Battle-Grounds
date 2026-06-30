# Rig templates

| File | Body class | Generate |
|------|------------|----------|
| `HumanoidHeavy_template.blend` | HumanoidHeavy | See below |
| `HumanoidLight_template.blend` | HumanoidLight | See below |

Templates are **not** committed until generated locally (binary, Blender version-specific).

```bash
blender --background --python tools/blender/scripts/build_rig_template.py
```

See [RIG-SPEC.md](../RIG-SPEC.md) for bone naming and body classes.
