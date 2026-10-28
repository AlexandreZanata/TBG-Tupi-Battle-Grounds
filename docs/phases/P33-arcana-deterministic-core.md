# P33 — Deterministic Arcana core (offline experiment)

**Depends on:** P32 reviewed, including control-safety and canon decisions.
**Branch:** `phase/p33-arcana-core`.
**Status:** planned; extra after P31, not public gameplay.
**Difficulty:** medium.

## Goal and rules

Implement the smallest testable modifier system in tbg-core. This phase
integrates an offline harness, not queues, persistence or production play.
GAME-DESIGN §9 (BR-AR01–BR-AR10) is the single source of gameplay values.
Keep all values Proposed. P34 can reject the experiment.

## Read first (only these)

- `AGENTS.md`; P32 report and approved combat decisions.
- `docs/GAME-DESIGN.md` §5 Evolution/Synergy, §9; DECISIONS D-04, D-05, D-25.
- Core: `lib.rs`, `types.rs`, `content.rs`, `state.rs`, `sim.rs`, `intent.rs`,
  `synergy.rs`, `combat.rs`, `hits.rs`, existing rule/contract tests and
  `examples/balance.rs`; server/client constructors only to verify neutrality.

## Affected files and limits

Add `src/arcanas.rs`, `tests/arcanas.rs`, `tests/arcana_stats.rs`,
`examples/arcana_balance.rs` and small native-example helpers as needed.
Modify content/state/sim/intent/lib and relevant existing tests only.
No dependency, I/O/clock/float in the library, generic modifier framework,
new resource, network message, account power, UI or deployment.
Keep library/tests/helpers within AGENTS.md's function/file limits.

## Exact contracts

```rust
pub enum ArcanaNode { Guard, Assault, Piercing, Endurance, Resilience, Renewal }
// Stable discriminants/index order 0..5 as GAME-DESIGN §9.
pub struct ArcanaPreset { ranks: [u8; 6] }
pub enum ArcanaError { RankOutOfBounds, OverBudget, Incomplete }
impl ArcanaPreset {
    pub const NEUTRAL: Self; // all zero, diagnostics only
    pub const DEFAULT: Self; // candidate vector from GAME-DESIGN §9
    pub fn new(ranks: [u8; 6]) -> Result<Self, ArcanaError>;
    pub fn ranks(self) -> [u8; 6];
    pub fn validate_complete(self) -> Result<Self, ArcanaError>;
}
impl MatchState {
    pub fn new_with_arcanas(seed: u64, presets: [ArcanaPreset; 2]) -> Self;
}
```

`new` validates ranks before summing in a wide integer and accepts partial
budgets for editors/experiments; `validate_complete` requires the exact
budget. Deserialization must call the same bounded validator, never bypass
private fields with unchecked derives. Store the validated preset per seat
in PlayerState. `MatchState::new(seed)` remains the neutral baseline API;
public online/practice entry points switch explicitly in later phases.
Expose types from lib.rs. No client path can mutate ranks during a match.

## Steps (in order; check and commit each)

### 1. Constants, validation and selection space

Keep node definitions, rank/point limits, default and basis-point values in
`content.rs`; arcanas.rs owns validation/selection, not balance literals.
Use fixed arrays; no maps, tree graph or dynamic allocation.
Tests: all 3^6 bounded vectors; exactly 141 complete vectors; partial and
neutral valid only as incomplete configurations; rank 3/255 and oversized
totals rejected without panic/overflow; deterministic rejection order.
**Check:** `cargo test -p tbg-core --test arcanas`.

### 2. Additive attribute calculation

Preserve `effective_stats(kind, tier, branch, synergy)` as a neutral wrapper.
Add `effective_stats_with_arcanas(kind, tier, branch, synergy, preset)`.
Choose eligible nodes by immutable base Archetype, not range, current damage,
Faction, display name, acquired status or current Evolution form.
Accumulate HP/damage/heal basis points with existing sources before the
single scale/truncation in GAME-DESIGN §9; use i64 intermediates and checked
final bounds. Preserve flat Evolution damage ordering. Do not apply a
second multiplier to already rounded Stats. Movement, range, splash radius,
interval, on-hit, costs, regeneration rate and slots are unchanged.

Remove the current temporary active-forms Vec using the existing two-form
array/iterators; no broad refactor. Derive per-seat static Arcana bonuses
once if useful. Do not add caches requiring many invalidation paths unless
the P34 profiler demonstrates a need.
**Check:** neutral Stats equal the old formula for every current valid
kind/form/Synergy combination; no new hot-path allocations.

### 3. Integrate all simulation entry points

`stats_of` reads the owning seat's preset. Purchase must use the same path
when initializing HP/max_hp (the old direct effective_stats call must not
bypass Arcanas). Evolution and Synergy refresh keep the current HP ratio,
including death/low-HP rounding rules; existing units are never healed to
full by recalculation. Subsequent buys use the frozen selection.
Do not add a PlayerIntention for preset edits. Statuses stay temporary and
follow the P32-approved rules after static attributes are calculated.
**Check:** purchase → Evolution → Synergy rise/drop → heal/status/damage →
outcome works with opposing presets and with neutral controls.

### 4. Numerical and property tests

At the planning roster, 8 kinds × 5 forms × 3 Synergy levels × 141 presets
gives 16,920 combinations; derive counts from current content if P31 changes
the roster. Enumerate without adding a property-testing dependency.
Assert positive max HP, nonnegative damage/heal, valid timing/ranges,
monotonic eligible attributes and unchanged ineligible attributes.
Representative starting-value assertions (refresh if P32 changes the base):
Shieldbearer 650→702 HP; fully defensive Humans tier-2 1560→1612;
Archer 45→47 damage and Marksman 81→83; Bloomkeeper 40→44 heal,
Blossom/Grove 60→64. Cite BR-AR03/04, not implementation details.
Test zero attributes, Bramble's retained heal priority, inherited splash
amounts, primary-only status, Plants integer regen and no Stronghold effect.
**Check:** `cargo test -p tbg-core --test arcanas --test arcana_stats`.

### 5. Determinism and neutral regression

Same seed, presets and intention script must produce identical events,
views, hashes and outcomes within the new state schema. Compare neutral
versus P32 using observable traces and normalized simulation fields; adding
a serialized preset legitimately changes old full-state hash bytes.
Never assert old/new schema hashes must be equal. Check both seat orders.
Run a WASM-executed trace comparison when the existing browser harness
supports it; compilation alone is not cross-platform execution evidence.
**Check:** deterministic scripted runs cover buy/evolve/status/Synergy and
end conditions, with fixture outputs recorded and no unexplained difference.

### 6. Minimal experimental example

`cargo run --release -p tbg-core --example arcana_balance -- --smoke`
runs a small deterministic named-preset set, outputs applied ranks/seeds,
finishes every match and includes a neutral comparison. Reuse P32 metrics.
P34 owns policies/search/statistical conclusions; do not claim balance now.
No Arcana constructor is called from public server/client entry points yet.
**Check:** example is integrated, reproducible and has documented CLI help.

## Verification

```bash
cargo fmt --all -- --check
cargo clippy --workspace -- -D warnings
cargo test --workspace
cargo check -p tbg-client -p tbg-core --target wasm32-unknown-unknown
```

## Acceptance and next-phase gate

- [ ] All exhaustive/invalid-input and integration tests pass.
- [ ] Neutral physics/outcomes match P32; new hashes are deterministic.
- [ ] No added per-tick allocation, runtime dependency, arbitrary client stats,
      new combat mechanic or online activation.
- [ ] Form/Synergy eligibility, truncation and zero-attribute cases verified.
- [ ] Reviewer approves experimental scope; P34 starts, not public rollout.

## Risks and reviewer checklist

Check purchase initialization, Synergy invalidation and ratio rounding first.
Additive HP still improves Plants regen indirectly; splash copies full damage.
Do not mistake a passing 141-preset enumerator for gameplay validation.
If these options cannot produce meaningful choices, return to design instead
of adding supremes or increasing numbers without evidence.

## Phase report

_Not executed. Fill in exact HEAD, commands/results, selected constants,
enumeration counts, neutral trace comparisons, determinism evidence,
allocation observations, experiment smoke output, deviations and blockers._
