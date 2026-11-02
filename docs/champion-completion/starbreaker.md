# CC10 — Starbreaker

**Status: Proposed.** Owner-authorized champion completion; numbers are initial local-preview hypotheses, not measured balance.

Display names: **Starbreaker** / **Martelo Estelar**. Celestials / DPS.

Default skin: Caged Star / Estrela Enjaulada.

## Identity and crowd space

Lean connected ivory construct with one star inset face and huge two-handed crystal hammer; no skull or bulky Justiciar fists.

Large prop stays vertical in rest and follows a short overhead arc; body remains narrow.

Use the shared vertex-color material, cached per-joint meshes and existing rig slots. Sample actual posed bounds for all five forms; retain one-dimensional authoritative anchors and existing crowd limits. No visual overlap waiver or simulation-body resizing.

## Five forms

Base uses scale 1.00. Each branch has two steps, at 1.08 and 1.16. Every step changes a major material surface and a silhouette piece; final forms gain one restrained signature artifact.

- A1: **Starmaul** / **Marreta Estelar**, scale 1.08.
- A2: **Constellation Breaker** / **Quebra-constelações**, scale 1.16.
- B1: **Hammer Dance** / **Dança do Martelo**, scale 1.08.
- B2: **Nova Forge** / **Forja da Nova**, scale 1.16.

## Props, shots and motion

### Weapon

Faceted star-cage hammer; heavy head carried above/along the forward axis.

### Projectile

None; short gold/blue planar hammer impact; no meteor or persistent terrain effect.

### Idle

Checks the hammer balance, exposed joints make tiny rigid settling motions.

### Walk

Grounded deliberate stride with visibly counterweighted hammer; not the heavy Tank march.

### Attack

Clear two-hand overhead anticipation, contact on Hit, visible recovery; no invented extra damage frames.

### Special

A restrained star pressure glyph appears on eligible high-base-HP enemy hits.

### Victory

Hammer rests vertically; two alternating heel lifts and one proud overhead half-raise.

### Death

Hammer head touches ground, shoulders fold around shaft and star face dims.

## Timing and verification

Attack preparation and impacts follow ART-DIRECTION section 4: ordinary melee contact at 0.30 s, projectile release at 0.18 s toward the authoritative impact, casts at 0.22 s. Cosmetic movement never chooses outcomes. Root/death cancel pending visual actions when the public state does. Victory stays eventless and bounded by the shared celebration resolver.

Required scoped evidence: five-form silhouette comparison against the 46 existing champions; actual per-form triangles/joints/materials and sampled animation bounds; unique idle/walk/attack/special/Hit/Death/Spawn/Evolve/Victory clips; both Evolution paths; actual ability regression; Linux preview capture. Windows and desktop web remain pending until observed.
