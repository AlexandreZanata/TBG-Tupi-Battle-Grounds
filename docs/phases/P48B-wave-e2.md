# P48-B — Wave E2: Puppetmaster, Smuggler, Crate (executor + review report)

> No reviewer spec existed for P48-B; the work is derived from
> ROSTER-EXPANSION §G (P48-B), §C.10, §C.11, §D, §E.5.
> Depends on P44–P48-A (`phase/p48a-wave-e1` tip). Branch: `phase/p48b-wave-e2`.
> A first executor agent wrote the kinds and mechanics; a reviewer pass
> re-checked them against the spec, fixed the bugs below and added tests.

## Goal (plan §G)

Puppetmaster confusion and betrayal (§C.10), Smuggler cargo Crates with
the per-seat cargo stream (§C.11), the Crate summon, placeholders and
tips. With it every character of ROSTER-EXPANSION exists in the staging
pool (`FULL_POOL`, 25 kinds).

## Commits

- `2a9acac` feat(p48b): wave-E2 kinds, tables, betray/cargo state; goldens refrozen once
- `8cc402f` feat(p48b): puppetry, betrayal, smuggler cargo crates
- `15be173` fix(p48b): cast routing, B2 deliveries, ward and immunity, boarding betrayal
- `af0baa2` test(p48b): wave-E2 scenarios (§C.10, §C.11)
- `47c1c24` fix(p48b): wave-E2 placeholders, crate figure, cloned recolors
- (this report)

## Review findings (all fixed, each with a regression test that fails on the old code)

1. The Puppetmaster cast never fired: the healing check ran first and
   swallowed it. Cast routing now precedes healing; the dead puppet branch
   inside the heal path is gone.
2. Smuggler B2 (Double Shipment) lost B1's 18 s interval: ability rows are
   absolute per form slot, so B2 now repeats `every: 360`.
3. The Antidote ward did not block confusion (§C.9/§D); warded units are
   no longer Puppet targets.
4. Oilpress A2 (Immovable Mass) could still be confused (§C.8); control
   immunity now covers confusion.
5. A betrayal windup on an ally that boarded a Cargo Crab hit the carried
   unit; the boarding redirect now covers betrayals.
6. The confused target could differ from the one `AttackStart` showed;
   the windup target is now preferred while it stays eligible.
7. Client placeholders: seven earlier-wave figures were clones (recolor
   keys the source figure never used), and the Crate drew as a Sawbot.
   Keys were fixed, the Crate got its own box figure, and
   `art/roster_tests.rs` asserts every kind has its own figure.

## Checks (real output)

- `cargo fmt --all -- --check`: clean.
- `cargo clippy --workspace -- -D warnings`: clean.
- `cargo test -p tbg-core --features lab`: all suites ok (incl.
  `wave_e2` 15/15, `contract_lab` 3/3 after the χ² test moved to integer math).
- `cargo test --workspace`: every suite ok except `tbg-server --test online`
  (28 passed, 2 failed: `reconnect_resumes_match`, `records_insert_and_count`).
  That run shared the CPU with a 16-thread lab gate; both tests play full
  matches against wall-clock deadlines and passed in the earlier full run
  on an idle machine. Server code is unchanged on this branch.
- `cargo check -p tbg-client -p tbg-core --target wasm32-unknown-unknown`: clean.
- `trunk build --release`: ok.
- Cargo table: `summon::tests::cargo_table_holds_over_100k_crates`
  (100 000 draws, each outcome within 1 pp, χ² below the 0.01 critical value).

## Lab (release, 25-kind preview pool)

- `tbg-balance duels` (git_head 47c1c24, fingerprint 312e36b912af10ce,
  wall 97.75 s): Puppetmaster base 15 / 15 / 15 %, every form in "never
  worth"; Smuggler base 52 / 46 / 48 %. Puppetmaster joins "weak
  everywhere", like the other damage-free supports (Drumguard 19 %,
  AntidoteBrewer 17 %, Bloodweaver 19 %): solo-form duels do not reward
  support value. Tuning stays P50.
- `gate --pairs 1024` and the `favor-smuggler` / `favor-puppetmaster` /
  `comp-puppetmaster-bonebrute` rows were **not run**: the owner stopped
  them because they saturated the machine. They remain open for P50.

## Visual check (built client, desktop 1280×800, in-app browser)

- Gallery: `?gallery=puppetmaster&lineup` (violet hexer, gold glow),
  `?gallery=smuggler&lineup`, `?gallery=crate` (timber box with bands),
  and `cargocrab`, `oilpress`, `bloodweaver` now read as distinct figures.
- Practice vs Bot (`?autoplay=practice`): the bot bought a Puppetmaster;
  the match ran with 20+ units and no runtime errors. No Smuggler was
  bought in that match, so live Crates were not seen; Crate behavior is
  covered by the headless tests.
- Known gap (not new): only the ten original kinds have Shop portraits
  (`assets/battle/portraits/*.webp`, owner art); the 15 staged kinds show
  an empty card and the HUD logs a 404 per missing portrait.

## Deviations

- D1: `UNIT_KINDS` stays 10; `FULL_POOL` is now 25 kinds.
- D2: `PUPPET_IMMUNITY` is a constant (240 ticks): the betrayal resolves
  after the Puppetmaster may have died, so the Ability's `immune` field is
  carried in the table but not read.
- D3: Cargo units are summons and are never Puppet targets.
- D4: Crate drops emit no `Spawn` event; `CrateOpened` is the signal.
- D5: Faster Deliveries' `atk_bp: 1_100` only changes the displayed
  interval; the drop interval comes from the ability row.
- D6: the gallery by-name lookup now searches `ALL_KINDS`, so staged kinds
  open by name; the index lookup still uses `UNIT_KINDS`.
- D7: P41 catalogs are absent on this branch; names fall back to content
  names, tips live in `tips.rs`.
- D8: `cargo clippy --workspace --all-targets` still reports lints in
  `wave_a.rs`, `wave_b.rs`, `wave_e1.rs` from earlier waves; the required
  clippy command is clean.

## Follow-up on `release/roster-online` (D-50)

Merging `origin/main` (D-46 Stronghold kinds) and putting all 25 kinds in
the online Shop exposed one more P48-B bug: `UnitView` could not mark
Smuggler cargo as a summon, so the bot kept sending an `Evolve` the server
refuses and, online, drained its message bucket (`RateLimited`, seen in
`leaderboard::br_r07_limit_clamped_to_100`). Fixed in `72ce205` with the
spare `UnitView.fx` bit (`FX_SUMMON`) and the regression test
`cargo_reads_as_a_summon_in_the_view`. In the same merge the War Banner
aura and Tower Fury switched to the unit-level summon check.

## Blocked

Nothing. Open: the gate and the wave-E2 lab rows (resource-heavy; P50).
