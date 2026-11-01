# CC05 — Spinehurler

**Status: Proposed.** Owner-authorized champion completion; numbers are initial local-preview hypotheses, not measured balance.

Display names: **Spinehurler** / **Arremessa-Espinhos**. Monsters / Ranged.

Default skin: Five-Quill Hunter / Caçador das Cinco Cerdas.

## Identity and crowd space

Slim upright porcupine reptile with a vertical fan of five broad quills, round snout and upright throwing needle.

Quills are tall rather than wide; needle remains upright outside attack and no infinite line trails.

Use the shared vertex-color material, cached per-joint meshes and existing rig slots. Sample actual posed bounds for all five forms; retain one-dimensional authoritative anchors and existing crowd limits. No visual overlap waiver or simulation-body resizing.

## Five forms

Base uses scale 1.00. Each branch has two steps, at 1.08 and 1.16. Every step changes a major material surface and a silhouette piece; final forms gain one restrained signature artifact.

- A1: **Keen Needle** / **Agulha Precisa**, scale 1.08.
- A2: **Spine Lancer** / **Lanceiro de Espinhos**, scale 1.16.
- B1: **Barbed Fan** / **Leque Farpado**, scale 1.08.
- B2: **Needle Phalanx** / **Falange de Agulhas**, scale 1.16.

## Props, shots and motion

### Weapon

One bone needle in hand and a compact dorsal supply fan; no bow or gun.

### Projectile

Ivory straight needle with turquoise binding; empowered shot leaves a short narrow line to one secondary victim.

### Idle

Selects one quill and checks its straightness; head makes short alert turns.

### Walk

Light steps, vertical needle tucked and quill fan rigid over the back.

### Attack

Overhand needle throw with a matching nocked prop hide/reload, release before impact.

### Special

A second thin needle trace is shown only when the server reports a valid piercing secondary.

### Victory

Balances the needle vertically, lets the quill fan ripple once and catches it.

### Death

Needle lowers first, fan folds backward as the body sits and rolls slightly aside.

## Timing and verification

Attack preparation and impacts follow ART-DIRECTION section 4: ordinary melee contact at 0.30 s, projectile release at 0.18 s toward the authoritative impact, casts at 0.22 s. Cosmetic movement never chooses outcomes. Root/death cancel pending visual actions when the public state does. Victory stays eventless and bounded by the shared celebration resolver.

Required scoped evidence: five-form silhouette comparison against the 46 existing champions; actual per-form triangles/joints/materials and sampled animation bounds; unique idle/walk/attack/special/Hit/Death/Spawn/Evolve/Victory clips; both Evolution paths; actual ability regression; Linux preview capture. Windows and desktop web remain pending until observed.
