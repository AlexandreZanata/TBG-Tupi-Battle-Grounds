# P70 pilot — Old Finn

Status: implemented candidate; final renderer inspection, live effects evidence,
focused regressions and owner aesthetic approval remain pending.

## Baseline and individual brief

The full-cohort baseline completed before these changes. Inspected original
Old Finn sheets: [Idle](../../.local/p70-evidence/before-cohort/OldFinn/ally-threequarter/00.png),
[Cast](../../.local/p70-evidence/before-cohort/OldFinn/ally-threequarter/04.png) and
[Victory](../../.local/p70-evidence/before-cohort/OldFinn/ally-threequarter/09.png).
Their adjacent JSON files record the original pose sampling and camera. These
are event-free model sheets, not evidence of projectile flight or live combat.

Observed at this framing: a narrow yellow coat, small single-point beard,
thin rod/reel, and weak large-surface distinction between the branches. The
cast's free arm and raised-side-arm celebration read as common humanoid poses.
The proposed recognition anchor remains the established old fisherman:
sou'wester, heavy oilskin, white beard, creel, rod and fish. No lore or ability
was added.

## Candidate decisions

- Rounded oilskin belly and shoulder caps, a larger divided beard and
  moustache, wider layered hat brim/back flap, dark rounded boot toes and a
  legible reel distinguish the body and equipment. Arm reach was fitted to
  the wider coat and reel so the free hand can work at its actual handle.
- Base retains yellow oilskin. A1/A2 use a large ochre coat region and red
  gloves, with a fuller basket and A2's existing large catfish/gold hook.
  B1/B2 add a large navy bib, hat band and net; the second rod remains, and
  B2 retains its hanging fish/lantern. Evolution scale remains
  `1.00 / 1.08 / 1.16` relative to Finn's base scale.
- The team marker is a fitted neckerchief with a knot and hanging tails.
  It remains separate from character color. This is a clothing refinement,
  not a claim that Finn reproduced the shared blue artifact.
- Named palette additions are owned centrally by the integration front:
  `OILSKIN_YELLOW #D8A92E`, `OILSKIN_SHADE #9D6F20`,
  `FISH_BACK #426676`, `FISH_SILVER #B7D1D5`. Other surfaces reuse existing
  palette colors.

## Individual motion

`old_finn/motion.rs` keeps the kit's authored performance separate from model
construction and reuses existing IK, foot planting and timing helpers.

- Idle: a restrained rod nibble, downward line check and free-hand pinch,
  returning toward the creel strap.
- Walk: short weighted boot steps, counter-rotation in the head/chest and
  the rod carried outside the right shoulder. Cadence still follows actual
  movement through the existing gait system; simulation speed is unchanged.
- Cast: right-shoulder load through `0.15 s`, free-hand line feed, forward
  whip at the existing `0.22 s` Release, then withdrawal and recovery through
  the existing `0.62 s` clip. The release socket remains the original rod tip.
- Victory: a planted rocking fishing jig with the free hand at the reel,
  restrained nods and an outside-the-head rod angle. P63 still owns the
  existing kit Victory through its resolver and root-motion bounds.
- Hit, Spawn, Death and Evolve retain the current life-event timing and
  shared movement helpers, with an appropriate held-rod/hand pose. Final
  all-form clipping and transition review is still required.

## Fish presentation and invariants

`ShotKind::Fish(u8)` carries only the already-known visual tier selected by
`Kit::Attack::Shot`. The existing release pipeline already calls the selector
with tier and branch; no protocol, core, server or extra per-character central
switch was needed.

`vfx/fish.rs` builds one cached nose-forward mesh with a broad forked tail,
dark back, pale belly, paired eyes/gill covers and distinct dorsal/pectoral
fins. Both seats see eyes and a separated fish silhouette. The cosmetic size
factors are `Base 1.00`, `tier 1 1.35`, `tier 2 1.70`, identical across the
two branches. The mesh itself is larger and more legible than the original;
these factors replace the larger preliminary planning factors, and remain
subject to final renderer tuning. Roll decreases from `22` to `6 rad/s`
so the fish silhouette stays legible during its short flight.

Unchanged: `0.22 s` release, `0.08 s` flight, authoritative `0.30 s` hit,
`fish_arc`, `FISH_MIN_APEX = 2.0`, destination, impact type, area data, the
existing Echo behavior/counts, all other projectile look data, and combat
stats. No mesh/material is created per frame. There is no new particle-driven
hit, extra fish event, slowdown or delay for screenshots.

Source inspection did not establish a separate client fish emission for
Echo. This change preserves the existing pipeline; it does not claim that
an unobserved Echo presentation was verified or implement an invented count.

## Validation and handoff

Level: work front. Executed on the candidate files:

```text
rustfmt --edition 2021 --check crates/tbg-client/src/art/kits/old_finn/mod.rs crates/tbg-client/src/art/kits/old_finn/model.rs crates/tbg-client/src/art/kits/old_finn/motion.rs crates/tbg-client/src/vfx/shots.rs crates/tbg-client/src/vfx/fish.rs
exit 0

git diff --check -- crates/tbg-client/src/art/kits/old_finn crates/tbg-client/src/vfx/shots.rs crates/tbg-client/src/vfx/fish.rs
exit 0
```

Written but not run here; consolidate once under the shared host lock:

```bash
flock -n -E 75 "/tmp/tbg-validation-$(id -u).lock" \
  env CARGO_BUILD_JOBS=2 cargo test -p tbg-client --bin tbg-client \
  fish -- --test-threads=1
```

The focused filter includes the extended
`fish_lands_at_hit_and_clears_allies`, new
`fish_tiers_keep_timing_and_other_shots_unchanged`, and new
`old_finn_all_forms_select_growing_fish` regressions. They guard form
propagation, monotonic cosmetic size, unchanged event time/trajectory and
isolation from unrelated shots.

No build, Cargo test, native/browser parity claim, or post-change screenshot
was performed on this work front. Integration owns the cached client check
and paired final capture. Still pending: all-form/all-applicable-clip final
sheets, front/profile/back and transitions, normal-distance fish emission /
travel / impact / cleanup at actual combat speed, both seats, branch A/B
and Echo observations, Low quality, Troops/training, P63 Result ownership,
desktop web, and owner recognition/beauty approval.
