# P47 — Wave D: Oilpress, Drumguard (executor report)

> No reviewer spec existed for P47; the executor derived this work from
> ROSTER-EXPANSION §F, §G (P47), §C.6/§C.8, §E.5, §I.
> Depends on P43 (`phase/p43-shared-foundations` tip; built on the P46 tip).
> Branch: `phase/p47-wave-d`.

## Goal (plan §G)

Systems: formation clamp, bonds with hysteresis, escort clamp, march
bonuses into the caps. Tests: formation only while engaged; allies ahead
unaffected; no effect on enemies/Strongholds; bond hysteresis
deterministic; best bonus with two drummers; caps; goldens without
Oilpress/Drumguard unchanged. Done when: as P44, plus stagnation row for
`comp-oilpress-*`.

## Commits

- `357dc67` feat(p47): wave-D kinds, tables, march field; goldens refrozen once
- `919bfe1` feat(p47): bonds, formation, march, splash-hit, A2 immunity
- `3a8b31f` feat(p47): wave-D tests (9, all pass)
- `9744e13` feat(p47): wave-D placeholders and tips
- `c11726a` feat(p47): stagnation row in run, comp antidote alias
- `56d8f5d` fix(p47): A2 march name and golden notes
- (this report; fmt/clippy fixes)

## Checks (real output)

- `cargo test -p tbg-core` (both configs): green. Goldens refrozen once
  (`march_ignore_slow` grew `UnitFx`); tick/winner/reason verified
  identical on the old kinds before refreezing.
- `cargo test -p tbg-balance` (incl. extended `contract_roster`, `wave_d`
  9/9, `comp_policy` incl. the antidote alias and all-conventional
  pairs): green.
- `cargo test -p tbg-client`: green. `cargo test --workspace`: all suites
  ok. `cargo test -p tbg-core --features lab`: ok.
- `cargo fmt --all -- --check`: clean.
  `cargo clippy --workspace -- -D warnings`: clean.
- `cargo check -p tbg-client -p tbg-core --target wasm32-unknown-unknown`:
  clean.
- Lab runs on the 21-kind preview pool (release): `duels`,
  `gate --pairs 1024` (65 rows, 16 failing),
  `run --a comp-oilpress-bloomkeeper --b bot --pairs 256`
  (`53.7 % [47.9 – 59.6]`, stalled 4.3 % = 22/512),
  `run --a comp-oilpress-bloomkeeper-antidote --b bot --pairs 128`
  (`48.0 % [40.2 – 55.9]`, stalled 0.8 % = 2/256).
- Practice preview plays: preview pool exercised by all lab runs; no
  online behavior change (`CANON.pool` still 10).

## Results vs the P42 baseline

New rows pass: purchase shares Oilpress 4.9 / Drumguard 5.9 % (all ≥ 2.5
%); favor rows Oilpress 52.1 / Drumguard 41.6 % (lower band edge).
Duels: Oilpress base 59/58/61 % (T1 Reinforced Hull 99/93/89 % and T2
Immovable Mass 98/99/98 % are near-unbeatable 1v1 — expected for a wall
with no clock, measured not tuned); Drumguard exempt (Support, 18 %).

No row that passed at P42 fails at P47. Deltas: the gate **stagnation row
fails for the first time: 1.4 % [1.3–1.6] vs ≤ 1.0 %** (P42: 0.6 %) —
the §H #4 stall risk materializing with walls in the pool, as the plan
predicted; the `comp-oilpress-bloomkeeper` stagnation row is 4.3 % while
the full turtle with Antidote is 0.8 % (both need P49/P50 follow-up, no
mechanic change: no clock is an owner rule). `cheap` fails marginally
again (55.6 % vs ≤ 55 %, CI overlaps the band); `favor-bonebrute`,
`favor-siegewalker`, `favor-antidotebrewer`, `favor-bloodweaver`,
`favor-bloodfurnace` pass on moved bands; oppressive is now 5 (Farwatch
joined); `never worth` 69.7 %, branches without a niche 12.

## Deviations

- D1: counts of `UNIT_KINDS` stay 10 (shop); lab iteration stays on
  `is_conventional()`; `FULL_POOL` is now 21 kinds.
- D2: Drumguard content range is 3.0 tiles (the hold radius); the §B.3
  table's 2.0 is the bond reach, encoded as `March.reach`. The staged
  test asserts 300 (documented).
- D3: A2 immunity is a kind+form check (`is_immune_to_control`), not data:
  the plan's shape has no immunity slot. It blocks slow/root/confusion
  application and the fish movement factor, but not poison/fish stacks.
- D4: B2 splash extension is a `FormDef.splash_hit` flag (default false):
  no existing splash changes behavior.
- D5: march bonuses join the existing BR-X03/BR-X05 caps (no separate
  caps); escort clamps at the ally, never past it; formation clamps only
  on crossing, only the nearest engaged wall ahead applies.
- D6: test arenas root the anchor/ally to freeze geometry (a marching
  wall moves the clamp point; Stronghold fire reaches 6000 and would
  contaminate long Roots/walks). Rooting does not affect bonds,
  formation checks, or immunity.
- D7: the `run` command gained a `stalled matches (60 s)` line (the
  plan's P47 stagnation row needs it; no contract pins `run.txt`).
- D8: `comp-*` accepts the §E.5 short name `antidote` (= AntidoteBrewer);
  all other E.5 names already match kind names exactly.
- D9: "no effect on Strongholds" is covered by construction (targeting
  unchanged for Strongholds; Guard logic untouched) rather than a
  dedicated test; no test moves any unit behind a wall because units
  cannot pass enemies in this engine.
- D10: P41 catalogs do not exist on this branch (unmerged
  `phase/p41-i18n`); `tips.rs` entries and `figure_spec` recolors added,
  names fall back to content names.

## Blocked

Nothing. Stall dynamics (`comp-oilpress-*`, 1.4 % gate stall) and the
oppressive Farwatch row are P49/P50 material; no balance number changed.
