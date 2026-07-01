# Deferred movement roadmap (post Phase 03b gate)

> **Not implemented.** Formal deferral per [ADR-005](../../docs/adr/ADR-005-deferred-movement-features.md).  
> Agents: do **not** start these without a phase TASKS.md entry.

## Index

| ID | Feature | Phase | Owner surface |
|----|---------|-------|---------------|
| [D1](#d1--full-body-ik-vrik-style) | Full-body IK | 06+ | `unity/.../Locomotion/` |
| [D2](#d2--motion-matching-database) | Motion matching DB | 06+ | `tools/blender/animation/motion_match/` (future) |
| [D3](#d3--ml-locomotion-controller) | ML locomotion | Post-MVP | `tools/ml/locomotion/` (future) |
| [D4](#d4--quadruped--mechanical-gait) | Non-humanoid gait | 06 | `walk/profiles.py`, clip library |
| [D5](#d5--nakama-move-speed-blend) | Server speed blend | 05 | Nakama module + `BehaviorPlayer` |

## Current MVP substitutes

| Need | Shipped in 03b |
|------|----------------|
| Foot plant on lane | `LocomotionFootIk` + `locomotion.footIk` |
| Speed variation | `walk/velocity.py` → `scale_walk_style_for_speed()` |
| Forward travel | In-place `Walk` + gameplay position, or `WalkRoot` |
| Attack variety | Procedural thrust + swing samplers |

---

## D1 — Full-body IK (VRIK-style)

**Problem:** Two-bone foot IK does not orient pelvis/spine on slopes or stairs.

**Prerequisites:** D1 optional; flat lane MVP complete; Phase 04 camera/lane scene.

**Planned approach:**

1. Evaluate Final IK / Animation Rigging VRIK vs custom spine chain (3–5 bones)
2. `LocomotionFullBodyIk.cs` behind `locomotion.fullBodyIk: false`
3. Reuse `TwoBoneIk` for legs; add spine lean from ground normal

**Acceptance:** No foot slide on 10° ramp; spine pitch within ±20° of terrain normal.

---

## D2 — Motion matching database

**Problem:** Procedural walk is uniform; heroes may need clip-like variety without per-unit mocap.

**Prerequisites:** Shared clip library stable; D2 index built from existing `Walk`/`WalkRoot` GLBs + CMU retarget extras.

**Planned approach:**

1. Package `motion_match/` — pose features, phase tags, KD-tree or flat index
2. Runtime search in Unity (C#) or bake-time clip selection in Blender
3. Fallback: `humanoid_*_v1` procedural profile

**Acceptance:** Index query &lt; 2 ms; visual match ≥ subjective parity with source clip on pilot unit.

---

## D3 — ML locomotion controller

**Problem:** Hand-tuned Perry curves do not cover all speeds/terrains.

**Prerequisites:** D2 or CMU dataset expanded; Python training env documented.

**Planned approach:**

1. Study [AI4AnimationPy](https://github.com/facebookresearch/ai4animationpy) FK + export pipeline
2. Train on retargeted `TBG_*` sequences only — no new skeleton
3. Export curve tables or ONNX → consumed by Application layer adapter (not Domain)

**Acceptance:** Trained output passes `tests/blender/walk/` loop + axis contracts; human sign-off in Animation Studio.

---

## D4 — Quadruped / Mechanical gait

**Problem:** `Organic` and `Mechanical` body classes use humanoid walk placeholders.

**Prerequisites:** `RIG-SPEC.md` extended for non-biped bone axes; pilot unit per class.

**Planned approach:**

1. `organic_quadruped_v1` / `mechanical_tread_v1` in `walk/profiles.py`
2. Separate sampler or `segment_gait` with quadruped phase offsets (4-beat)
3. Rebake `content/characters/animations/clips/Organic/Walk.glb`, `Mechanical/Walk.glb`

**Acceptance:** Contract tests for leg axis + loop closure per body class.

---

## D5 — Nakama move speed blend

**Problem:** Lane units move at gameplay speed; walk cadence should match without desync.

**Prerequisites:** Phase 05 match module publishes unit kinematics; Phase 04 client applies position.

**Planned approach:**

1. Nakama `UnitState.moveSpeedMps` authoritative field
2. Client: `BehaviorPlayer.ApplyMoveSpeed(mps)` → `Animator.SetFloat("WalkSpeed", …)` or sample scale from `velocity.py` port in C#
3. Optional: server validates hit frames still use `behavior.timings` (unchanged)

**Acceptance:** Integration test — server fixture 1.0 m/s vs 2.0 m/s → cadence ratio ≈ √2 within tolerance.

---

## Agent protocol

1. Confirm target phase TASKS.md schedules the ID.
2. Read ADR-005 acceptance criteria before coding.
3. Add contract tests first; never weaken 03b gait/attack contracts.
4. Update this file when an item moves from deferred → active.
