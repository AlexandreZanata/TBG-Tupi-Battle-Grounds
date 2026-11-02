# CC15 — Needlewing

**Status: Proposed.** Owner-authorized champion completion; numbers are initial local-preview hypotheses, not measured balance.

Display names: **Needlewing** / **Libélula Agulha**. Insects / Ranged.

Default skin: Still Sight / Mira Imóvel.

## Identity and crowd space

Dragonfly with long narrow abdomen, large focused eyes, four planted legs and four narrow folded wing planes.

Wing planes kept nearly vertical, folded rearward; aim line and projectile stay narrow.

Use the shared vertex-color material, cached per-joint meshes and existing rig slots. Sample actual posed bounds for all five forms; retain one-dimensional authoritative anchors and existing crowd limits. No visual overlap waiver or simulation-body resizing.

## Five forms

Base uses scale 1.00. Each branch has two steps, at 1.08 and 1.16. Every step changes a major material surface and a silhouette piece; final forms gain one restrained signature artifact.

- A1: **Keen Sight** / **Mira Precisa**, scale 1.08.
- A2: **Hunt Window** / **Janela de Caça**, scale 1.16.
- B1: **Needle Volley** / **Salva de Agulhas**, scale 1.08.
- B2: **Amber Guide** / **Guia de Âmbar**, scale 1.16.

## Props, shots and motion

### Weapon

Organic needle guide on the forearm, natural ivory needle; no rifle or metal scope.

### Projectile

Thin straight ivory needle with a tiny amber tip; charged version gets one short dark-amber tail.

### Idle

Eye/head tracking and a very small wing fold twitch; never hovers.

### Walk

Four-foot measured walking gait with rearward wing panels and stable needle guide.

### Attack

Braces the guide, releases the needle, then makes one precise reload motion.

### Special

Opportunity bonus checks the locked target windup at impact; it never interrupts or reads player intentions.

### Victory

Needle-guide salute, two paired wing clicks and a small four-foot step pattern.

### Death

Needle guide lowers, abdomen rests, wings fold together and head bows.

## Timing and verification

Attack preparation and impacts follow ART-DIRECTION section 4: ordinary melee contact at 0.30 s, projectile release at 0.18 s toward the authoritative impact, casts at 0.22 s. Cosmetic movement never chooses outcomes. Root/death cancel pending visual actions when the public state does. Victory stays eventless and bounded by the shared celebration resolver.

Required scoped evidence: five-form silhouette comparison against the 46 existing champions; actual per-form triangles/joints/materials and sampled animation bounds; unique idle/walk/attack/special/Hit/Death/Spawn/Evolve/Victory clips; both Evolution paths; actual ability regression; Linux preview capture. Windows and desktop web remain pending until observed.
