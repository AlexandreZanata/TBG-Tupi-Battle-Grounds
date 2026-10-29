# P23 — Character refinement: Shieldbearer

**Depends on:** P22 reviewed.
**Branch:** `phase/p23-shieldbearer`.
**Difficulty:** medium–high (art and animation quality).

## Goal

Bring the **Shieldbearer** (`Shieldbearer`) to final medieval quality in all
five forms, with professional animation, its effects and sounds, inside
the budgets — and consistent with every character already finished.
Follow ART-DIRECTION §8 ("Character refinement procedure") step by step.

## Unit sheet

| Field | Value |
|-------|-------|
| Faction / Archetype / body plan / slots | Humans / Tank / Humanoid / 1 |
| Simulation (read only, GAME-DESIGN §5) | cost 3 · HP 650 · 30 dmg every 1.2 s · range 0.8 · speed 0.8 |
| Identity | A knight-at-arms behind a tall kite shield with the team's heraldry, flanged mace, open-faced helm, mail and a surcoat. Steady, grounded, protective. |

| Form | Adds |
|------|------|
| Base | kite shield (team heraldry), flanged mace, mail, open helm |
| A1 Bulwark | tower shield with iron rim, plate pauldrons |
| A2 Fortress | full plate, great helm with crest, shield spikes; gold accent |
| B1 Vanguard | spear instead of mace, lighter round shield, faster stance |
| B2 Champion | long spear, pauldrons, back banner on the `Back` spring; gold accent |

## Read first (only these)

- `AGENTS.md`
- `docs/ART-DIRECTION.md` §2–§5, §8, §9
- `docs/GAME-DESIGN.md` §5 rows for this unit (stats and evolution tree)
- Previous character phase reports (lineup and lessons)
- Client: `src/art/roster/humans/` (create `shieldbearer.rs` if missing),
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
ART-DIRECTION §8 step 4. Archetype set **Tank**. Signature: shield-raise block flinch when hit from the front; Fortress hits stamp a small ground crack (slow on hit); Champion's attack B is a wide spear sweep matching its splash 1.0.
Timing table in the report (clip, duration, key times, event time).

### 4. Effects and sound
Metal sparks on mace hits; Fortress slow shows the spectral chains (P21); Champion sweep shows the exact splash ring. Sounds: shield bash, mace hit variants, armor clank on footsteps (quiet).

### 5. Portrait and card
Re-render the Shop-card portrait (D-29 gallery portrait tool) and check the card at Shop size on desktop and phone.

### 6. Consistency lineup
`?gallery=lineup`: this unit's five forms next to every finished unit;
screenshot in the report (ART-DIRECTION §8 step 6).

### 7. Performance
ART-DIRECTION §8 step 7 (`?stress=30` of this unit, Low tier, mid-range
phone ≥ 30 FPS; draw calls recorded for diagnostics; no fixed maximum, D-28).

### 8. Validation and owner approval
ART-DIRECTION §8 step 8. Screenshots to `docs/media/units/shieldbearer/`.

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
