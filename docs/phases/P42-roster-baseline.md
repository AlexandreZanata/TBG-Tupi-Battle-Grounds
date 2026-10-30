# P42 — Roster baseline: canon rows, lab instrumentation, measured baseline

**Depends on:** `main` at `0154dd4` or later (D-45: a purchase redraws all
five Shop offers). Plan: [ROSTER-EXPANSION.md](../ROSTER-EXPANSION.md).
**Runs before:** P43 (shared foundations). Owner decisions: O-R1 (both
Shatterback and Cargo Crab), O-R2 (roster health ≥ 2.5 %), recorded as
D-48 in step 1.
**Branch:** `phase/p42-roster-baseline` (already exists on GitHub with the
plan; work on it).
**Difficulty:** low. No gameplay, protocol, server or client change.

## Goal

Before any of the fifteen new units exists, make the balance lab able to
see what the plan needs to judge them, and freeze a baseline of the
current game to compare every later phase against:

1. Canon: the planned roster is written into GAME-DESIGN as **Proposed,
   not active**; the roster-health threshold becomes 2.5 % (O-R2); D-48.
2. Per-kind combat counters and a stall measure in the lab (behind the
   existing `lab` feature only).
3. New lab rows: duration percentiles and "matches with a 60 s stall".
4. A generic `comp-<a>-<b>[-<c>]` lab policy (buys any of the named units).
5. A committed baseline on current `main`.

## Read first (only these)

- `AGENTS.md`
- `docs/ROSTER-EXPANSION.md` §A, §B.2, §E, §F.6, §G (P42), §I
- `docs/GAME-DESIGN.md` §5 (roster table), §6 (balance lab targets)
- `docs/DECISIONS.md` D-33, D-44, D-45
- Core: `crates/tbg-core/src/{lab.rs,state.rs,hits.rs,combat.rs}`,
  `crates/tbg-core/Cargo.toml` (`lab` feature)
- Lab: `crates/tbg-balance/src/{metrics.rs,runner.rs,gate.rs,policy.rs,report.rs,commands.rs}`,
  `crates/tbg-balance/tests/contract.rs`

## Do NOT

- Change gameplay, balance numbers, tick order, protocol, views, events,
  the server or the client. `PROTOCOL_VERSION` stays as it is.
- Move any golden hash or trace (`contract_determinism.rs`,
  `contract_lab.rs`). If a hash moves, the change is wrong: every new
  core field is `#[serde(skip)]` and every new core line is behind
  `#[cfg(feature = "lab")]`.
- Implement any new unit, ability, `UnitKind`, or `Ruleset` field.
- Add dependencies.
- Change any gate threshold other than the purchase share (O-R2).
- Touch `archive/`.

## Steps (in order; check and commit each)

### 1. Canon first

Files: `docs/GAME-DESIGN.md`, `docs/DECISIONS.md`, `docs/ROADMAP.md`,
`crates/tbg-balance/src/gate.rs`.

- GAME-DESIGN §6, row "Roster health": target `≥ 2.5 %` instead of
  `≥ 5 %`, with the note "(owner decision O-R2, 2026-10-29: 5 % is
  impossible with 25 units, whose even share is 4.0 %)".
- GAME-DESIGN: add a new section at the end, `## 11. Roster expansion
  (planned, not active)`: one paragraph saying fifteen units are planned
  in `ROSTER-EXPANSION.md`, every value is **Proposed, not active**, none
  is in the Shop; then copy the summary table of ROSTER-EXPANSION §B.3
  exactly (15 rows).
- DECISIONS: append `## D-48 · Roster expansion: fifteen units, staged by
  ruleset pool — 2026-10-29 (owner decision)`, with **Decision** (the 15
  units of §B.1, Shatterback and Cargo Crab both in, O-R1; roster health
  2.5 %, O-R2; units enter through the phases P42–P53), **Why** (one
  paragraph from ROSTER-EXPANSION §A.3 and §B.2), **Reopen when** (a wave
  shows a unit that numbers cannot balance).
- ROADMAP: set the P42 row's status to "in progress" (the row and the
  plan paragraph already exist on this branch).
- `gate.rs`, function `fun_rows`, the `purchase share` row: band
  `(0.05, 1.0)` becomes `(0.025, 1.0)`. Add a one-line comment citing
  O-R2. If a test in `tbg-balance/tests/contract.rs` asserts 5 % for this
  row, change it **in this same commit** (append-only rule, AGENTS.md)
  and name O-R2 in the commit message.

**Check:** `cargo test -p tbg-balance`.
**Done when:** committed (`docs(p42): roster canon, D-48, roster health 2.5 %`).

### 2. Lab counters in the core (feature-gated)

Files: `crates/tbg-core/src/lab.rs`, `state.rs`, `hits.rs`, `combat.rs`.

Add to `lab.rs` (the module is already compiled only with `lab`):

```rust
/// Per-seat, per-kind combat totals for the balance lab (P42). Never
/// serialized, so it is not part of `hash()`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LabStats {
    /// HP removed by hits of this seat's units of each kind (primary and splash).
    pub dealt: [[u64; ALL_KINDS.len()]; 2],
    /// HP removed from this seat's units of each kind, by any source.
    pub taken: [[u64; ALL_KINDS.len()]; 2],
    /// HP restored by this seat's units of each kind (direct, splash, lifesteal).
    pub healed: [[u64; ALL_KINDS.len()]; 2],
    /// Status ticks (slow or root) applied by this seat's units of each kind.
    pub status_ticks: [[u64; ALL_KINDS.len()]; 2],
    /// HP removed from units by each seat's Stronghold.
    pub stronghold_dealt: [u64; 2],
    /// Last combat tick with Stronghold damage or a conventional death.
    pub last_progress: Tick,
    /// Longest gap, in combat ticks, between two such ticks so far.
    pub longest_stall: Tick,
}
```

`Default` sets every array to zero and `last_progress` to `RUNNING_START`
(write `impl Default` by hand). Add small `pub(crate)` functions in
`lab.rs`, one per counter (`dealt`, `taken`, `healed`, `status`,
`stronghold`, `progress`). `progress(state)` does:
`longest_stall = max(longest_stall, tick − last_progress); last_progress = tick`.
Add a public `pub fn stall_at_end(&self, end: Tick) -> Tick` returning
`max(longest_stall, end − last_progress)`.

`MatchState` gets one field:

```rust
#[cfg(feature = "lab")]
#[serde(skip, default)]
pub lab: crate::lab::LabStats,
```

and `sim.rs::lab(...)` initializes it with `Default::default()` under the
same `cfg`.

Hooks: each is one line preceded by `#[cfg(feature = "lab")]`.

| Where | Hook |
|-------|------|
| `hits.rs::resolve_hit`, after the primary `damage(...)` | `dealt(state, attacker, removed)` |
| `hits.rs::splash`, each `damage(...)` on a unit | `dealt(state, attacker, removed)` (use the returned value) |
| `hits.rs::damage`, unit branch, after `removed` is known | `taken(state, owner, victim.kind, removed)` |
| `hits.rs::damage`, Stronghold branch, when `removed > 0` | `progress(state)` |
| `hits.rs::heal` | make it return the effective amount (`i32`, 0 when nothing healed); `resolve_hit` records `healed(state, attacker, amount)` for the direct heal, the splash heals and the lifesteal heal |
| `hits.rs::apply_on_hit` | make it return the applied ticks (`u16`, 0 if the unit is gone); `resolve_hit` records `status(state, attacker, ticks)` |
| `combat.rs::stronghold_fire`, after `damage(...)` | `stronghold(state, seat, removed)` |
| `combat.rs::remove_dead`, if at least one conventional unit died this tick | `progress(state)` once |

The return-type changes of `heal` and `apply_on_hit` are the only edits
outside `cfg`; they change no behavior (callers may ignore the value).

**Check:**
- `cargo test -p tbg-core` (feature off) and
  `cargo test -p tbg-core --features lab`: every golden identical.
- New test `crates/tbg-core/tests/lab_stats.rs` (compiled only with
  `#![cfg(feature = "lab")]`): on seeds 1–20, bot vs bot played with
  `tbg_core::headless::play`, for both seats
  `Σ taken[seat] == Σ dealt[other] + stronghold_dealt[other]` (every HP
  a unit loses today comes from an enemy unit or the enemy Stronghold),
  and `stall_at_end(final tick) ≤ final tick`.
- `cargo check -p tbg-client -p tbg-core --target wasm32-unknown-unknown`
  (the feature is off there; nothing must leak).

**Done when:** committed (`feat(p42): lab combat counters`).

### 3. Lab rows and per-kind report

Files: `crates/tbg-balance/src/{metrics.rs,runner.rs,gate.rs,report.rs,commands.rs}`.

- `MatchResult` gains `stats: LabStats` (copied from the state when the
  match ends, **seat order converted to participant order** with
  `participant()`) and `stall: u32` (`state.lab.stall_at_end(state.tick)`).
- `gate.rs::fun_rows` adds, over the strategic games:
  - `p90 duration (s)`, `p99 duration (s)`, `max duration (s)`: report
    rows with band `(0.0, f64::MAX)` (they never fail; ROSTER-EXPANSION §I
    compares later phases to these values);
  - `matches with a 60 s stall`: share of games whose `stall` ≥ 1 200
    ticks, band `(0.0, 0.01)` (ROSTER-EXPANSION §I).
- `gate` (and `run`) write `roster.csv` next to `gate.csv`: one line per
  kind in `ALL_KINDS` order with `kind, purchases, dealt, taken, healed,
  status_ticks, dealt_per_energy, taken_per_energy`, where the per-Energy
  columns divide by `purchases × cost` (empty when 0). Summons (Beetling)
  have no cost: leave their per-Energy columns empty.

**Check:** `cargo test -p tbg-balance`; `cargo run --release -p
tbg-balance -- gate --pairs 16` prints the four new rows and writes
`roster.csv`.
**Done when:** committed (`feat(p42): duration, stall and per-kind rows`).

### 4. `comp-*` policy

File: `crates/tbg-balance/src/policy.rs`.

- New variant `Comp([Option<UnitKind>; 3])`: the shipped bot, but it buys
  the first affordable offer that is one of the named kinds; if none is
  offered it buys the cheapest affordable offer to redraw the Shop (the
  D-44 refresh-by-buying behavior already used by `Favor`). Reuse the
  `Favor` code path; do not copy it.
- `name()`: `comp-<a>-<b>[-<c>]` with the lowercase kind names in the
  given order; `parse()` accepts 2 or 3 names, rejects unknown names,
  duplicates, and summons.
- `key()`: a new variant number; the index is the three kind indices
  packed into one `usize`.
- Not part of `pool()` or `strategic()`: it is used with `--a/--b` and by
  later phases.

**Check:** unit tests for `parse`/`name` round-trip and rejections;
`cargo run --release -p tbg-balance -- run --a comp-archer-bonebrute --b bot --pairs 16`.
**Done when:** committed (`feat(p42): comp policy`).

### 5. Baseline on current main

Run in release on the development PC and save the **text** outputs (no
large CSV) under `docs/media/roster/`:

```bash
cargo run --release -p tbg-balance -- gate --pairs 1024 --out target/balance/p42-gate
cargo run --release -p tbg-balance -- duels --out target/balance/p42-duels
cargo run --release -p tbg-balance -- run --a comp-archer-bonebrute --b bot --pairs 256 --out target/balance/p42-comp
```

Copy `gate.txt`, `duels.txt`, `roster.csv` (it is small) and each
`manifest.txt` to `docs/media/roster/baseline-*.{txt,csv}`. Run the gate a
second time with the same flags and confirm the output is byte-identical.

**Done when:** committed (`docs(p42): baseline on main`).

### 6. Phase report

Fill in the report below with real outputs and commit
(`docs(p42): phase report`). Stop and hand over for review.

## Acceptance checklist

- [ ] `cargo fmt --all -- --check`
- [ ] `cargo clippy --workspace -- -D warnings`
- [ ] `cargo test --workspace`
- [ ] `cargo test -p tbg-core --features lab`
- [ ] `cargo check -p tbg-client -p tbg-core --target wasm32-unknown-unknown`
- [ ] Every golden hash and trace unchanged (no edit to `contract_determinism.rs` or `contract_lab.rs`)
- [ ] `git diff main -- crates/tbg-core/src` shows only `cfg(feature = "lab")` code plus the two return-type changes
- [ ] Baseline files committed and reproducible byte for byte
- [ ] GAME-DESIGN §6 and §11, D-48 and ROADMAP updated

## Reviewer checklist

- Hash and trace goldens untouched; the counters cannot reach the server
  or the client (feature off there; contract C10 still passes).
- `Σ taken == Σ dealt + stronghold_dealt` test is meaningful (not
  tautological) and passes.
- The 2.5 % change is the only threshold change and cites O-R2.
- `Comp` reuses the `Favor` path.
- Baseline numbers in the report match the committed files.

## Phase report

### Status

Done. All six steps executed in order, each checked before commit. No
gameplay, protocol, server or client change. Every golden hash and trace
unchanged (no edit to `contract_determinism.rs` or `contract_lab.rs`).

Commits on `phase/p42-roster-baseline` (on top of `ba2d2e7`):

- `344214e` docs(p42): roster canon, D-48, roster health 2.5 %
- `5185de2` feat(p42): lab combat counters
- `8633c6a` feat(p42): duration, stall and per-kind rows
- `a9af5dc` feat(p42): comp policy
- `58e9220` docs(p42): baseline on main
- `484c93a` style(p42): cargo fmt

### Commands run (real output)

`cargo test -p tbg-balance` (step 1, after the 2.5 % band change):

```text
test result: ok. 6 passed; 0 failed (unit)
test result: ok. 7 passed; 0 failed (contract, 90.87 s)
test result: ok. 12 passed; 0 failed (value)
... every suite ok, 0 failed
```

`cargo test -p tbg-core` (feature off) and
`cargo test -p tbg-core --features lab` (step 2): every suite ok in both
configurations, goldens identical; new test
`lab_stats::lab_counters_balance_on_bot_vs_bot` passes (1.19 s, 20 seeds).
The first version failed on seed 1 (`taken 19805 != dealt + stronghold
24170`): the primary-hit hook also counted direct hits on Strongholds.
Fixed by recording `dealt` for unit targets only (see Deviations).

`cargo check -p tbg-client -p tbg-core --target wasm32-unknown-unknown`:

```text
Finished `dev` profile [unoptimized + debuginfo] target(s) in 5.34s
```

(no warnings; the feature is off there, nothing leaks).

`cargo run --release -p tbg-balance -- gate --pairs 16` (step 3 smoke):
the four new rows print (`p90 duration (s)`, `p99 duration (s)`,
`max duration (s)`, `matches with a 60 s stall`) and `roster.csv` is
written (11 kinds in `ALL_KINDS` order, Beetling per-Energy columns empty).

`cargo run --release -p tbg-balance -- run --a comp-archer-bonebrute
--b bot --pairs 16` (step 4 smoke):

```text
comp-archer-bonebrute vs bot: 68.8% [43.8 – 87.5] over 16 seed pairs
```

Step 5 baselines (release, 16 threads):

```text
gate --pairs 1024: 352256 games in ~165 s, ~2140 games/s
duels: wall_s 0.93
run --a comp-archer-bonebrute --b bot --pairs 256:
  comp-archer-bonebrute vs bot: 66.0% [60.2 – 71.9] over 256 seed pairs
```

Reproducibility: the gate was run twice to two output dirs; `cmp` on
`gate.txt`, `gate.csv`, `roster.csv`, `duels.txt`, `duels.csv`,
`payoff.csv` reports IDENTICAL for all six (manifests differ only in
`wall_s`, as expected).

Acceptance checklist runs (step 6):

```text
cargo fmt --all -- --check        -> FMT_OK (after one `cargo fmt --all`)
cargo clippy --workspace -- -D warnings -> Finished, no warnings
cargo test --workspace            -> 39 suites "test result: ok", 0 failed
cargo test -p tbg-core --features lab -> no failures (incl. lab_stats)
cargo check ... wasm32-unknown-unknown -> Finished, clean
```

`git diff ba2d2e7 -- crates/tbg-core/src` (non-`cfg` lines only): the two
return-type changes (`heal -> i32`, `apply_on_hit -> u16`), their
underscore bindings, the `Target::Unit` guard on the primary `dealt`
hook, and `pub mod lab`. No other behavior change.

### Baseline summary (from `docs/media/roster/`)

`gate --pairs 1024`, canon, seeds 1000000..1001024. 43 rows, 16 failing
(the plan's §A.3 counted 12 of 39; the 4 new rows add 3 passing duration
references + 1 passing stall row).

| Row | Value | Target | Pass |
|-----|-------|--------|------|
| Seat A, bot vs bot | 49.3 % [48.8–49.9] | 48.5–51.5 | PASS |
| Median duration | 107 s | 150–240 s | FAIL |
| p90 / p99 / max duration (new) | 169 / 271 / 549 s | reference only | PASS |
| Ended by Stronghold kill | 100.0 % | ≥ 60 % | PASS |
| Draws | 0.0 % | ≤ 2 % | PASS |
| Faction-majority Humans / Undead / Machines / Plants | 37.8 / 64.1 / 79.9 / 24.2 % | 45–55 % | FAIL ×4 |
| Purchase share, all 10 kinds (new band) | 5.2–16.7 % (min Nester) | ≥ 2.5 % | PASS ×10 |
| Comebacks | 26.9 % [25.9–27.9] | ≥ 20 % | PASS |
| Matches with a 60 s stall (new) | 0.6 % [0.5–0.6] | ≤ 1 % | PASS |
| Strongest strategic style (saver) | 95.7 % | ≤ 60 % | FAIL |
| heavy vs bot | 57.6 % [54.6–60.5] | ≤ 55 % | FAIL |
| cheap vs bot | 47.7 % [44.7–50.7] | ≤ 55 % | PASS |
| favor vs bot (median band 32.3–52.3) | 3 fail: archer 52.5, bonebrute 61.4, siegewalker 58.6 | median ± 10 pp | FAIL ×3 |
| Duel weak / oppressive / counterless | 2 / 3 / 0 | 0 / 0 / 0 | FAIL, FAIL, PASS |
| Duel Evolutions never worth Energy | 47.5 % | ≤ 25 % | FAIL |
| Duel branches without a niche | 2 (Archer T1, Bloomkeeper T2) | 0 | FAIL |
| Nash support / largest weight | 1 / 100.0 % (saver) | ≥ 3 / ≤ 50 % | FAIL ×2 |
| comp-archer-bonebrute vs bot (256 pairs) | 66.0 % [60.2–71.9] | n/a (baseline) | — |

Duels: weak everywhere Sawbot, Rootcaller; oppressive BoneBrute,
SiegeWalker, Nester; 19 evolved forms never worth their Energy
(full list in `baseline-duels.txt`). Roster per-Energy leaders:
SiegeWalker dealt 0.56, Archer 0.37; BoneBrute taken 0.63
(`baseline-roster.csv`). Counts: 7 665 549 Shieldbearer-scale purchases
over 352 256 games; Beetling dealt 70 444 312 with 0 purchases
(summons are never bought; per-Energy columns empty).

### Deviations

- D1: the `resolve_hit` primary `dealt` hook fires only for unit targets
  (`if let Target::Unit(_)`). The spec lists it unconditionally, but the
  spec's own invariant (Σ taken == Σ dealt + stronghold_dealt) requires
  the condition: direct hits on Strongholds have no `taken` counter. The
  first test run proved this (seed 1 mismatch quoted above).
- D2: `crates/tbg-core/src/lib.rs`: `mod lab` became `pub mod lab` so the
  spec's `pub lab` field and `pub stall_at_end` are nameable from the
  lab crate. No behavior change: the module still exists only with the
  `lab` feature, which the server and client never enable (C10 holds;
  wasm check is clean).
- D3: hook-only bindings use underscore names (`_restored`, `_applied`,
  `_removed`) so the feature-off build stays warning-free; without the
  feature the values are ignored, exactly as the spec allows.
- D4: the four new gate rows use `searched: false` (reference-only, like
  the Nash rows). They never fail by construction (`(0.0, f64::MAX)` /
  the stall band passes at 0.6 %), so the search loss is unaffected
  either way. The `f64::MAX` band prints as a long number in `gate.txt`;
  kept spec-literal.

### Blocked

Nothing. No step hit the same error twice; no `> blocked:` note needed.
