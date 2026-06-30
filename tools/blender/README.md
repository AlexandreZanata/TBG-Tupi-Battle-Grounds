# TBG Blender toolchain

Authoring-only scripts for the Real 3D Character Pipeline (Phase 02).

| Path | Purpose |
|------|---------|
| [RIG-SPEC.md](RIG-SPEC.md) | Bone naming, body classes, animation names |
| [tbg_export/](tbg_export/) | Blender add-on — glTF export + validation |
| [blender_cmd.sh](blender_cmd.sh) | Resolve `blender`, Flatpak (repo mount), or Docker (`TBG_BLENDER_DOCKER_IMAGE`) |
| [setup.sh](setup.sh) | One-time setup — verify Blender + generate rig template |
| [export_unit.py](export_unit.py) | Headless CLI entry |
| [export_game_unit.py](export_game_unit.py) | Pilot game mesh orchestrator (prefers Blender) |
| [scripts/sculpt_to_game_mesh.py](scripts/sculpt_to_game_mesh.py) | Decimate showcase → low GLB (bpy) |
| [scripts/attach_tbg_rig.py](scripts/attach_tbg_rig.py) | Attach body-class armature (bpy) |
| [scripts/retarget_clip_library.py](scripts/retarget_clip_library.py) | Embed named clip actions (bpy) |
| [animation/](animation/) | Modular clip generators (`walk/`, future `idle/`, `attack/`) |
| [animation/walk/](animation/walk/) | Parametric in-place walk cycle |
| [scripts/build_walk_clip.py](scripts/build_walk_clip.py) | CLI — export Walk GLB |
| [scripts/export_game_unit_bpy.py](scripts/export_game_unit_bpy.py) | In-Blender entry (no PIL) |
| [scripts/build_rig_template.py](scripts/build_rig_template.py) | Generate `HumanoidHeavy_template.blend` |
| [templates/](templates/) | Rig `.blend` templates |

## Setup (required once per machine)

```bash
./tools/blender/setup.sh
```

Installs nothing — detects native `blender` or Flatpak `org.blender.Blender`, then generates `templates/HumanoidHeavy_template.blend` if missing.

Flatpak (Pop!_OS / Linux) — mounts the repo (required when the project lives outside `$HOME`, e.g. `/data/dev/...`):

```bash
flatpak install flathub org.blender.Blender
./tools/blender/setup.sh
```

Docker (optional):

```bash
export TBG_BLENDER_DOCKER_IMAGE=nytimes/blender:3.6.4-ubuntu22.04  # example
./tools/blender/blender_cmd.sh --background --python tools/blender/scripts/build_walk_clip.py -- HumanoidLight
```

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
./tools/blender/setup.sh   # once per machine
python3 tools/blender/export_game_unit.py demo_sentinel_01
```

Uses **Blender** when available (`blender_cmd.sh` → native or Flatpak). Pipeline: decimate showcase sculpt → attach `HumanoidHeavy` rig → embed clip actions → resize textures + Draco.

Python fallback (proxy mesh) runs only when Blender is missing.

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
