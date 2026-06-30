# ADR-004: Dual-Tier Mesh and Animation Pipeline

**Status:** Accepted  
**Date:** 2026-06-30  
**Deciders:** Product / engineering (solo dev)  
**Supersedes:** Partial extension of [ADR-003](ADR-003-real-3d-character-pipeline.md)

## Context

Phase 02 established real 3D units: authored sculpt GLBs (~1.5M tris, static, 35–77 MB) for web showcase and C# spawn via `Mesh3DCharacterFactory`. Procedural browser animations (whole-mesh bob) were acceptable for art review but not for in-game combat presentation.

TBG is a **single-lane** auto battler. Units need:

- **Lane march** — looping `Walk` (in-place cycle; position driven by gameplay)
- **Repeated attacks** — `Attack` / `Attack2` with optional round-robin (`combat.attackCycle`)
- **Ultimates** — one-shot `Ultimate` blocking other actions until `ultimateDurationMs`
- **Mobile budgets** — ≤ 8k tris, ≤ 64 bones, ≤ 2 MB target GLB per unit

You cannot animate the high-poly sculpt in place. Retopo, rig attach, clip retarget, and validation must be automated and CI-gated.

## Decision

Adopt a **dual-tier asset model** per unit id:

| Tier | Path | Purpose | Shipped in mobile build |
|------|------|---------|-------------------------|
| **Showcase** | `meshes/showcase/{id}.glb` | Carousel, Character Studio art review | No |
| **Game** | `meshes/game/{id}.glb` | Unity runtime + Animation Studio game preview | Yes |

Definition JSON fields:

```json
{
  "meshShowcase": "content/characters/meshes/showcase/demo_sentinel_01.glb",
  "mesh": "content/characters/meshes/game/demo_sentinel_01.glb",
  "animationSet": "humanoid_heavy_combat_v1",
  "gameLod": { "triTarget": 8000, "textureSize": 1024 }
}
```

**Shared clip library per body class** (`HumanoidLight`, `HumanoidHeavy`):

- Base clips authored once → `content/characters/animations/clips/{bodyClass}/`
- Animation set manifest maps action keys → clip names → `content/characters/animation_sets/{id}.json`
- Per-unit export retargets library clips onto the unit skeleton and embeds them in the game GLB

**Automated export pipeline** (Blender headless, GPL external only):

1. Decimate showcase → low mesh (`sculpt_to_game_mesh.py`)
2. Attach `TBG_*` rig + auto weights (`attach_tbg_rig.py`)
3. Retarget clips (`retarget_clip_library.py`)
4. Post-process: texture resize, ORM pack, Draco (`export_game_unit.py`)
5. Validate budgets (`validate_game_glb.py` in `./scripts/verify.sh`)

**Runtime (C#):**

- `Mesh3DCharacterFactory` loads **game** `mesh`, not showcase
- `AnimationSetLoader` resolves clip names from manifest; clips embedded in GLB at import
- `BehaviorPlayer` v2: `EnterWalk` / `ExitWalk`, `PlayAttackRepeat`, `PlayUltimate`, `OnClipComplete`
- `CharacterLodGroup`: LOD0 game mesh, LOD1 static mesh in `meshes/game/lod1/`

**Web Animation Studio:** toggle Showcase vs Game tier; game tier plays real skeletal clips when present.

**Behavior timings** (`ultimateWindupMs`, `attackHitMs`, …) remain in behavior JSON — server sync deferred to Phase 04.

## Consequences

### Positive

- Same unit id serves art review and mobile runtime without duplicate JSON
- One clip library scales to all units sharing a body class
- CI enforces mobile budgets before merge
- Showcase carousel unchanged — still high-fidelity sculpts

### Negative

- Two GLB files per unit increases repo size (mitigated: game tier ≤ 2 MB target)
- Blender required for production-quality weights; Python fallback is dev/CI stub only
- `clipSource: library` merge at Editor import time not yet implemented — MVP embeds clips in game GLB

### Non-goals (this ADR)

- Open-world locomotion blend trees
- Per-unit bespoke mocap for all heroes in MVP
- Nakama sync of animation hit frames (Phase 04)
- Git LFS policy for showcase tier (deferred)

## Alternatives considered

| Option | Rejected because |
|--------|------------------|
| Single mesh tier (decimate sculpt only) | No skeleton; cannot retarget shared clips |
| Runtime retarget from showcase GLB | 1.5M tris exceeds mobile budget |
| Unique skeleton per hero | Art and engineering cost; shared `TBG_*` rig sufficient for MVP |
| External clip-only GLBs at runtime | Deferred; embedded clips simpler for glTFast MVP |

## Follow-up

- Phase 04: Nakama authoritative sync of behavior timings and hit frames
- Investigate KTX2 / BasisU when Unity glTFast path supports it
- Optional: deduplicate clip bytes via shared clip-only GLBs + `BodyClassClipRegistry`

## References

- [tools/mesh-pipeline/README.md](../../tools/mesh-pipeline/README.md) — workflow diagram
- [tools/mesh-pipeline/BUDGETS.md](../../tools/mesh-pipeline/BUDGETS.md) — numeric limits
- [com.tbg.character-library README](../../unity/Packages/com.tbg.character-library/README.md) — runtime API
- Phase 03 tasks: `.local/phases/03-game-animation-studio/`
