# P27 — Character refinement: Sawknight

**Depends on:** P26 reviewed.
**Branch:** `phase/p27-sawbot`.
**Difficulty:** medium–high (art and animation quality).

## Goal

Bring the **Sawknight** (`Sawbot`) to final medieval quality in all
five forms, with professional animation, its effects and sounds, inside
the budgets — and consistent with every character already finished.
Follow ART-DIRECTION §8 ("Character refinement procedure") step by step.

Display name **Sawknight** replaces "Sawbot" in every player-facing text (card, info, tips); the code id `Sawbot` stays.

## Unit sheet

| Field | Value |
|-------|-------|
| Faction / Archetype / body plan / slots | Machines / DPS / Humanoid / 1 |
| Simulation (read only, GAME-DESIGN §5) | cost 3 · HP 270 · 55 dmg every 1.05 s · range 0.8 · speed 1.3 |
| Identity | A clockwork knight: riveted iron shell, brass gears at the joints, a furnace glow in the chest grille, one arm ending in a circular saw blade that spins. Fast and twitchy. |

| Form | Adds |
|------|------|
| Base | iron shell, saw-blade arm, chest grille glow, team-color pennant on the back |
| A1 Overclock | exhaust vents on the back puffing steam |
| A2 Shredder | second saw on the other arm, bigger furnace glow; forge accent |
| B1 Plating | riveted extra plates, shoulder guard |
| B2 Juggernaut | tower helm, plate skirt, larger blade; forge accent |

## Read first (only these)

- `AGENTS.md`
- `docs/ART-DIRECTION.md` §2–§5, §8, §9
- `docs/GAME-DESIGN.md` §5 rows for this unit (stats and evolution tree)
- Previous character phase reports (lineup and lessons)
- Client: `src/art/roster/machines/` (create `sawbot.rs` if missing),
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
ART-DIRECTION §8 step 4. Archetype set **DPS**. Signature: the saw joint spins continuously (idle slow, attack fast, ramp 0.2 s); dash-in on attack A; Juggernaut attack B is a spinning sweep matching splash 0.8.
Timing table in the report (clip, duration, key times, event time).

### 4. Effects and sound
Sparks on every saw hit; steam puffs from vents (Overclock+); Juggernaut splash ring (exact). Sounds: saw whir loop (attack speeds it up), saw hit, gear clicks on footsteps.

### 5. Portrait and card
Re-render the Shop-card portrait (D-29 gallery portrait tool) and check the card at Shop size on desktop and phone.

### 6. Consistency lineup
`?gallery=lineup`: this unit's five forms next to every finished unit;
screenshot in the report (ART-DIRECTION §8 step 6).

### 7. Performance
ART-DIRECTION §8 step 7 (`?stress=30` of this unit, Low tier, mid-range
phone ≥ 30 FPS; draw calls recorded for diagnostics; no fixed maximum, D-28).

### 8. Validation and owner approval
ART-DIRECTION §8 step 8. Screenshots to `docs/media/units/sawbot/`.

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
