# tools/asset-factory/animations/

Named animation sets and bake/retarget conventions (Phase 02+).

Contracts:

- One named set per body class: `idle`, `walk`, `attack`, `attack2`,
  `death`, `spawn`, `ultimate` (names inherited from legacy behavior
  profiles in `archive/content/characters/behaviors/`).
- Sets target shared base skeletons, so parts reuse clips by construction.
- Timing values are presentation data, not balance canon.
