# CC16 — Nectar Nurse

**Status: Proposed.** Owner-authorized champion completion; numbers are initial local-preview hypotheses, not measured balance.

Display names: **Nectar Nurse** / **Ama de Néctar**. Insects / Support.

Default skin: Honey Thread / Fio de Mel.

## Identity and crowd space

Rounded bee medic, short apron, two compact upright wings, honey bowl and short curved dipper; no queen crown.

Round abdomen is compact, wings upright; thread is a thin line rather than a wide healing cone.

Use the shared vertex-color material, cached per-joint meshes and existing rig slots. Sample actual posed bounds for all five forms; retain one-dimensional authoritative anchors and existing crowd limits. No visual overlap waiver or simulation-body resizing.

## Five forms

Base uses scale 1.00. Each branch has two steps, at 1.08 and 1.16. Every step changes a major material surface and a silhouette piece; final forms gain one restrained signature artifact.

- A1: **Honey Keeper** / **Guardiã do Mel**, scale 1.08.
- A2: **Golden Remedy** / **Remédio Dourado**, scale 1.16.
- B1: **Silken Nectar** / **Néctar Sedoso**, scale 1.08.
- B2: **Hive Lifeline** / **Fio Vital da Colmeia**, scale 1.16.

## Props, shots and motion

### Weapon

Honey bowl in left hand, curved dipper in right; one controlled nectar thread.

### Projectile

No ballistic shot; one thin amber thread joins current patient only while a real channel is active.

### Idle

Stirs once and checks the honey thread; antennae tilt asymmetrically.

### Walk

Small four-foot rocking walk, honey bowl stable and wing panels tucked.

### Attack

Starts the bounded heal channel instead of performing another normal heal cycle.

### Special

Patient remains locked for the channel; pulses stop on death, incapacity or loss of range.

### Victory

Honey-dipper stir, bowl-balanced heel taps and a little wing flutter close to the back.

### Death

Dipper rests across bowl, legs fold and channel thread snaps immediately.

## Timing and verification

Attack preparation and impacts follow ART-DIRECTION section 4: ordinary melee contact at 0.30 s, projectile release at 0.18 s toward the authoritative impact, casts at 0.22 s. Cosmetic movement never chooses outcomes. Root/death cancel pending visual actions when the public state does. Victory stays eventless and bounded by the shared celebration resolver.

Required scoped evidence: five-form silhouette comparison against the 46 existing champions; actual per-form triangles/joints/materials and sampled animation bounds; unique idle/walk/attack/special/Hit/Death/Spawn/Evolve/Victory clips; both Evolution paths; actual ability regression; Linux preview capture. Windows and desktop web remain pending until observed.
