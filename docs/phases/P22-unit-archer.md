# P22 — Character refinement: Archer

**Depends on:** P21 reviewed (and P21 effects kit merged).
**Branch:** `phase/p22-archer`.
**Difficulty:** medium–high (art and animation quality).

## Goal

Bring the **Archer** (`Archer`) to final medieval quality in all
five forms, with professional animation, its effects and sounds, inside
the budgets — and consistent with every character already finished.
Follow ART-DIRECTION §8 ("Character refinement procedure") step by step.

Because this unit is the reference, finish it first and use its lineup in every later unit phase.

## Unit sheet

| Field | Value |
|-------|-------|
| Faction / Archetype / body plan / slots | Humans / Ranged / Humanoid / 1 |
| Simulation (read only, GAME-DESIGN §5) | cost 3 · HP 220 · 45 dmg every 1.0 s · range 4.5 · speed 1.0 |
| Identity | The reference character (style lock from P15). A hooded longbowman: linen tunic, leather bracer and quiver, a longbow taller than he is, team-color tabard and fletching. Calm, precise, economical motion. |

| Form | Adds |
|------|------|
| Base | style-lock model from P15 (re-check against the final kit) |
| A1 Longbow | longer yew bow with grip wrap; feathered cap |
| A2 Marksman | recurve longbow, half-cape, eye-glint on release; gold accent |
| B1 Volley | second quiver on the hip, shorter faster draw |
| B2 Arrow Rain | twin quivers, fire-arrow glow on the nocked arrow; gold accent |

## Read first (only these)

- `AGENTS.md`
- `docs/ART-DIRECTION.md` §2–§5, §8, §9
- `docs/GAME-DESIGN.md` §5 rows for this unit (stats and evolution tree)
- Previous character phase reports (lineup and lessons)
- Client: `src/art/roster/humans/` (create `archer.rs` if missing),
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
ART-DIRECTION §8 step 4. Archetype set **Ranged**. Signature: a nock-and-check idle every 4–6 s; Volley/Arrow Rain use a snappier draw (anchor shortened to 20 ms); Marksman holds the anchor longer (release still at 180 ms by compressing the draw).
Timing table in the report (clip, duration, key times, event time).

### 4. Effects and sound
Arrow projectile with team fletching; Arrow Rain arrows arc high and land as a splash ring; Marksman arrows leave a faint trail. Sounds: bow twang variants; Marksman: heavier twang.

### 5. Portrait and card
Re-render the portrait (P19) and check the card at Shop size on a phone.

### 6. Consistency lineup
`?gallery=lineup`: this unit's five forms next to every finished unit;
screenshot in the report (ART-DIRECTION §8 step 6).

### 7. Performance
ART-DIRECTION §8 step 7 (`?stress=30` of this unit, Low tier, mid-range
phone ≥ 30 FPS; draw calls recorded for diagnostics; no fixed maximum, D-28).

### 8. Validation and owner approval
ART-DIRECTION §8 step 8. Screenshots to `docs/media/units/archer/`.

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
