# P44 — Wave A: Longaxe, Farwatch, Bombard (executor report)

> No reviewer spec existed for P44; the executor derived this work from
> ROSTER-EXPANSION §F, §G (P44), §C.2/§C.4/§C.7, §E.5, §I.
> Depends on P43 (`phase/p43-shared-foundations` tip).
> Branch: `phase/p44-wave-a`.

## Goal (plan §G)

Systems: `splash_bp`, conditional damage (above/below HP), on-hit heal
cut, `NoStronghold` targeting, friendly blast. Tests cite rules.
Done when: duels and favor rows for the three units reported; no row that
passed at P42 fails; placeholder figurines (blockout recolors), names and
tips in the P41 catalogs; Practice preview plays.

## Commits

- `91ba056` feat(p44): wave-A kinds, tables and forms; contract covers §11 rows
- `b248d69` feat(p44): lab covers staged kinds, preview pool; extend c8 flat roster
- `0651bf3` feat(p44): NoStronghold, splash share, conditional damage, form heal cut, friendly blast
- `34ce328` feat(p44): wave-A placeholders and tips
- `1a751d2` feat(p44): FULL_POOL staging pool for lab and preview
- (this report; fmt fixes if any)

## Checks (real output)

- `cargo test -p tbg-core` (both configs), `cargo test -p tbg-balance`
  (incl. extended `contract_roster`, extended c8, `comp_policy`,
  `wave_a` 8/8): all green.
- `cargo test -p tbg-client`: green.
- `cargo fmt --all -- --check`: clean.
  `cargo clippy --workspace -- -D warnings`: clean.
- `cargo test --workspace`: all suites ok.
- `cargo check -p tbg-client -p tbg-core --target wasm32-unknown-unknown`:
  clean.
- New tests `crates/tbg-core/tests/wave_a.rs` (8, all pass): Farwatch
  holds at both seats, Guard forces Farwatch onto the Rampart, Bombard
  self-hit (24), ally full damage (all 24s), own Stronghold untouched,
  friendly-fire kill credits the enemy (Plunder bounty), Longaxe strikes
  over its tank undamaged, splash-share math.
- Lab runs on the 13-kind preview pool (release):
  `gate --pairs 1024`, `duels`, `run --a comp-archer-bonebrute --b bot
  --pairs 256` (`comp-archer-bonebrute vs bot: 72.3 % [66.8 – 77.5]`).
  Preview pool exercised by 352 256 matches, all ended (no safety-bound
  hit); Practice uses the same constructor (`with_preview`).
- Online crates unchanged in behavior: server still plays `CANON.pool`
  (10); protocol still v13; `cargo test -p tbg-server --test online`
  covered by the workspace run.

## Results vs the P42 baseline (49 rows, 16 failing — same count, one swap)

New rows pass: purchase shares Longaxe 4.3 / Farwatch 12.1 / Bombard 6.9 %
(all ≥ 2.5 %); favor rows Longaxe 59.5 / Farwatch 67.0 / Bombard 62.2 %
(all inside the shifted median band 50.0–70.0).
Duels: Longaxe base 75/58/48 %, Farwatch base 83/76/70 %, Bombard base
67/77/77 % (none oppressive, none weak); most wave-A evolutions are
situational in duels and land in "never worth" (Finishing Shot 20/23/23 %
— the <35 % condition rarely pays in equal duels; data for P50, no tuning
in this phase).

**One P42-passing row now fails: `cheap vs bot` 47.7 % → 64.6 %
[61.7–67.7] (band ≤ 55 %).** Cause in the data: the cheap policy now
frequently fields Farwatch (12.1 % share; favor-farwatch 67.0 %).
`favor-archer` flipped the other way (fail → pass) as the favor median
moved 42.3 → 60.0 with the new rows. No threshold was changed
(product decision, owner). Cheap-spam vs the field is now carried by a
staged unit: input for P50/outlier work, recorded here, not fixed here.

Other moves: faction Humans 37.8 → 28.7, Undead 64.1 → 75.8, Machines
79.9 → 77.3, Plants 24.2 → 36.6 (all still failing, same direction except
Plants improving); Hexer joined "weak everywhere" (now 3);
`duel Evolutions never worth` 47.5 → 55.8 % (15 situational forms added).

## Deviations

- D1: counts of `UNIT_KINDS` stay 10 (shop); lab roster iteration moved to
  `is_conventional()` over `ALL_KINDS`. `valid()` gained the
  NoStronghold range exception (§C.2) and checks staged kinds too.
- D2: splash share is highest-active-form-wins (like `on_hit`), radius
  still sums; tested in `wave_a`.
- D3: conditional damage sums per-form bonuses under per-form thresholds
  (BR-X11, read at hit time); form heal cut sums bp with max duration and
  applies after the Augment cut, on primary and splash.
- D4: splash vs Strongholds uses the shared amount (uniform rule).
- D5: P41 catalogs do not exist on this branch (unmerged
  `phase/p41-i18n`); the equivalents here are `names.rs` (fallback already
  yields the content names) and `tips.rs` (explicit §11-identity entries
  added). Placeholder figures are recolor arms in `figure_spec`.
- D6: `CrateOpened{unit, opened}` renames the spec's illegal `crate`
  field (P43, kept).
- D7: kill-credit test uses Plunder bounty as the observable; the credit
  path itself is untouched code.
- D8: c8 and `contract_roster` were extended (not weakened): staged kinds
  join the flat roster; §11 rows checked; every old assertion kept.

## Blocked

Nothing. Tuning the cheap/Farwatch finding is P50's job (numbers with
lab evidence, owner decision); this phase changes no balance number.
