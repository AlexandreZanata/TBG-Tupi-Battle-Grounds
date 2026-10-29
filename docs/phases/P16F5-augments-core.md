# P16-F5 — Augments core: rounds, mirrored draws, the first 14 (offline)

**Depends on:** P16-F4 reviewed, canon switched (kinds and Fortifications
online).
**Branch:** `phase/p16f5-augments-core`.
**Difficulty:** high. Step 2 (round machinery and draw) and step 4
(lifesteal, shield) need care; step 6 decides whether the system continues.
**Owner decisions:** O-6 (identical options), O-7 (three rounds), O-10
(public Augments) at step 7. Nothing is switched on online here.

## Goal

1. Build the `AugmentRound` machinery in `tbg-core`: schedule, mirrored
   draw, purchase, Forgo, expiry, compensation, three-Augment limit.
2. Implement the 14 ★ Augments of GAME-DESIGN §10.8 with nine effect kinds,
   inside one closed enum.
3. Measure the system in the lab — value of each Augment, Forgo rate,
   seat bias, interaction with kinds and policies — before the full catalog
   (P16-F6) or any UI (P16-F7).

## Starting state (verify first)

Record HEAD, `PROTOCOL_VERSION`, the P16-F4 gate. Expected: per-seat
Stronghold (`stronghold_max`, `kind`, `fort_*`), Bounty with one source
(`bounty_window`), `PlayerState::spend`, Interest `low`, Siege at 4:00.
Facts at planning time: `Hit` carries the requested amount (`hits.rs`
`damage` clamps HP at 0 but does not return what it removed); `Death` has no
cause; Evolution cost is read from `rules.evolve_cost` in `intent.rs` and
hard-coded in the client (`ui/evolve.rs`); `effective_stats` (`ruleset.rs`)
sums forms and Synergy; RNG streams in use: 1/2 Shop, 5 practice bot kind,
7 bot, 11 lab controllers.

## Rules

GAME-DESIGN §10.6 (BR-AG01–AG11), §10.7 (BR-X01–X11), §10.8 (★ rows),
§10.9 Augment rows; BR-E08–E09; BR-M09 (Collapse ignores shields).

## Technical decisions

- **Module** `tbg-core/src/augment.rs` (one concern: Augment data
  helpers, draw, and the per-player sums); round lifecycle lives in
  `augment.rs` too, called from `sim.rs`; hooks stay in the module they
  modify (`ruleset::effective_stats`, `hits.rs`, `combat.rs`, `intent.rs`).
- **Data** (`content.rs`): `Rarity { Bronze, Silver, Gold, Diamond }`,
  `Tag { Universal, Archetype(Archetype), Faction(Faction), Kind(UnitKind),
  Beetlings, TwoFactions }`, `AugmentDef { id: u8, rarity, tag, effect:
  AugmentEffect }`, `AUGMENTS` — the 14 ★ entries, each with its §10.8
  index as `id`; P16-F6 appends the rest. Ids are a serialization contract
  and never renumbered.
- **Effects** (closed enum, no trait, no callback list):
  `UnitStat { filter: Filter, hp_bp, dmg_bp, atk_bp, move_bp }` (`Filter
  { Conventional, Archetype(a), Faction(f), Evolved }`),
  `StrongholdStat { dmg_bp }`, `VsStronghold { filter, bp }`,
  `VsArchetype { archetype, bp }`, `Bounty { per_kill, cap }`,
  `EvolveDiscount { amount }`, `Lifesteal { bp }`,
  `LastStand { below_bp, shield_bp, ticks }`, `Comeback { bp }`.
- **Ruleset**: `augment_rounds: Option<[u32; 3]>` (running ticks 600, 2 400,
  4 200; canon `None` until P16-F7), `augment_window: u32` (600),
  `augment_cost: [u32; 4]` (1 800, 3 600, 6 000, 9 000), `forgo_energy: u32`
  (1 200), `augments: [AugmentDef; N]` (numbers tunable; tags and ids
  untunable). Register knobs (C5): costs ±1 Energy, Forgo 600–1 800 step
  300, each effect's bp ±50 % step 100.
- **State** (serialized): `MatchState.augment_rng: Pcg32` (stream 3,
  seeded like the Shop), `MatchState.round: Option<OpenRound { index: u8,
  options: [u8; 3], closes_at: Tick }>`, `MatchState.offered: u64` (bit per
  id, never re-offered); `PlayerState.augments: [u8; 3]` (255 = empty, in
  purchase order), `PlayerState.resolved: u8` (bit per round),
  `PlayerState.sums: AugmentSums` (fixed-size bp sums per filter and
  attribute, folded at purchase; a conditional effect is never folded),
  `PlayerState.shield: i32`, `shield_until: Tick`, `last_stand_used: bool`.
- **Round lifecycle** in `step()` right after the Shop step:
  open at `RUNNING_START + rounds[i]` (draw BR-AG04 with `augment_rng`,
  push `Event::RoundOpened { round }`); at `closes_at` every unresolved
  player gets Forgo (+`forgo_energy`, `Event::Forgone { seat, round }`),
  then `round = None`. Intentions are applied before `step()`, so a purchase
  on the closing tick wins.
- **Intentions** append `ChooseAugment { round: u8, option: u8 }`,
  `Forgo { round: u8 }`; **rejections** append `RoundClosed` (no open
  round, wrong round, or already resolved by this player), `BadOption`;
  reuse `NotEnoughEnergy`, `Unavailable` (rounds disabled). Purchase uses
  `spend` (Interest low-water) and pushes `Event::AugmentChosen { seat, id }`.
- **Hooks**: `effective_stats` adds `sums` for the unit's filters (never
  for summons); hit resolution adds VsStronghold, VsArchetype and Comeback
  bp to Σbp (BR-X04), then applies lifesteal from the HP actually removed
  (change `hits::damage` to return it; the `Hit` event amount stays
  unchanged); Stronghold damage adds `StrongholdStat`; Evolution cost
  subtracts `EvolveDiscount` (minimum 600); Bounty joins the P16-F3 Bounty
  with its own cap and the shared 1-Energy ceiling (track per-source window
  amounts in a fixed array, no allocation); LastStand is checked after
  damage to the Stronghold: first time HP < 25 % of max → `shield =
  max × 1 200 / 10 000` for 300 ticks, `Event::ShieldRaised { seat }`;
  unit and Stronghold damage hit `shield` first; Collapse never does.
  Comeback compares fractions with `i64(hp_a) × max_b` vs
  `i64(hp_b) × max_a` (strictly lower).
- **View** appends `round: Option<RoundView { round, options: [u8; 3],
  ticks_left, resolved: bool }>`, `augments: [[u8; 3]; 2]` (public,
  BR-AG09), `shield: [i32; 2]`, `evolve_cost: [u32; 2]` (own, after
  discounts, so the client stops hard-coding it). Events appended:
  `RoundOpened`, `AugmentChosen`, `Forgone`, `ShieldRaised`. Bump
  `PROTOCOL_VERSION` in step 8 (canon rounds still disabled).
- **Lab**: an `AugmentPolicy { Greedy, Forgo, Force(id), Random }` per
  participant (like the kind): `Greedy` buys the highest-rarity affordable
  option whose tag fits its army (Universal, or an Archetype/Faction/Kind
  it owns at least 2 of), else waits until 5 s before closing and Forgoes;
  lab-only `--force-augment ID` (feature `lab`) places that id in the
  slot of its rarity in the first round that has one, for both seats, so
  "bot + X" and "bot + Forgo" play the same seeds. Δ value per Augment =
  score(bot + Force(X)) − score(bot + Forgo).

## Read first (only these)

`AGENTS.md`; GAME-DESIGN §4, §10 (all); DECISIONS D-33, D-34, D-38; P16-C
steps 3, 6–8, 10; P16-F3 report; core `content.rs`, `ruleset.rs`,
`types.rs`, `state.rs`, `sim.rs`, `intent.rs`, `shop.rs`, `combat.rs`,
`hits.rs`, `rng.rs`, `view.rs`, `protocol.rs`, `lab.rs`,
`tests/contract_*.rs`; lab `policy.rs`, `runner.rs`, `metrics.rs`,
`gate.rs`, `knobs.rs`, `search.rs`, `tests/contract.rs`.

## Files

Core new `augment.rs`, new `tests/augments.rs`; `content.rs`,
`ruleset.rs`, `state.rs`, `sim.rs`, `intent.rs`, `combat.rs`, `hits.rs`,
`view.rs`, `protocol.rs`, `lab.rs`, `lib.rs`. Lab `cli.rs`, `runner.rs`,
`policy.rs`, `metrics.rs`, `gate.rs`, `knobs.rs`, `report.rs`. Client:
compile fixes only, plus reading `evolve_cost` from the view in
`ui/evolve.rs` and the desktop HUD (no Augment UI). Evidence
`docs/media/p16f5/`.

## Do NOT

- Add the other 26 Augments or their effect kinds (P16-F6).
- Add UI for rounds (P16-F7), a fourth round, a reroll, sell, swap, or an
  automatic pick.
- Let the client choose, see the draw before it opens, or send an effect.
- Use `HashMap`, floats, or per-tick allocation (C1, P16-B rule).
- Apply any Augment to summons in this phase (no ★ Augment names Beetlings).

## Steps (in order; check and commit each)

### 1. Data, state and disabled lifecycle

Types, table, fields, intentions, rejections, events; with rounds `None`
nothing happens. Prove unchanged behavior before refreezing hashes (as in
P16-F1 step 2).
**Done when:** committed (`refactor(p16f5): disabled Augment state (BR-AG01)`).

### 2. Rounds, draw, purchase, Forgo, expiry

BR-AG01–AG08 behind a lab ruleset.
**Check:** tests 1–10.
**Done when:** committed (`feat(p16f5): Augment rounds (BR-AG01–AG08)`).

### 3. Stat effects

`UnitStat`, `StrongholdStat`, `VsStronghold`, `VsArchetype`,
`EvolveDiscount` and the caps of BR-X01–X04 in one function per attribute.
**Check:** tests 11–15.
**Done when:** committed (`feat(p16f5): stat Augments (BR-X01–X04)`).

### 4. Behavior effects

`Bounty` (shared ceiling), `Lifesteal` (HP actually removed, BR-X06),
`LastStand` shield (BR-X10, Collapse bypass), `Comeback`.
**Check:** tests 16–21; the termination bound (P16-F1 test 10) extended to
a shield and the largest heal (none yet) still passes.
**Done when:** committed (`feat(p16f5): behavior Augments`).

### 5. Lab policies and metrics

`AugmentPolicy`, `--force-augment`, metrics: per round offered/bought/
Forgo/expired, Energy spent on Augments, Δ per Augment (with intervals),
pick rate when offered, win rate by equipped set, seat bias with rounds on.
Gate rows: §10.9 Augment rows restricted to the 14, plus seat bias
±1.5 pp, plus every row that passed in P16-F4.
**Check:** C3, C5, C7 (canary: a planted ×3 `UnitStat` Augment is flagged
by the Δ row), C8 (all Augments with zero effect → every Δ within ±1 pp),
C11.
**Done when:** committed (`feat(p16f5): Augment lab metrics`).

### 6. Measure and decide whether to continue

Discovery: each ★ Augment forced at its natural round × 4 kinds × the
strategic pool (2 000 pairs per cell); Greedy vs Forgo-always vs Random;
compare round timing variants 0:30/2:00/3:30 vs 1:30/3:00 (O-7 evidence)
and identical vs per-seat draws (O-6 evidence: per-seat draws use streams
3 and 4). Confirmation: 20 000 pairs for the final rows. Tune numbers only
(`search` limited to Augment knobs, ≤ 8 changes). Save `values.txt`
(Δ table), `rounds.txt`, `draws.txt`, `gate-after.txt`, `proposal.md`.
**Continue rule:** if Greedy is not better than Forgo-always by ≥ 2 pp,
or more than 4 of the 14 are outside their Δ band after tuning, stop and
report: the owner decides whether to change rarities/costs (a DECISIONS
entry) before P16-F6.
**Done when:** committed (`balance(p16f5): Augment evidence and tuned values`).

### 7. Owner checkpoint (stop here)

Present Δ table, Forgo rates, round-timing and draw comparisons. Owner
decides O-6, O-7, O-10 and the tuned values; record them; apply accepted
values to `content.rs` and GAME-DESIGN §10.6/§10.8 (still **Proposed**).
**Done when:** committed (`docs(p16f5): owner decision`).

### 8. Wire and report

Protocol contract tests and `PROTOCOL_VERSION` + 1 (rounds stay disabled
online; `ChooseAugment`/`Forgo` answer `Unavailable`); C4a/C4b;
`army_load`; view bytes; report.
**Done when:** committed (`docs(p16f5): phase report`).

## Acceptance tests

`tests/augments.rs`:

1. `br_ag01_rounds_open_at_their_ticks_and_close_after_thirty_seconds`
2. `br_ag02_round_profiles_match_the_rarity_table`
3. `br_ag03_both_seats_receive_identical_options` (1 000 seeds)
4. `br_ag04_no_duplicate_and_no_reoffer_and_one_universal` (1 000 seeds,
   ★ pool; includes the fallback when a rarity has no Universal left)
5. `br_ag05_one_purchase_per_round_per_player`
6. `br_ag06_forgo_and_expiry_grant_two_energy_once`
7. `br_ag06_reconnect_snapshot_grants_nothing_extra` (serialize mid-round,
   continue: same Energy and hash as the uninterrupted run)
8. `br_ag07_short_of_energy_can_wait_and_buy_later_in_the_window`
9. `br_ag07_match_ending_with_an_open_round_grants_nothing`
10. `br_ag08_never_more_than_three_augments`
11. `br_x01_bonuses_add_in_bp_and_truncate_once` (Evolution + Synergy +
    Augment on the same attribute, exact integers)
12. `br_x02_attack_interval_cap_and_min_interval`
13. `br_x04_situational_damage_joins_the_same_sum`
14. `evolve_discount_never_below_one_energy`
15. `br_ag10_stat_augments_skip_summons`
16. `br_e08_plunder_and_treasury_share_the_one_energy_ceiling`
17. `br_x06_lifesteal_uses_hp_removed_and_skips_splash_summons_strongholds`
18. `br_x06_lifesteal_never_exceeds_max_hp`
19. `br_x10_last_bastion_triggers_once_absorbs_damage_and_expires`
20. `br_m09_collapse_ignores_the_shield`
21. `comeback_compares_fractions_without_overflow` (max HP values, ties,
    full health)

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

Measured: step mean/p99 at 10/40/100 units a side with three Augments per
side (budget ≤ +10 % over P16-F4); view bytes (+≈ 10 B, round view only
during a window); per-match memory (+≈ 100 B). No allocation per tick.

## Acceptance checklist

- [ ] Tests 1–21 pass; goldens refrozen with proof
- [ ] Δ table, round timing and draw comparisons saved; continue rule applied
- [ ] Owner decisions recorded; accepted values in `content.rs` and GAME-DESIGN
- [ ] Online unchanged (rounds disabled, intentions `Unavailable`)
- [ ] Docs: GAME-DESIGN §10.6/§10.8, ARCHITECTURE §4, roadmap

## Risks and what to do

| Problem | Action |
|---------|--------|
| Δ measures the bot, not the Augment | Force both seats into the same round; paired seeds; several policies; owner playtest later (P16-F8) |
| One Augment always picked | Pick-rate row ≤ 60 %; lower its value before raising others |
| Forgo always better | Greedy vs Forgo-always row; costs and Forgo are knobs |
| Lifesteal + healing stall | Collapse bound test with the shield; Grievous Wounds arrives in P16-F6 |
| Mirrored options feel samey | Keep the per-seat comparison in `draws.txt` for O-6 |

## Reviewer checklist

- Draw order and stream: identical options for both seats, no client path
  to the RNG, no re-offer.
- Every cap of BR-X01–X06 is applied in exactly one function.
- `hits::damage` returns HP removed; `Hit` events unchanged on the wire.
- Shield math: unit damage, Stronghold damage, Collapse (bypass), expiry.

## Owner checkpoint

Step 7 (O-6, O-7, O-10, tuned values). Step 6 may stop early under the
continue rule.

## Phase report

_Not executed._ Fill in: starting state; refreeze proof; Δ table with
intervals; Forgo and pick rates; round timing and draw comparisons; tuned
values; owner decisions; performance; deviations; blocked items;
limitations.
