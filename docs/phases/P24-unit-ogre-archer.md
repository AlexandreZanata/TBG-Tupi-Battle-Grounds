# P24 — Character refinement: Ogre Archer

**Depends on:** P23 reviewed (and P21 effects kit merged).
**Branch:** `phase/p24-siegewalker`.
**Difficulty:** medium–high (art and animation quality).

## Goal

Bring the **Ogre Archer** (`SiegeWalker`) to final medieval quality in all
five forms, with professional animation, its effects and sounds, inside
the budgets — and consistent with every character already finished.
Follow ART-DIRECTION §8 ("Character refinement procedure") step by step.

Display name **Ogre Archer** replaces "Siege Walker" in every player-facing text (card, info, tips); the code id `SiegeWalker` stays.

## Unit sheet

| Field | Value |
|-------|-------|
| Faction / Archetype / body plan / slots | Machines / Ranged / Large / 2 |
| Simulation (read only, GAME-DESIGN §5) | cost 5 · HP 700 · 85 dmg every 2.5 s · range 6.0 · speed 0.6 · splash 1.5 |
| Identity | A hulking ogre (tusks, green-grey skin, leather straps, iron and brass plates) carrying a geared great-bow — an arbalest with a crank, brass gears, and iron limbs — bigger than a man. The Ironworks hired muscle: slow, heavy, devastating. |

| Form | Adds |
|------|------|
| Base | ogre, geared great-bow, bolt rack on the back, team-color loincloth and back banner |
| A1 Longshot | longer bow limbs, a brass sighting ring |
| A2 Artillery | double-limb arbalest, iron helm with plume; forge-glow accent |
| B1 Shrapnel | barbed bolts in the rack, spiked shoulder plate |
| B2 Concussion | hammer-head bolts, stone ammunition bag, heavier crank; forge-glow accent |

## Read first (only these)

- `AGENTS.md`
- `docs/ART-DIRECTION.md` §2–§5, §8, §9
- `docs/GAME-DESIGN.md` §5 rows for this unit (stats and evolution tree)
- Previous character phase reports (lineup and lessons)
- Client: `src/art/roster/machines/` (create `siegewalker.rs` if missing),
  `src/art/clips/`, `src/art/controller.rs`, `src/vfx/`, `src/audio.rs`, `src/gallery.rs`

## Do NOT

- Change any simulation value, event, or timing; the card text comes from
  `tbg-core` content.
- Touch another unit's files (except the shared lineup gallery list).
- Exceed the triangle budget (≤ 3 000 per form) or add textures.

## Steps

### 1. Brief and silhouette
ART-DIRECTION §8 steps 1–2. Silhouette row screenshot at 64 px height.

### 2. Model: five forms
ART-DIRECTION §8 step 3 using the form table above. Team color on the
tabard/banner/pennant slot. Remove this unit's old 12-joint spec and its
adapter entry when the new forms are in.

### 3. Animation
ART-DIRECTION §8 step 4. Archetype set **Ranged** on the **Large** plan. Signature: between shots (2.5 s interval) it plays a whole-body crank-reload loop, so on `AttackStart` only the final heave-draw plays and the release lands at 180 ms; then a recoil half-step back. Ground thud on footsteps.
Timing table in the report (clip, duration, key times, event time).

### 4. Effects and sound
Heavy bolt projectile with spin and dust trail, arcing; impact = splash ring of radius 1.5 (exact, from content), dust and splinters; Concussion root shows roots/chains per P21 root visual. Sounds: heavy bolt release, crank ratchet, ogre grunt (low volume), Large footstep thud.

### 5. Portrait and card
Re-render the portrait (P19) and check the card at Shop size on a phone.

### 6. Consistency lineup
`?gallery=lineup`: this unit's five forms next to every finished unit;
screenshot in the report (ART-DIRECTION §8 step 6).

### 7. Performance
ART-DIRECTION §8 step 7 (`?stress=30` of this unit, Low tier, mid-range
phone ≥ 30 FPS; draw calls within budget).

### 8. Validation and owner approval
ART-DIRECTION §8 step 8. Screenshots to `docs/media/units/siegewalker/`.

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
