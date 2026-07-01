# ADR-007: Modular Character Assembler (pipeline pivot)

**Status:** Accepted  
**Date:** 2026-07-01  
**Deciders:** Product / character pipeline  
**Supersedes:** Extends [ADR-003](ADR-003-real-3d-character-pipeline.md), deprecates active work on [ADR-006](ADR-006-biped-auto-rig-pipeline.md) seed path

---

## Context

Phases 02–04b built a **per-unit mesh3d pipeline**: import AI GLB → attach TBG rig → weight paint gate → retarget clips → export game GLB.

Quality gate for Phase 04b: ≥90% of 26 seed units pass weight gate without manual paint. **Actual: ~8% (2/25).** Production time per unit remains 2–6 hours — incompatible with MVP roster velocity.

Industry pattern for mobile games at scale: **modular character kits** (body base + interchangeable parts + shared skeleton), not bespoke rigging per hero.

---

## Decision

Pivot to **Modular Character Assembler** (Phase 05):

1. **Library-first content** under `content/characters/library/` (bases, parts, materials, rigs, exports)
2. **MB-Lab** as default base body authoring tool (free, in-Blender, rig included)
3. **Rig once per base archetype** (male, female, small, large) — never per assembled character
4. **Web Assembly Studio** as orchestration UI above headless Blender
5. **Archive** seed GLB demo roster to `archive/content/characters/seed-glb-pipeline/`

---

## Consequences

### Positive

- Target **&lt;30 min** per new character (select + export)
- Reuses Phase 03 clip library and Unity CharacterLibrary
- Artists author reusable parts, not one-off rigs
- Free toolchain (Blender + MB-Lab)

### Negative

- Less silhouette freedom vs fully custom AI sculpts
- Initial investment: 4 bases + part library before first export
- Phase 04b tooling becomes legacy maintenance

### Neutral

- Procedural Phase 01 JSON units unchanged
- Dual-tier showcase/game concept may simplify (assembled export is game-ready)

---

## Alternatives considered

| Option | Rejected because |
|--------|------------------|
| Continue Phase 04b automation | Weight paint remains bottleneck; 8% pass rate |
| Paid auto-rig plugins | Violates free-toolchain rule |
| Pure procedural (no mesh3d) | Art direction requires readable 3D units |
| External SaaS character creator | Vendor lock-in; no Blender control |

---

## Implementation

- Spec: [modular-character-assembler.md](../design/modular-character-assembler.md)
- Tasks: `.local/phases/05-modular-character-assembler/TASKS.md`
- Web UI: `assembly-studio.html`
- Blender: `tools/assembly_studio/blender/`
