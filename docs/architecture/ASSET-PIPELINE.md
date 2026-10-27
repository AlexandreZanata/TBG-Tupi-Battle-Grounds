# Asset Pipeline — TBG 2.0

> Planned production flow for game art. Directory contracts and quality
> gates only; no generators, rigs, or validators are implemented in
> Phase 00 (`tools/asset-factory/` holds placeholders and contracts).

## Production flow (proposed)

```
Codex (procedural scripts, versioned in tools/asset-factory/)
  → Blender, headless/offline (mesh assembly, rigging, animation baking)
  → GLB export (glTF 2.0)
  → Validation (validators/ gates; tbg-assets schemas)
  → Optimized assets (shared materials, capped geometry/textures)
  → Bevy client (tbg-client loads packaged GLBs)
```

Hard constraints:

- No generative-AI inference and no Blender services run during gameplay
  or on player devices. Generation is an offline, developer-side step.
- No asset pipeline component is part of the game server or client builds.
- Every shipped GLB passes validation before packaging; failures block
  the content commit, not the game build.

## Reusable-rig strategy (lesson from ADR-007)

The retired per-character pipeline (import AI GLB → attach bespoke rig →
weight-paint gate) measured ~8% automatic pass rates and 2–6 hours of
manual work per unit. It is not rebuilt. Instead:

- **Modular parts**: shared bases per body class + interchangeable parts
  (heads, limbs, accessories, faction kits).
- **Shared rigs**: rig once per base archetype (small set of skeletons),
  never per assembled character. Articulated rigid-part rigs preferred
  where skinning cost is unjustified.
- **Reusable animation sets**: one named set per body class
  (`idle`, `walk`, `attack`, `attack2`, `death`, `spawn`, `ultimate`),
  retargeted by construction since parts share the base skeleton.
- **Stylized low-poly look**: professional, readable at mobile-browser
  scale; shared materials and optimized geometry by default.

## Directory contracts (`tools/asset-factory/`)

| Directory | Owns | Phase 00 state |
|-----------|------|----------------|
| `generators/` | Versioned procedural generation scripts (parts, bodies, variants) | Placeholder + contract README |
| `rigs/` | Shared skeleton definitions and rig-once-per-base tooling | Placeholder + contract README |
| `animations/` | Named animation sets and retarget/bake conventions | Placeholder + contract README |
| `validators/` | GLB acceptance gates (schema, triangle/texture budgets, rig checks) | Placeholder + contract README; runtime schemas live in `tbg-assets` |

## Asset quality gates (proposed; enforced from Phase 02)

1. Valid glTF 2.0 / GLB container, Bevy-loadable.
2. Triangle count and texture resolution within per-class budgets
   (budgets set in Phase 02, recorded in `tbg-assets` schemas).
3. Skeleton is a known shared rig; no per-unit bespoke skeletons.
4. Named animation set complete for the unit's body class.
5. Shared materials only; no embedded duplicates.
6. License audit: every source texture, script output, and reference
   asset must have a recorded license compatible with MIT distribution.

## Content layout (`content/`)

| Directory | Owns |
|-----------|------|
| `content/units/` | Per-unit data files (Phase 03+; schema from `tbg-assets`) |
| `content/factions/` | Faction definitions and synergy tables (Phase 03+) |
| `content/balance/` | Tunable numbers, versioned separately from code (Phase 03+) |
| `content/assets/` | Packaged, validated GLBs + manifests (Phase 02+) |

Until Phase 02 defines schemas, these directories hold a `README.md`
contract and (where harmless) one minimal example file each.
