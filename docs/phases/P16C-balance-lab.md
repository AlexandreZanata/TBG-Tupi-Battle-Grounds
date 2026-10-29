# P16-C — Balance lab: millions of headless matches, data-driven tuning

**Depends on:** P16-B merged (open army, five offers, Nester, Rampart).
**Runs before:** P16 (class animations are timed from simulation intervals,
so the numbers settle first). Owner decision: D-33.
**Branch:** `phase/p16c-balance-lab`.
**Difficulty:** high in method, medium in code. Steps 3 and 5 need care.

## Goal

1. Run **millions of complete matches** offline, each one provably the same
   match the server would play at real speed (1×).
2. Turn every balance number (unit cost and stats, Evolution forms,
   Synergy, Energy, Stronghold, Hatch) into **data** a lab can vary without
   touching the online game.
3. Measure balance with **many opponent styles**, unit-versus-unit
   cost-efficiency duels, paired seeds, confidence intervals, and a
   meta-game equilibrium — not one bot's opinion.
4. Search thousands of candidate rulesets and propose a **small, reviewed
   set of number changes** (first target: the P16-B heavy-spam finding).
5. Leave **contract tests that keep the lab honest forever**: they iterate
   the roster, so a new unit or a new attribute is covered automatically or
   fails to compile until it is registered.

## The key idea: faster, never different

The game already runs on a fixed 20 Hz integer tick (ARCHITECTURE §4). The
server waits 50 ms of wall clock between `step()` calls; nothing inside a
step reads time. A headless loop calls the **same `step()` back to back**.
That is not "10× speed": it is as fast as the CPU allows, with zero change
in behavior.

**Never scale the tick length** (bigger time steps or fewer ticks): that
would change movement, cooldowns, and targeting order, and the result would
no longer be the game. Speed comes only from (a) not waiting and (b) many
matches in parallel on all cores.

Measured on the development PC before this phase (P16-B rules, release
build, one thread): `balance 2000` → 2,000 complete bot matches in 2.6 s,
≈ 770 matches/s, mean step 0.32 µs. A median match is 138 s of game time,
so one core plays ≈ **100,000× real time**. With 16 cores: ≈ 1 million
matches in a few minutes, tens of millions overnight. Step 4 measures this
again with thread scaling.

Equality with 1× is proven, not assumed (step 2 and step 5):
- the simulation has no clock, float, OS randomness, or hash-ordered input
  (source guard);
- per-tick state hash traces are frozen (goldens);
- the real server code, driven by a virtual clock and by the real 50 ms
  clock, produces the same final hash as the headless replay of the
  intentions it applied;
- lab results are byte-identical for 1 or N threads and across reruns.

## Deliverable

- `Ruleset`: every tunable number as data in `tbg-core`; the canonical
  ruleset is assembled from `content.rs` and changes no golden hash.
- `crates/tbg-balance`: native-only lab binary (no new dependencies):
  `run`, `duels`, `meta`, `screen`, `search`, `confirm`, `gate`.
- Contract tests C1–C12 (step 10), generic over the roster.
- One reviewed balance proposal applied to `content.rs` and GAME-DESIGN §5
  (still **Proposed**), with held-out evidence; or an owner decision that
  numbers alone cannot fix it (step 9).
- Evidence summaries in `docs/media/p16c/` (small text files only; raw
  CSVs stay outside git).

## Read first (only these)

- `AGENTS.md`
- `docs/GAME-DESIGN.md` §4, §5, §6; `docs/DECISIONS.md` D-14, D-32, D-33
- `docs/ARCHITECTURE.md` §3, §4, §13
- `docs/phases/P16B-open-army-and-new-units.md` (phase report: balance
  finding and performance)
- `docs/phases/P34-arcana-simulation-and-balance.md` step 3 and step 5
  only (paired seeds and cluster bootstrap; reuse the method, not Arcanas)
- Core: `content.rs`, `state.rs`, `sim.rs`, `intent.rs`, `shop.rs`,
  `summon.rs`, `synergy.rs`, `combat.rs`, `hits.rs`, `targeting.rs`,
  `bot.rs`, `view.rs`, `rng.rs`, `examples/balance.rs`,
  `tests/contract_determinism.rs`, `tests/contract_roster.rs`
- Server: `src/match_task.rs`, `tests/online.rs`, `src/botclient.rs`

## Do NOT

- Change the tick rate, tick order, protocol, wire layout, or server
  behavior seen by players. `PROTOCOL_VERSION` stays as it is.
- Change any golden hash in steps 1–8. If a refactor moves a hash, the
  refactor is wrong.
- Give the online server or the WASM client any way to load a non-canonical
  ruleset (no file, flag, env var, or message). The lab is offline only.
- Add dependencies (no rayon, rand, serde_json, clap, statrs): `std`
  threads, the existing PCG32, and hand-written CSV are enough.
- Change rule mechanics (new abilities, a cost that grows with the army,
  new limits). The search varies **numbers** only; a mechanic change needs
  an owner decision first (step 9).
- Tune on the confirmation seeds, or report a number without its data.
- Write a general AI or optimization framework. Policies are a small enum;
  the search is a few dozen lines.

## Steps (in order; check and commit each)

### 1. Baseline before any change

Run, in release, and save the outputs to `docs/media/p16c/baseline-*.txt`:
`examples/balance.rs` with 20000 seeds for `bot`, `heavy`, `cheap`; the
three golden hashes; `cargo test --workspace` count. Record wall time and
matches/s. This is the "before" for every later comparison.
**Check:** files exist; numbers match a second run exactly.
**Done when:** baseline committed (`docs(p16c): baseline`).

### 2. Speed-invariance contracts (before refactoring)

File: `crates/tbg-core/tests/contract_lab.rs` (new).
- **C1 purity guard.** Read every `crates/tbg-core/src/**/*.rs` except an
  explicit allow-list (`account.rs`, `rating.rs`, `protocol.rs`, plus the
  test's own reason comment per entry) and fail on any of: `Instant`,
  `SystemTime`, `std::time`, `thread`, `sleep`, `f32`, `f64`, `HashMap`,
  `HashSet`, `thread_local`, `static mut`, `std::env`, `getrandom`.
  New simulation modules are covered automatically because the list is an
  allow-list, not a block-list of files.
- **C2 trace goldens.** `trace(seed)` folds `state.hash()` of **every
  tick** of a bot-vs-bot match (FNV, like `hash()`); freeze the trace and
  final hash for seeds 1, 7, 42, 1000, 65535. Stronger than final-hash
  goldens: a divergence that later converges is still caught.
- Move the bot-match loop from `contract_determinism.rs` into
  `tbg_core::headless` (new module, step 4 uses it):
  `pub fn play(state: &mut MatchState, decide: impl FnMut(Seat, &MatchView) -> Option<PlayerIntention>) -> MatchSummary`.
  It reproduces the server cadence exactly: intentions are decided on
  views at even ticks (10 Hz), applied before the next `step()`, events
  drained into the next view. `contract_determinism.rs` and
  `examples/balance.rs` call it; existing goldens stay identical.
**Check:** `cargo test -p tbg-core`; goldens unchanged; C1 fails when you
temporarily add `use std::time::Instant;` to `combat.rs` (then revert).
**Done when:** committed (`test(p16c): purity guard and trace goldens`).

### 3. Ruleset as data (refactor; hashes must not move)

File: `crates/tbg-core/src/ruleset.rs` (new); every simulation module.
- `pub struct Ruleset` holds every **tunable** value: Energy start, cap,
  regen (running, overtime); `EVOLVE_COST`; Stronghold HP, damage,
  interval, range; `SYNERGY_THRESHOLDS`, `SYNERGIES`; `ROTATION_TICKS`;
  Hatch values; and `units: [UnitDef; N]` with their forms.
- **Structural** values stay `const` in `content.rs` and are never tuned:
  `TICK_HZ`, `LANE_LEN`, `SPAWN_OFFSET`, `ENERGY_SCALE`, `BP`, `OFFERS`,
  `COUNTDOWN`, `REGULATION`, `OVERTIME`, `WINDUP`, `MIN_INTERVAL`.
  Document the split in the `ruleset.rs` header.
- `pub const CANON: Ruleset` is built from the existing `content.rs`
  constants and `UNITS` (each number still written once, in `content.rs`).
- `MatchState` gains `#[serde(skip, default = "Ruleset::canon")] rules:
  &'static Ruleset`. `MatchState::new(seed)` uses `&CANON`;
  `MatchState::with_rules(seed, &'static Ruleset)` is the lab entry point.
  Skipped in serialization, so `hash()` and views are unchanged.
- Every simulation read (`def`, `form_of`, `effective_stats`,
  `hatch_rule`, Energy, Stronghold, Synergy, Shop rotation) goes through
  `state.rules`. The free functions in `content.rs` stay as thin wrappers
  over `CANON` for the client UI and the server.
- `Bot::with_rules(seed, &'static Ruleset)`; `Bot::new` uses `CANON`.
- Lab rulesets are leaked once per candidate (`Box::leak`): a few KB each,
  freed at process exit. `ponytail:` comment naming this.
- `crates/tbg-core/Cargo.toml` gains an empty feature `lab` (reason comment:
  offline balance lab only) that exposes `MatchState::lab_spawn(seat, kind,
  tier, branch, x)` for duels (step 6). Nothing else is behind it.
**Check:** all goldens (C2 and `contract_determinism.rs`) byte-identical;
`examples/balance.rs 20000` output identical to the step-1 baseline;
`cargo check -p tbg-client -p tbg-core --target wasm32-unknown-unknown`.
**Done when:** committed (`refactor(p16c): ruleset as data`).

### 4. Lab crate, parallel runner, throughput

Crate `crates/tbg-balance` (workspace member, binary, depends on
`tbg-core` with feature `lab`; never a dependency of server or client).
One concern per module: `cli.rs`, `runner.rs`, `policy.rs`, `metrics.rs`,
`stats.rs`, `duels.rs`, `meta.rs`, `knobs.rs`, `search.rs`, `report.rs`.
- **Runner:** a job list `(ruleset, policy_a, policy_b, seed, swapped)`;
  `std::thread::scope` workers take the next index from an `AtomicUsize`
  and write the result into slot `index`. Output order is the job order,
  never completion order → identical bytes for any thread count.
- **Paired seeds:** every seed is played twice with seats swapped and each
  participant keeping its own Shop stream and policy seed (P34 step 3:
  `MatchState` binds RNG streams to seats, so swap the participant's seed
  mapping, not only the policy). The pair is the statistical unit.
- **Seed blocks:** discovery `0..`, screening `200_000..`, confirmation
  `1_000_000..`. Confirmation seeds are never used to choose a value; the
  manifest records every block used.
- **Manifest** per run (outside git, `--out DIR`, default
  `target/balance/<run-id>/`): git HEAD, ruleset fingerprint (FNV of the
  postcard bytes of the `Ruleset`), policies, seed blocks, thread count,
  command line, wall time, matches/s, game-seconds simulated / wall second.
- `run` command: `tbg-balance run --policy-a bot --policy-b heavy --pairs
  10000 --threads 16`.
**Check:** throughput table for 1, 2, 4, 8, 16 threads (matches/s and
speed-up versus real time) in the report; `run` output identical for
`--threads 1` and `--threads 16` (C3).
**Done when:** committed (`feat(p16c): lab runner`).

### 5. Proof against the real server (1× equivalence)

Files: `crates/tbg-server/src/match_task.rs`, `crates/tbg-server/tests/lab_equivalence.rs` (new).
- The match task records each **applied** intention as `(tick, seat,
  intention)` and the final `state.hash()`. Expose them to tests through
  the smallest path that fits the existing code (the finished-match record
  or a test-only hook); record the choice under *Deviations*. It never
  reaches clients and adds no protocol message.
- **C4a (default `cargo test`):** `#[tokio::test(start_paused = true)]`
  plays a full match through the real server (WebSocket, two bot clients)
  on a virtual clock — real server code, seconds of wall time. Replaying
  the recorded intentions with `tbg_core::headless` on the same seed must
  give the same outcome, end tick, and final hash.
- **C4b (`#[ignore]`, run once in this phase and before every release):**
  the same test with the real 50 ms clock (a full 1× match, 2–4 minutes).
  Paste its output in the report.
- Latency and reaction time change **which** intentions a player sends and
  **when**, never how the simulation resolves them; the lab models that
  with policy reaction times (step 6), not with a different simulation.
**Check:** both tests pass; a deliberately wrong replay seed fails C4a.
**Done when:** committed (`test(p16c): server and headless are the same match`).

### 6. Opponent pool and unit duels

`policy.rs`: one `enum Policy`; each variant decides from `(&MatchView,
&Ruleset)` plus its own seeded state (a `Pcg32`, or the wrapped `Bot`); no
access to enemy Energy, offers, seeds, or future draws. Every variant also has a **reaction** in views
between decisions (1 = 100 ms bot-fast, 5 = 500 ms human-like, 10 = slow).

| Policy | Behavior |
|--------|----------|
| `Bot` | current `tbg_core::bot::Bot`, unchanged (reference) |
| `Heavy` / `Cheap` | most expensive / cheapest affordable offer, never evolve (P16-B) |
| `Evolver` | secure one Tank, then evolve the oldest unit before buying |
| `Saver` | bank to the Energy cap, then spend all of it |
| `Faction(f)` | prefer offers of faction `f` until Synergy level 2, then `Bot` |
| `Mono(k)` | buy unit `k` whenever offered and affordable, else wait; evolve `k` |
| `Random` | uniformly random legal action (sanity floor; must lose clearly) |

`Faction(f)` and `Mono(k)` are generated from `FACTIONS` and `UNIT_KINDS`,
so a new faction or unit joins the pool with no code change.

`duels.rs` (feature `lab`): **Energy-equal duels** without the Shop. For
every pair of forms `(kind, tier, branch)` × `(kind, tier, branch)` and
budgets of 10, 20, and 40 Energy (Evolution cost included in the budget),
spawn each side's units at its Stronghold one per second until the budget
is spent; Strongholds do not fire and Energy does not regenerate (the duel
ruleset sets Stronghold damage and regen to 0 — no extra simulation code);
the side with units left (or more total HP after 60 s) wins. Run both seat orders. Output an N×N win matrix. It
answers, cheaply and exactly: which form is cost-inefficient everywhere
(dominated), which beats almost everything (oppressive), and whether every
form has at least one counter (GAME-DESIGN §6 "Counterplay").
**Check:** `Mono(k)` buys only `k`; `Random` loses ≥ 80 % to `Bot`;
duels are symmetric (a form against itself: 50 % ± 1 pp over both orders).
**Done when:** committed (`feat(p16c): opponent pool and duels`).

### 7. Metrics, statistics, meta-game

`metrics.rs` per match: duration, end reason, draw, Stronghold HP at 2:00,
purchases and Energy spent per kind, Evolutions per branch, damage dealt
and taken per kind, max army, Synergy levels reached.
`stats.rs`: score `(wins + 0.5 × draws) / games`; 95 % intervals by
deterministic bootstrap over seed pairs (10,000 resamples, own fixed PCG32
stream); never independent-Bernoulli intervals on paired games. Sequential
stopping: a comparison stops early once its interval half-width ≤ 1.5 pp
or it is decided against the threshold; the stop rule is in the manifest.
`meta.rs`: payoff matrix of every policy against every policy (paired
seeds), then the **symmetric Nash equilibrium** by fictitious play
(deterministic, fixed iterations). Report the equilibrium mix, its support,
and each policy's exploitability. A healthy game has several policies in
the support and no single one above 50 % weight.
`gate` command prints the table below with intervals, PASS/FAIL per row.

**Balance targets** (§6 rows are canon; the rest are **Proposed** and go
into GAME-DESIGN §6 in this step's commit, subsection "Balance lab"):

| Measure (pool: all policies except `Random`, reaction 5) | Target |
|---|---|
| Seat bias, `Bot` vs `Bot` | 50 % ± 1.5 pp |
| §6 median duration / Stronghold kills / overtime / draws | 2.5–4 min / ≥ 60 % / ≤ 30 % / ≤ 2 % |
| §6 faction-majority win rate | 45–55 % each |
| §6 purchase share per unit (pool average) | ≥ 5 % |
| §6 comebacks (behind at 2:00, then won) | ≥ 20 % |
| `Heavy` vs `Bot` and `Cheap` vs `Bot` | ≤ 55 % each |
| `Mono(k)` against the pool, every `k` | 35–60 % |
| Duel matrix: forms dominated at all three budgets | none |
| Duel matrix: forms without a counter (≥ 60 % against it at some budget) | none |
| Each Evolution branch: wins duels its sibling loses, or wins more in matches | both branches of every unit |
| Nash support / largest weight | ≥ 3 policies / ≤ 50 % |

**Check:** C9 statistics self-tests pass (step 10); `gate` on the canonical
ruleset runs end to end and its output is saved as
`docs/media/p16c/gate-canon.txt` (it is expected to FAIL rows — that is
the work list).
**Done when:** committed (`feat(p16c): metrics, statistics, meta-game, gate`).

### 8. Knobs and search

`knobs.rs`: the search space, generated from a `Ruleset`. Each knob has a
getter/setter, a canonical value, a granularity, and bounds:
unit cost (whole Energy, ±1 or ±2), HP and damage (steps of 5, ±25 %),
interval (whole ticks, ±25 %), range and speed (50 milli-tiles, ±20 %),
heal and splash, every form's bp fields (steps of 500 bp), Evolution
costs, Synergy bp, Energy regen, Stronghold HP and damage.
The registry **destructures** `Ruleset`, `UnitDef`, and `FormDef`
exhaustively (no `..`): adding a field fails to compile until the field is
registered as a knob or explicitly marked untunable (C5).
**Role invariants** (Proposed, GAME-DESIGN §5 note): no unit range ≥
Stronghold range (P7 decision); cost ≥ 1 Energy; interval ≥
`MIN_INTERVAL`; each Archetype keeps its role — Tank has the most HP per
Energy of its faction, Ranged the longest range of its faction, Support
keeps a heal. Candidates breaking one are discarded before any match.

`search.rs`, staged, all deterministic from `--search-seed`:
1. **Screen** (`screen`): every knob at −10 % and +10 % (or ±1 step),
   2,000 pairs each, common seeds. Rank knobs by effect on the loss below.
2. **Search** (`search`): a (1+λ) evolution strategy over the top 24
   knobs, λ = 16, each child changing 1–3 knobs by one or two steps.
   **Common random numbers**: every candidate in a generation plays the
   same seed pairs, so differences come from the ruleset, not luck.
   **Successive halving**: 256 pairs for all children, the best quarter
   to 1,024, the best to 4,096. Stop after 60 generations or when the gate
   passes on discovery seeds.
3. **Confirm** (`confirm`): the best three candidates on 20,000 fresh
   confirmation pairs, with `Random`-free full pool, every reaction time,
   and duels. A candidate that fails becomes discovery data; take a new
   seed block before searching again.

Loss = Σ over gate rows of the squared distance outside the target band
(scaled to percentage points) + 0.5 × number of changed knobs + Σ
|relative change|. Small, readable proposals win over large ones; a
proposal changes at most 8 numbers.
Output `proposal.md`: knob, old → new, the gate rows it moved, intervals
before/after; plus the exact `content.rs` lines to change.
Budget (measured in step 4, estimate here): one generation ≈ 16 × 256 ×
2 + 4 × 1,024 × 2 + 4,096 × 2 ≈ 24,500 matches ≈ 3 s on 16 threads; 60
generations ≈ 1.5 M matches. If any stage exceeds 30 minutes on the
development PC, split it into resumable batches by seed block; never lower
a gate or rent compute to finish.
**Check:** `search` with the same flags twice gives the same proposal;
C7 canaries pass (step 10).
**Done when:** committed (`feat(p16c): knobs and search`).

### 9. Apply with evidence (owner checkpoint)

1. Run `screen`, `search`, `confirm` from the canonical ruleset. Save
   summaries (`gate-before.txt`, `proposal.md`, `gate-after.txt`,
   `confirm.txt`, throughput) in `docs/media/p16c/`.
2. Present the proposal to the owner with the P16-B heavy-spam finding
   first. If numbers alone cannot bring `Heavy` vs `Bot` ≤ 55 % without
   failing another row, **stop** and ask the owner for a mechanic decision
   (for example P16-B candidate 2, a cost that grows with the army),
   recorded as a new DECISIONS entry before any mechanic code. That
   mechanic would be a new phase, not part of this one.
3. On the owner's yes, apply the accepted numbers to `content.rs` and the
   GAME-DESIGN §5 tables (**Proposed**, with a note "P16-C lab evidence,
   <date>") and update `contract_roster.rs`, the goldens (C2 and
   `contract_determinism.rs`) **in the same commit**. Re-run `gate` on
   confirmation seeds and paste the result.
**Check:** `gate` after the change passes every row it was meant to fix
and fails no row that passed before (or the report says why).
**Done when:** committed (`balance(p16c): <short summary>`).

### 10. Contract tests that outlive this phase

These guard the lab itself. They iterate `UNIT_KINDS`, `FACTIONS`, forms,
and knobs, never named units, so new content is covered automatically.
Fast versions run in `cargo test` (debug, total ≤ 60 s; if slower, add
`[profile.test.package.tbg-core] opt-level = 2` with a reason comment);
heavy versions run through `tbg-balance gate`.

| ID | Contract | Where |
|----|----------|-------|
| C1 | Simulation sources stay free of clocks, floats, OS randomness, hash-ordered containers (allow-list) | core `contract_lab.rs` |
| C2 | Per-tick trace goldens and final-hash goldens | core `contract_lab.rs`, `contract_determinism.rs` |
| C3 | Lab output is byte-identical for 1 and N threads and across reruns | balance `tests/contract.rs` |
| C4 | Server (virtual clock; `#[ignore]` real 1× clock) equals headless replay | server `lab_equivalence.rs` |
| C5 | Every `Ruleset`/`UnitDef`/`FormDef` field is a knob or marked untunable (compile-time); `CANON` equals the `content.rs` values; every shop unit has knobs, a `Mono` policy, and duel rows | balance + core |
| C6 | For every unit kind, a ×3 damage variant changes a duel result: the simulation reads the ruleset, not a hidden constant | balance |
| C7 | **Canaries:** for every unit kind, ×2 HP and ×2 damage makes the lab flag it oppressive; ×3 cost makes it flag dominated. The lab must catch a planted imbalance in any unit, including future ones | balance |
| C8 | **Null tests:** a ruleset where every unit has identical stats gives a flat duel matrix (50 % ± 2 pp) and flags nothing; `Bot` vs `Bot` seat bias within target | balance |
| C9 | Statistics self-tests: bootstrap interval covers a known rate; fictitious play gives ⅓/⅓/⅓ on rock-paper-scissors and 0 weight to a dominated strategy | balance |
| C10 | Isolation: `tbg-server` and `tbg-client` manifests never enable `tbg-core/lab`; no non-canonical ruleset is reachable from server or client code | core (scans `Cargo.toml` files) |
| C11 | Every policy only produces intentions the server accepts or rejects cleanly; none panics over 1,000 random seeds | balance |
| C12 | `gate` on `CANON` passes every row marked required in GAME-DESIGN §6 "Balance lab" (release run; reviewers run it on every balance commit) | `tbg-balance gate` |

**Immutability rule** (write it into AGENTS.md "Tests" in this commit):
files named `contract_*` and the lab's `tests/contract.rs` are append-only.
A test in them may change only in the same commit as the canon change it
guards (GAME-DESIGN or DECISIONS), and the commit message names that
change. Thresholds live in one table (`gate` targets) mirrored in
GAME-DESIGN §6; tuning a threshold is a product decision, not a test fix.
**Check:** temporarily add an eleventh unit copied from Archer with ×3
damage on a scratch branch: C5 forces its registration, C7 flags it,
`gate` fails. Record this drill in the report; do not commit it.
**Done when:** committed (`test(p16c): balance lab contracts`).

## Verification

```bash
cargo fmt --all -- --check
cargo clippy --workspace -- -D warnings
cargo test --workspace
cargo check -p tbg-client -p tbg-core --target wasm32-unknown-unknown
cargo test -p tbg-server --test lab_equivalence -- --ignored   # 1× real clock, once
cargo run --release -p tbg-balance -- gate --pairs 20000 --threads 16
```

Client change: none expected; if `content.rs` numbers changed, run
`trunk build --release` in `crates/tbg-client/` and play one practice match
in the browser (costs and stats on cards match the new values).

## Acceptance checklist

- [ ] All verification commands pass; goldens unchanged through step 8
- [ ] Throughput table (1–16 threads) and speed-up versus real time reported
- [ ] C4a and C4b pass: the real server at 1× and the lab replay agree
- [ ] Duel matrix, payoff matrix, Nash mix, and gate saved in `docs/media/p16c/`
- [ ] A proposal confirmed on held-out seeds and applied, or an owner
      decision recorded that a mechanic change is needed
- [ ] C1–C12 pass; the eleventh-unit drill recorded
- [ ] GAME-DESIGN §5/§6, DECISIONS D-33, ARCHITECTURE §3/§13, AGENTS.md, and
      the roadmap updated

## Risks and what to do

| Problem | Action |
|---------|--------|
| Lab balances the bots, not the game | Many policies and reaction times, Nash mix instead of one matchup, held-out policies in `confirm`; final values stay **Proposed** until the owner's playtest |
| Overfitting to lucky seeds | Common seeds for comparison, successive halving, fresh confirmation block, cluster bootstrap |
| Refactor silently changes the game | Trace goldens and baseline output must match exactly before step 4 |
| Search proposes many tiny changes | Loss penalty per changed knob, at most 8 changes per proposal |
| Numbers cannot fix heavy spam | Stop at step 9.2 and ask the owner; no mechanic code in this phase |
| Tests too slow in debug | Fast versions in `cargo test`, heavy ones in `gate` (release) |

## Reviewer checklist

- Re-run C4b (real clock) and `gate` yourself; compare with the report.
- `git diff` of `contract_*` files: every change tied to a canon change.
- Confirmation seeds never appear in a `screen`/`search` manifest.
- No ruleset loading path in server or client; `lab` feature only in
  `tbg-balance`.
- Proposal numbers equal `content.rs`, GAME-DESIGN §5, and
  `contract_roster.rs`.

## Later (not in this phase)

- P34 (Arcanas) reuses this lab: paired seeds, policies, bootstrap, gate,
  instead of a separate example.
- Calibrate policies against real ranked matches (purchase and win shares
  from the server's match records) once enough exist.
- Rule-mechanic experiments behind lab-only ruleset fields, after an owner
  decision.

## Phase report

Executed 2026-10-29 by Claude (executor and self-reviewer) on
`phase/p16c-balance-lab`, branched from P16-B (`15f914d`). An independent
review agent read the diff afterwards; its two real findings (a late
intention after the outcome was not recorded for the replay; a worker
panic would shift parallel results) and two minor ones are fixed in
`fix(p16c): review findings`.

### What exists now

- **`tbg-core`**: `Ruleset` (`ruleset.rs`; `CANON` assembled from
  `content.rs`, skipped in serialization), `MatchState::with_rules`,
  `Bot::with_rules`, `headless::play` (the server loop without a clock),
  `replay::Replay`, feature `lab` with `lab_spawn` (`lab.rs`).
- **`crates/tbg-balance`** (no new dependencies): `run`, `duels`, `meta`,
  `gate`, `throughput`, `screen`, `search`, `confirm`. Modules: `cli`,
  `commands`, `runner`, `policy`, `metrics`, `stats`, `duels`, `meta`,
  `gate`, `knobs`, `search`, `report`.
- **`tbg-server`**: `Config::replays` (tests only, never from the
  environment) records applied intentions and forfeits per tick plus the
  final hash.
- **Contracts**: core `contract_lab.rs` (C1, C2, C10), server
  `lab_equivalence.rs` (C4a, C4b), balance `tests/contract.rs` (C3, C5–C9,
  C11). C12 is `tbg-balance gate --strict`.

### Step results

1. **Baseline** (`docs/media/p16c/baseline-*.txt`, 20,000 matches each,
   second run identical except timing): bot vs bot median 136 s, seat A
   50.0 %; heavy vs bot 63.1 %; cheap vs bot 41.6 %; 23–25 s per 20,000
   matches on one thread.
2. **Speed invariance**: C1 allow-list scan passes (and fails when
   `use std::time::Instant;` is added to `combat.rs`); C2 per-tick traces
   for seeds 1, 7, 42, 1000, 65535 frozen; the existing final-hash goldens
   unchanged.
3. **Ruleset as data**: every golden byte-identical; the three 20,000-match
   baselines reproduced line for line (`diff` shows only the timing line).
4. **Throughput** (`docs/media/p16c/throughput.txt`, bot vs bot, idle PC):

   | threads | games/s | game-seconds per wall-second |
   |---|---|---|
   | 1 | 800 | 111,809× |
   | 2 | 1,596 | 223,108× |
   | 4 | 2,944 | 411,591× |
   | 8 | 5,013 | 700,848× |
   | 16 | 5,644 | 789,164× |

   One million games ≈ 3 minutes on this PC. The full gate (344,000
   games + 7,350 duels) took 141 s while a search shared the CPU. Every
   thread count gives identical results (checked by `throughput` itself
   and by C3).
5. **1× proof**: C4a (real WebSocket bot match at a 5 ms tick) and C4b
   (production 50 ms tick, `--ignored`) both replay headless to the
   server's final hash and outcome; a wrong seed does not.
   C4b output: `tick 50 ms: 38 commands, end tick 2365, outcome
   Some(Outcome { winner: Some(B), reason: StrongholdDestroyed }), hash
   a0852a9cebc2849c` — `ok`, 118.42 s.
   Re-run after the review fix: `tick 50 ms: 31 commands, end tick 1777,
   ... hash 23be0711733de518` — `ok`, 88.83 s. Final `cargo test --workspace`:
   121 passed, 0 failed, 1 ignored (C4b).
6. **Pool and duels**: policies `bot`, `heavy`, `cheap`, `evolver`,
   `saver`, `faction-*` ×4, `favor-*` ×10, `random`; duels over 50 forms
   × 3 budgets × both seat orders (14,700 duels, 0.15 s).
7. **Gate on the canonical ruleset** (`docs/media/p16c/gate-canon.txt`,
   confirmation seeds 1,000,000.., 1,000 pairs, 344,000 games): 40 rows,
   18 failing. See *Findings*.
8. **Search** and 9. **Apply**: see *Search and confirmation*.
10. **Contracts**: all pass; debug `cargo test -p tbg-balance` ≈ 33 s.
    **Eleventh-unit drill** (scratch worktree, not committed): an Archer
    copy with ×3 damage ("Longarm") needed only its `UnitKind`, roster,
    and content rows — the lab registered its knobs, `favor-longarm`
    policy, and five duel forms by itself; duels flagged it oppressive;
    `favor-longarm` scored 83.6 % [80.4–86.8] against `bot`; all C3–C11
    ran on it and passed; the determinism goldens failed as they must.
    Adding a field `armor` to `UnitDef` stopped `tbg-balance` from
    compiling (`E0027: pattern does not mention field armor` in
    `knobs.rs`) until registered.

### Findings (canonical P16-B rules)

Confirmation seeds, 4,096 pairs (1.4 M games per ruleset), 95 % intervals:

1. **Banking beats trickling.** `saver` (wait for 10 Energy, then buy the
   most expensive offers at once) averages 92.8 % against the other
   strategic styles and beats `bot` 95.5–96.0 % at 100 ms, 500 ms, and
   1 s reactions (500 pairs each); the Nash mix is 100 % `saver`.
   Concentrated waves arrive together and win every local fight; any
   human can do this. This is a mechanic-level finding (Lanchester), not
   a single bad number.
2. **Machines dominate, Plants and Humans lag.** Faction-majority score:
   Machines 85.2 %, Undead 48.8 %, Humans 35.5 %, Plants 21.5 %.
   Siege Walker: duel base form 96–100 %, `favor-siegewalker` 66.7 %;
   `favor-archer` 66.3 %.
3. **Evolution is overpriced** (confirms P16-B): 70 % of evolved forms
   lose to their own base form Energy for Energy; `evolver` scores 0.1 %
   against the Nash mix. Heavy spam beats `bot` 62.8 % [61.3–64.2].
4. **Duels:** oppressive base forms Bone Brute, Siege Walker, Nester; weak
   base forms Sawbot, Rootcaller; Hexer tier 1 has a branch never better
   than its sibling. No form is counterless.
5. Comebacks 6.4 % (target ≥ 20 %), median 118 s among strategic styles
   (target 150–240 s). Seat bias is clean: 50.0 % [49.8–50.3].

### Search and confirmation

`tbg-balance search --pairs 1024 --generations 30 --lambda 16 --top 24`
(screen: 257 variants on the screening block; 30 generations on the
discovery block; 2,531 s): discovery loss 5,765 → 2,264 with 8 changes
(`docs/media/p16c/proposal.md`, `search.log`):

| Knob | Canon | Proposed |
|---|---|---|
| Evolution cost tier 1 | 3 Energy | 2 Energy |
| Stronghold damage | 65 | 53 |
| Shieldbearer cost | 3 | 4 |
| Hexer damage | 30 | 29 |
| Sawbot cost / HP | 3 / 270 | 4 / 231 |
| Siege Walker speed | 0.60 | 0.66 tiles/s |
| Rampart cost | 5 | 6 |

`confirm --pairs 4096` on the untouched confirmation block
(`docs/media/p16c/confirm.txt`, 851 s), canon → proposal:
Machines 85.2 → 71.8 %, Humans 35.5 → 44.0 %, Plants 21.5 → 28.7 %,
Undead 48.8 → 56.8 % (now above band), strongest strategic style 92.8 %
(`saver`) → 78.8 % (`faction-machines`), heavy vs bot 62.8 → 59.7 %
(58.4–60.7 % across reactions), Evolutions never worth their Energy
70 → 47.5 %, comebacks 6.4 → 9.0 %, `favor-archer` 66.3 → 58.7 %,
`favor-bonebrute` 52.9 → 60.1 %, branches without a niche 1 → 2. Failing
rows 18 → 18; the Nash mix moves from 100 % `saver` to 100 %
`faction-machines`.

**Decision per step 9.2: not applied.** The confirmed proposal is a real
improvement on held-out seeds, but numbers alone do not bring heavy spam
to ≤ 55 % or give the meta-game more than one equilibrium style; the
required rows still fail. `content.rs` and GAME-DESIGN §5 are unchanged,
so every golden stays valid. **Owner decisions needed:** (a) apply the
8-number proposal as Proposed values for playtest
(`--apply evolve_cost.0=1200,stronghold_damage=53,Shieldbearer.cost=2400,Hexer.damage=29,Sawbot.cost=2400,Sawbot.hp=231,SiegeWalker.speed=33,Rampart.cost=3600`);
(b) a mechanic against banked waves and spam (options to evaluate in a
new phase: a lower Energy cap, a per-seat deploy cooldown, or a cost that
grows with the living army); (c) whether the gate's Proposed thresholds
are right. C12 (`gate --strict`) fails on CANON until then.

### Deviations

- `Mono(k)` became `Favor(k)` (the shipped bot, but always buying `k`):
  a one-unit army scored 0 % for 8 of 10 units, measuring the policy's
  handicap, not the unit. Its row is relative (median ± 10 pp).
- Duel budgets are 20/40/60 Energy, not 10/20/40: at 10 Energy integer
  unit counts wasted up to 40 % of the budget for evolved forms.
- Duel weak/oppressive flags compare base forms with base forms; evolved
  forms are judged against their own base form (new row "Evolutions never
  worth their Energy", ≤ 25 %). Support is exempt from "weak". A form is
  "counterless" only if it also wins on average (a flat roster would
  otherwise flag every form).
- New searched row "strongest strategic style ≤ 60 %": the Nash rows need
  the full pool and are gate-only, and the search needed a cheap proxy.
- C4a uses the real server at a 5 ms tick (the existing test convention)
  instead of tokio's paused clock: paused time auto-advances while
  sockets wait and would fire client timeouts. 5 ms and 50 ms both equal
  the headless replay, which is the stronger statement.
- Per-kind damage dealt/taken is not measured: `Event::Hit` carries no
  attacker; adding one would change the wire. Purchases, Evolutions per
  branch, army size, and the 2:00 checkpoint are measured.
- C7 canaries: "flagged, or the base form's average moves ≥ 20 pp at
  every budget" (a ×3 cost does not make a strong unit weak, only
  average). Pure healers (no damage) use a match canary (`favor-<unit>`
  vs `bot` +10 pp, 24 pairs).
- Knob steps: 5 % of the canonical value (±25 %), 500 bp for form
  percentages (±4 steps), whole Energy within ±25 % for costs (one step
  for every current cost, after a smoke search proposed halving the
  Nester's cost).
- Reaction time: non-`bot` policies decide every 5 views (500 ms);
  `favor-*` falls back to the bot at the bot's own pace.
- Fixed a pre-existing race in `tests/accounts.rs`
  (`br_p04_account_messages_refused_in_match`): under full CPU load a
  `View` arrived before `AccountError`.

### Limitations

- Bots are not humans. A passing gate is necessary, never sufficient;
  every value stays **Proposed** until the owner's playtest.
- The lab varies numbers only. Mechanic options (for example a cost that
  grows with the army, or a deploy cooldown against banking) need an
  owner decision and their own phase.
- Duels have no Shop or Stronghold fire; they measure cost efficiency,
  not the whole game.
