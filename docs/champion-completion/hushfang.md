# CC04 — Hushfang

**Status: Proposed.** Owner-authorized champion completion; numbers are initial local-preview hypotheses, not measured balance.

Display names: **Hushfang** / **Presa Silenciosa**. Monsters / DPS.

Default skin: Quiet Hunt / Caçada Silenciosa.

## Identity and crowd space

Low long-backed four-legged panther-lizard, broad saber fangs and swept ears; never a humanoid werewolf.

Longitudinal footprint with close-set feet; tail bends along the body axis instead of sweeping lateral lanes.

Use the shared vertex-color material, cached per-joint meshes and existing rig slots. Sample actual posed bounds for all five forms; retain one-dimensional authoritative anchors and existing crowd limits. No visual overlap waiver or simulation-body resizing.

## Five forms

Base uses scale 1.00. Each branch has two steps, at 1.08 and 1.16. Every step changes a major material surface and a silhouette piece; final forms gain one restrained signature artifact.

- A1: **Silent Prowler** / **Espreitador Silencioso**, scale 1.08.
- A2: **Fang Warden** / **Guardião das Presas**, scale 1.16.
- B1: **Flint Hunter** / **Caçador de Sílex**, scale 1.08.
- B2: **Midnight Pounce** / **Bote da Meia-noite**, scale 1.16.

## Props, shots and motion

### Weapon

Jaw and paired saber fangs; tail is a rearward visual counterweight.

### Projectile

None; one small fang flash on the charged first melee hit.

### Idle

Alternating forepaw knead, small ear turns and slow tail coil.

### Walk

Diagonal feline four-beat gait, head low; charge changes posture without added movement.

### Attack

Draws the head back and bites at normal melee reach; no jump across targets.

### Special

Fang bases acquire a restrained turquoise readiness glint, removed immediately by qualifying control.

### Victory

Two front-paw steps, a grounded back stretch and a small head shake.

### Death

Forelegs bend, chest settles, rear folds afterward; tail follows without a ragdoll.

## Timing and verification

Attack preparation and impacts follow ART-DIRECTION section 4: ordinary melee contact at 0.30 s, projectile release at 0.18 s toward the authoritative impact, casts at 0.22 s. Cosmetic movement never chooses outcomes. Root/death cancel pending visual actions when the public state does. Victory stays eventless and bounded by the shared celebration resolver.

Required scoped evidence: five-form silhouette comparison against the 46 existing champions; actual per-form triangles/joints/materials and sampled animation bounds; unique idle/walk/attack/special/Hit/Death/Spawn/Evolve/Victory clips; both Evolution paths; actual ability regression; Linux preview capture. Windows and desktop web remain pending until observed.
