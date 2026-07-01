# Modular Character Assembler

> Web orchestration layer above Blender. **Blender does all mesh work** — merge, materials, export.

## Surfaces

| Surface | Path |
|---------|------|
| Web UI | `static/assembly-studio.html` (served by Character Studio) |
| Blender scripts | `blender/assemble_character.py` (Phase 05 §3) |
| Content library | `content/characters/library/` |
| Agent tasks | `.local/phases/05-modular-character-assembler/` |

## Run locally

```bash
pip install -r tools/requirements.txt
python3 -m tools.character_studio serve
# Open http://127.0.0.1:8765/assembly-studio.html
```

## Flow

```text
Select base body → head → parts → material → Export (Blender job) → library/exports/
```

## Related

- [modular-character-assembler.md](../../docs/design/modular-character-assembler.md)
- [mblab_bases.md](./blender/mblab_bases.md) — MB-Lab base authoring
