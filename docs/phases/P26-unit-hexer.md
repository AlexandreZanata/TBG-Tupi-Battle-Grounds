# P26 — Character refinement: Hexer

**Depends on:** P25 reviewed.
**Branch:** `phase/p26-hexer`.
**Difficulty:** medium–high (art and animation quality).

## Goal

Bring the **Hexer** (`Hexer`) to final medieval quality in all
five forms, with professional animation, its effects and sounds, inside
the budgets — and consistent with every character already finished.
Follow ART-DIRECTION §8 ("Character refinement procedure") step by step.

## Unit sheet

| Field | Value |
|-------|-------|
| Faction / Archetype / body plan / slots | Undead / Control / Humanoid / 1 |
| Simulation (read only, GAME-DESIGN §5) | cost 4 · HP 300 · 30 dmg every 1.4 s · range 3.5 · slow 40% for 2 s |
| Identity | A gaunt hooded necromancer with a skull-topped staff and floating soul-fire motes; violet robes, bone charms. Slow, deliberate, eerie gestures. |

| Form | Adds |
|------|------|
| Base | hood, violet robe, skull staff, two soul-fire motes |
| A1 Withering | longer crooked staff, bone necklace |
| A2 Doom | horned crown, staff skull glowing; soul-fire accent |
| B1 Plague | censer hanging from the staff, green mist |
| B2 Pestilence | bigger censer, mist cloak, three motes; soul-fire accent |

## Read first (only these)

- `AGENTS.md`
- `docs/ART-DIRECTION.md` §2–§5, §8, §9
- `docs/GAME-DESIGN.md` §5 rows for this unit (stats and evolution tree)
- Previous character phase reports (lineup and lessons)
- Client: `src/art/roster/undead/` (create `hexer.rs` if missing),
  `src/art/clips/`, `src/art/controller.rs`, `src/vfx/`, `src/audio.rs`, `src/gallery.rs`

## Do NOT

- Change any simulation value, event, or timing; the card text comes from
  `tbg-core` content.
- Touch another unit's files (except the shared lineup gallery list).
- Exceed the triangle budget (≤ 1 500 per form) or add textures.

## Steps

### 1. Brief and silhouette
ART-DIRECTION §8 steps 1–2. Silhouette row screenshot at 64 px height.

### 2. Model: five forms
ART-DIRECTION §8 step 3 using the form table above. Team color on the
tabard/banner/pennant slot. Remove this unit's old 12-joint spec and its
adapter entry when the new forms are in.

### 3. Animation
ART-DIRECTION §8 step 4. Archetype set **Control**. Signature: motes orbit the staff while idle and gather on cast; release at 220 ms.
Timing table in the report (clip, duration, key times, event time).

### 4. Effects and sound
Hex orb projectile (curving, soul-fire glow); slow = spectral chains at the target's feet for the status duration; Plague/Pestilence splash = green mist ring (exact radius). Sounds: hex whisper on cast, soft impact hiss.

### 5. Portrait and card
Re-render the Shop-card portrait (D-29 gallery portrait tool) and check the card at Shop size on desktop and phone.

### 6. Consistency lineup
`?gallery=lineup`: this unit's five forms next to every finished unit;
screenshot in the report (ART-DIRECTION §8 step 6).

### 7. Performance
ART-DIRECTION §8 step 7 (`?stress=30` of this unit, Low tier, mid-range
phone ≥ 30 FPS; draw calls recorded for diagnostics; no fixed maximum, D-28).

### 8. Validation and owner approval
ART-DIRECTION §8 step 8. Screenshots to `docs/media/units/hexer/`.

## Acceptance checklist

- [ ] All verification commands from `AGENTS.md` pass; `trunk build --release` passes
- [ ] Five forms within the triangle budget (gallery log)
- [ ] Attack event times match ART-DIRECTION §4 (controller tests still pass)
- [ ] Lineup, contact sheets, and in-match screenshots saved
- [ ] ≥ 30 FPS stress result recorded
- [ ] Owner approval recorded

## Reviewer checklist

- Readable at phone size in a busy match (side, unit, form, health).
- Same detail density and palette rules as the reference (Archer).
- No change outside this unit's files, the lineup list, and sounds.

## Phase report

_Executor fills this in: brief, timing table, triangle counts, FPS,
screenshots, owner approval, deviations._
