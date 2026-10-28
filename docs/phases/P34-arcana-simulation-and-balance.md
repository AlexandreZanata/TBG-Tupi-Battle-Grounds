# P34 — Arcana simulation, adversarial search and balance gate

**Depends on:** P33 reviewed; P32 combat decisions remain satisfied.
**Branch:** `phase/p34-arcana-balance`.
**Status:** planned; offline only, before any online integration.
**Difficulty:** high in interpretation, small in implementation.

## Goal and read first

Test whether six choices add strategy without a universal preset, mandatory
tree or bot-specific illusion. Reject weak hypotheses before production.
Read `AGENTS.md`, GAME-DESIGN §6/§9, DECISIONS D-25, P32/P33 reports,
`bot.rs`, `rng.rs`, Shop draw/refill and both balance examples/helpers.
Only inspect combat/stat code for a measured anomaly.

## Affected files and scope

Extend `examples/arcana_balance.rs` with small `examples/arcana_balance/`
modules for policies, metrics, pairing and statistics (each ≤200 lines).
Reuse P32 observation; compile offline examples only on native targets.
The optional tuning feature below may touch core Cargo.toml/arcanas/state.
Adjust content/GAME-DESIGN together only after evidence and the owner's
recorded rule decision; retain Proposed status until human approval.
No network, UI, persistence, optimization dependency or general AI framework.

## Steps (in order; check and commit each)

### 1. Fixed named fixtures and CLI

Use the schema order Guard, Assault, Piercing, Endurance, Resilience, Renewal:

| Name | Ranks | Meaning |
|------|-------|---------|
| melee | [2,2,2,0,0,0] | frontline offense with ranged damage |
| ranged | [2,0,2,2,0,0] | durable ranged offense |
| defensive | [2,0,0,2,2,0] | HP across all eligible roles |
| support | [2,0,0,0,2,2] | frontline plus support sustain |
| control | [0,0,0,2,2,2] | sustain/control composition, no CC bonus |
| hybrid | [1,1,1,1,1,1] | every option |
| generalist | [2,0,2,0,2,0] | candidate default |
| extreme | [0,2,2,0,0,2] | damage/healing, no HP |
| neutral | [0,0,0,0,0,0] | diagnostic control, never valid online |

Support/control are composition-policy labels, not promises of extra rules.
Provide `--smoke`, `--matrix`, `--search`, `--seeds N`, `--seed-start S`,
`--policy current|evolution|siege|sustain|all`, `--sensitivity`, `--out PATH`,
and `--help`.
Reject zero/overflowing counts and unknown flags with a useful error.
Write CSV summaries plus a manifest with HEAD, feature flags, rules/parameters, presets,
seed range, policy and invocation; keep large outputs outside git.
**Check:** smoke repeats exactly in outcomes/counts; failures exit nonzero.

### 2. Four small, reproducible policies

Keep `current` behavior unchanged. `evolution`: after securing a Tank,
reserve the next Evolution cost for the oldest owned eligible unit before
buying again; choose branch A, retaining an existing branch for tier 2.
`siege`: after securing a Tank, save for offered Siege Walker, otherwise
buy Archer; if neither is offered use current policy, not omniscient offers.
`sustain`: after securing a Tank, prioritize offered Plants to reach the
next Synergy threshold, then Hexer; use branch A and current fallback.
Share the existing seeded reaction schedule, tie-breaking and legal-view
interface. No access to enemy Energy/offers or future RNG draws.
Freeze each policy for a comparison; document its weaknesses. Compare
the same policy with different presets before comparing policy strengths.
**Check:** policies buy, save and evolve in targeted fixtures; the siege
policy actually samples expensive units and sustain samples control/healing.

### 3. Mirrored counterfactual experiments

Each sample is a pair of positions with the same participants: preserve
their bot seeds and initial Shop RNG streams/offers across the seat swap.
`MatchState::new` binds RNG streams to seats; merely moving presets does
not preserve participant randomness. Map initial player RNG/Shop state
with the participant in the native harness, before any step/intention.
Also reverse same-tick intention order in dedicated bias fixtures.

Run matched neutral controls for each policy, then the preset matrix.
Separate open-loop scripted stat fixtures from closed-loop bot decisions;
changed purchases consume RNG differently, so later offers may diverge.
Record draw counts/offer availability and time without eligible purchases;
never force weighted offers or claim identical future Shop exposure.
**Check:** swapped initial draws belong to the same participant; seeds,
positions and controls are identifiable in every output row.

### 4. Staged sample budget and metrics

Time the smoke run before scaling. Discovery uses seed-start 20000;
confirmation uses 100000 and is never used to tune parameters.
Start with 16 paired seeds, then 128 per unordered named-preset matchup.
Nine presets give 36 matchups ×128×2 = 9,216 games for one policy, or
36,864 for four matched-policy matrices. Run critical cross-policy pairs
after identifying contenders; do not mix them into the same-policy estimate.
Measure duration, endings, overtime, comeback at 120 s, Stronghold damage
and remaining HP, buys, Evolution branches, categories, source damage,
effective healing, overheal, root uptime/longest lock and Energy idle time.
Attribute the same metrics to strict faction majority/plurality separately.
**Check:** wins+losses+draws reconcile; no non-ending match, missing
controller, unsupported metric or silently omitted expensive-unit scenario.

### 5. Uncertainty and tuning

Main score is `(wins + 0.5 * draws) / games`; print draws separately.
Compute 95% intervals by deterministic bootstrap of seed-pair clusters
(10,000 resamples using a separate fixed RNG). Keep both seat games and
their neutral control in the same cluster. Report policies separately;
never apply independent-Bernoulli Wilson intervals to correlated seat games.

Screen ±1 percentage point around HP/damage/heal per-rank coefficients,
one dimension at a time (center plus six variants). Add an empty, non-default
Cargo feature `arcana-experiments` with an offline-only reason comment.
Only under that feature expose `ArcanaTuning { hp_bp: u16, damage_bp: u16,
heal_bp: u16 }` (per-rank coefficients) and
`MatchState::new_with_arcana_tuning(seed, presets, tuning)`. The ordinary
constructor still uses content constants. `--sensitivity` requires this
feature and runs exactly the seven variants. Experimental fields/APIs must
not affect default serialized state or wire layout. Never enable the feature
for an online server or distributed WASM build; no network tuning surface.
Then enumerate all 141 complete presets against melee, ranged, defensive,
support, control and generalist references: 16 paired discovery seeds
requires 141×6×16×2 = 27,072 games for one fixed reference policy.
Verify promising/abusive presets across all four policies. Inspect payoff
vectors for dominance/Pareto trade-offs; do not declare noisy near-ties
dominated. Re-run finalists with ≥512 held-out paired seeds per critical
matchup, targeting interval half-width ≤5 percentage points.
If confirmation fails, those seeds become discovery data; reserve a fresh
unused seed block before tuning again and record that block in the manifest.
**Check:** all chosen parameters have held-out evidence; otherwise mark
inconclusive. If a stage exceeds 30 minutes on the development PC, retain
completed batches and split/resume deterministically; never lower the gate
or add paid compute just to finish. Record wall time and throughput.

### 6. Gate: accept, revise or reject

Apply GAME-DESIGN §9's aggregate, matchup, default and diversity hypotheses.
Competitive aggregates exclude neutral and weight complete opponent presets
equally within each policy; compare the default to the best held-out
generalist aggregate on that same opponent pool. State all weights.
Evaluate both improvements over neutral and counterplay among complete
presets; neutral need not score 50% against positive bonuses. Broad 50%
averages do not excuse a counterless matchup. Inspect each tree's marginal
benefit via one-node ablations using partial diagnostic presets.
For any >55% aggregate or >65% matchup, record uncertainty, policy effects,
offer exposure and tested counters; unresolved advantage blocks P35.
Require at least three competitive non-dominated choices and a satisfactory
default. Human fun remains unproven even when all statistical gates pass.
Change only justified coefficients, update GAME-DESIGN and increment the
candidate rules revision in the manifest; no new nodes to rescue a failure.
If choices are imperceptible or mathematically compulsory, reject/rewrite
the experiment and revise downstream specs before continuing.
**Check:** reviewer and owner record the tuning decision; P35 cannot start
with unresolved balance or control-safety concerns.

## Verification

```bash
cargo fmt --all -- --check
cargo clippy --workspace -- -D warnings
cargo test --workspace
cargo check -p tbg-client -p tbg-core --target wasm32-unknown-unknown
cargo run --release -p tbg-core --example arcana_balance -- --smoke
cargo test -p tbg-core --features arcana-experiments
```

Run and record matrix/search/confirmation commands with actual output paths.
Example full discovery command (output is outside git):
`cargo run --release -p tbg-core --example arcana_balance -- --matrix --seeds 128 --seed-start 20000 --policy all --out /tmp/tbg-arcana-discovery.csv`.
For sensitivity use `cargo run --release -p tbg-core --features arcana-experiments --example arcana_balance -- --sensitivity`.
No benchmark or confidence interval is reported without its executed data.

## Acceptance and next-phase gate

- [ ] Named matrices, neutral ablations, policy coverage and search executed.
- [ ] Held-out cluster intervals and worst matchups support the chosen tuning.
- [ ] No unresolved compulsory choice, control lock or untested Shop dependency.
- [ ] Reviewer/owner approve the evidence; otherwise P35 remains blocked.

## Reviewer checklist

- Check paired RNG mapping, policy confounds, raw counts and held-out seeds.
- Read the worst matchup, cheapest exploit, inactive nodes and Shop droughts.
- Public behavior is still unchanged; no unapproved optimization dependency.

## Phase report

_Not executed. Fill in invocations/manifests, matrices and cluster intervals,
bot coverage/limitations, sensitivity and search results, adversarial cases,
chosen constants, measured runtime, deviations/blockers, reviewer/owner
gate decision and explicit human-validation limitations._
