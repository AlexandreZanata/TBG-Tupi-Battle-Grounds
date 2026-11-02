# CC08 — Sunlancer

**Status: Proposed.** Owner-authorized champion completion; numbers are initial local-preview hypotheses, not measured balance.

Display names: **Sunlancer** / **Lanceiro da Aurora**. Celestials / DPS.

Default skin: First Light / Primeira Luz.

## Identity and crowd space

Light spear fencer with triangular chest harness and narrow open sun crest; no shield or winged archer silhouette.

Long spear aligns with lane axis; no broad flag; crest is thin and vertical.

Use the shared vertex-color material, cached per-joint meshes and existing rig slots. Sample actual posed bounds for all five forms; retain one-dimensional authoritative anchors and existing crowd limits. No visual overlap waiver or simulation-body resizing.

## Five forms

Base uses scale 1.00. Each branch has two steps, at 1.08 and 1.16. Every step changes a major material surface and a silhouette piece; final forms gain one restrained signature artifact.

- A1: **Dawn Point** / **Ponta da Aurora**, scale 1.08.
- A2: **Sun Pact** / **Pacto Solar**, scale 1.16.
- B1: **Ember Lance** / **Lança da Centelha**, scale 1.08.
- B2: **Daybreak Oath** / **Juramento da Alvorada**, scale 1.16.

## Props, shots and motion

### Weapon

Long solar spear with one clear spearhead; stows almost vertical between attacks.

### Projectile

None; one gold spear contact glint and one small consumable spark glyph on the primary victim.

### Idle

Rolls the spear grip and shifts to a balanced fencing guard.

### Walk

Light high-knee approach with a compact vertical spear and fixed crest.

### Attack

Linear planted-foot thrust, rear hand follows the haft and recoils exactly after contact.

### Special

An ally melee strike consumes the single victim spark in one small gold cross-flash.

### Victory

Four-step lance salute, shaft half-roll close to chest and a forward nod.

### Death

Spear braces, knee touches, grip releases and crest tips forward.

## Timing and verification

Attack preparation and impacts follow ART-DIRECTION section 4: ordinary melee contact at 0.30 s, projectile release at 0.18 s toward the authoritative impact, casts at 0.22 s. Cosmetic movement never chooses outcomes. Root/death cancel pending visual actions when the public state does. Victory stays eventless and bounded by the shared celebration resolver.

Required scoped evidence: five-form silhouette comparison against the 46 existing champions; actual per-form triangles/joints/materials and sampled animation bounds; unique idle/walk/attack/special/Hit/Death/Spawn/Evolve/Victory clips; both Evolution paths; actual ability regression; Linux preview capture. Windows and desktop web remain pending until observed.
