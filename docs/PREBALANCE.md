# Pre-balance value model (P16-D)

> Offline research only. No server, client, protocol, or competitive rule
> reads this model. Balance numbers change only after the owner's decision
> and the normal gate. The executable phase is
> [P16D-prebalance-value-model.md](phases/P16D-prebalance-value-model.md).

## 1. What it answers

The model estimates the **marginal strategic value** of a change before a
full balance pass: +20 % HP on a Shieldbearer, one more tile of range, a
stronger slow, a cheaper Evolution, or one Energy at a given time. It also
measures interactions (damage × attack speed, root × ranged, HP × heal)
and flags changes that are dominated, dead, or extreme.

It does not claim a universal exchange rate between HP, damage, time, and
control. Value is conditional on a declared context distribution; the
reported number is only meaningful with its context, interval, and source.

## 2. Definitions

A context `c` is a ruleset, a board state, both compositions, the phase and
tick, the policy pair, and the horizon. `D_ref` is a fixed, pre-registered
set of contexts.

```text
U(θ; c)  = expected match score (win 1, draw 0.5, loss 0) for configuration θ
ΔU(x; c) = U(θ ⊕ x; c) − U(θ; c)        one intervention x, paired seeds
λ(c)     = best marginal score per Energy at c
λ_ref    = median λ(c) over the reference contexts
PVE(x)   = 100 · E_{c ~ D_ref}[ ΔU(x; c) ] / λ_ref
```

`100 PVE = one Energy at the reference` is a **display convention**. It is
not a claim that one Energy always equals a fixed amount of HP or damage.
If `λ_ref` is not distinguishable from zero, PVE is not reported; the raw
`ΔU` is the result.

For a package with Energy cost `k`, `efficiency = PVE / k`. A free balance
change has no in-game cost; report PVE and the Energy equivalent
`PVE / 100`, never an efficiency that divides by zero.

Interactions for a small set use the Shapley value; for pairs:

```text
I(i, j) = V({i, j}) − V({i}) − V({j})
```

Dominance is Pareto dominance with uncertainty intervals. A dead effect
has a PVE interval containing zero in at least 80 % of contexts. An
oppressive effect has PVE per Energy above twice the median with no tested
counter. A game-theoretic dominance check reuses the P16-C payoff matrix
and Nash mix.

## 3. Measurement ladder

| Layer | What it provides | Cost |
|-------|------------------|------|
| Static | Effective stats, cost ratios, `HP × damage / interval / cost` baseline | negligible |
| Micro-scenario | Exact lane positions, fixed spawns, effective damage, overkill, healing, control uptime, time-to-contact | deterministic, milliseconds |
| Match snapshot | Clone a real state, apply one change, continue for a horizon | cheap, but not implemented yet |
| Full-match counterfactual | Paired seeds, both seat orders, one-sided change, win score | the ground truth; thousands of games |
| Surrogate | Predict full-match ΔU from micro features | **not implemented**; direct measurement is cheap at the current catalog size |

The first implementation measures one-sided changes with
`Favor(k) vs avoid-k`: the treated participant buys the changed unit, the
control never does. This isolates the change but measures a policy that
leans on the unit; report it as a one-sided upper bound, not as average
match value.

## 4. Reference contexts and commands

The current context is the canonical P16-C ruleset, paired seeds, and the
full-match score. The reference Energy price uses the shipped bot and four
ticks (0, 600, 1200, 1800). Micro-scenarios are deterministic and have no
RNG.

```bash
cargo run --release -p tbg-balance -- value --mode shadow --pairs 4096 --threads 16
cargo run --release -p tbg-balance -- value --mode reference --pairs 4096 --threads 16
cargo run --release -p tbg-balance -- value --mode interactions
cargo run --release -p tbg-balance -- validate --pairs 4096 --threads 16
```

Outputs are CSV/text under `target/balance/`; small summaries live in
`docs/media/p16d/`.

## 5. Measured evidence (2026-10-29)

Ruleset: P16-C final proposal, `content.rs` at the P16-D implementation
commit. 4,096 paired seeds per row, 95 % cluster-bootstrap intervals.

Energy price (score per 1 Energy):

| Tick | λ | 95 % interval |
|------|----|----------------|
| 0 | 0.0073 | −0.0089 .. 0.0235 |
| 600 | 0.0302 | 0.0182 .. 0.0426 |
| 1,200 | 0.0287 | 0.0213 .. 0.0362 |
| 1,800 | 0.0103 | 0.0059 .. 0.0148 |

`λ_ref = 0.028748`. Energy at 0:00 is not distinguishable from zero;
Energy at 0:30 and 1:00 is clearly valuable; at 1:30 it is smaller but
still positive. This is a bot lower bound, not an expert-player price.

Reference value table (one-sided; free changes have no Energy cost):

| Change | Δ score | PVE |
|--------|---------|-----|
| Archer range +1 tile | +14.57 pp | 507 |
| Bloomkeeper heal +20 % | +8.58 pp | 299 |
| Archer damage +20 % | +8.21 pp | 286 |
| Shieldbearer HP +20 % | +3.70 pp | 129 |
| Hexer slow 40 % → 60 % | +2.17 pp | 75 |
| Rootcaller root +0.4 s | +1.15 pp | 40 |
| Siege Walker splash +1 tile | +0.49 pp | 17 |
| Rampart HP +20 % | +0.35 pp | 12 |
| Sawbot interval −10 % | +0.25 pp | 9 (interval crosses 0) |
| Canary: Archer damage ×2 | +30.75 pp | 1,070 |
| Canary: Shieldbearer cost +1 Energy | −3.25 pp | −113 |

The canaries behave as required: the ×2 damage is extreme, and the +1
Energy cost is negative. The `global_evolution_cost_minus_1` row is a
global ruleset change, not a one-sided value; it is reported as a balance
effect (−1.15 pp in the bot pool) and must not be read as a PVE.

Validation: discovery vs confirmation rank correlation is **0.991**;
confirmation vs the static `HP × damage / interval / cost` baseline is
**0.073**. The direct measurement is stable out of sample and the crude
static baseline does not reproduce its ranking. This is evidence that the
model is measuring something beyond a renamed stat formula; it is not a
proof of human value.

Interactions (deterministic HP margin):

| Pair | Base | A | B | A+B | Interaction |
|------|------|---|---|-----|-------------|
| HP + heal | 1,890 | 2,020 | 1,890 | 2,020 | 0 |
| Damage + attack speed | −650 | 1,414 | 1,315 | 1,714 | −1,665 |
| Root + ranged damage | −1,935 | 1,450 | −1,764 | 2,477 | +856 |
| Null (HP + unrelated range) | 810 | 940 | 810 | 940 | 0 |

Damage and attack speed show diminishing returns in this scenario (the
target dies before the full product is realized); root and ranged damage
are super-additive. The null pair is exactly zero, which validates the
interaction arithmetic.

## 6. Planned systems

No value is claimed for an unimplemented system. Each phase that adds one
must use this contract:

| System | Value question | Data needed | Phase |
|--------|----------------|-------------|-------|
| Interest / uncapped Energy | option value of holding Energy; λ(t) | `Ruleset` fields, `banker-N`, low-water balances | P16-F1 |
| Siege / Collapse | value of Stronghold HP and defense under percentage collapse | phase, Collapse rate, time-to-end | P16-F1 |
| Shop Reserve | option premium of holding one offer | shop state, hold timer | P16-F2 |
| Stronghold kinds | PVE of HP/damage/passive vs Classic | per-seat kind and stats | P16-F3/F4 |
| Fortifications | PVE per Energy of each branch and tier | tier, branch, cost | P16-F3/F4 |
| Augments | Δ vs Forgo, conditional interactions | forced-Augment policy, round state | P16-F5/F6 |
| Arcanas | PVE of ranks and presets | preset schema, forced presets | P32–P38 |

## 7. Limitations and open work

- The full-match counterfactual is one-sided (`Favor(k) vs avoid-k`); it is
  an upper bound for a player who leans on that unit. Closed-loop average
  value and match-snapshot contexts are not implemented yet.
- The Energy price uses the shipped bot and measures one Energy, not the
  full marginal curve. The 0:00 interval includes zero.
- The surrogate is intentionally not implemented: the direct 4,096-pair
  run takes about 80 s and 196,608 games, and the static baseline ranks
  poorly. If the catalog grows, the surrogate must beat the static baseline
  on held-out units and policies before it is used.
- The P16-C findings (`saver`, heavy spam, Evolution value, faction spread)
  are not closed by this first table. The tool now provides the evidence
  needed to attack them; P16-F1 remains blocked until the owner decides.
- Micro-scenarios do not include the Shop, Synergy, or Stronghold
  interactions unless the scenario defines them.
