# Biped Game-Ready Pipeline

> Phase 04b — scale AI/sculpt meshes to validated game GLBs with shared `TBG_*` skeletons.  
> **Operational detail:** [tools/mesh-pipeline/README.md](../../tools/mesh-pipeline/README.md)  
> **Related:** [ADR-006](../adr/ADR-006-biped-auto-rig-pipeline.md), [real-3d-character-pipeline.md](real-3d-character-pipeline.md)

---

## Mission

Every biped combat unit defaults to a **body-class skeleton** (`HumanoidLight`, `HumanoidHeavy`) with automated rig, weights, clip retarget, and mobile budget validation. Manual rigs are **exceptions only** ([rig-exception-workflow.md](rig-exception-workflow.md)).

The bottleneck is game-readiness (rig + weights + clips), not mesh generation.

---

## Single command

```bash
python3 tools/blender/import_game_ready_unit.py \
  --source /path/raw.glb --id demo_scout_01 \
  --body-class HumanoidLight \
  --animation-set humanoid_light_combat_v1 \
  --mixamo-fallback
```

**Steps:** prep (optional) → showcase optimize → `export_game_unit` → weight gate → `validate_game_glb`

Blocked when `rigException: true` in the unit definition.

---

## Pipeline overview

```text
AI / sculpt GLB
    → prep_ai_mesh.py (normalize 1.8 m, Y-up)
    → optimize_glb.sh --mode visual (showcase tier)
    → sculpt_to_game_mesh + attach_tbg_rig + retarget_clip_library
    → post-process (ORM, Draco) → meshes/game/{id}.glb
    → weight gate + budget validation
```

**Fallback when weight gate fails:** [Mixamo path](../../tools/blender/animation/retarget/BONE_MAP_MIXAMO_TO_TBG.json) (dev rig bootstrap only — ship body-class clips, not Mixamo mocap).

**Batch:**

```bash
./tools/mesh-pipeline/batch_seed_to_game.sh   # TBG_WEIGHT_GATE=1 default
python3 tools/mesh_pipeline/baseline_audit.py --write
```

---

## Key artifacts

| Topic | Doc / path |
|-------|------------|
| Rig bone names | [RIG-SPEC.md](../../tools/blender/RIG-SPEC.md) |
| AI mesh input | [PREP-AI-MESH-SPEC.md](../../tools/blender/scripts/PREP-AI-MESH-SPEC.md) |
| Weight gate | `tools/mesh_pipeline/weight_quality.py`, `TBG_WEIGHT_GATE=1` |
| Exception rigs | [rig-exception-workflow.md](rig-exception-workflow.md) |
| Runtime hitboxes | [physics-hitbox-spec.md](physics-hitbox-spec.md) (Phase 04 stub) |
| Free tools policy | `.local/phases/04b-biped-game-ready-pipeline/FREE-TOOLS-MATRIX.md` (local) |

---

## Quality gate status

Phase 04b **tooling is complete**; roster **pass rate is still open**:

| Criterion | Target | Status (2026-06-30) |
|-----------|--------|---------------------|
| Unified import CLI | `import_game_ready_unit.py` | **Done** |
| End-to-end on 3 AI GLBs | Pipeline runs; weight gate may fail | **Partial** — validated on `demo_scout_01`, `demo_warden_01`, `demo_shade_01` |
| Batch weight gate | ≥90% of `fixed_test_imports.json` | **Open** — 2/25 pass (8%) |
| `verify.sh` with weight gate | Green | **Open** — fails on weight + mesh size budget |
| Mesh island integrity | Largest island ≥75% | **Open** — decimated game meshes fragment (tracked separately) |

**Next work:** improve bone coverage (neck/head/arms) in `rig_weight_bind.py`, optional Mixamo batch for failures, decimation weld pass for island gate.

**Experimental:** [tools/experimental/auto_rig/](../../tools/experimental/auto_rig/) — standalone headless PoC (prep → template align → bind → export). Blender 5.1 headless `ARMATURE_AUTO` assigns zero weights; default `--bind-mode fallback` uses proximity weights.

Run audit: `python3 tools/mesh_pipeline/baseline_audit.py`

---

## Design principles

1. One skeleton per body class — shared clip library via retargeting  
2. Automate the hot path — batch first; manual only on `rigException`  
3. 100% free toolchain — Blender, Mixamo (fallback), gltf-transform, CMU mocap  
4. Physics ≠ mesh — bone colliders at runtime, not in GLB  
5. Dual-tier unchanged — showcase for art review, game GLB for Unity / Animation Studio  

---

**Updated:** 2026-06-30
