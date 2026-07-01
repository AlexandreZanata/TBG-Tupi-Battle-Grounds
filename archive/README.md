# Archive — legacy TBG character pipeline

> **Not part of the active repo.** Backup only — 755+ files.  
> **Active work:** [modular character assembler](../docs/design/modular-character-assembler.md)

Do **not** import from `archive/` in CI, runtime, or new code.

## Contents

| Path | What was archived |
|------|-------------------|
| `content/characters/seed-glb-pipeline/` | 26 `demo_*` seed units, meshes, rig-sessions |
| `content/characters/` | animations, behaviors, materials, textures, cards, meshes-legacy |
| `tools/blender/` | Walk/attack bake, rig, retarget, export |
| `tools/mesh-pipeline/` + `mesh_pipeline/` | Game GLB batch, weight gate, validate |
| `tools/auto_rig/` | Browser Rig Studio Blender jobs |
| `tools/animation_sets/` | Placeholder clip generator |
| `tools/material-maker/` | PBR texture export |
| `tools/unity/` | Animator controller scripts |
| `character_studio/` | Rig / Animation / Arena / Showcase UI + rig API |
| `character_studio/static-js/` | Three.js viewer, behavior playback |
| `unity/` | Full Unity client + `com.tbg.character-library` package |
| `tests/` | Blender, mesh, behavior, material-maker, docs tests |
| `docs/` | ADR-003–006, Phase 02–04b design specs, research |
| `.local/phases/04b-*` | Superseded agent tasks |

## Active replacement

```text
content/characters/library/     modular bases + parts
tools/assembly_studio/          Blender merge (rebuild)
tools/character_studio/         JSON CRUD + Assembly Studio HTML
unity/README.md                 placeholder — client rebuild Phase 04
```

## Restore one file

```bash
cp archive/tools/blender/RIG-SPEC.md tools/assembly_studio/blender/
```
