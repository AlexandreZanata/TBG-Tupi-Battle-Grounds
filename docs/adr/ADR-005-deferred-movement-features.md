# ADR-005: Deferred Movement Features (Post Phase 03b Gate)

**Status:** Accepted (deferred — not in MVP scope)  
**Date:** 2026-06-30  
**Deciders:** Product / engineering (solo dev)  
**Extends:** [MOVEMENT-MODULE.md](../../tools/blender/animation/MOVEMENT-MODULE.md), Phase 03b §7

## Context

Phase 03b delivered a **code-first movement module**: mocap-calibrated segment gait, root-motion `WalkRoot`, kinetic-chain attacks, lightweight foot IK, and velocity scaling contracts. The lane MVP does **not** require slopes, motion matching, ML controllers, non-humanoid gaits, or server-driven animation blend.

Building these now would expand scope beyond the single-lane auto-battler gate and duplicate work planned for Phases 04–06.

## Decision

**Defer** the following to post-gate phases. Each item has a target phase, prerequisites, and acceptance criteria — see [DEFERRED-ROADMAP.md](../../tools/blender/animation/DEFERRED-ROADMAP.md).

| ID | Feature | Target phase | Rationale |
|----|---------|--------------|-----------|
| D1 | Full-body IK (VRIK-style) | 06+ | Lane is flat; two-bone foot IK sufficient for MVP |
| D2 | Motion matching clip DB | 06+ | Procedural + shared library covers 12 MVP units |
| D3 | ML locomotion controller | Post-MVP | Requires dataset + training infra; study-only refs in phase README |
| D4 | Quadruped / Mechanical gait | 06 | `Organic` / `Mechanical` body classes exist; humanoid gate first |
| D5 | Nakama move speed → animation blend | 05 | Authoritative speed lives on server; client stub exists (`velocity.py`) |

**Do not implement** D1–D5 until the target phase TASKS.md explicitly schedules them.

## Prerequisites (shared)

- Phase 03b gate passed (`MOVEMENT-MODULE.md`, `tests/blender/` green)
- Phase 04: Unity lane controller + `BehaviorPlayer` root-motion sync
- Phase 05: Nakama match tick + unit state replication

## Per-item acceptance (summary)

### D1 — Full-body IK

- Spine/pelvis adjust to ground normal on slopes ≤ 15°
- Stairs: foot IK + pelvis offset without foot slide
- Feature flag `locomotion.fullBodyIk` default `false`
- Does not replace FK `TBG_*` bone axis contract

### D2 — Motion matching

- Indexed clip database per body class (pose + velocity + phase)
- Search: nearest neighbor on lower-body features @ 30 Hz
- Fallback to procedural `segment_gait` when no match within threshold
- CI: index build reproducible from `content/characters/animations/clips/`

### D3 — ML locomotion

- Offline training only (Python); no PyTorch in Unity runtime
- Export: ONNX or baked curve tables consumed by `walk/sampler.py`
- Human-in-the-loop: no auto-commit of ML weights without visual review

### D4 — Quadruped / Mechanical gait

- New `WalkStyle` presets in `walk/profiles.py` for `Organic` and `Mechanical`
- Bone axis contract documented per body class in `RIG-SPEC.md`
- Rebake `clips/Organic/Walk.glb` and `clips/Mechanical/Walk.glb`

### D5 — Nakama-synced animation blend

- Server publishes `moveSpeedMps` per unit each tick
- Client: `scale_walk_style_for_speed()` or animator `Speed` param
- Desync tolerance: ±1 frame vs `behavior.timings` unchanged
- Contract test: server fixture → expected cadence/stride scale

## Consequences

### Positive

- Phase 03b closes without scope creep
- Future agents have explicit entry criteria per feature
- `velocity.py` and `LocomotionFootIk` remain the MVP runtime surface

### Negative

- Sloped arenas not supported until D1
- Walk variety limited to procedural presets until D2/D3

## Alternatives considered

| Option | Rejected because |
|--------|------------------|
| Implement VRIK in 03b | Overkill for flat lane; foot IK already shipped |
| Start motion matching now | No gameplay need for clip variety in MVP roster |
| Ship ML walk in Blender | Training loop not owned; mocap calibration sufficient |

## References

- Phase 03b open-source table: `.local/phases/03b-biomechanical-movement-module/README.md`
- [AI4Animation](https://github.com/sebastianstarke/AI4Animation) / [AI4AnimationPy](https://github.com/facebookresearch/ai4animationpy) — study patterns for D2/D3
- [ossos](https://github.com/sketchpunklabs/ossos) — IK/retarget reference for D1
