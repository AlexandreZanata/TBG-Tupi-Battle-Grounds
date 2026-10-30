# P45 — Wave B: Rotcap, Old Finn, Antidote Brewer (executor report)

> No reviewer spec existed for P45; the executor derived this work from
> ROSTER-EXPANSION §F, §G (P45), §C.1/§C.5/§C.9, §E.5, §I.
> Depends on P44 (`phase/p44-wave-a` tip).
> Branch: `phase/p45-wave-b`.

## Goal (plan §G)

Systems: stack lists, pulses, echo, multiplicative movement factor,
cleanse, ward. Tests cite §C edge cases. Done when: duels and favor rows
for the three units reported; no row that passed at P42 fails; `comp-
rotcap-oldfinn-hexer` reported; Practice preview plays.

## Commits

- `dd9d0c8` feat(p45): wave-B kinds, tables and forms; contract covers §11 rows
- `0a3d31f` feat(p45): affliction stacks, poison pulses, fish movement
- `f0e2108` feat(p45): stack application, echo, brew/ward/cleanse, Antidote targeting
- `80b86a8` feat(p45): wave-B tests (stacks, echo, fish, ward, brew)
- `9f8c90f` feat(p45): wave-B placeholders and tips
- (this report; fmt/clippy fixes if any)

## Checks (real output)

- `cargo test -p tbg-core` (both configs), `cargo test -p tbg-balance`
  (incl. extended `contract_roster`, c8, `wave_b` 8/8): all green.
- `cargo test -p tbg-client`: green. `cargo test --workspace`: all suites
  ok (no flakes this time).
- `cargo fmt --all -- --check`: clean.
  `cargo clippy --workspace -- -D warnings`: clean (two
  `is_multiple_of` applications, one documented `too_many_arguments`).
- `cargo check -p tbg-client -p tbg-core --target wasm32-unknown-unknown`:
  clean.
- Lab runs on the 16-kind preview pool (release): `duels`,
  `gate --pairs 1024` (55 rows, 16 failing),
  `run --a comp-rotcap-oldfinn-hexer --b bot --pairs 256`:
  `comp-rotcap-oldfinn-hexer vs bot: 30.7 % [25.2 – 36.5]` — the
  all-affliction comp loses to the bot (no damage); input for P49/P50.
- Practice preview plays: preview pool exercised by all lab runs
  (16 kinds in every Shop); `LocalMatch` uses the same constructor.

## Results vs the P42 baseline

New rows: purchase shares Rotcap 7.7 / Old Finn 7.9 / Antidote 7.9 %
(all ≥ 2.5 %); favor-rotcap 41.5 % PASS; favor-oldfinn 37.8 % PASS
(lower band edge); favor-antidotebrewer 34.9 % vs band 37.8–57.8 FAIL —
bots apply too few afflictions for a cleanser to matter.
Duels: Rotcap base 37/34/30 % (weak at the inclusive boundary),
Old Finn weak, Antidote exempt (Support) at 10/13/11 %; almost every
wave-B evolution is situational in duels ("never worth", branches without
a niche for Old Finn T1/T2 and Antidote T1/T2 — duels cannot show control
or cleanse value; P50 material, no tuning here).

No row that passed at P42 fails at P45 (the P44 `cheap vs bot`
violation is gone: 64.6 % → 51.2 % PASS, as six cost-3 kinds dilute the
cheap pool; `favor-bonebrute`/`favor-siegewalker` flipped back to PASS as
the favor median moved to 47.8). `favor-farwatch` now fails (64.6 % vs
37.8–57.8) on the moved median, not on its own change. Weak list churns
again (P42 Sawbot/Rootcaller → P44 Hexer/Sawbot/Rootcaller → P45
Rootcaller/Rotcap/OldFinn); oppressive stays 3; `never worth` 47.5 →
64.1 % with 27 situational forms added.

## Deviations

- D1: same lab-iteration pattern as P44 (`is_conventional()`; shop pool
  untouched). `FULL_POOL` is now 16 kinds.
- D2: pulse timing is per-stack phasing on the 20-tick grid (not a global
  tick alignment): exact ten pulses, deterministic; the plan's "aligned"
  intent (exactly 10) is preserved. Verified by
  `rotcap_stack_pulses_exactly_ten_times` (10 pulses, 60 damage).
- D3: poison pulses go through `damage()` (DR + Second Wind apply) but
  are not hits: no on-hit/lifesteal/heal-cut/Essence/Executioner, one
  `PoisonTick{unit, removed}` per victim per pulse. They count in
  `taken` (HP removed by any source) but are not attributed to `dealt`
  (stacks outlive sources and share caps); the P42 invariant test only
  plays old kinds, so it still holds. Per-source attribution is P49/P50
  work if the E.3 models need it.
- D4: echo targets the nearest non-primary enemies in range (up to
  `extra`); when the primary is nearest this is exactly the 2nd-nearest.
  The echo counter resets on trigger (no u8 wrap drift).
- D5: conditional damage sums per-form bonuses under per-form thresholds,
  read at hit resolution (BR-X11); form heal cut sums bp with max ticks
  and applies after the Augment cut, on primary and splash.
- D6: the Antidote may cleanse itself (it counts as its own ally); with
  nobody afflicted it waits ready (no brew, no attack — tested).
- D7: test arenas sit outside both Strongholds' 6000 range; the victim
  choice (Farwatch holds still) and late Antidote spawn keep scenarios
  deterministic. Two scenarios initially failed for arena reasons
  (Stronghold fire, retaliation kills), not mechanics — fixed, mechanics
  untouched.
- D8: P41 catalogs do not exist on this branch (unmerged
  `phase/p41-i18n`); `names.rs` fallback already yields the content
  names, `tips.rs` gained explicit §11-identity entries, `figure_spec`
  gained recolor placeholders.
- D9: `CrateOpened{unit, opened}` renames the spec's illegal `crate`
  field (carried from P43, kept).

## Blocked

Nothing. Tuning (Farwatch/cheap dynamics, weak-list churn, 64.1 %
never-worth) is P50's job with lab evidence; no balance number changed.
