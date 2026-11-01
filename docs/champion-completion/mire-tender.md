# CC06 — Mire Tender

**Status: Proposed.** Owner-authorized champion completion; numbers are initial local-preview hypotheses, not measured balance.

Display names: **Mire Tender** / **Curandeiro do Brejo**. Monsters / Support.

Default skin: Reed Broth / Caldo de Junco.

## Identity and crowd space

Squat toad with broad triangular head, shallow stone bowl and vertical spoon staff; no plant humanoid or mushrooms.

Wide low head but short limbs; bowl is front-centered and reed staff is vertical, not a large lateral branch.

Use the shared vertex-color material, cached per-joint meshes and existing rig slots. Sample actual posed bounds for all five forms; retain one-dimensional authoritative anchors and existing crowd limits. No visual overlap waiver or simulation-body resizing.

## Five forms

Base uses scale 1.00. Each branch has two steps, at 1.08 and 1.16. Every step changes a major material surface and a silhouette piece; final forms gain one restrained signature artifact.

- A1: **Brothkeeper** / **Guardião do Caldo**, scale 1.08.
- A2: **Deep Remedy** / **Remédio Profundo**, scale 1.16.
- B1: **Reed Tender** / **Zelador do Junco**, scale 1.08.
- B2: **Mire Sage** / **Sábio do Brejo**, scale 1.16.

## Props, shots and motion

### Weapon

Spoon-shaped reed and stone broth bowl; tools stay clear of the face.

### Projectile

Single turquoise medicinal droplet; quiet regeneration is a small reed-shaped glyph, no large healing fog.

### Idle

Slow throat breath, gentle bowl tilt and one wrist stir.

### Walk

Short webbed-foot steps, bowl stabilized and spoon following the stride.

### Attack

Scoops broth, tips the spoon and releases one droplet toward the patient.

### Special

Regeneration glyph disappears on actual HP damage; protected hits do not invent damage.

### Victory

Bowl-balanced two-step, three spoon stirs and a pleased throat bob.

### Death

Protects the bowl, sits down, spoon slides to the ground and eyelids lower.

## Timing and verification

Attack preparation and impacts follow ART-DIRECTION section 4: ordinary melee contact at 0.30 s, projectile release at 0.18 s toward the authoritative impact, casts at 0.22 s. Cosmetic movement never chooses outcomes. Root/death cancel pending visual actions when the public state does. Victory stays eventless and bounded by the shared celebration resolver.

Required scoped evidence: five-form silhouette comparison against the 46 existing champions; actual per-form triangles/joints/materials and sampled animation bounds; unique idle/walk/attack/special/Hit/Death/Spawn/Evolve/Victory clips; both Evolution paths; actual ability regression; Linux preview capture. Windows and desktop web remain pending until observed.
