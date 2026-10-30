# P42 — Roster baseline: canon rows, lab instrumentation, measured baseline

**Depends on:** `main` at `0154dd4` or later (D-45: a purchase redraws all
five Shop offers). Plan: [ROSTER-EXPANSION.md](../ROSTER-EXPANSION.md).
**Runs before:** P43 (shared foundations). Owner decisions: O-R1 (both
Shatterback and Cargo Crab), O-R2 (roster health ≥ 2.5 %), recorded as
D-46 in step 1.
**Branch:** `phase/p42-roster-baseline` (already exists on GitHub with the
plan; work on it).
**Difficulty:** low. No gameplay, protocol, server or client change.

## Goal

Before any of the fifteen new units exists, make the balance lab able to
see what the plan needs to judge them, and freeze a baseline of the
current game to compare every later phase against:

1. Canon: the planned roster is written into GAME-DESIGN as **Proposed,
   not active**; the roster-health threshold becomes 2.5 % (O-R2); D-46.
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
- DECISIONS: append `## D-46 · Roster expansion: fifteen units, staged by
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
**Done when:** committed (`docs(p42): roster canon, D-46, roster health 2.5 %`).

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
- [ ] GAME-DESIGN §6 and §11, D-46 and ROADMAP updated

## Reviewer checklist

- Hash and trace goldens untouched; the counters cannot reach the server
  or the client (feature off there; contract C10 still passes).
- `Σ taken == Σ dealt + stronghold_dealt` test is meaningful (not
  tautological) and passes.
- The 2.5 % change is the only threshold change and cites O-R2.
- `Comp` reuses the `Favor` path.
- Baseline numbers in the report match the committed files.

## Phase report

*(executor fills this in)*

### Status

### Commands run (real output)

### Baseline summary (from `docs/media/roster/`)

| Row | Value | Target | Pass |
|-----|-------|--------|------|

### Deviations

### Blocked
