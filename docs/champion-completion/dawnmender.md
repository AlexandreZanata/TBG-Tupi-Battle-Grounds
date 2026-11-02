# CC11 — Dawnmender

**Status: Proposed.** Owner-authorized champion completion; numbers are initial local-preview hypotheses, not measured balance.

Display names: **Dawnmender** / **Tecelã da Alvorada**. Celestials / Support.

Default skin: Held Sunrise / Alvorada nas Mãos.

## Identity and crowd space

Short bell-shaped healer coat, exposed round face and a circular lantern held with both hands; no wizard staff or head halo.

Lantern stays within torso/front volume; collar is narrow, coat hem stays above ankles.

Use the shared vertex-color material, cached per-joint meshes and existing rig slots. Sample actual posed bounds for all five forms; retain one-dimensional authoritative anchors and existing crowd limits. No visual overlap waiver or simulation-body resizing.

## Five forms

Base uses scale 1.00. Each branch has two steps, at 1.08 and 1.16. Every step changes a major material surface and a silhouette piece; final forms gain one restrained signature artifact.

- A1: **Calm Lantern** / **Lanterna Serena**, scale 1.08.
- A2: **Sanctuary Light** / **Luz do Santuário**, scale 1.16.
- B1: **Battle Beacon** / **Farol de Batalha**, scale 1.08.
- B2: **Dawn Chorus** / **Coro da Aurora**, scale 1.16.

## Props, shots and motion

### Weapon

One chest-height sunrise lantern; hands grip separate fixed rim sockets.

### Projectile

A compact warm light mote from the lantern; contextual heal bonus is a small bright lantern-rim pulse.

### Idle

Gently cradles lantern and breathes; shoulder collar counter-sways.

### Walk

Small steady steps; keeps the lantern level and turns the head toward the patient.

### Attack

Raises the lantern a little, opens its shutters, sends one light mote and closes them.

### Special

Front-pressure bonus is snapshotted at preparation; no permanent aura or unlimited enemy-count scaling.

### Victory

Two-step lantern cradle dance, gentle dip and a short sunrise presentation.

### Death

Kneels protecting the lantern, shutters close and the glow extinguishes last.

## Timing and verification

Attack preparation and impacts follow ART-DIRECTION section 4: ordinary melee contact at 0.30 s, projectile release at 0.18 s toward the authoritative impact, casts at 0.22 s. Cosmetic movement never chooses outcomes. Root/death cancel pending visual actions when the public state does. Victory stays eventless and bounded by the shared celebration resolver.

Required scoped evidence: five-form silhouette comparison against the 46 existing champions; actual per-form triangles/joints/materials and sampled animation bounds; unique idle/walk/attack/special/Hit/Death/Spawn/Evolve/Victory clips; both Evolution paths; actual ability regression; Linux preview capture. Windows and desktop web remain pending until observed.
