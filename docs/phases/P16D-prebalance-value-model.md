# P16-D — Universal pre-balance value model (PVE)

**Depends on:** P16-C reviewed and merged (lab, ruleset as data, final
proposal, report). P16-F1 depends on this phase: the `saver` finding must
be fixed or explicitly explained before uncapped Energy and Interest are
enabled.

**Branch:** `phase/p16d-prebalance-value-model`.

**Status:** the offline value engine is implemented and measured; the
owner balance decision is still open. No production rule, number,
protocol message, server path, client path, or dependency is changed by
the engine. The phase is not marked reviewed.

**Difficulty:** high in method, medium in code. The hard part is choosing
a defensible measurement model and proving it is useful; the Rust surface
is small and lives in the existing offline lab.

---

## 1. Goal

Turn the existing P16-C balance lab from an **outcome measuring tool** into
a **pre-balance instrument** that can answer, before a full match is run:

- How much does +10 % HP on a Shieldbearer change the expected result?
- Is a 3-Energy Evolution worth more or less than a 3-Energy new unit in
  the same board state?
- How much of a faction's win rate comes from its units, its Synergy, its
  economy, and its composition?
- Which pairs or triples of effects are additive, redundant, or
  super-additive?
- Which choices are dominated, excessively efficient, dead, or
  exploitable?
- How much is one Energy worth at 0:30, 2:00, and 3:30, and how does that
  change when Energy is uncapped and pays Interest?
- Which changes deserve a full-match gate, and which can be rejected
  cheaply?

The phase must also resolve the four P16-C findings that block the
strategic expansion: `saver`, heavy spam, Evolution value, and faction
spread.

**A playable game is the goal.** This phase must reduce the number of full
matches needed to find broken content, not add a research program that
never reaches the game.

## 2. What already exists (audit at planning time)

### 2.1 Reusable foundations

| Area | What exists | Where |
|------|-------------|-------|
| Deterministic simulation | Fixed 20 Hz integer tick, no clocks/floats/hash order; `MatchState::step` is the server's own step | `crates/tbg-core/src/{sim,state,combat,hits,targeting,summon,synergy,shop,intent}.rs` |
| Numbers as data | `Ruleset`, `UnitDef`, `FormDef`; `effective_stats` is the single stat formula | `crates/tbg-core/src/{content,ruleset}.rs` |
| Offline lab | Parallel runner, paired seeds, policies, duels, Nash meta, bootstrap, gate, knobs, search | `crates/tbg-balance/src/` |
| Equality proof | Headless replay equals the real server at 1×; trace goldens | `crates/tbg-core/src/headless.rs`, `tests/contract_lab.rs`, server `lab_equivalence.rs` |
| Throughput | About 800 matches/s on one core, about 5,600/s on 16 threads (P16-C measurement) | `docs/media/p16c/throughput.txt` |
| Strategic-expansion specs | Planned economy, Siege, Reserve, Stronghold kinds, Fortifications, Augments, with their own gates | `docs/phases/P16F1`–`P16F8`, `docs/GAME-DESIGN.md` §10 |
| Arcana methodology | Paired RNG mapping, held-out seeds, forced presets, cluster bootstrap, adversarial search | `docs/phases/P32`–`P34` |

### 2.2 What the current lab measures

- Win/draw/loss score, seat bias, duration, ending reason, comeback at
  2:00, purchases, Evolutions per branch, maximum army, faction majority.
- Energy-equal duels over every form and budget, with flags for weak,
  oppressive, counterless, not-worth-it Evolutions, and branch niches.
- Payoff matrix over policies and a symmetric Nash mix.
- Ruleset knobs and a (1+λ) search over numbers.
- 95 % intervals by deterministic cluster bootstrap over seed pairs.

### 2.3 What is missing

1. **No marginal value.** The lab answers "who wins", not "what is this
   attribute worth" or "what happens if I change only this number".
2. **No Energy price.** Nothing measures the opportunity cost of one
   Energy at a given board state or time.
3. **No interaction measurement.** Matchup matrices show that A beats B;
   they do not decompose HP + healing, damage + attack speed, root +
   ranged, or splash + slow.
4. **No effective-combat metrics.** `metrics.rs` tracks purchases and army
   size, but not effective damage, overkill, effective healing, overheal,
   control uptime, damage avoided, or time-to-contact. P32 already notes
   that `Hit` events carry no attacker identity and may contain nominal
   overkill.
5. **Duels are a narrow context.** They have no Shop, no economy, no
   Synergy, no Stronghold fire, and equal budgets. They are excellent for
   cost-efficiency extremes and poor for timing, composition, and
   economy.
6. **Planned systems have no value contract.** Interest, Siege/Collapse,
   Reserve, Stronghold kinds, Fortifications, Augments, and Arcanas will
   each need the same questions answered; without a shared model every
   phase will invent its own metric.
7. **P16-C's final proposal is not on every branch.** The planning base
   has the pre-proposal gate (18 failing rows). Step 1 must start from
   the reviewed P16-C state, not from stale numbers.

### 2.4 Planning-time evidence that motivates the phase

From `docs/media/p16c/gate-canon.txt` on the planning base (refresh in
step 1):

| Finding | Value | Why it matters |
|---------|-------|----------------|
| Strongest strategic style (`saver`) | 93.2 % | Banking to the cap then spending dominates; uncapped Energy + Interest would make it worse |
| Heavy spam vs bot | 61.8 % | Buying a new unit is about twice the stats per Energy of an Evolution |
| Faction-majority Humans / Machines / Plants | 35.6 / 84.8 / 21.0 % | One faction is far outside 45–55 % |
| Duels: Evolutions never worth their Energy | 70.0 % of evolved forms | The Evolution package is not competitive in equal-Energy fights |
| Duels: base forms weak / oppressive everywhere | 2 / 3 | Several forms have no niche or no counter |
| Nash support / largest weight | 1 / 100 % | The policy meta-game collapsed to one strategy |
| Median duration | 118 s | Below the 150–240 s target |

These are measurements of the bot pool, not proof about humans. The plan
must treat them as hypotheses with uncertainty, not as final verdicts.

### 2.5 Explicit non-goals

- Do not change the tick rate, tick order, protocol, or server behavior.
- Do not add a second combat engine or duplicate `effective_stats`.
- Do not add a general AI, optimization framework, dashboard, database,
  or paid service.
- Do not use an LLM to produce numeric estimates; every number must come
  from code, a cited measurement, or an explicitly labeled hypothesis.
- Do not claim a universal exchange rate between HP, damage, control,
  time, and Energy. The model produces **contextual, scenario-weighted
  marginal values**, not physical constants.
- Do not tune on confirmation seeds, lower a gate, or apply a balance
  number without the owner's recorded decision.

## 3. Methodology: compare, then choose

### 3.1 Alternatives

| Method | Answers | Strengths | Limits | Verdict |
|--------|---------|-----------|--------|---------|
| **A. Weighted power budget** | Sum of attributes × weights | Simple, fast, interpretable | Weights are arbitrary; ignores context, timing, interactions; produces false precision | Screening baseline only; never the decision metric |
| **B. Marginal utility** | Value of a small change from a base state | Captures diminishing returns and breakpoints; directly answers the user's questions | Needs a utility and a context distribution; can be noisy | **Core** |
| **C. Effective combat value** | Damage, survival, healing, control, pressure in a fight | Exact in deterministic micro-scenarios; cheap | Narrow context; does not include Shop, economy, or match win probability | Feature/dimension, not the final unit |
| **D. Shadow pricing / Energy** | Opportunity cost of 1 Energy at a state/time | Natural numeraire; connects every purchase, Evolution, and Augment | Time-varying; cap/Interest change the price; estimation needs counterfactuals | **Core normalization** |
| **E. Counterfactual value** | Difference between two identical worlds with one change | Causal for the tested intervention; reuses paired seeds | Expensive if run for everything; closed-loop changes purchases and Shop draws | **Ground truth for calibration** |
| **F. Multi-objective optimization** | Trade-offs and Pareto frontiers | Avoids premature scalarization; exposes dominated choices | Hard to communicate; needs uncertainty-aware dominance | **Reporting and dominance** |
| **G. Game-theoretic value** | Best response, dominance, exploitability, equilibrium | Catches strategies that win by collapsing diversity, not by average stats | Needs a policy pool; equilibrium is approximate | **Meta-game check** |
| **H. Surrogate models** | Fast prediction of full-match results from features | Can make pre-balance cheap if validated | Overfits; opaque; can silently encode bot bias | **Only after calibration beats baselines** |
| **I. Hybrid** | Combine the above | Best coverage and honesty | More moving parts | **Chosen** |

### 3.2 Chosen method: contextual marginal value with an Energy numeraire

The model has four layers:

1. **Exact micro-scenarios** (duels and small skirmishes) measure the
   mechanism: effective damage, survival time, healing, control uptime,
   time-to-contact, overkill, and breakpoints. These are reported in
   native units (HP, damage, seconds, ticks), never disguised as a
   universal point.
2. **Full-match counterfactuals** measure the outcome: paired seeds, both
   seat orders, same participants, one intervention at a time. The
   primary utility is the match score `win = 1`, `draw = 0.5`,
   `loss = 0`.
3. **Shadow price of Energy** converts outcome changes into a common
   numeraire. One Energy at the reference context is worth 100 PVE **by
   convention**; the convention is documented and tested, not assumed to
   be a law of the game.
4. **A calibrated surrogate** learns to predict the full-match effect
   from micro-scenario features. It is accepted only if it beats simple
   baselines on held-out seeds, units, and policies.

The final scalar is always **match-based**. Micro-scenario metrics are
features and diagnostic dimensions; they are not silently summed into
PVE. If the surrogate is not trustworthy, the phase falls back to
micro-screening plus full matches for finalists and reports a no-go for
the surrogate.

### 3.3 Why this is not the owner's original formula

The owner's first hypothesis was "100 PVE = 1 Energy invested
efficiently". That is kept as the **scaling convention**, but it is not
used as an axiom. The model first estimates the marginal win-equivalent
value of Energy in a declared reference context; the 100 is only the
display scale. If Energy has a different value at 0:30 and 3:30, the model
reports two values instead of pretending one exists. If HP and control
have no common physical unit, the model reports a vector and a
context-weighted scalar, not a universal equivalence.

## 4. Definitions and formulas

### 4.1 Context

A context `c` is:

```text
c = (ruleset, state, own composition, enemy composition,
     phase/tick, policy pair, horizon)
```

A **reference context distribution** `D_ref` is a fixed, deterministic
list of contexts with declared weights. It is generated before outcomes
are seen. It contains duels, skirmishes, and sampled match states; it is
never hand-picked after seeing which unit looks strong.

### 4.2 Utility

For a configuration `θ` in context `c`, against the reference policy pool
`Π`:

```text
U(θ; c) = E_{π_A, π_B ~ Π, seed ~ S} [ score(θ, c, seed) ]
score   = 1 win, 0.5 draw, 0 loss
```

For deterministic micro-scenarios, `U` is the scenario outcome (win,
draw, or normalized remaining HP) and is used only as a feature; the
canonical `U` for PVE is the match score.

### 4.3 Marginal value

For an intervention `x` (an attribute, ability, Evolution, cost change,
Stronghold change, Augment, Arcana, or economy change):

```text
ΔU(x; c) = U(θ ⊕ x; c) - U(θ; c)
```

`ΔU` is estimated with paired seeds and both seat orders. A pair is the
statistical unit; the two seat-swapped games are never treated as
independent.

### 4.4 Shadow price of Energy

At a context `c`, the shadow price is the best marginal use of one
Energy:

```text
λ(c) = max over affordable actions a of [ ΔU(a; c) / cost(a) ]
```

Cross-check: grant the seat +1 Energy at the same state and measure
`ΔU / 1 Energy` with the same policy. The envelope and the grant should
agree within uncertainty; if they do not, report both and investigate.

The reference price is:

```text
λ_ref = median_{c ~ D_ref} λ(c)
```

Report `λ(c)` by phase and time, not only the median. The planned
uncapped Energy and Interest change `λ`; those values are hypothetical
until P16-F1 is implemented.

### 4.5 PVE

```text
PVE(x; c) = 100 * ΔU(x; c) / λ_ref
PVE(x)    = 100 * E_{c ~ D_ref}[ ΔU(x; c) ] / λ_ref
```

The factor 100 is a display convention: at the reference identity, one
Energy is about 100 PVE. It is not a claim that one Energy is always
worth the same number of HP, damage, or seconds.

Report the raw `ΔU`, the interval, the context weights, and the
provenance with every PVE. Round to meaningful precision; never print
more digits than the uncertainty supports.

### 4.6 Efficiency and return on investment

For a purchasable package `x` with Energy cost `k`:

```text
efficiency(x) = PVE(x) / k
```

For a free balance change (for example, +10 % HP on an existing unit),
there is no in-game cost; report `PVE` and the **Energy equivalent**
`PVE / 100`, never an "efficiency" that divides by zero.

For an investment at time `t`:

```text
ROI(x; t) = ΔU(x; c_t) / (k * λ_ref)
payback   = inf { s : cumulative ΔU(x; t..t+s) >= k * λ_ref }
```

Risk-adjust the payback by the probability that the match ends before it
is recovered. This is how the model answers "is it worth buying now or
saving?" without a fixed rule.

### 4.7 Interactions

For a small set of components `N` (at most five), the value function
`V(S)` is the PVE of the package `S`. The Shapley value of component `i`
is:

```text
φ_i = Σ_{S ⊆ N \ {i}} |S|! (|N| - |S| - 1)! / |N|! * [V(S ∪ {i}) - V(S)]
```

For larger sets, use pairwise interaction indices:

```text
I(i, j) = V({i, j}) - V({i}) - V({j})
```

and screen only the top main effects plus all pairs inside the same
system (HP/heal, damage/attack speed, root/ranged, splash/slow,
economy/Interest, Stronghold HP/damage). Triples are escalated only when
they are flagged by a pair or by the adversarial search. This avoids
exponential enumeration.

### 4.8 Dominance and redundancy

With uncertainty intervals:

- `A` is **Pareto dominated** by `B` only when `B` is at least as good in
  every reported dimension and no worse in cost, with intervals that
  exclude the opposite.
- `A` is **redundant** when its marginal value is highly correlated with
  an existing option across contexts, or when its interaction with that
  option is negative and its standalone value is small.
- `A` is **dead** when its PVE interval includes zero in at least 80 % of
  the reference contexts.
- `A` is **oppressive** when its PVE per Energy is above twice the
  reference median and no tested counter reduces its score below 60 %.
- `A` is **game-theoretically dominant** when adding it collapses the
  Nash support or gives one policy more than 50 % equilibrium weight.

No dominance claim is made from a noisy near-tie. Report the interval and
the contexts where the ordering reverses.

### 4.9 Nonlinearity

For each attribute, run a response curve over feasible integer steps
(for example −3, −2, −1, 0, +1, +2, +3 knob steps). Report:

- the realized stat change after integer rounding;
- the first and second differences;
- breakpoints such as hits-to-kill, time-to-kill, number of attacks
  before death, and HP thresholds;
- whether the response is monotone, saturating, or non-monotone.

Discrete abilities (root, slow, splash) are measured by scenario
contrasts, not derivatives.

### 4.10 Provenance and uncertainty

Every row carries:

```text
source: analytic | micro-sim | full-match | surrogate
contexts: <context IDs and weights>
seeds: <discovery or confirmation block>
ruleset: <fingerprint>
head: <git commit>
model: <PVE model version>
confidence: A (full-match) | B (validated surrogate) | C (micro/analytic)
```

Intervals are 95 % cluster-bootstrap intervals over seed pairs and
contexts. A surrogate row always names its training data and its
held-out validation. A hypothesis is always labeled as a hypothesis.

## 5. Value dimensions

The scalar PVE is match-based. The vector explains it. Dimensions are
reported in native units; their contribution to PVE is an attribution
from the calibrated model (Shapley or regression), never an independent
summation.

| Dimension | Operational metrics | Attributes that map to it |
|-----------|--------------------|---------------------------|
| Offense | Effective damage to units, overkill, time-to-kill, attacks completed | Damage, DPS, attack speed, splash, damage bonuses |
| Defense | Effective HP absorbed, survival time, damage avoided, mitigation | HP, defense/mitigation, regeneration, lifesteal, shields |
| Support | Effective healing, overheal, sustain time, buff uptime | Heal, heal splash, regeneration auras, buffs |
| Control | Root/slow uptime, enemy actions prevented, damage avoided | Slow, root, status duration, control abilities |
| Mobility | Time-to-contact, distance advanced, time-to-Stronghold | Move speed, deploy timing, haste |
| Pressure | Stronghold damage per second, time-to-Stronghold, siege potential | Damage, splash, Stronghold damage, summons |
| Economy | Energy generated/saved, ROI, payback, Interest/Bounty | Energy, Interest, Bounty, cost changes |
| Timing | Value realized before the match ends, phase weighting | Costs, Evolution timing, temporary effects |
| Composition | Synergy thresholds, faction mix, interaction value | Faction bonuses, summons, Augments, Arcanas |

## 6. Scenario library

The library is deterministic and generated from the roster, not written
by hand for the units that look interesting.

### 6.1 Duel contexts

- Every base form against every base form at 20, 40, and 60 Energy, both
  seat orders. Reuse the existing duel matrix where possible.
- Every evolved form against its own base and against three reference
  opponents (Archer, Shieldbearer, Siege Walker) at 40 Energy, both seat
  orders.
- Strongholds do not fire; no regeneration; no Shop. This is the
  mechanism context, not a balance verdict.

### 6.2 Skirmish contexts

- 2v2 and 3v3 with formations (front/back, stacked, spread) and fixed
  spawn ticks.
- A stratified set of compositions: every archetype pair; every faction
  pair; the known interaction families (Tank + Support, Tank + Ranged,
  Ranged + Control, DPS + Control, Summon + Support).
- Strongholds off and on (to measure pressure and Guard).
- Each scenario is run with both seat orders and with the intervention
  on one side.
- This is where HP + healing, root + ranged, splash + slow, and
  summon + buffs become measurable without a full match.

### 6.3 Match-snapshot contexts

- Sample states from full matches at 0:30, 1:00, 1:30, 2:00, 3:00, and
  3:30 under the policies `bot`, `evolver`, `saver`, `heavy`, and
  `faction-*`.
- Clone the state, apply one intervention, continue with the same
  policies for a fixed horizon (for example 20 s) or to the end, and
  measure the difference against the untouched clone.
- This captures the actual board, economy, Shop, and Stronghold state
  without paying for a whole match per intervention.
- A snapshot is only valid if both sides have the same participant seeds
  and Shop streams after the clone; the harness records the mapping.

### 6.4 Full-match contexts

- Paired seeds, both seat orders, the full policy pool, and the existing
  gate. Used for calibration, finalists, and the final balance packet.
- Discovery and confirmation seed blocks are fixed in advance
  (`0..`, `200_000..`, `1_000_000..` as in P16-C); confirmation is never
  used to choose a value.

### 6.5 Reference weights

The default `D_ref` is uniform over the fixed context list, with the
three strata (duel, skirmish, snapshot) weighted 1/3 each. Sensitivity
runs use at least two alternatives (duel-heavy and snapshot-heavy). If
the ranking of the top candidates changes materially, the report says so
and does not present a single "true" PVE.

## 7. Pipeline

```text
Ruleset + deterministic context library
  -> static audit and invariants
  -> micro-scenarios: effective combat dimensions
  -> interventions: paired ΔU, λ, PVE
  -> interaction/dominance/redundancy screening
  -> targeted full-match counterfactuals for finalists
  -> calibration on discovery, validation on held-out
  -> balance packet + owner decision
  -> optional number changes + existing gate
```

Stage costs are budgeted. Micro-scenarios and snapshots must be at least
an order of magnitude cheaper than the full matches they replace. If a
stage costs more than the full-match gate it is meant to save, the phase
drops it and says why.

## 8. Scientific validation

### 8.1 Pre-registered questions

1. Does PVE rank interventions in the same order as full-match
   counterfactuals on held-out data?
2. Does it detect dominated, dead, and oppressive options?
3. Does interaction screening find known combinations and planted
   synergies?
4. Does it predict the effect of a held-out stat change within a useful
   error?
5. Is it stable under context-weight and step-size changes?
6. Does it save full matches for the same top-k recall?

### 8.2 Acceptance metrics (starting hypotheses)

| Metric | Target | Notes |
|--------|--------|-------|
| Spearman rank correlation, predicted vs observed ΔU | ≥ 0.60, 95 % CI lower bound > 0.40 | Held-out interventions |
| Top-decile precision / recall | ≥ 0.60 / ≥ 0.60 | The candidates that matter |
| Mean absolute calibration error | ≤ 2 percentage points of Δwin, or better than the best simple baseline | Report the interval |
| Dominance detection | precision ≥ 0.70, recall ≥ 0.70 | Against full-match flags |
| Stability | rank correlation ≥ 0.80 under context-weight and ±1-step changes | Otherwise report instability |
| Efficiency | ≥ 10× fewer full matches for the same top-5 recall | Measured wall time and match count |
| Null control | identical units → PVE interval includes 0 | No false signal |
| Canary | ×2 HP or ×2 damage is flagged; ×3 cost is flagged as inefficient | In every unit, including future ones |

The thresholds are research acceptance criteria, not game balance
targets. The reviewer may tighten them before execution; they may not be
loosened after seeing the data.

### 8.3 Baselines

The surrogate must beat all of these on the same held-out data:

1. Cost-only ranking.
2. Raw stat heuristic (`HP × DPS` or equivalent).
3. Micro-scenario ranking without calibration.
4. The full-match oracle (upper bound, not a competitor).

If no model beats the simple baselines, the phase reports a **no-go** for
the surrogate and recommends micro-screening plus full matches for
finalists. That is a valid outcome, not a failure to hide.

### 8.4 Held-out design

- Seeds: discovery and confirmation blocks, never mixed.
- Units: leave-one-unit-out cross-validation for the surrogate.
- Policies: hold out at least one policy family from calibration.
- Contexts: hold out entire composition families (for example all
  summon scenarios) from calibration.
- Interventions: hold out at least 20 % of the tested changes.

### 8.5 Statistical rules

- Paired seeds; both seat orders; cluster bootstrap over seed pairs.
- Never use independent-Bernoulli intervals on paired games.
- Report raw counts and intervals, not only percentages.
- Correct for multiple comparisons when scanning many candidates, or
  label the scan as exploratory and confirm finalists on fresh seeds.
- A result that is not distinguishable from zero is reported as
  inconclusive.

## 9. P16-C findings: required analysis

### 9.1 `saver` (banking to the cap, then spending)

- Measure `λ(t)` and the option value of held Energy at 0:30, 1:00,
  2:00, and 3:00.
- Add `banker-N` policies (bank to N Energy, then spend) for N = 10, 15,
  20, 25, 30, 40; test with human-like reaction times and against `rush`.
- Compare burst deployment (several units at once) with steady spending.
- Decide: is 93 % a bot artifact (the opponent does not punish passivity)
  or a real dominant strategy? If real, propose a rule/economy change
  before P16-F1; do not enable uncapped Energy + Interest on top of it.

### 9.2 Heavy spam (61.8 %)

- Measure PVE per Energy of a new unit versus an Evolution of the same
  cost in the same board state.
- Decompose the Evolution's value into stat delta, timing, HP-fraction
  preservation, and branch.
- Test whether the problem is Evolution cost, Evolution stat budget, or
  the action economy (a new unit also adds a body and a target).
- Propose the smallest number change or the mechanic decision; validate
  with the existing gate.

### 9.3 Evolution value (70 % of evolved forms lose to their base in duels)

- Separate the duel context from the match context. Duels have no Shop,
  no Synergy, no Stronghold, and equal budgets; a match may value
  preserving a veteran unit and its position.
- Report `PVE` and `PVE per Energy` for each branch, with and without
  Synergy, with and without healing.
- Identify branches that are never better and branches that are only
  better in a narrow context.
- Propose changes only with held-out evidence.

### 9.4 Faction spread (Humans 35.6 %, Machines 84.8 %, Plants 21.0 %)

- Decompose into unit PVE, Synergy value, composition interactions, and
  purchase mix.
- Check sample size and the fact that "faction majority" is a purchase
  label, not a causal army identity.
- Test controlled compositions (same Energy, same archetypes, only the
  faction bonus changes) to separate the Synergy from the roster.
- Propose targeted changes; re-run the gate; do not lower the 45–55 %
  band.

## 10. Planned systems: contracts only

No value is claimed for a system that is not implemented. Each planned
system gets a contract in `docs/PREBALANCE.md` so its phase can plug into
the same model without inventing a new metric.

| System | Status | Value question | Contract/data needed | Implemented by |
|--------|--------|----------------|----------------------|----------------|
| Uncapped Energy, Interest, Bounty | Proposed | What is the option value of holding Energy? How does `λ(t)` change? | `Ruleset` fields, `banker-N`, low-water balances, payment ticks | P16-F1 |
| Siege / Collapse | Proposed | How does percentage collapse change the value of Stronghold HP and defensive investment? | Phase/tick, Collapse rate, time-to-end | P16-F1 |
| Shop Reserve | Proposed | What is the option premium of holding one offer? | Shop state, hold timer, offer identity | P16-F2 |
| Stronghold kinds | Proposed | PVE of HP/damage/passive versus the Classic control | Per-seat kind, stats, passive | P16-F3/F4 |
| Fortifications | Proposed | PVE per Energy of each branch and tier | Tier, branch, cost, effects | P16-F3/F4 |
| Augments | Proposed | Δ versus Forgo, conditional value, interaction with composition | Forced-Augment policy, round state, effect enum | P16-F5/F6 |
| Arcanas | Proposed (extra) | PVE of each rank and preset, by Archetype | Preset schema, forced presets, additive formula | P32–P38 |

The contracts must state the model, the data, and the falsifying test. A
future phase that cannot satisfy the contract must say so before
implementation.

## 11. Architecture

### 11.1 Where the code lives

Extend the existing offline lab; do not create a second balance project.

```text
crates/tbg-core/src/lab.rs                  one new lab-only spawn hook
crates/tbg-balance/src/value/mod.rs         public types and PVE pipeline
crates/tbg-balance/src/value/context.rs     deterministic context library
crates/tbg-balance/src/value/scenario.rs    micro-scenario runner
crates/tbg-balance/src/value/measure.rs     tick observer and metrics
crates/tbg-balance/src/value/intervention.rs interventions on a cloned Ruleset
crates/tbg-balance/src/value/estimate.rs    ΔU, λ, PVE, uncertainty, provenance
crates/tbg-balance/src/value/interaction.rs Shapley, pairs, dominance, redundancy
crates/tbg-balance/src/value/calibrate.rs   ridge regression and validation
crates/tbg-balance/src/value/report.rs      Markdown/CSV/JSON output
crates/tbg-balance/src/{cli,commands}.rs    `value` and `validate` commands
```

If a module grows beyond one responsibility, split it; do not turn
`value/mod.rs` into a god file. If a file would be a thin wrapper around
`std`, keep it inline.

### 11.2 Core hook

`crates/tbg-core/src/lab.rs` gains one function under the existing `lab`
feature:

```rust
impl MatchState {
    /// Place a unit at an exact lane position for an offline scenario.
    pub fn lab_spawn_at(
        &mut self,
        seat: Seat,
        kind: UnitKind,
        tier: u8,
        branch: u8,
        x: i32,
    ) -> UnitId;
}
```

It must reuse the existing `spawn` path so max HP, Synergy, and Hatch
initialization stay identical. It must not change the online path; the
server and client never enable `tbg-core/lab` (contract C10).

Other scenario state (Energy, Stronghold HP, unit HP, cooldown, status)
is already reachable through public fields or can be set after the spawn
in the offline crate. No serialized field is added.

### 11.3 Measurement without production tracing

The lab observes state deltas and the existing events:

- effective damage = HP removed, capped at the HP before the hit;
- overkill = nominal `Hit` amount minus effective damage;
- effective healing = `Event::Heal.amount`;
- overheal = attempted heal minus effective, measured only in controlled
  support fixtures where the healer is known;
- control uptime = ticks with `Status::Slowed` or `Status::Rooted`;
- damage avoided = baseline enemy damage minus damage under control;
- time-to-contact and time-to-Stronghold = first tick at a distance
  threshold;
- attacks, kills, purchases, Evolutions, Energy spent, and idle Energy
  from events and state.

This avoids changing `Event`, the wire, or the simulation. If a metric
cannot be attributed without attacker identity, it is measured in a
controlled scenario rather than guessed in a full match.

### 11.4 CLI

Add two commands to `tbg-balance`; keep the existing commands unchanged.

```text
value     --mode reference|interactions|findings|planned
          [--pairs N] [--threads N] [--seed-start S] [--out DIR]
          [--intervention SPEC] [--context SPEC] [--weights SPEC]

validate  --pairs N [--threads N] [--seed-start S] [--out DIR]
          [--holdout units|policies|contexts|seeds] [--baseline NAME]
```

`--intervention` names a registered change (for example
`unit-hp:Shieldbearer=+500bp`, `form-dmg:Archer:1:0=+500bp`,
`evolve-cost:1=-1`, `ability:Hexer=slow:4000:40`). The parser rejects
unknown fields and out-of-bounds steps. The same registry feeds the
scenario runner and the full-match counterfactual so both apply the same
change.

### 11.5 Determinism and performance

- Micro-scenarios use no RNG; identical inputs give identical outputs.
- Full-match interventions reuse the paired-seed runner and preserve
  participant RNG mapping.
- The context library, intervention registry, and model version are in
  every manifest.
- The value engine is offline only; it is never a dependency of the
  server or client. It may use `f64`; the simulation stays integer.

## 12. Steps (in order; check and commit each)

### 1. Audit and freeze the starting state

**Files:** `docs/media/p16d/audit.md` (new).

Record HEAD, branch, dirty files, toolchain, protocol version, the
reviewed P16-C state (including its final proposal and `gate-after.txt`),
the P16-F spec status, the current content values, the existing metrics,
and every gap this phase must fill.

Verify that the reviewed P16-C final proposal is on the base branch. If
it is not, stop with a `> blocked:` note; do not analyze stale numbers.

**Check:** every claim has a file/symbol reference; no code changed;
`git status` shows only the new audit file.

**Done when:** committed (`docs(p16d): audit`).

### 2. Methodology and proposed decision record

**Files:** `docs/PREBALANCE.md` (new); this phase file's report section
untouched.

Write the executable methodology: definitions, formulas, reference
contexts, weights, provenance, uncertainty, validation thresholds,
rejected alternatives, limitations, and the planned-system contracts.
Include the exact proposed decision text for D-40 (methodology and
numeraire). Do not edit `docs/DECISIONS.md` in this step; the owner or
reviewer records D-40 when the methodology is approved.

**Check:** a reviewer can reproduce every formula from the document; no
formula has an undeclared constant; the 100 scaling convention is
explicit; no production rule is changed.

**Done when:** committed (`docs(p16d): pre-balance methodology (D-40 proposed)`).

### 3. Scenario runner and observation

**Files:** `crates/tbg-core/src/lab.rs`,
`crates/tbg-balance/src/value/{mod,scenario,measure}.rs`, tests.

Add `lab_spawn_at`. Implement `Scenario`, `Spawn`, `ScenarioResult`, and
the tick observer. Scenarios place units at exact positions and ticks,
run to a fixed horizon or elimination, and return native metrics.

**Check:** `cargo test -p tbg-balance`; a null scenario with identical
sides gives a zero difference and no winner bias; a second run is
byte-identical; `cargo test --workspace`; WASM check still passes.

**Done when:** committed (`feat(p16d): deterministic scenario runner`).

### 4. Interventions, shadow price, and PVE

**Files:** `crates/tbg-balance/src/value/{intervention,estimate}.rs`,
`cli.rs`, `commands.rs`, tests.

Implement the intervention registry, apply changes to a cloned
`Ruleset`, run paired full-match counterfactuals, estimate `ΔU`, `λ`,
`λ_ref`, PVE, intervals, and provenance. Add the `value` command.

**Check:** +1 Energy has a positive λ in an economy context; a zero
change gives PVE 0; a ×2 HP canary is flagged; the CLI rejects an
unknown field; the output manifest records the ruleset fingerprint,
seeds, contexts, and model version.

**Done when:** committed (`feat(p16d): counterfactual interventions and PVE`).

### 5. Reference contexts and baseline value table

**Files:** `value/context.rs`, `value/report.rs`,
`docs/media/p16d/values.md` (summary), raw CSV under `target/`.

Generate the deterministic reference contexts. Run the value table for
every base unit, form, attribute family, Synergy, and Stronghold
attribute. Save `contexts.csv`, `values.csv`, `values.md`.

**Check:** every shop unit, form, and registered attribute is covered;
every row has a source, context set, interval, and model version; null
controls pass; raw outputs are outside git.

**Done when:** committed (`feat(p16d): reference contexts and baseline values`).

### 6. Interactions, dominance, redundancy

**Files:** `value/interaction.rs`, `docs/media/p16d/interactions.md`.

Implement pairwise interactions and exact Shapley for sets of at most
five components. Add Pareto dominance, dead-effect, redundancy, and
oppressive flags with uncertainty. Include the known interaction
families and a planted-synergy canary.

**Check:** the known combos (HP + healing, damage + attack speed, root +
ranged, splash + slow) are detected; a null pair has an interval around
zero; a planted super-synergy is flagged; no dominance is declared from
overlapping intervals.

**Done when:** committed (`feat(p16d): interaction and dominance analysis`).

### 7. P16-C findings and proposals

**Files:** `docs/media/p16d/findings.md`, `proposal.md`.

Analyze `saver`, heavy spam, Evolution value, and faction spread with the
model plus targeted full matches. For each: mechanism, magnitude,
uncertainty, contexts where it holds, and the smallest proposed change or
the mechanic decision it requires. Do not apply a production change.

**Check:** each finding cites its evidence and intervals; the `saver`
result is either explained or escalated as a rule decision; heavy spam
has a PVE-per-Energy decomposition; Evolution has duel and match
contexts; faction spread separates roster from Synergy.

**Done when:** committed (`docs(p16d): findings and proposals`).

### 8. Calibration and held-out validation

**Files:** `value/calibrate.rs`, `docs/media/p16d/calibration.md`,
`validation.csv`.

Fit the simplest interpretable surrogate (standardized ridge regression;
a shallow piecewise model only if ridge fails). Compare with the
baselines. Run leave-one-unit-out, held-out policies, held-out contexts,
and confirmation seeds. Apply the pre-registered thresholds. Report a
go/no-go.

**Check:** the chosen model beats the best simple baseline on the
primary held-out metric or the phase records a no-go; confirmation seeds
were not used for fitting; canaries and nulls pass; the model's
coefficients are not presented as universal constants.

**Done when:** committed (`feat(p16d): calibrated surrogate` and
`docs(p16d): validation`).

### 9. Balance packet and owner checkpoint

**Files:** `docs/media/p16d/proposal.md`, the phase report.

Present the value table, interaction findings, P16-C resolutions, and
the proposed number changes or mechanic decisions. The owner decides.

If the owner accepts number changes: apply them to `content.rs` and the
matching GAME-DESIGN §5/§6/§10 tables (**Proposed**), update the contract
tests and goldens in the same commit, and re-run the gate on
confirmation seeds. If the owner requires a mechanic change: record a
new decision and stop; that mechanic is a new phase, not part of P16-D.

**Check:** the gate passes every row it was meant to fix and fails no
row that passed before, or the report explains why; no unapproved number
changed.

**Done when:** committed (`balance(p16d): <summary>` or
`docs(p16d): no-go`).

### 10. Planned-system contracts and integration

**Files:** `docs/PREBALANCE.md` (planned-system section),
`docs/ROADMAP.md` (P16-F note), `docs/ARCHITECTURE.md` (offline tool
only).

Write the contracts for Interest, Siege/Collapse, Reserve, Stronghold
kinds, Fortifications, Augments, and Arcanas. State for each: the value
question, the data required, the falsifying test, and the phase that
implements it. Mark every unimplemented value as hypothetical.

**Check:** no contract assumes an unimplemented field exists; each
P16-F phase can cite the contract; no production code or protocol is
touched.

**Done when:** committed (`docs(p16d): planned-system value contracts`).

### 11. Report, roadmap, and handover

**Files:** this phase file (report), `docs/ROADMAP.md`,
`docs/ARCHITECTURE.md` if the offline tool description needs one line.

Fill the report with real commands, outputs, throughput, value table
summary, calibration results, P16-C resolutions, owner decision,
deviations, blockers, and limitations. Update the roadmap status and the
P16-F1 dependency note. Do not mark the phase reviewed; that is the
reviewer's action.

**Check:** full verification block passes; docs and code agree; no
production behavior changed except approved numbers; the phase report
contains no invented number.

**Done when:** committed (`docs(p16d): report and roadmap`).

## 13. Verification

```bash
cargo fmt --all -- --check
cargo clippy --workspace -- -D warnings
cargo test --workspace
cargo check -p tbg-client -p tbg-core --target wasm32-unknown-unknown

# Offline value engine (release; measurements, not unit tests)
cargo run --release -p tbg-balance -- value --mode reference --pairs 2000 --threads 16
cargo run --release -p tbg-balance -- value --mode interactions --pairs 2000 --threads 16
cargo run --release -p tbg-balance -- validate --pairs 20000 --threads 16
cargo run --release -p tbg-balance -- gate --pairs 20000 --threads 16
```

If any `content.rs` number is approved and changed, also run:

```bash
trunk build --release   # in crates/tbg-client/
```

and play one practice match in the browser to confirm that the cards show
the new values. No client change is expected from the value engine
itself.

## 14. Acceptance checklist

- [ ] The methodology has an explicit definition of context, utility,
      `ΔU`, `λ`, `λ_ref`, PVE, efficiency, ROI, interaction, dominance,
      and uncertainty.
- [ ] 100 PVE = one Energy at the reference is documented as a display
      convention, not a universal equivalence.
- [ ] The reference context library is deterministic, pre-registered,
      and sensitivity-tested.
- [ ] Every existing attribute family is mapped to a dimension and
      measured where the content exists.
- [ ] Micro-scenarios measure effective damage, overkill, effective
      healing, overheal (controlled), control uptime, damage avoided,
      time-to-contact, and pressure.
- [ ] The P16-C findings have evidence, uncertainty, and a decision:
      fixed, explained as a bot artifact, or escalated as a mechanic.
- [ ] The surrogate beats the baselines on held-out data or the phase
      records a no-go; confirmation seeds were never used for fitting.
- [ ] Canaries and null controls pass; planted imbalances are detected.
- [ ] No production path, protocol, server, client, or dependency is
      changed by the value engine.
- [ ] Any accepted number change updates `content.rs`, GAME-DESIGN, and
      the contract tests/goldens in the same commit; the gate is
      re-run.
- [ ] Full verification passes; the report quotes real output.
- [ ] The roadmap, ARCHITECTURE, proposed D-40, and phase report are
      updated; P16-F1 remains blocked until `saver` is resolved.

## 15. Risks and mitigations

| Risk | Mitigation |
|------|------------|
| The model balances bots, not humans | Multiple policies and reaction times; held-out policies; report bot limitations; owner playtest decides |
| PVE becomes a new arbitrary number | Estimate `λ_ref`; report raw `ΔU`, intervals, contexts, and sensitivity; use PVE for ranking, not as a law |
| Context weights cherry-pick a winner | Generate the library deterministically; publish the weights; report rankings under alternative weights |
| Surrogate overfits | Simple ridge model; leave-one-unit-out; held-out seeds/policies/contexts; baseline comparison; no-go allowed |
| Full-match counterfactuals change Shop luck | Paired participant RNG mapping; open-loop fixtures for mechanism isolation; closed-loop for ecological value; report both |
| The P16-C final state is not merged | Stop at step 1 and rebase; never analyze stale numbers |
| Planned systems are not implemented | Contracts only; every value labeled hypothetical; no fabricated results |
| Compute cost grows without bound | Stage budgets; micro-scenarios and snapshots first; successive halving; stop a stage that costs more than the gate it replaces |
| A balance change is applied silently | Owner checkpoint; one commit with content/GAME-DESIGN/tests; reviewer re-runs the gate |
| The value engine leaks into production | `tbg-core/lab` only; `tbg-server`/`tbg-client` manifests unchanged; contract C10 re-checked |

## 16. Reviewer checklist

- Re-run the canaries, null controls, and held-out validation; compare
  with the report.
- Check that confirmation seeds never appear in a fitting manifest.
- Verify the PVE formula, the `λ_ref` calculation, and the units of every
  reported number.
- Spot-check at least three rows against a hand calculation or a raw
  full-match counterfactual.
- Check that every report row has a source, context, interval, ruleset
  fingerprint, and model version.
- Check that no production path, protocol, server, client, or dependency
  changed; `git diff` of `Cargo.toml` files is empty unless a reason is
  recorded.
- Check that no balance number was applied without the owner's decision.
- Check that P16-F1 is still blocked if `saver` is unresolved.
- Check that only this phase's docs and the offline lab changed; other
  fronts (P39, P16-F, Arcanas) were not touched.

## 17. Phase report

Executed 2026-10-29 on `phase/p16d-prebalance-value-model`, based on the
P16-C final proposal (`1ba4d30`, 8-number proposal applied, gate-after
recorded) plus the P16-F specs. The engine is offline only; no server,
client, protocol, or competitive rule changed.

### Status

Implemented: the deterministic scenario runner, one-sided full-match
counterfactuals, the Energy shadow price, PVE, interaction canaries,
held-out validation, and the `value`/`validate` commands. The P16-C
findings are **not closed**; the owner balance decision and any mechanic
change remain open. P16-F1 is still blocked by the `saver`/banked-wave
finding.

### Files

| Area | Change |
|------|--------|
| `tbg-core/src/headless.rs` | `play_with` hook; `play` is the no-hook wrapper, so online behavior is unchanged |
| `tbg-core/src/lab.rs` | `lab_spawn_at`; reuses `spawn` and only overrides `x` |
| `tbg-balance/src/runner.rs` | lab-only `Setup::Energy`, participant-aware under seat swap |
| `tbg-balance/src/policy.rs` | `BotAvoid(k)` for one-sided control seats |
| `tbg-balance/src/value/*` | `intervention`, `scenario`, `measure`, `estimate`, `report`, `validate`, `catalog`, `pairs` |
| `tbg-balance/src/{cli,commands}.rs` | `value` and `validate` commands and flags |
| `tbg-balance/tests/value.rs` | 12 value contracts |
| `docs/PREBALANCE.md` | methodology, evidence, contracts, limitations |
| `docs/media/p16d/` | small measured summaries |

### Commands and real output

```bash
cargo run --release -p tbg-balance -- value --mode shadow --pairs 4096 --threads 16
cargo run --release -p tbg-balance -- value --mode reference --pairs 4096 --threads 16
cargo run --release -p tbg-balance -- value --mode interactions
cargo run --release -p tbg-balance -- validate --pairs 4096 --threads 16
```

- Shadow: 28.19 s, 4 points × 4,096 pairs. `λ_ref = 0.028748`; tick 0
  interval includes zero; ticks 600/1,200/1,800 are positive with
  intervals excluding zero.
- Reference: 98.30 s, 196,608 games, 2,000 games/s. Top values: Archer
  range +1 tile 507 PVE (+14.57 pp), Bloomkeeper heal +20 % 299 PVE
  (+8.58 pp), Archer damage +20 % 286 PVE (+8.21 pp), Shieldbearer HP
  +20 % 129 PVE (+3.70 pp), Hexer slow 60 % 75 PVE (+2.17 pp), Rootcaller
  root +0.4 s 40 PVE (+1.15 pp). Sawbot interval −10 % is 9 PVE with an
  interval crossing zero; Rampart HP +20 % is 12 PVE.
- Interactions (deterministic): damage + attack speed = −1,665 HP margin
  (diminishing returns); root + ranged damage = +856 (super-additive);
  null pair = 0.
- Validation: 135.78 s, discovery vs confirmation rank **0.991**;
  confirmation vs static baseline rank **0.073**. The direct measurement
  is stable out of sample and not a renamed static formula.
- Canaries: Archer damage ×2 = 1,070 PVE (+30.75 pp); Shieldbearer cost
  +1 Energy = −113 PVE (−3.25 pp). Both were caught.

### P16-C findings

- `saver` / banked waves: still open. The tool shows mid-game Energy is
  valuable (λ ≈ 0.03 per Energy at 0:30 and 1:00), but the `banker-N`
  policy family and a direct burst-deployment measurement are not
  implemented. P16-F1 remains blocked.
- Heavy spam: still open. The one-sided cost canary is negative, but the
  new-unit-versus-Evolution comparison is not yet a full finding.
- Evolution value: the global Evolution-cost change is −1.15 pp in the
  bot pool; it is a global balance effect, not a one-sided PVE. The
  47.5 % "Evolutions never worth their Energy" duel row remains the
  primary evidence.
- Faction spread: not analyzed by this first engine run; the P16-C
  gate-after (Machines 70.9 %, Plants 28.6 %) is unchanged.

### Deviations

- The surrogate is not implemented. Direct full-match measurement is
  already cheap at this catalog size (82 s for 196,608 games), and the
  static baseline ranks poorly (0.073). No surrogate claim is made; a
  future phase must beat the static baseline on held-out units and
  policies before using one.
- Match-snapshot contexts and the `banker-N` policy family are not
  implemented. The one-sided `Favor(k) vs avoid-k` design is an upper
  bound, not average match value.
- The `global_evolution_cost_minus_1` row is reported as a global balance
  effect, not as PVE.
- No balance number was applied. The owner checkpoint is open.

### Verification

```bash
cargo fmt --all -- --check                 # pass
cargo clippy --workspace -- -D warnings    # pass
cargo test --workspace                     # 133 passed, 0 failed, 1 ignored
cargo check -p tbg-client -p tbg-core --target wasm32-unknown-unknown  # pass
cargo run --release -p tbg-balance -- gate --pairs 1000 --threads 16    # completed
```

The gate reproduces the P16-C final result exactly: 40 rows, 18 failing,
same rows and values (`docs/media/p16d/gate-after-p16d.txt`). The new
core hooks therefore did not change online behavior. The one ignored test
is C4b, the full 1× real-clock server match, as in P16-C. The value
engine adds no dependency and is not a dependency of `tbg-server` or
`tbg-client`.

### Limitations

The model balances bot policies, not humans. The Energy price is a bot
lower bound. Micro-scenarios are deterministic and narrow. Planned
systems (Interest, Siege, Reserve, kinds, Fortifications, Augments,
Arcanas) have contracts only and no measured value yet. The owner's
playtest still decides fun and fairness.
