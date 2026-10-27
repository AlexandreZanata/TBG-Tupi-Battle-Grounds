# TBG Factions and Archetypes

> The identity layer of the roster. Names and roles are confirmed legacy;
> visual direction is adapted to the 2.0 low-poly GLB pipeline, and all
> gameplay bonuses are deferred to content phases.

## Factions (confirmed legacy names)

| Faction | Legacy visual language (reference, not mandate) |
|---------|--------------------------------------------------|
| `Humans` | Disciplined guard motifs — shields, plate, warm earthy palettes (cf. legacy `tank_human_01` "Iron Guard") |
| `Undead` | Dark, spectral motifs |
| `Machines` | Constructed, mechanical motifs |
| `Plants` | Organic, overgrown motifs |

Notes:

- Faction determines visual language and synergy grouping — never raw
  power by itself (BR-Y01, BR-U05).
- Detailed art direction (palettes, silhouettes, modular part kits) is a
  Phase 02/05 deliverable, produced through `tools/asset-factory/`.
- No legacy source fixes faction bonus values; none are stated here.

## Archetypes (confirmed legacy roles)

| Archetype | Battlefield role |
|-----------|------------------|
| `Tank` | Absorbs pressure, holds the line |
| `DPS` | Primary damage output |
| `Support` | Sustains or amplifies allies |
| `Ranged` | Deals damage from distance |
| `Control` | Disrupts enemy units and tempo |

Legacy behavior contracts (`archive/content/characters/behaviors/*.json`)
map each archetype to a presentation profile (e.g. `tank_melee_v1`,
`ranged_shot_v1`); those JSON timings and animation bindings are
**retired references**, useful as naming inspiration for 2.0 animation
sets but not as balance or timing canon.

## Roster composition

- Every unit is one `Faction` × one `Archetype` with a `SlotCount` of 1
  or 2 (BR-U02, BR-U03).
- Launch target: 12 unique units (`MVPUnitRoster`); 2.0 proposes shipping
  the first 8 as the playable-battle-loop milestone (see
  [OVERVIEW.md](OVERVIEW.md) scope decisions).
- Faction × archetype coverage matrix and the concrete 8-unit starter set
  are Phase 05 content decisions, not established here.

## Synergy grouping

- Default grouping is by `Faction`; defined cross-faction combos are
  allowed by BR-Y01 but none are established.
- Thresholds, bonus effects, and stacking rules: deferred to Phase 03
  simulation work with `tbg-core` data tables.
