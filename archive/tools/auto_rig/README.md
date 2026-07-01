# Experimental auto-rig (Blender headless)

> **Status:** experimental — not wired into `import_game_ready_unit.py`.  
> **Goal:** validate a standalone path from raw AI GLB (mesh + textures, no skeleton) to a skinned GLB on the shared TBG body-class template.

Production pipeline (`attach_tbg_rig.py` + decimation + clip retarget) currently passes the weight gate on ~8% of the pilot roster. This folder isolates a **simpler orchestration**:

```text
raw GLB/OBJ
  → load TBG body-class template armature
  → import + prep (join, merge, normalize 1.8 m, optional voxel remesh, decimate for rig)
  → align skeleton to mesh bounds
  → bind weights (--bind-mode, default fallback)
  → export skinned GLB
```

No clip retarget in this PoC — rig only.

---

## Blender 5.1 headless finding

`bpy.ops.object.parent_set(type='ARMATURE_AUTO')` returns `FINISHED` but assigns **zero** vertex groups in headless Blender 5.1 (Docker and native). The same regression affects production `rig_weight_eval.paint_automatic_weights`.

**Default `--bind-mode fallback`** uses anatomical proximity weights (same as `rig_weight_bind.py`) and picks the better result. Use `--bind-mode proximity` to force proximity only; `--bind-mode automatic` to probe ARMATURE_AUTO (expect failure until Blender fixes headless parenting).

---

## Prerequisites

```bash
./tools/blender/setup.sh   # generates HumanoidLight/HumanoidHeavy templates
```

Blender 3.6+ via native binary, Flatpak, or Docker (`TBG_BLENDER_DOCKER_IMAGE`).

---

## Usage

```bash
./tools/blender/blender_cmd.sh --background \
  --python tools/experimental/auto_rig/auto_rig_mesh.py -- \
  content/characters/meshes/showcase/demo_bruiser_01.glb \
  .local/tmp/auto_rig/demo_bruiser_01_rigged.glb \
  --body-class HumanoidLight \
  --voxel-remesh \
  --bind-mode fallback \
  --report-json
```

| Flag | Default | Purpose |
|------|---------|---------|
| `--body-class` | `HumanoidLight` | Template under `tools/blender/templates/` |
| `--height` | `1.8` | Target character height (meters, Y-up) |
| `--voxel-remesh` | off | Remesh dense AI meshes (&gt;500k faces auto, or force) |
| `--max-faces` | `100000` | Decimate before bind so weight solvers stay tractable |
| `--bind-mode` | `fallback` | `proximity`, `automatic`, or `fallback` (proximity vs automatic) |
| `--z-up` | off | Rotate Z-up source meshes to glTF Y-up |
| `--report-json` | off | Print weight metrics JSON on stdout |

Exit code `0` = weight gate pass; `2` = rigged but gate failed (still writes output).

---

## Python runner (tests / automation)

```python
from pathlib import Path
from tools.experimental.auto_rig.run_auto_rig import run_auto_rig_mesh

result = run_auto_rig_mesh(
    Path("content/characters/meshes/showcase/demo_bruiser_01.glb"),
    Path(".local/tmp/auto_rig/out.glb"),
    voxel_remesh=True,
)
print(result.report)
```

---

## Validate output

```bash
PYTHONPATH=tools python3 -c "
from pathlib import Path
from mesh_pipeline.glb_inspect import inspect_glb
print(inspect_glb(Path('.local/tmp/auto_rig/out.glb')))
"
```

Preview in Character Studio or Motion Arena after export.

---

## Motion Arena single-pilot dev slice

Arena lists only pilots in `arenaPilotConfig.js` (default: **`demo_scout_01`**, **`demo_guard_01`** — rig session walk test).

```bash
PYTHONPATH=tools python3 tools/experimental/auto_rig/promote_arena_pilot.py --id demo_scout_01 --fast
```

Open Character Studio → `/motion-arena.html`.

---

## Rig Studio — manual skeleton + walk test

Browser UI: `/rig-studio.html`. Saves live session to `.local/rig-studio/sessions/{unitId}/bones.json`.

**Archive skeleton (git-tracked, do not lose):**

```bash
python3 tools/experimental/auto_rig/archive_rig_session.py --id demo_guard_01
```

**Bake walk on your placement** (needs `session.blend` from prep + game GLB from finalize):

```bash
python3 tools/experimental/auto_rig/promote_rig_session_walk.py --id demo_guard_01
```

Format spec: [docs/design/rig-studio-bones-format.md](../../docs/design/rig-studio-bones-format.md).  
Archived bones: [content/characters/rig-sessions/](../../content/characters/rig-sessions/).

| File | Role |
|------|------|
| [rig_studio_bpy.py](rig_studio_bpy.py) | Prep / sync / finalize |
| [rig_bone_names.py](rig_bone_names.py) | glTF ↔ Blender bone names |
| [archive_rig_session.py](archive_rig_session.py) | `.local` → `content/characters/rig-sessions/` |
| [bake_session_walk_bpy.py](bake_session_walk_bpy.py) | Walk clip from session + bones.json |
| [embed_unit_walk_bpy.py](embed_unit_walk_bpy.py) | Embed unit Walk into game GLB |
| [promote_rig_session_walk.py](promote_rig_session_walk.py) | Bake + embed (full pipeline) |

---

## Files

| File | Role |
|------|------|
| [auto_rig_mesh.py](auto_rig_mesh.py) | CLI entry (bpy) |
| [auto_rig_ops.py](auto_rig_ops.py) | Template load, bind modes, export |
| [auto_rig_align.py](auto_rig_align.py) | Armature ↔ mesh alignment |
| [auto_rig_decimate.py](auto_rig_decimate.py) | Pre-bind decimation cap |
| [auto_rig_fixture.py](auto_rig_fixture.py) | Headless limb-proxy smoke test |
| [run_auto_rig.py](run_auto_rig.py) | Subprocess wrapper for pytest |
| [archive_rig_session.py](archive_rig_session.py) | Archive Rig Studio bones to `content/` |
| [bake_session_walk_bpy.py](bake_session_walk_bpy.py) | Walk bake from session + bones |
| [run_session_walk_test.py](run_session_walk_test.py) | Python wrapper for walk bake |

Reuses: `prep_ai_mesh_ops.py`, `rig_weight_bind.py`, `rig_weight_paint.py`, `tools/blender/templates/*_template.blend`.

---

## Next steps (if PoC succeeds)

1. Compare weight gate pass rate vs full `import_game_ready_unit.py` on `fixed_test_imports.json`.
2. Merge best bind path into production when pass rate improves.
3. Add clip retarget (`retarget_clip_library.py`) once weights are acceptable.

**Related:** [biped-game-ready-pipeline.md](../../docs/design/biped-game-ready-pipeline.md), [RIG-SPEC.md](../../tools/blender/RIG-SPEC.md), [rig-studio-bones-format.md](../../docs/design/rig-studio-bones-format.md).
