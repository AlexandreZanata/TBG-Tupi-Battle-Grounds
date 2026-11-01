# CC03 — Cragclaw

**Status: Proposed.** Owner-authorized champion completion; numbers are initial local-preview hypotheses, not measured balance.

Display names: **Cragclaw** / **Garra-de-Basalto**. Monsters / DPS.

Default skin: Obsidian Prow / Proa de Obsidiana.

## Identity and crowd space

Lean wedge-shaped lizard, small hornless head, one broad chisel forearm and one tiny balancing arm.

Claw carried along forward lane axis; attack arc is local and not an oversized horizontal disk.

Use the shared vertex-color material, cached per-joint meshes and existing rig slots. Sample actual posed bounds for all five forms; retain one-dimensional authoritative anchors and existing crowd limits. No visual overlap waiver or simulation-body resizing.

## Five forms

Base uses scale 1.00. Each branch has two steps, at 1.08 and 1.16. Every step changes a major material surface and a silhouette piece; final forms gain one restrained signature artifact.

- A1: **Edgeclaw** / **Garra Afiada**, scale 1.08.
- A2: **Obsidian Chisel** / **Cinzel de Obsidiana**, scale 1.16.
- B1: **Splitclaw** / **Garra Fendida**, scale 1.08.
- B2: **Shardsweeper** / **Varredor de Estilhaços**, scale 1.16.

## Props, shots and motion

### Weapon

One asymmetric slab claw; broad planar cutting edge rather than two identical weapons.

### Projectile

None; short flat turquoise shard fan at melee contact, capped and immediately fading.

### Idle

Lowers the small shoulder, flexes the single heavy claw and balances with the tail.

### Walk

Uneven quick stride with a counter-rotating small arm; weight never reads as a Tank.

### Attack

Low compact slash; empowered sweep has visible shoulder coil and a longer recovery.

### Special

A shallow wedge on the ground marks the local sweep; no displacement or stun.

### Victory

Three claw-tip ground taps followed by a compact shoulder shimmy.

### Death

Heavy claw touches first; narrow body folds behind it and head rests on its edge.

## Timing and verification

Attack preparation and impacts follow ART-DIRECTION section 4: ordinary melee contact at 0.30 s, projectile release at 0.18 s toward the authoritative impact, casts at 0.22 s. Cosmetic movement never chooses outcomes. Root/death cancel pending visual actions when the public state does. Victory stays eventless and bounded by the shared celebration resolver.

Required scoped evidence: five-form silhouette comparison against the 46 existing champions; actual per-form triangles/joints/materials and sampled animation bounds; unique idle/walk/attack/special/Hit/Death/Spawn/Evolve/Victory clips; both Evolution paths; actual ability regression; Linux preview capture. Windows and desktop web remain pending until observed.
