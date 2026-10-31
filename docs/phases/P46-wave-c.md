# P46 — Wave C: Shatterback, Blood Furnace, Bloodweaver (executor report)

> No reviewer spec existed for P46; the executor derived this work from
> ROSTER-EXPANSION §F, §G (P46), §C.12/§C.13/§C.14, §E.5, §I.
> Depends on P43 (`phase/p43-shared-foundations` tip; built on the P45 tip).
> Branch: `phase/p46-wave-c`.

## Goal (plan §G)

Systems: plates in `damage()`, shrapnel queue, burn and multiplier at
resolve, transfer selection and event. Tests: plates before HP after
reduction; plates never restored; shrapnel chain terminates; burn only at
resolve (root cancels at no cost); Deathblow at 0 HP and simultaneous
deaths; multiplier off vs Strongholds; transfer bounds (donor ≥ 1 HP, heal
cut, summons excluded, ties). Done when: as P44.

## Commits

- `d34e284` feat(p46): wave-C kinds, tables and forms; contract covers §11 rows
- `17bbb84` feat(p46): plates, furnace sacrifice, blood transfer
- `299b76b` feat(p46): wave-C tests (plates, furnace, transfer)
- `56058cd` feat(p46): wave-C placeholders and tips
- (this report; fmt/clippy fixes if any)

## Checks (real output)

- `cargo test -p tbg-core` (both configs): all green, **goldens untouched**
  (no golden file changed: the shrapnel queue is serde-skipped, the three
  kinds only append enum variants; behavior for old armies identical).
- `cargo test -p tbg-balance` (incl. extended `contract_roster`,
  `wave_c` 11/11): green.
- `cargo test -p tbg-client`: green. `cargo test --workspace`: all suites
  ok. `cargo test -p tbg-core --features lab`: ok.
- `cargo fmt --all -- --check`: clean.
  `cargo clippy --workspace -- -D warnings`: clean (one useless-conversion
  fix).
- `cargo check -p tbg-client -p tbg-core --target wasm32-unknown-unknown`:
  clean.
- Lab runs on the 19-kind preview pool (release): `duels`,
  `gate --pairs 1024` (61 rows, 17 failing),
  `run --a comp-archer-bonebrute --b bot --pairs 16`
  (`62.5 % [37.5 – 87.5]`, wide interval at 16 pairs: smoke only).
- Practice preview plays: preview pool exercised by all lab runs; the new
  units are unreachable online (`CANON.pool` still 10).

## Results vs the P42 baseline

New rows pass: purchase shares Shatterback 5.2 / Blood Furnace 3.9 /
Bloodweaver 6.5 % (all ≥ 2.5 %); favor rows Shatterback 51.4 / Blood
Furnace 49.6 % PASS, **favor-bloodweaver 39.2 % vs band 41.4–61.4 FAIL**
(a pure redistributor scores below the favor median — expected, same
shape as the Antidote row). Duels: Shatterback base 84/71/68 % (hot but
not oppressive), Blood Furnace base 77/59/52 %, Bloodweaver exempt
(Support, 10/13/11 %); most wave-C evolutions are situational in duels.

No row that passed at P42 fails at P46 (`favor-bonebrute`,
`favor-siegewalker` and `cheap` all pass on the moved bands; the P44
`cheap` violation is gone). New-row movement: `favor-farwatch` fails
again (64–65 % vs a 41–51 band), `favor-antidotebrewer` fails again;
Farwatch joined "oppressive everywhere" (now 4); `never worth` 55.8 →
69.7 % with 12 more situational forms; Undead faction-majority improved
75.8 → 56.7 % (still failing). All P50 material; no balance number changed.

## Deviations

- D1: shrapnel is a runtime-only queue field on `MatchState`
  (`#[serde(skip)]`, like `pending`): no golden impact. It drains fully
  each tick in `remove_dead` before removal; termination is structural
  (plates only decrease).
- D2: Shatterback speed 28 (0.56 t/s) stands in for the model's 0.55:
  milli-tiles are integers; ±0.01 t/s is model noise.
- D3: A2's 1.0-tile shrapnel radius is a named constant at the drain
  site (no per-form radius field exists in the plan's shape).
- D4: the per-broken-plate bonus reads `Plates::dmg_bp` (that field's
  purpose in the plan's shape); the B2 slot repeats B1's 800 (absolute
  slots, no inheritance).
- D5: Rampage overrides splash only when the unit has no other splash;
  share is full. Lifesteal can save a Deathblow (burn → resolve incl.
  lifesteal → end-of-tick death): natural order, noted, not special-cased.
- D6: transfer pair requires headroom ≥ 1 (donor stays ≥ 1 HP even after
  the A2 double); the receiver gains donor-loss minus heal cut, clamped;
  ties break by nearest, then lowest id; the weaver itself may donate.
- D7: the Furnace burn pays on every resolving hit (Strongholds
  included); the multiplier never applies vs Strongholds.
- D8: P41 catalogs do not exist on this branch (unmerged
  `phase/p41-i18n`); `tips.rs` gained §11-identity entries, `figure_spec`
  recolor arms, names fall back to content names.
- D9: `CrateOpened{unit, opened}` renames the spec's illegal `crate`
  field (carried from P43, kept).
- D10: P48-B integration points left open on purpose: betrayal hits
  (Leechroot essence exclusion, Furnace sacrifice inclusion) and the
  betrayal damage path (no situational/Arcana-conditional).

## Blocked

Nothing. Tuning (weaver/antidote favor rows, oppressive Farwatch, 69.7 %
never-worth) is P50's job with lab evidence.
