# Modular Character Assembler

> **Phase 05** — Replace per-unit rigging with library-based assembly.  
> **Target:** New character in **&lt;30 minutes**.

---

## Problem

The seed GLB pipeline (AI → rig → weight paint → Mixamo → fix → export) does not scale:

| Step | Typical time |
|------|--------------|
| AI mesh cleanup | 15–60 min |
| Rig + weight paint | 1–4 hours |
| Clip retarget fixes | 30–120 min |
| **Total** | **2–6 hours per unit** |

For 26+ MVP units this is a production blocker.

---

## Solution

**Modular assembly** — rig once per base body, swap interchangeable parts at export time.

```text
Library
  ↓
Select body (male / female / small / large)
  ↓
Select head · hair · armor · weapons · …
  ↓
Select material palette
  ↓
Export (Blender headless)
  ↓
Done
```

---

## Architecture

### Content library

Path: `content/characters/library/`

```text
bases/       # MB-Lab authored bodies (4 archetypes)
parts/       # Socket-attached pieces (heads, hair, armor, weapons, …)
materials/   # Faction tint profiles
rigs/        # One canonical armature per base
exports/     # Assembled game GLBs
```

### Web layer

**Assembly Studio** (`tools/character_studio/static/assembly-studio.html`) — thin UI using existing `studio.css` shell:

- Browse library manifests
- Preview selection in Three.js
- Queue Blender export job
- Poll job status → download GLB

### Blender layer

`tools/assembly_studio/blender/assemble_character.py` — headless script:

1. Load base body + rig
2. Attach parts at socket bones (bone-parented meshes; base skin weights only)
3. Apply material profile from `library/materials/{materialId}.json`
4. Export to `library/exports/{unit_id}.glb`

CLI:

```bash
tools/assembly_studio/blender/blender_cmd.sh --background \
  --python tools/assembly_studio/blender/assemble_character.py -- \
  --recipe path/to/recipe.json
```

Recipe schema: `library/_schema.recipe.json` — `{ unitId, baseId, parts[], materialId }`.

**MB-Lab** is the recommended base body generator (free, open source, in-Blender, includes rig). See `tools/assembly_studio/blender/mblab_bases.md`.

### Runtime (unchanged)

- Shared clip library per body class (`HumanoidLight`, `HumanoidHeavy`)
- `com.tbg.character-library` spawns from exported GLB + behavior profile

---

## What we keep from prior phases

| Asset | Path | Reuse |
|-------|------|-------|
| Behavior profiles | `content/characters/behaviors/` | Yes |
| Material profiles | `content/characters/materials/` | Extend for palettes |
| Animation sets | `content/characters/animation_sets/` | Yes |
| Walk / attack clips | `animations/clips/Humanoid*/` | Yes |
| Unity CharacterLibrary | `unity/Packages/com.tbg.character-library/` | Yes |

## What is archived

Seed GLB demo roster → `archive/content/characters/seed-glb-pipeline/`

Phase 04b scripts remain for reference; new work goes to `tools/assembly_studio/`.

---

## Quality gate

1. Four base bodies authored (MB-Lab → TBG skeleton)
2. ≥3 parts per core category (heads, hair, armor, weapons)
3. Web UI → Blender export → Unity spawn with walk clip
4. Documented timed run **&lt;30 minutes** for a new unique character

---

## Related

- [ADR-007](../adr/ADR-007-modular-character-assembler.md)
- [character-creation-guide.md](./character-creation-guide.md)
- Agent tasks: `.local/phases/05-modular-character-assembler/`
