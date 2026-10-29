# P29 — Character refinement: Rootcaller

**Depends on:** P28 reviewed (and P21 effects kit merged).
**Branch:** `phase/p29-rootcaller`.
**Difficulty:** medium–high (art and animation quality).

## Goal

Bring the **Rootcaller** (`Rootcaller`) to final medieval quality in all
five forms, with professional animation, its effects and sounds, inside
the budgets — and consistent with every character already finished.
Follow ART-DIRECTION §8 ("Character refinement procedure") step by step.

## Unit sheet

| Field | Value |
|-------|-------|
| Faction / Archetype / body plan / slots | Plants / Control / Humanoid / 1 |
| Simulation (read only, GAME-DESIGN §5) | cost 4 · HP 300 · 20 dmg every 1.6 s · range 3.0 · root 0.8 s |
| Identity | A small treant shaman: bark body, moss beard, glowing amber eyes, long root-claw hands. Knobbly, rooted, deliberate. |

| Form | Adds |
|------|------|
| Base | bark body, moss beard, root-claw hands |
| A1 Deep Roots | longer claws, roots trailing from the feet |
| A2 Strangle | vine coils around the arms, amber eyes brighter; amber accent |
| B1 Spores | mushroom cap on the head |
| B2 Overgrowth | bigger cap with spore sacs, moss cloak; amber accent |

## Read first (only these)

- `AGENTS.md`
- `docs/ART-DIRECTION.md` §2–§5, §8, §9
- `docs/GAME-DESIGN.md` §5 rows for this unit (stats and evolution tree)
- Previous character phase reports (lineup and lessons)
- Client: `src/art/roster/plants/` (create `rootcaller.rs` if missing),
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
ART-DIRECTION §8 step 4. Archetype set **Control**. Signature: casts by plunging the claws into the ground (release at 220 ms) instead of thrusting a staff.
Timing table in the report (clip, duration, key times, event time).

### 4. Effects and sound
Ground ripple projectile to the target, roots erupt and wrap (root status duration); Spores/Overgrowth splash = spore puff ring (exact radius). Sounds: roots rumble on cast, wood creak on footsteps.

### 5. Portrait and card
Re-render the portrait (P19) and check the card at Shop size on a phone.

### 6. Consistency lineup
`?gallery=lineup`: this unit's five forms next to every finished unit;
screenshot in the report (ART-DIRECTION §8 step 6).

### 7. Performance
ART-DIRECTION §8 step 7 (`?stress=30` of this unit, Low tier, mid-range
phone ≥ 30 FPS; draw calls recorded for diagnostics; no fixed maximum, D-28).

### 8. Validation and owner approval
ART-DIRECTION §8 step 8. Screenshots to `docs/media/units/rootcaller/`.

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
