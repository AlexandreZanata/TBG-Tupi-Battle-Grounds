# CC12 — Prismancer

**Status: Proposed.** Owner-authorized champion completion; numbers are initial local-preview hypotheses, not measured balance.

Display names: **Prismancer** / **Prismante**. Celestials / Ranged.

Default skin: True Alignment / Alinhamento Perfeito.

## Identity and crowd space

Angular short cape, geometric brow band and forearm-braced triangular prism; no tall staff, bow or halo.

Prism and cape are diagonal but close; beam is flat along the BattleLine rather than a screen-filling shaft.

Use the shared vertex-color material, cached per-joint meshes and existing rig slots. Sample actual posed bounds for all five forms; retain one-dimensional authoritative anchors and existing crowd limits. No visual overlap waiver or simulation-body resizing.

## Five forms

Base uses scale 1.00. Each branch has two steps, at 1.08 and 1.16. Every step changes a major material surface and a silhouette piece; final forms gain one restrained signature artifact.

- A1: **Keen Focus** / **Foco Preciso**, scale 1.08.
- A2: **Crystal Needle** / **Agulha de Cristal**, scale 1.16.
- B1: **Open Prism** / **Prisma Aberto**, scale 1.08.
- B2: **Triune Prism** / **Prisma Trino**, scale 1.16.

## Props, shots and motion

### Weapon

Gold fork and triangular crystal braced to right forearm; left fingertips adjust the focus.

### Projectile

Basic tiny azure shard; special uses one thin fixed segment warning followed by a single bounded beam.

### Idle

Turns the prism minutely and checks a short geometric sight line.

### Walk

Measured cross-steps, prism braced close to shoulder and cape kept short.

### Attack

Aligns fork, releases a single shard and restores the forearm guard.

### Special

Warning segment is fixed at cast start and remains fixed; beam resolves once without tracking targets.

### Victory

Traces a small triangle with the focus, shoulder sway and brief prism presentation.

### Death

Prism lowers, left hand catches the fork and body folds to the side; crystal dims intact.

## Timing and verification

Attack preparation and impacts follow ART-DIRECTION section 4: ordinary melee contact at 0.30 s, projectile release at 0.18 s toward the authoritative impact, casts at 0.22 s. Cosmetic movement never chooses outcomes. Root/death cancel pending visual actions when the public state does. Victory stays eventless and bounded by the shared celebration resolver.

Required scoped evidence: five-form silhouette comparison against the 46 existing champions; actual per-form triangles/joints/materials and sampled animation bounds; unique idle/walk/attack/special/Hit/Death/Spawn/Evolve/Victory clips; both Evolution paths; actual ability regression; Linux preview capture. Windows and desktop web remain pending until observed.
