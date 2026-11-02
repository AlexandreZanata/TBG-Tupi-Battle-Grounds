# CC01 — Gravecutter

**Status: Proposed.** Owner-authorized champion completion; numbers are initial local-preview hypotheses, not measured balance.

Display names: **Gravecutter** / **Ceifador de Correntes**. Undead / DPS.

Default skin: Barrow Duelist / Duelista do Túmulo.

## Identity and crowd space

Exposed skull, narrow split fencing jacket, curved forearm blade and compact chain; no hood or staff.

Tall narrow crescent profile; chain does not contribute to idle lateral radius; jacket is above the feet.

Use the shared vertex-color material, cached per-joint meshes and existing rig slots. Sample actual posed bounds for all five forms; retain one-dimensional authoritative anchors and existing crowd limits. No visual overlap waiver or simulation-body resizing.

## Five forms

Base uses scale 1.00. Each branch has two steps, at 1.08 and 1.16. Every step changes a major material surface and a silhouette piece; final forms gain one restrained signature artifact.

- A1: **Gravetaker** / **Coveiro**, scale 1.08.
- A2: **Final Harvester** / **Ceifador Final**, scale 1.16.
- B1: **Chain Dancer** / **Dançarino de Correntes**, scale 1.08.
- B2: **Soul Reaver** / **Ceifador de Almas**, scale 1.16.

## Props, shots and motion

### Weapon

Short articulated chain crescent; chain stays coiled near the forearm between attacks.

### Projectile

None; one small crescent contact streak, never a simulated ranged attack.

### Idle

Checks the wrist chain and tilts the bare skull; jacket tails counter-sway.

### Walk

Quick heel-to-toe fencing steps, chain wrist tucked; no sliding or long swinging chain.

### Attack

Coils wrist, diagonal hook at the authoritative hit, then reels the blade back close.

### Special

A short green crescent tightens at the blade when the target qualifies for the finisher.

### Victory

Two heel taps, wrist-chain twirl held vertically, blade-over-heart bow.

### Death

Knees fold in sequence; chain slackens, blade settles beside the body; soul clasp dims.

## Timing and verification

Attack preparation and impacts follow ART-DIRECTION section 4: ordinary melee contact at 0.30 s, projectile release at 0.18 s toward the authoritative impact, casts at 0.22 s. Cosmetic movement never chooses outcomes. Root/death cancel pending visual actions when the public state does. Victory stays eventless and bounded by the shared celebration resolver.

Required scoped evidence: five-form silhouette comparison against the 46 existing champions; actual per-form triangles/joints/materials and sampled animation bounds; unique idle/walk/attack/special/Hit/Death/Spawn/Evolve/Victory clips; both Evolution paths; actual ability regression; Linux preview capture. Windows and desktop web remain pending until observed.
