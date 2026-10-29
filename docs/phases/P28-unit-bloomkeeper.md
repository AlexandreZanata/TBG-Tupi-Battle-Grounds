# P28 — Character refinement: Bloomkeeper

**Depends on:** P27 reviewed (and P21 effects kit merged).
**Branch:** `phase/p28-bloomkeeper`.
**Difficulty:** medium–high (art and animation quality).

## Goal

Bring the **Bloomkeeper** (`Bloomkeeper`) to final medieval quality in all
five forms, with professional animation, its effects and sounds, inside
the budgets — and consistent with every character already finished.
Follow ART-DIRECTION §8 ("Character refinement procedure") step by step.

## Unit sheet

| Field | Value |
|-------|-------|
| Faction / Archetype / body plan / slots | Plants / Support / Humanoid / 1 |
| Simulation (read only, GAME-DESIGN §5) | cost 3 · HP 280 · heal 40 every 1.5 s · range 3.0 |
| Identity | A druid in a leaf cloak with a flowering staff and a crown of small blossoms; bark-skinned hands. Gentle, flowing motion. |

| Form | Adds |
|------|------|
| Base | leaf cloak (Back spring), flowering staff, blossom crown |
| A1 Blossom | larger blooms on the staff, petal shoulder |
| A2 Grove | blooming antler-crown, petal aura; bloom accent |
| B1 Bramble | thorn armor pieces, gains a briar whip (attacks) |
| B2 Thornwarden | full thorn armor, briar whip with roots; amber accent |

## Read first (only these)

- `AGENTS.md`
- `docs/ART-DIRECTION.md` §2–§5, §8, §9
- `docs/GAME-DESIGN.md` §5 rows for this unit (stats and evolution tree)
- Previous character phase reports (lineup and lessons)
- Client: `src/art/roster/plants/` (create `bloomkeeper.rs` if missing),
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
ART-DIRECTION §8 step 4. Archetype set **Support** (heal) and, for Bramble/Thornwarden, an added short whip attack (Control-style release at 220 ms). Signature: petals drift off the cloak while walking.
Timing table in the report (clip, duration, key times, event time).

### 4. Effects and sound
Petal stream to the healed ally + heal glow; Grove heal splash = flower ring (exact radius); Thornwarden root = roots wrapping (P21). Sounds: bloom chime on heal, whip crack for Bramble line.

### 5. Portrait and card
Re-render the portrait (P19) and check the card at Shop size on a phone.

### 6. Consistency lineup
`?gallery=lineup`: this unit's five forms next to every finished unit;
screenshot in the report (ART-DIRECTION §8 step 6).

### 7. Performance
ART-DIRECTION §8 step 7 (`?stress=30` of this unit, Low tier, mid-range
phone ≥ 30 FPS; draw calls recorded for diagnostics; no fixed maximum, D-28).

### 8. Validation and owner approval
ART-DIRECTION §8 step 8. Screenshots to `docs/media/units/bloomkeeper/`.

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
