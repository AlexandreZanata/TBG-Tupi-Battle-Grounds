# Modular Character Assembler

> Web orchestration layer above Blender. **Blender does all mesh work** — merge, materials, export.

## Surfaces

| Surface | Path |
|---------|------|
| Web UI | `static/assembly-studio.html` (served by Character Studio) |
| MB-Lab Studio | `static/mblab-studio.html` — parametric bodies via Blender + [MB-Lab](https://github.com/animate1978/MB-Lab) |
| Blender scripts | `blender/assemble_character.py` (Phase 05 §3) |
| Content library | `content/characters/library/` |
| Agent tasks | `.local/phases/05-modular-character-assembler/` |

## Run locally

```bash
pip install -r tools/requirements.txt
python3 -m tools.character_studio serve
# Open http://127.0.0.1:8765/assembly-studio.html
```

## Seed library fixtures

```bash
./scripts/seed_library.sh   # blocky TBG bases + box parts (dev)
./scripts/verify.sh         # contract tests
```

MB-Lab production export:

```bash
./scripts/vendor-mblab.sh          # once — clone tag 1_8_1 into third_party/MB-Lab
./scripts/install-mblab.sh         # once — copy into Blender add-ons
# Author content/characters/mblab/projects/{male,female,small,large}.blend in Blender GUI
./scripts/export-mblab-base.sh male # → library/bases/male/body.glb
```

See [`blender/mblab_bases.md`](blender/mblab_bases.md) and [`docs/design/mblab-web-authoring.md`](../../docs/design/mblab-web-authoring.md).

## Assemble export (§3)

Recipe JSON (`library/exports/{unitId}.recipe.json`):

```json
{
  "unitId": "demo_guard_male_01",
  "baseId": "male",
  "parts": ["head_round_01", "weapon_sword_01"],
  "materialId": "default"
}
```

```bash
tools/assembly_studio/blender/blender_cmd.sh --background \
  --python tools/assembly_studio/blender/assemble_character.py -- \
  --recipe content/characters/library/exports/demo_guard_male_01.recipe.json
```

Output: `content/characters/library/exports/{unitId}.glb`

Pure-Python GLB merge runs automatically when `bpy` is unavailable (CI).

## Web API (§4)

| Endpoint | Method | Purpose |
|----------|--------|---------|
| `/api/assembly/library` | GET | Bases, parts, materials catalog |
| `/api/assembly/export` | POST | Queue assemble job (recipe JSON) |
| `/api/assembly/jobs/{id}` | GET | Poll job status |
| `/api/assembly/exports/{unitId}/mesh` | GET | Download assembled GLB |
| `/api/assembly/mblab/status` | GET | MB-Lab template readiness |
| `/api/assembly/mblab/generate` | POST | Queue MB-Lab morph + export job |
| `/api/assembly/mblab/jobs/{id}` | GET | Poll MB-Lab job |

## Flow

```text
Select base body → head → parts → material → Export (Blender job) → library/exports/
```

## Related

- [modular-character-assembler.md](../../docs/design/modular-character-assembler.md)
- [mblab_bases.md](./blender/mblab_bases.md) — MB-Lab base authoring
