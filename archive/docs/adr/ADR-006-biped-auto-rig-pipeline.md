# ADR-006: Biped Auto-Rig Pipeline (Phase 04b)

**Status:** Accepted  
**Date:** 2026-06-30  
**Deciders:** Product / engineering (solo dev)  
**Extends:** [ADR-003](ADR-003-real-3d-character-pipeline.md), [ADR-004](ADR-004-dual-tier-mesh-animation.md)

## Context

Phase 03 delivered dual-tier meshes and body-class clip retargeting for hand-tuned pilots. Scaling to 26+ seed units (and eventually hundreds) requires:

- A **default** path from AI/sculpt GLB → game GLB without per-unit Blender sessions
- **CI enforcement** of skin weight quality (not just triangle/bone budgets)
- A **free** fallback when proximity/auto weights fail (Mixamo auto-rig → `TBG_*` rename)
- A documented **exception** path for non-biped or organic silhouettes (`rigException: true`)

Baseline audit (Phase 04b §0) showed ~92% of units collapsed to 100% `TBG_Spine` dominance before the weight gate and auto-weight fallback.

## Decision

1. **Default rig profile:** `rigProfile: "tbg_biped_auto"` — attach body-class `TBG_*` template via `attach_tbg_rig.py` with proximity vs automatic weight fallback (`rig_weight_bind.py`).

2. **Unified orchestrator:** `import_game_ready_unit.py` wraps prep → showcase → export → weight gate. Batch integration via `batch_seed_to_game.sh` with per-unit weight summary table.

3. **Weight quality gate:** `validate_weight_quality()` — dominance <85%, no unweighted verts, every `TBG_*` bone has max weight >0.01. Enabled in CI via `TBG_WEIGHT_GATE=1` in `verify.sh`.

4. **Mixamo fallback (dev-only):** Free Adobe auto-rig when gate fails; `BONE_MAP_MIXAMO_TO_TBG.json` renames bones/weights. **Ship** body-class clip library only — not Mixamo mocap downloads.

5. **Exception path:** `rigException: true` blocks auto pipeline; manual checklist in [rig-exception-workflow.md](../design/rig-exception-workflow.md).

6. **Physics decoupled:** Runtime bone colliders spec'd in [physics-hitbox-spec.md](../design/physics-hitbox-spec.md); not embedded in GLB.

## Consequences

### Positive

- Single command and batch path for roster production
- Weight failures visible in CI instead of silent bad deformation
- Free toolchain preserved (no Auto Rig Pro)
- Clear escape hatch for organic/mechanical exceptions

### Negative / open

- **Roster pass rate still low** (2/25 at acceptance) — neck/head/arm coverage needs bind improvements or Mixamo batch
- **Mesh island integrity** on decimated game meshes regressed — separate fix from weight gate
- Mixamo dependency for worst-case rigs (acceptable dev-only per ADR-003 §2.4)

## References

- [biped-game-ready-pipeline.md](../design/biped-game-ready-pipeline.md)
- [tools/mesh-pipeline/README.md](../../tools/mesh-pipeline/README.md)
- Phase plan: `.local/phases/04b-biped-game-ready-pipeline/`
