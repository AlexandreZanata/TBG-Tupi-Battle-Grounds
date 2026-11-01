# CC09 — Eclipse Dancer

**Status: Proposed.** Owner-authorized champion completion; numbers are initial local-preview hypotheses, not measured balance.

Display names: **Eclipse Dancer** / **Duelista do Eclipse**. Celestials / DPS.

Default skin: Divided Sky / Céu Dividido.

## Identity and crowd space

Compact athletic duelist, asymmetric short waist panels, split sun/moon halo and two strongly different curved blades.

Blades describe compact forearm arcs; no world-yaw spinning or wide skirt.

Use the shared vertex-color material, cached per-joint meshes and existing rig slots. Sample actual posed bounds for all five forms; retain one-dimensional authoritative anchors and existing crowd limits. No visual overlap waiver or simulation-body resizing.

## Five forms

Base uses scale 1.00. Each branch has two steps, at 1.08 and 1.16. Every step changes a major material surface and a silhouette piece; final forms gain one restrained signature artifact.

- A1: **Sun Step** / **Passo Solar**, scale 1.08.
- A2: **Zenith Blade** / **Lâmina do Zênite**, scale 1.16.
- B1: **Moon Step** / **Passo Lunar**, scale 1.08.
- B2: **Midnight Waltz** / **Valsa da Meia-noite**, scale 1.16.

## Props, shots and motion

### Weapon

Gold solar saber right, ivory lunar crescent left; both remain close to the torso in crowds.

### Projectile

None; gold or cool-ivory narrow contact crescent selected by the authoritative alternating counter.

### Idle

Alternating blade grip checks; half-halo and waist panels overlap softly.

### Walk

Cross-step fencing walk with no root spin and blades carried down at distinct angles.

### Attack

Right solar cut alternates with left lunar hook; one actual hit per basic cycle.

### Special

Lunar self-heal flash only follows actual unit HP damage, never shield damage or tower hits.

### Victory

Two compact crossing steps, sun blade up and moon blade down, then reverses the pose.

### Death

Blades cross protectively, knees fold and the half-halo settles close to the back.

## Timing and verification

Attack preparation and impacts follow ART-DIRECTION section 4: ordinary melee contact at 0.30 s, projectile release at 0.18 s toward the authoritative impact, casts at 0.22 s. Cosmetic movement never chooses outcomes. Root/death cancel pending visual actions when the public state does. Victory stays eventless and bounded by the shared celebration resolver.

Required scoped evidence: five-form silhouette comparison against the 46 existing champions; actual per-form triangles/joints/materials and sampled animation bounds; unique idle/walk/attack/special/Hit/Death/Spawn/Evolve/Victory clips; both Evolution paths; actual ability regression; Linux preview capture. Windows and desktop web remain pending until observed.
