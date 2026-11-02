# CC13 — Leafcutter

**Status: Proposed.** Owner-authorized champion completion; numbers are initial local-preview hypotheses, not measured balance.

Display names: **Leafcutter** / **Saúva-Lâmina**. Insects / DPS.

Default skin: Serrated Mandible / Mandíbula Serrilhada.

## Identity and crowd space

Ant with bent antennae, narrow waist, two tool arms and four planted legs; one serrated blade, no Mantis scythe arms.

Additional feet tuck under the narrow thorax; blade is diagonal-forward rather than sideways.

Use the shared vertex-color material, cached per-joint meshes and existing rig slots. Sample actual posed bounds for all five forms; retain one-dimensional authoritative anchors and existing crowd limits. No visual overlap waiver or simulation-body resizing.

## Five forms

Base uses scale 1.00. Each branch has two steps, at 1.08 and 1.16. Every step changes a major material surface and a silhouette piece; final forms gain one restrained signature artifact.

- A1: **Keen Mandible** / **Mandíbula Afiada**, scale 1.08.
- A2: **Deep Serration** / **Serrilha Profunda**, scale 1.16.
- B1: **Swift Saw** / **Serra Veloz**, scale 1.08.
- B2: **Amber Reaper** / **Ceifadora de Âmbar**, scale 1.16.

## Props, shots and motion

### Weapon

One flat mandible sword, teeth are broad geometric notches rather than tiny saw noise.

### Projectile

None; compact serrated contact streak and existing bounded bleed indicator.

### Idle

Antenna alternation and a small mandible sword test with no full-body sway.

### Walk

Four-foot alternating ant gait; sword points along the lane and abdomen stays steady.

### Attack

Low single serrated pull-cut; no extra hits from the tooth animation.

### Special

Bleed uses the existing strongest refresh family and cleanup rules; no new venom stack.

### Victory

Four-foot marching shuffle, antenna salute and a diagonal blade tap.

### Death

Four legs tuck sequentially and sword rests on its flat side; antennae lower last.

## Timing and verification

Attack preparation and impacts follow ART-DIRECTION section 4: ordinary melee contact at 0.30 s, projectile release at 0.18 s toward the authoritative impact, casts at 0.22 s. Cosmetic movement never chooses outcomes. Root/death cancel pending visual actions when the public state does. Victory stays eventless and bounded by the shared celebration resolver.

Required scoped evidence: five-form silhouette comparison against the 46 existing champions; actual per-form triangles/joints/materials and sampled animation bounds; unique idle/walk/attack/special/Hit/Death/Spawn/Evolve/Victory clips; both Evolution paths; actual ability regression; Linux preview capture. Windows and desktop web remain pending until observed.
