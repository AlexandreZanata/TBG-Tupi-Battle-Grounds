# TBG Blender toolchain

Authoring-only scripts for the Real 3D Character Pipeline (Phase 02).

| Path | Purpose |
|------|---------|
| [RIG-SPEC.md](RIG-SPEC.md) | Bone naming, body classes, animation names |
| [tbg_export/](tbg_export/) | Blender add-on — glTF export + validation |
| [export_unit.py](export_unit.py) | Headless CLI entry |
| [export_game_unit.py](export_game_unit.py) | Pilot game mesh orchestrator (Step C) |
| [scripts/sculpt_to_game_mesh.py](scripts/sculpt_to_game_mesh.py) | Decimate showcase → low GLB (bpy) |
| [scripts/attach_tbg_rig.py](scripts/attach_tbg_rig.py) | Attach body-class armature (bpy) |
| [scripts/retarget_clip_library.py](scripts/retarget_clip_library.py) | Embed clip library (bpy) |
| [scripts/build_rig_template.py](scripts/build_rig_template.py) | Generate `HumanoidHeavy_template.blend` |
| [templates/](templates/) | Rig `.blend` templates |

## Quick start

```bash
# Generate HumanoidHeavy rig template (requires Blender 3.6+)
blender --background --python tools/blender/scripts/build_rig_template.py

# Export unit from template (replace template path as needed)
blender --background tools/blender/templates/HumanoidHeavy_template.blend \
  --python tools/blender/export_unit.py -- tank_human_01
```

Output: `content/characters/meshes/{unit_id}.glb`

## Game mesh export (Phase 03 pilot)

```bash
# Python fallback (no Blender) — proxy skinned mesh + clip stubs
python3 tools/blender/export_game_unit.py demo_sentinel_01

# Blender pipeline (when template + bpy available)
blender --background --python tools/blender/export_game_unit.py -- demo_sentinel_01
```

Output: `content/characters/meshes/game/demo_sentinel_01.glb`  
Validate: `python3 tools/mesh-pipeline/validate_game_glb.py content/characters/meshes/game/demo_sentinel_01.glb`

## Install add-on (interactive)

1. Blender → Edit → Preferences → Add-ons → Install
2. Select `tools/blender/tbg_export` folder (or zip)
3. Enable **TBG Export**

## Tests (pure validation rules)

```bash
python3 -m pytest tests/blender/ -q
```
