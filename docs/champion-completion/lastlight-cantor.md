# CC02 — Lastlight Cantor

**Status: Proposed.** Owner-authorized champion completion; numbers are initial local-preview hypotheses, not measured balance.

Display names: **Lastlight Cantor** / **Cantora do Último Sino**. Undead / Support.

Default skin: Mourning Bell / Sino de Luto.

## Identity and crowd space

Bare skull in front of a mourning fan, bell-shaped skirt and one large handbell; no wizard hood or pole.

Vertical fan and chest-level bell; skirt hem stays above feet and does not spread into adjacent figures.

Use the shared vertex-color material, cached per-joint meshes and existing rig slots. Sample actual posed bounds for all five forms; retain one-dimensional authoritative anchors and existing crowd limits. No visual overlap waiver or simulation-body resizing.

## Five forms

Base uses scale 1.00. Each branch has two steps, at 1.08 and 1.16. Every step changes a major material surface and a silhouette piece; final forms gain one restrained signature artifact.

- A1: **Vigil Bell** / **Sino da Vigília**, scale 1.08.
- A2: **Sanctuary Bell** / **Sino do Santuário**, scale 1.16.
- B1: **Funeral Choir** / **Coro Fúnebre**, scale 1.08.
- B2: **Last Requiem** / **Último Réquiem**, scale 1.16.

## Props, shots and motion

### Weapon

Broad bronze handbell with one rigid clapper socket.

### Projectile

Soft green bell ripple travels from the bell to the healed ally; echo uses a single small wrist glyph.

### Idle

Bell breath-sway and gentle free-hand finger opening.

### Walk

Short processional steps; bell lags the shoulder and the fan stays vertical.

### Attack

Raises bell, sounds one measured toll, then presents the free palm toward the patient.

### Special

Echo rune closes once when nonlethal HP damage consumes it; no resurrection or death cancellation.

### Victory

Three alternating bell swings, two small side steps and a ceremonial fan bow.

### Death

Kneels holding the bell upright; clapper stops last and the head bows into the fan.

## Timing and verification

Attack preparation and impacts follow ART-DIRECTION section 4: ordinary melee contact at 0.30 s, projectile release at 0.18 s toward the authoritative impact, casts at 0.22 s. Cosmetic movement never chooses outcomes. Root/death cancel pending visual actions when the public state does. Victory stays eventless and bounded by the shared celebration resolver.

Required scoped evidence: five-form silhouette comparison against the 46 existing champions; actual per-form triangles/joints/materials and sampled animation bounds; unique idle/walk/attack/special/Hit/Death/Spawn/Evolve/Victory clips; both Evolution paths; actual ability regression; Linux preview capture. Windows and desktop web remain pending until observed.
