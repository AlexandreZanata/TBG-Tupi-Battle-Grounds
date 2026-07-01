# Archive — legacy TBG project

> **Not part of the active repo.** Backup only.  
> **Active repo:** Agent Harness + `.local/` tasks only.

Do **not** import from `archive/` in CI, runtime, or new code unless a task explicitly restores a path.

## Archived 2026-07-01 (full project pivot)

All application code moved here. Root keeps harness symlinks, `AGENTS.md`, `.cursor/rules/`, and `archive/`.

| Path | Contents |
|------|----------|
| `content/` | JSON definitions only — **GLB/FBX assets stripped** (see below) |
| `THIRD_PARTY_NOTICES.md` | MB-Lab + Mixamo notices (binaries removed) |
| `docs/` | ADRs, architecture, design specs, glossary |
| `tools/` | Assembly Studio, Character Studio, Blender scripts, MB-Lab glue |
| `tests/` | Assembly studio, character studio, behavior contracts |
| `unity/` | Unity client placeholder + `com.tbg.character-library` |
| `scripts/` | verify.sh, studio, MB-Lab, Mixamo import scripts |
| ~~`third_party/`~~ | MB-Lab — **deleted** (~279 MB) |
| `character_studio/` | Legacy Rig / Animation / Arena UI (pre-pivot) |
| `.local/` | Phases 01–05 agent tasks, rig-studio sessions, tmp |
| `.cursor/skills/` | Project-specific Cursor skills (e.g. walk animation) |
| `pytest.ini` | Former test config (`testpaths = archive/tests` if re-run) |

## Earlier archive (pre-2026-07-01)

| Path | Notes |
|------|-------|
| `content/characters/seed-glb-pipeline/` | Phase 04b seed units |
| `tools/blender/` | Walk bake, retarget, rig scripts |
| `docs/adr/003–006` | Superseded ADRs |

## Stripped assets (2026-07-01)

Removed from git to shrink the repo (~3.5 GB):

- `third_party/MB-Lab/` — re-vendor from upstream tag `1_8_1`
- `content/characters/seed-glb-pipeline/` — seed GLB units
- `content/characters/imports/mixamo/` — Mixamo FBX downloads
- `content/characters/library/` — assembled GLB exports
- `content/characters/mblab/` — MB-Lab blend templates
- `.local/tmp/`, `.local/rig-studio/` — local Blender sessions

Code, docs, and JSON configs remain under `archive/`.

## Restore one file

```bash
cp archive/docs/GLOSSARY.md ../docs/GLOSSARY.md   # after creating docs/ again
cp archive/scripts/verify.sh ../scripts/verify.sh
```

## Run archived tests (optional)

```bash
pip install -r archive/tools/requirements.txt
pytest -c archive/pytest.ini archive/tests/
```
