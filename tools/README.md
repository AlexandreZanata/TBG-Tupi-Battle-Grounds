# Tools (Phase 05 — clean slate)

Active tooling for the **modular character assembler**. Legacy modules live in [`../archive/`](../archive/) — not imported by CI.

| Path | Purpose |
|------|---------|
| [`assembly_studio/`](assembly_studio/) | Blender assembly scripts + MB-Lab docs + library validation |
| [`character_studio/`](character_studio/) | Web UI — unit JSON CRUD + Assembly Studio shell |
| [`requirements.txt`](requirements.txt) | Python deps for studio server + tests |

## Run

```bash
pip install -r tools/requirements.txt
./scripts/seed_library.sh   # optional: regenerate dev library fixtures
python3 -m tools.character_studio serve
# Studio:    http://127.0.0.1:8765/
# Assembler: http://127.0.0.1:8765/assembly-studio.html
```

## Rebuild from archive

To recover a deleted module, copy from `archive/tools/` — do not wire back without a Phase 05 task.
