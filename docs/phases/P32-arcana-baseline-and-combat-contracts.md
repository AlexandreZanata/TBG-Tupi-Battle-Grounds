# P32 — Arcanas extra: baseline and combat contracts

**Depends on:** P31 reviewed, released, and owner checkpoints satisfied.
**Branch:** `phase/p32-arcana-baseline`.
**Status:** planned; optional extra after all P0–P31 work.
**Difficulty:** medium–high. Evidence first; no Arcana implementation.

## Goal

Establish how the released game actually works before adding strategic
modifiers. Reproduce dangerous interactions and resolve canon conflicts.
The planning-time audit is evidence to verify, never a frozen future HEAD.

## Read first (only these)

- `AGENTS.md`; `docs/ROADMAP.md` extra sequence and P31 report.
- `docs/GAME-DESIGN.md` §3–§6, §9; `docs/DECISIONS.md` D-14, D-18, D-25.
- `docs/ARCHITECTURE.md` §3–§5, §8, §10 and the Arcanas addendum.
- Core: `content.rs`, `combat.rs`, `hits.rs`, `targeting.rs`, `synergy.rs`,
  `sim.rs`, `state.rs`, `intent.rs`, `shop.rs`, `bot.rs`, `rng.rs`,
  `view.rs`, `protocol.rs`, `tests/rules.rs`, `examples/balance.rs`.
- Server: pairing, match task/report, account session ownership, store
  schema/ranked settlement; client: current unit details/status rendering.

## Scope and affected files

Core contract tests (`tests/combat_contracts.rs`, `tests/combat_control.rs`),
the existing balance example and small `examples/balance/` helpers if needed,
plus GAME-DESIGN/ARCHITECTURE/DECISIONS when a discrepancy is resolved.
Runtime combat files change only for a reproduced, approved correction.
No dependencies, Arcana state, network features, or public deployment.
Paths are landmarks: P31 may have split modules; resolve their current
locations without reviving old implementations or reading `archive/`.

## Steps (in order; check and commit each)

### 1. Record the actual starting point

Record HEAD, branch, dirty-file inventory, toolchain, protocol version,
schema migration level, and P31 sign-off. Preserve unrelated work.
Extract the current roster's cost, SlotCount, HP, damage/heal, interval,
movement, range, splash, status, both Evolution branches and Synergies.
Document Energy scale/start/cap/regen, Shop refill/rotation/RNG, spawn
positions, target ordering, slot cap, victory, overtime and forfeit rules.
For every number, identify the executed constant and matching canon rule.
**Check:** every listed value has a file/symbol reference; code/document
differences are separate rows, not silently reconciled.

Review the progression choice against the refreshed roster: five deep
trees/ten points offer breadth but overlap Evolution/Synergy; three compact
trees/six points permit exhaustive screening; mutually exclusive talents
offer clearer effects but add distinct rules/counters. Record why the small
candidate remains appropriate, or amend downstream specs before coding.

### 2. Reproduce combat contracts and risks

Add deterministic scenarios, citing existing rule IDs, for:
- Rootcaller base, Deep Roots and Overgrowth against a windup attacker,
  single and multiple casters; reverse ID/seat order. Measure longest
  uninterrupted lock, root uptime, completed attacks and movement windows.
- Slow replacement/refresh, stronger-to-weaker transitions, root priority,
  status expiry and the fact cooldown keeps ticking while rooted.
- Archer/Sawbot haste thresholds, six-tick windup and the effective minimum
  attack period; range at acquisition versus resolution of a live target.
- Nearest absolute-distance target, ties by ID, Stronghold reach, and
  Longshot outside the Stronghold firing range.
- Splash damage to clustered units and Stronghold; on-hit status only on
  the primary target; Grove healing, self-healing and Bramble priority.
- Plants integer regeneration, max-HP transitions and HP ratio retention;
  no healing of Strongholds; current absolute-HP time-up and overtime ties.
**Check:** `cargo test -p tbg-core --test combat_contracts --test combat_control`.
Characterization tests describe observed behavior; a conflict with canon
must remain explicitly labeled, never renamed into an approved rule.

### 3. Decide only demonstrated corrections

At planning HEAD `c5dd2de`, Deep Roots was 32 ticks versus 28 documented;
Overgrowth was 28 versus 24. Architecture called Energy milli-energy but
the actual scale was 600. Targeting did not enforce "ahead". Recheck all.
Recommend the smallest correction and record the owner decision where
game rules differ. Re-run the contract scenarios and baseline after it.

The conditional root-episode/immunity candidate is in GAME-DESIGN §9.
If needed, the reviewer must first amend this step with the approved exact
state fields, timer ordering, snapshot/visual indication and assertions;
then implement and verify it. Immunity after natural expiry alone cannot
solve indefinitely refreshed roots. If unresolved control lock would be
amplified by longer-lived Control units, block P33 rather than hiding it.
Do not add variable Stronghold HP, normalize equal maxima unnecessarily,
rewrite attack timing, change targeting or propagate status through splash
merely to make Arcanas easier. Unchanged intentional behavior is a valid
decision only with explicit counterplay evidence and owner approval.
**Check:** approved behavior has passing regression tests; no unresolved
competitive rule/canon conflict is inherited by the experiment.

### 4. Improve baseline observation, without changing bot decisions

Keep the old bot and seed schedule. Output exact win/draw counts, seat
results, regulation/time-up/overtime separately, successful intentions,
purchase counts/mixes, Evolution branches, unit categories, Energy cap
time, Stronghold HP/damage, healing/overheal and status-lock metrics.
Attribute effective damage (exclude overkill) and healing to their source
in an optional native-example observer; do not serialize diagnostic traces
into MatchView or allocate a telemetry object on every production tick.
Audit `Hit` events: planning-time events omit attacker identity and can
contain nominal overkill; infer nothing they cannot establish. If an
observer is necessary, document its minimal interface before adding it.
Replace the misleading faction-majority label: either enforce >50% of
purchases or print unique plurality, ties and mixed compositions separately.
Never treat purchase share or faction attribution as causal unit balance.
**Check:** event totals reconcile on hand-built damage/healing fixtures;
original bot decisions and outcomes remain equal when observation is off/on.

### 5. Measure and compare

Run `cargo run --release -p tbg-core --example balance -- 2000`.
Separate compile time from execution. Record raw counts, elapsed time and
per-match throughput. After warm-up, repeat timing over the same 256
seeds five times; report mean/p50/p95/p99/max, units alive and host load.
Use the legal maximum (currently 16 one-slot units across both players);
a 30-unit synthetic stress case is diagnostic, not a legal match.
**Check:** same build/seeds/policy reproduce outcomes; scheduling outliers
are reported and investigated, never used to claim a CPU budget passed.

## Historical measurements (not this phase's execution report)

The planning audit on 2026-10-28 used HEAD `c5dd2de`, seeds 0–1999:
2,000 matches; median/p90 149/183 s; Stronghold/time 70.7/29.4%; draws
1.2%; 7.7 Evolutions/match; max 16 living units; 26 core tests passed.
Faction unique-plurality wins: Humans 46.7%/2467 armies, Undead 45.3%/258,
Machines 60.3%/174, Plants 53.1%/565. Siege Walker purchase share 3.5%.
Other shares: Shieldbearer 21.8, Archer 15.4, Bone Brute 11.1, Hexer 8.8,
Sawbot 15.3, Bloomkeeper 15.4, Rootcaller 8.7 percent.
Step mean 3.84 us, max 1,786,883.6 us: contaminated timing, no performance
approval. A 149 s median is just below the existing 150 s lower target.
These rounded aggregates do not supply raw confidence-interval counts.

## Risk matrix to refresh in the report

Likelihood is a scenario assessment, not a measured population frequency.

| Risk | Severity | Scenario likelihood / exploitability | Mitigation |
|------|----------|--------------------------------------|------------|
| Refreshed root + cancelled windup | critical | high in fixtures / easy | reproduce, decide global counterplay before HP buffs |
| Variable Stronghold max HP + absolute time-up | critical if added | certain / trivial | no Stronghold Arcana; future cross-products first |
| Haste quantization and windup floor | high | certain thresholds / easy | omit haste; accumulator only after benefit proof |
| Extra range beyond Stronghold fire | high | existing Evolution / easy | no range Arcana; test current counterplay |
| Full splash + Machines damage | high | clustered fights / easy | source-aware effective damage and base pressure |
| Grove + HP + Plants regen | high | sustain armies / moderate | effective heal, overheal, lock/stall duration |
| Narrow build + random Shop | medium–high | frequent / inherent | offer-access metrics; no build-weighted draws |
| Bot spending and ID/seat ordering | medium–high | systematic / moderate | varied policies, participant-owned RNG, mirrored runs |

With eight practically uniform roster choices, three independent offers
miss Bloomkeeper with probability (7/8)^3 ≈67.0%, or both Ranged kinds with
(6/8)^3 ≈42.2%. These are initial-draw estimates, not conditional guarantees
after purchase/refill decisions. Recompute if the roster/draw rule changes.

## Verification and acceptance

```bash
cargo fmt --all -- --check
cargo clippy --workspace -- -D warnings
cargo test --workspace
cargo check -p tbg-client -p tbg-core --target wasm32-unknown-unknown
```

If runtime corrections touch the client, also build with Trunk, inspect a
browser, capture desktop/phone screenshots for changed status visuals,
and complete two-client/reconnect regression for multiplayer changes.
**Advance only when:** baseline, all risk reproductions, canon decisions,
required checks and owner rule decisions are recorded and reviewed.
Failure twice: follow AGENTS.md's blocked-note rule; never fabricate data.

## Reviewer checklist

- No speculative combat rewrite, new feature or Shop weighting.
- Any known root-lock issue has a decision and evidence, not a waiver by bot win rate.
- Current behavior and historical observations are clearly distinguished.

## Phase report

_Not executed. Fill in HEAD/toolchain, commands and raw results, contract
table, updated risk matrix, bot limitations, timing repetitions, approved
rule changes or explicit unchanged decisions, deviations, blockers and
reviewer/owner decisions. Link artifacts; do not commit large traces._
