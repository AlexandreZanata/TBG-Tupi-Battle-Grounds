# P16-F3 — Stronghold kinds and Fortifications: rules and lab (offline)

**Depends on:** P16-F2 reviewed, canon switched.
**Branch:** `phase/p16f3-stronghold-kinds-core`.
**Difficulty:** medium–high. Step 2 (per-seat Stronghold) touches every
Stronghold read; step 6 decides the numbers.
**Owner decisions:** O-2 (names), O-9 (Fortification values), O-12 (Banner
aura, Ballista variant) at step 7. Nothing is switched on online here.

## Goal

1. Turn the Stronghold into per-seat data: kind, max HP, Fortification
   tier and branch.
2. Implement the four kinds, the Treasury Bounty (BR-E08), the Banner aura
   and the shared Walls/Arms tree in `tbg-core`.
3. Measure kinds and Fortifications in the lab against the strategic pool;
   propose reviewed numbers.

Online play keeps `Classic` on both seats and Fortification disabled until
P16-F4.

## Starting state (verify first)

At planning time the Stronghold has no struct: `PlayerState.stronghold_hp:
i32` and `stronghold_cooldown: u16`; max HP is `rules.stronghold_hp`
everywhere (init in `sim.rs`, client `ui/battle_hud.rs`,
`ui/hud_update.rs`, `arena/towers.rs`); `combat::stronghold_fire` has no
windup, targets the nearest enemy unit in range with Ramparts first (BR-U09),
and deals `rules.stronghold_damage` at once; nothing heals a Stronghold;
`Death` events carry no cause, and only summons expire. Record HEAD,
`PROTOCOL_VERSION`, and the P16-F2 gate output.

## Rules

GAME-DESIGN §10.4 (BR-K01–K08: this phase implements K06, K07 and the data
for the others), §10.5 (BR-F01–F05), BR-E08 (Bounty), §10.7 (BR-X01, X04,
X08, X09), §10.9 kind and Fortification rows.

## Technical decisions

- `types.rs`: `enum StrongholdKind { Classic, Treasury, Redoubt, Ballista,
  Banner }` — this order is a serialization contract; `SELECTABLE_KINDS:
  [StrongholdKind; 4]` excludes `Classic`.
- `content.rs` / `ruleset.rs`: `KindDef { hp_bp: i32, dmg_bp: i32, splash:
  i32, bounty_per_kill: u32, bounty_cap: u32, aura_range: i32,
  aura_dmg_bp: i32 }`, `Ruleset.kinds: [KindDef; 5]` (index = enum order);
  values from GAME-DESIGN §10.4 (Ballista `splash` 0; the splash variant is
  a lab ruleset only). `Ruleset.fortify_cost: [u32; 2]` (3 600, 6 000) and
  `Ruleset.fortify: [FortDef; 4]` in `[A1, A2, B1, B2]` order, `FortDef {
  hp_bp, dmg_bp, on_hit: OnHit, splash_add }`; `Ruleset.fortify_enabled:
  bool` canon `false` until P16-F4. `BOUNTY_CEILING` = 600 is structural.
  Register every field in `knobs.rs` (C5): kind bp ±1 000 step 100,
  bounty 60–180 step 30, aura bp 500–1 500 step 250, aura range 3 000–6 000
  step 500, Fortification bp 1 000–2 500 step 250, costs ±2 Energy.
- `state.rs`: `PlayerState` gains `kind`, `stronghold_max: i32`,
  `fort_tier: u8`, `fort_branch: u8`, `bounty_window: u32`. One helper
  `scale_hp(hp, old_max, new_max) -> i32` (`i64`, ≥ 1 if hp ≥ 1) used by
  units (replacing the body of `refresh_max_hp`) and Strongholds (BR-K07).
- `MatchState::new(seed)` keeps both seats `Classic` (server and goldens
  unchanged in meaning); `MatchState::new_with_kinds(seed, [kind; 2])` is a
  canonical constructor (kinds are player choices, not rules), allowed in
  the server from P16-F4; C10 stays as is.
- Stronghold damage per shot: `floor(D × (10 000 + kind.dmg_bp +
  fort.dmg_bp) / 10 000)` (Augments join in P16-F5); Stronghold max:
  `floor(H × (10 000 + kind.hp_bp + fort.hp_bp) / 10 000)`. Range and
  interval never change (BR-K06).
- Curtain Wall: `stronghold_fire` applies the `OnHit::Slow` through the
  same function `hits.rs` uses for units (replacement rules unchanged).
  War Engines and the Ballista variant: Stronghold splash uses the existing
  unit splash routine (`hits.rs`) with the Stronghold as attacker.
- Banner aura: evaluated at hit resolution (`hits.rs`): attacker is an own
  conventional unit with `|x − stronghold_x(seat)| ≤ aura_range` → add
  `aura_dmg_bp` to Σbp (BR-X04). No per-tick state.
- Bounty (BR-E08): in `combat::remove_dead`, for each removed conventional
  unit (summons never; only summons expire, so every conventional removal is
  a death by damage), the other seat gains `min(per_kill, cap −
  window_from_this_source, BOUNTY_CEILING − bounty_window)`. Track one
  `bounty_window` per player; with a single source in this phase that is
  enough — P16-F5 extends it per source. Window resets on rotation ticks
  (the Interest ticks).
- `PlayerIntention` appends `Fortify { branch: u8 }`; `Rejection` appends
  `MaxFortification`; reuses `WrongBranch`, `NotEnoughEnergy`,
  `Unavailable` (disabled). Spend through `PlayerState::spend` (Interest
  low-water, P16-F1). `Event` appends `Fortified { seat, tier, branch }`.
- `MatchView` appends `stronghold_max: [i32; 2]`, `fort: [(u8, u8); 2]`
  (tier, branch), `kinds: [StrongholdKind; 2]`. These are public. Bump
  `PROTOCOL_VERSION` by one in step 8 (the wire changes even though
  Fortify stays disabled online).
- Lab: kind is a participant attribute (`--kind-a`, `--kind-b`, `--kinds
  all`), carried through paired-seed swaps; new policies `Fortifier(b)`
  (bot that buys tier 1 then tier 2 of branch `b` once Energy ≥ cost + 3),
  `FortifyAlways` (fortifies before any purchase), `Turtle` (Walls to tier
  2, then `Eco`). Duels (P16-C) are unchanged: Strongholds do not fire there.

## Read first (only these)

`AGENTS.md`; GAME-DESIGN §4, §5, §10.1, §10.4, §10.5, §10.7, §10.9;
DECISIONS D-23, D-33, D-34, D-37; P16-C steps 3, 6–8, 10; core
`content.rs`, `ruleset.rs`, `types.rs`, `state.rs`, `sim.rs`, `intent.rs`,
`combat.rs`, `hits.rs`, `targeting.rs`, `view.rs`, `protocol.rs`,
`tests/contract_*.rs`; lab `policy.rs`, `runner.rs`, `metrics.rs`,
`gate.rs`, `knobs.rs`, `meta.rs`, `tests/contract.rs`.

## Files

Core `types.rs`, `content.rs`, `ruleset.rs`, `state.rs`, `sim.rs`,
`intent.rs`, `combat.rs`, `hits.rs`, `view.rs`, `protocol.rs`; new
`tests/stronghold_kinds.rs`. Lab `cli.rs`, `runner.rs`, `policy.rs`,
`metrics.rs`, `gate.rs`, `knobs.rs`, `report.rs`. Client: only the
compile fixes for new view fields (no new UI; P16-F4). Evidence in
`docs/media/p16f3/`.

## Do NOT

- Add selection, protocol messages for choosing, persistence, lobby UI or
  3D work (P16-F4).
- Change unit stats, Shop, Energy, Interest or Siege values.
- Add kind-exclusive Fortifications, a third branch, or tier 3.
- Let anything heal a Stronghold (Stone Mend is P16-F6).
- Read Stronghold max from `rules.stronghold_hp` after step 2 anywhere in
  the simulation.

## Steps (in order; check and commit each)

### 1. Baseline

`gate` (confirmation seeds) and `army_load` at the P16-F2 HEAD into
`docs/media/p16f3/baseline.txt`.
**Done when:** committed (`docs(p16f3): baseline`).

### 2. Per-seat Stronghold, all `Classic`

Add kinds, `KindDef` table, per-seat fields, `scale_hp`, and route every
Stronghold read (init, fire, Collapse, view) through the seat's values.
With both seats `Classic` and Fortify disabled the game is unchanged;
hashes move (new fields): prove unchanged behavior as in P16-F1 step 2,
then refreeze.
**Check:** `cargo test --workspace`; `grep -rn "stronghold_hp" crates/tbg-core/src`
shows only the `Ruleset` field and the canonical base reads.
**Done when:** committed (`refactor(p16f3): per-seat Stronghold (BR-K07)`).

### 3. Kinds and Bounty

Implement BR-K06 values, Treasury Bounty (BR-E08) and the Banner aura;
Ballista splash path (used only by the lab variant).
**Check:** tests 1–6.
**Done when:** committed (`feat(p16f3): Stronghold kinds and Bounty (BR-K06, BR-E08)`).

### 4. Fortifications

`Fortify` intention, tree, costs, HP scaling, Curtain Wall slow, War
Engines splash, `Fortified` event; enabled only when `fortify_enabled`.
**Check:** tests 7–12 with a lab ruleset enabling Fortify.
**Done when:** committed (`feat(p16f3): Fortifications (BR-F01–F05)`).

### 5. Termination bound with the new maxima

Extend `br_m11_server_limit_is_unreachable` and the analytic bound to the
largest possible max (Redoubt + Walls tier 2) and every kind; Collapse is a
percentage, so the bound must not move — the test proves it.
**Done when:** committed (`test(p16f3): termination bound for every kind (BR-M10)`).

### 6. Lab: kinds × pool, Fortification value, proposal

- Matrix: 4 selectable kinds + `Classic` control × strategic pool,
  paired seeds, both seat orders (2 000 pairs per cell on discovery,
  confirmation 20 000 for the final table).
- Head-to-head kind matrix (5 × 5, bot vs bot).
- Fortification value: for each kind and branch, `Fortifier(b)` vs `Bot`
  and vs `Rush`/`Banker(25)`; `FortifyAlways` vs pool.
- Variants: Ballista pure damage vs splash variant; Banner aura 500 /
  1 000 / 1 500 bp.
- Metrics: kind win share, Bounty Energy per match and its share of
  income, how often the Bounty ceiling binds, Fortification purchase time
  and branch, Collapse share per kind.
- `search` (P16-C, at most 8 changes, knobs limited to kind and
  Fortification fields) only if a §10.9 kind row fails; `confirm` on fresh
  seeds. Save `kinds-matrix.txt`, `fortify.txt`, `proposal.md`,
  `gate-after.txt`.
**Check:** C3, C5, C7 (canaries extended: a kind with ×2 damage is flagged),
C8 (all kinds `Classic` → flat matrix), C11.
**Done when:** committed (`feat(p16f3): kind and Fortification lab evidence`).

### 7. Owner checkpoint (stop here)

Present the matrices, the Fortification value table, the variants and the
proposal. Owner decides O-2, O-9, O-12 and the proposal. Apply accepted
numbers to `content.rs` and GAME-DESIGN §10.4/§10.5 (still **Proposed**),
contract tests in the same commit.
**Done when:** committed (`balance(p16f3): <summary of accepted values>`).

### 8. Wire and report

Append the view fields and the intention/event variants to the protocol
contract tests, bump `PROTOCOL_VERSION`, keep online canon `Classic` and
Fortify disabled (server sends `Unavailable` to any `Fortify`). C4a/C4b.
Measure `army_load` and view bytes. Fill in the report.
**Done when:** committed (`docs(p16f3): phase report`).

## Acceptance tests

`tests/stronghold_kinds.rs`:

1. `br_k06_kind_values_scale_the_canonical_stronghold` (every kind;
   generic over `SELECTABLE_KINDS`)
2. `br_k06_kinds_never_change_range_or_interval`
3. `br_e08_treasury_bounty_counts_conventional_deaths_only` (a Beetling
   death and expiry give nothing)
4. `br_e08_bounty_respects_cap_and_ceiling_and_resets_each_window`
5. `br_x04_banner_aura_applies_inside_four_tiles_only` (3 999 vs 4 001 mm)
6. `ballista_splash_variant_hits_every_unit_within_one_tile`
7. `br_f01_fortify_order_and_branch_lock` (tier 0→1 either branch;
   1→2 same branch; no third)
8. `br_f02_rejections_are_exact` (`MaxFortification`, `WrongBranch`,
   `NotEnoughEnergy`, `Unavailable` when disabled)
9. `br_k07_max_hp_change_keeps_the_fraction_and_never_heals_it` (full,
   half, 1 HP, odd values)
10. `curtain_wall_slow_follows_status_replacement_rules`
11. `fortify_lowers_interest_low_water` (BR-E06)
12. `fortification_survives_reconnect_snapshot` (serialize/deserialize the
    state mid-match: identical continuation hash)
13. `br_m10_bound_holds_for_every_kind_and_fortification`

## Verification

```bash
cargo fmt --all -- --check
cargo clippy --workspace -- -D warnings
cargo test --workspace
cargo check -p tbg-client -p tbg-core --target wasm32-unknown-unknown
cargo test -p tbg-server --test lab_equivalence -- --ignored
cargo run --release -p tbg-balance -- gate --pairs 20000 --threads 16
```

## Performance

Step mean/p99 at 10/40/100 units a side (budget ≤ +5 % over baseline: the
aura check is one comparison per hit); view bytes (+≈ 12 B). Measured only.

## Acceptance checklist

- [ ] All Stronghold reads are per seat; goldens refrozen with proof
- [ ] Kinds, Bounty, aura, Fortifications pass tests 1–13
- [ ] Lab tables saved; owner decisions recorded; accepted values applied
- [ ] Online behavior unchanged (`Classic`, Fortify `Unavailable`)
- [ ] Docs: GAME-DESIGN §10.4/§10.5, ARCHITECTURE §4, roadmap

## Risks and what to do

| Problem | Action |
|---------|--------|
| Treasury snowballs (Bounty to the side already winning fights) | Bounty share-of-income and ceiling-binding metrics; lower the cap before raising HP |
| Ballista + War Engines makes rushing impossible | Rush row and Ballista head-to-head; splash variant only if pure damage fails |
| Redoubt + Walls favors stalling | Collapse share per kind ≤ 15 %; Collapse is a percentage, so HP does not outlast it |
| Fortifications never worth their Energy (like 70 % of Evolutions) | Fortification value table; owner may lower costs within the knob bounds |
| A missed `rules.stronghold_hp` read | grep check in step 2; per-seat test with different kinds on each side |

## Reviewer checklist

- Run the 5 × 5 head-to-head yourself on confirmation seeds.
- `scale_hp` is the only place max HP changes for units and Strongholds.
- Bounty never triggers on summons or expiry; the ceiling holds per window.
- Server and client still construct `Classic` matches only.

## Owner checkpoint

Step 7 (O-2, O-9, O-12 and the proposal), with the matrices in hand.

## Phase report

_Not executed._ Fill in: starting state; refreeze proof; kind matrix;
head-to-head; Fortification value; variants; proposal and confirmation;
owner decisions; performance; deviations; blocked items; limitations.
