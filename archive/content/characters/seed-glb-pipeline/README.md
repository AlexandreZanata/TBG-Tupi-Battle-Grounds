# Archived — Seed GLB pipeline (Phases 02–04b)

> **Status:** Archived 2026-07-01. Superseded by [modular character assembler](../../../docs/design/modular-character-assembler.md).

This folder holds assets from the **per-unit AI → rig → weight paint → Mixamo → export** pipeline. That approach does not scale to MVP roster production (&lt;30 min per character target).

## What lives here

| Subfolder | Contents |
|-----------|----------|
| `definitions/` | `demo_*` unit JSON (26 seed roster) |
| `meshes/showcase/` | High-poly seed sculpt GLBs |
| `meshes/game/` | Retopo + rigged game GLBs (+ `lod1/`) |
| `cards/` | Procedural card PNGs for demo units |
| `rig-sessions/` | Browser Rig Studio session archives |
| `animations/clips/units/` | Per-unit walk clip overrides |
| `manifests/` | `pilot_roster.json`, `fixed_test_imports.json` |

## Still active (not archived)

Shared infrastructure reused by the modular pipeline:

- `content/characters/behaviors/` — behavior profiles
- `content/characters/materials/` — PBR material profiles
- `content/characters/animation_sets/` — body-class clip manifests
- `content/characters/animations/clips/HumanoidHeavy|HumanoidLight/` — shared clips
- `tools/blender/animation/` — walk bake, retarget (library-driven)
- `unity/Packages/com.tbg.character-library/` — runtime spawn

## Restore (if needed)

```bash
# Example: restore one definition
cp _archive/seed-glb-pipeline/definitions/demo_scout_01.json definitions/
```

## New pipeline

See `content/characters/library/` and `tools/assembly_studio/`.
