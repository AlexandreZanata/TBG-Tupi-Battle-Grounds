# P48-A — Wave E1: Leechroot, Cargo Crab (executor report)

> No reviewer spec existed for P48-A; the executor derived this work from
> ROSTER-EXPANSION §F, §G (P48-A), §C.3/§C.15, §E.5, §I.
> Depends on P44–P47 (`phase/p47-wave-d` tip). Branch: `phase/p48a-wave-e1`.

## Goal (plan §G)

Essence; carrier/passenger fields, shared targeting filter, position
sync, windup redirect, dismount, A2 shield on the plate absorb path.
Tests: §C.3 and §C.15 edge cases; the targeting-filter test enumerating
every selection function (enemy units, Strongholds, echoes, betrayal,
Farwatch). Done when: as P44, plus leech-fortress,
`comp-leechroot-cargocrab`, `comp-bloodfurnace-cargocrab` rows.

## Commits

- `d02ce36` feat(p48a): wave-E1 kinds, tables, surge field; goldens refrozen once
- `1054f4a` fix(p48a): correct 65535 trace grouping
- `b027b2e` feat(p48a): essence, lifesteal form, A2 shield, crab boarding and filter
- `66f3741` feat(p48a): wave-E1 tests, placeholders and tips
- (this report; fmt/clippy fixes)

## Checks (real output)

- `cargo test -p tbg-core` (both configs): green. Goldens refrozen once
  (`surge_until` grew `UnitFx`); tick/winner/reason verified identical.
- `cargo test -p tbg-balance` (incl. extended `contract_roster`,
  `wave_e1` 10/10, `comp_policy` all-conventional pairs): green.
- `cargo test -p tbg-client`: green. `cargo test --workspace`: all suites
  ok. `cargo test -p tbg-core --features lab`: ok (incl. the P42
  taken/dealt invariant — poison-free old-kind matches).
- `cargo fmt --all -- --check`: clean.
  `cargo clippy --workspace -- -D warnings`: clean.
- `cargo check -p tbg-client -p tbg-core --target wasm32-unknown-unknown`:
  clean.
- Lab runs on the 24-kind preview pool (release): `duels`,
  `gate --pairs 1024` (69 rows, 18 failing),
  `comp-leechroot-cargocrab` (41.4 % [33.2–49.6], stalled 2.0 %),
  `comp-bloodfurnace-cargocrab` (53.1 % [44.9–61.3], stalled 1.6 %),
  `comp-leechroot-oilpress-bloodweaver` as the leech-fortress stand-in
  (37.9 % [29.7–46.1], stalled 1.6 %),
  `comp-leechroot-oilpress-drumguard` (33.6 % [25.8–41.8], stalled 2.0 %).
- Practice preview plays: preview pool exercised by all lab runs; no
  online behavior change (`CANON.pool` still 10).

## Results vs the P42 baseline

69 rows, 18 failing — the new rows pass (purchase shares Leechroot 5.2 /
Crab 4.1 % ≥ 2.5 %; favor rows 43.2 / 50.7 % inside the moved band).
Duels: Leechroot base joins "weak everywhere" (needs time, as designed);
Crab excluded from weak/oppressive lists? — recorded below.
No row that passed at P42 fails except dilution and stall: `cheap`
marginal again (55.3 % vs ≤ 55 %), Sawbot purchase share at the band
edge (2.5 % [2.4–2.5] vs ≥ 2.5 %), `favor-drumguard` 0.3 pp under its new
band; stall stays failing (1.2 %). Median duration passes again.
Undead improved to 56.7 % (was 64–76 %). Duel "never worth" 71.7 %,
branches without a niche 15 — 12 more situational forms, P50 material.

## Deviations

- D1: counts of `UNIT_KINDS` stay 10 (shop); lab iteration stays on
  `is_conventional()`; `FULL_POOL` is now 23 kinds.
- D2: Leechroot essence scales the stack by `dps × st.damage / base`
  (static bonus included, situational excluded up to truncation noise —
  verified exact for the model table in tests).
- D3: A2's 6 s ward... (no — A2's 6 s shield): implemented as a temporary
  absorb pool via the existing unit `fx.shield/shield_until` (P43
  fields), drained before plates; the 6 s / 120-tick duration and the
  15 s / 4 s Overdrive timings are named constants at the use sites.
- D4: B2 Overdrive needs one new `UnitFx.surge_until` tick (re-armed 220
  ticks after it lapses); P48-A golden refreeze covers it (no behavior
  change on old kinds, verified).
- D5: the shared carried-unit filter lives in `targeting.rs` and is used
  by enemy attacks, Stronghold fire and echoes (betrayal joins in
  P48-B); heals, cleanses, transfers and bonds intentionally bypass it.
- D6: windup redirect only converts enemy windups (same-seat heals keep
  their target).
- D7: transfer and brew share the "action unit" engage slot; no new
  PlayerState fields (the cargo stream question belongs to P48-B's
  Smuggler work, not to carried units).
- D8: `comp-*` accepts the §E.5 short name `antidote` (= AntidoteBrewer);
  all other E.5 names already match kind names exactly.
- D9: `leech-fortress` has no dedicated policy in the pool; reported as
  `comp-leechroot-oilpress-bloodweaver` (+ `…-drumguard` variant), the
  §E.5 carry-fortress compositions.
- D10: P41 catalogs do not exist on this branch (unmerged
  `phase/p41-i18n`); `tips.rs` entries and `figure_spec` recolors added,
  names fall back to content names.

## Blocked

Nothing. Betrayal interaction (Leechroot exclusion, carried targeting)
is P48-B work by construction (the betrayal path does not exist yet);
tuning stays P50.
