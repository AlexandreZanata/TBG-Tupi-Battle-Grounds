# Blender animation authoring

Modular clip generators for the TBG FK rig (`TBG_*` bones). Each locomotion or combat clip type lives in its own subpackage.

| Package | Status | Purpose |
|---------|--------|---------|
| `walk/` | **Active** | Level-2 biomechanical in-place walk (`BIOMECHANICAL-WALK-SPEC.md`) |
| `retarget/` | **Active** | CMU BVH → TBG bone map + gait curve calibration |
| `root_motion/` | **Active** | Optional `WalkRoot` forward translation on `TBG_Root` |
| `idle/` | Planned | Breathing / weight-shift idle |
| `attack/` | **Active** | Thrust + heavy swing (`THRUST-ATTACK-SPEC.md`, `SWING-ATTACK-SPEC.md`) |

**Index:** [MOVEMENT-MODULE.md](MOVEMENT-MODULE.md)

## Layout convention

```
animation/<clip_type>/
  constants.py   # fps, frame count
  types.py       # shared type aliases
  style.py       # tunable parameters (dataclass)
  sampler.py     # pure math — no bpy
  profiles.py    # unit/body-class profile registry
  api.py         # stable public sampler API
  bake.py        # Blender action bake (bpy only)
  export.py      # standalone GLB export CLI logic
  README.md      # clip-specific docs
```

`tools/blender/scripts/` keeps **thin entrypoints** only (`build_walk_clip.py`, `build_attack_clip.py`, shims for legacy imports).

## Agent skill

See [.cursor/skills/tbg-walk-animation/SKILL.md](../../../.cursor/skills/tbg-walk-animation/SKILL.md) for authoring workflow.
