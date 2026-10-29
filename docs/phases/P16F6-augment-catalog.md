# P16-F6 — Augment catalog: the remaining 26 and adversarial balance

> **Owner revision 2026-10-29 (D-41).** All 40 Augments are implemented
> and active (each copy stacks). The adversarial balance work of this
> phase was **not** done; it stays the open follow-up. Report:
> [P16-F1 phase report](P16F1-economy-and-siege.md#phase-report).

**Depends on:** P16-F5 reviewed, its gate passed on confirmation seeds, and
its owner checkpoint done (round machinery and the 14 ★ Augments work
offline). Canon rounds stay **disabled** through this phase; P16-F7 turns
them on online.
**Branch:** `phase/p16f6-augment-catalog`.
**Difficulty:** high. Many small effects, each with an exploit to close;
steps 2–5 and 8 need care.
**Owner decisions:** O-8 (revised catalog), O-10 (equipped Augments are
public; nothing in this phase hides or reveals more than BR-AG09).

## Goal

1. Implement the 26 Augments of GAME-DESIGN §10.8 that P16-F5 did not
   ship, and the effect kinds they need, inside the closed
   `AugmentEffect` enum of `tbg-core/src/augment.rs`.
2. Keep every stacking cap of BR-X01–X11 enforced at one place per
   attribute, with integer math only.
3. Measure each Augment, every Augment triple, and their interaction with
   `StrongholdKind`, `Fortification` and policy in the P16-C lab; tune
   numbers only; leave a confirmed, reviewed catalog for the owner (O-8).
4. Document why the owner's original list became §10.8, with the risks of
   the final catalog, so later tuning starts from evidence.

## Starting state (verify first)

Record each item in the report with a file/symbol reference; a mismatch
is a `> blocked:` note, not a silent fix.

- `augment.rs` exists with `AugmentEffect` variants `UnitStat`,
  `StrongholdStat`, `VsStronghold`, `VsArchetype`, `Bounty`,
  `EvolveDiscount`, `Lifesteal`, `LastStand`, `Comeback` (P16-F5).
- `content.rs` has `AUGMENTS` with the 14 ★ entries at their §10.8 index
  and placeholders (or nothing) for the other 26; the rarity of an index is
  `index / 10` (0 Bronze … 3 Diamond).
- `Ruleset` has the Augment fields (round ticks, window, costs, Forgo,
  per-Augment numbers) registered as knobs (C5); canon rounds disabled.
- Lab: `AugmentPolicy {Greedy, Forgo, Force(id)}`, `--force-augment`, and
  the Δ metric (score(bot + forced) − score(bot + Forgo), paired seeds).
- The P16-F5 report's step mean at 100 units/side (`examples/army_load.rs`):
  this phase's performance baseline.
- Facts the effects depend on (re-check the lines):
  - `Hit` events carry the requested amount, not HP removed
    (`hits.rs:66-81`).
  - `Death` carries no cause (`combat.rs:122-127`); summon expiry and kills
    look alike. P16-F3 added the eligible-kill rule (BR-E08); reuse its
    helper, do not invent a second one.
  - Stronghold fire has no windup and targets a Rampart first
    (`combat.rs:95-120`).
  - A slow of 10 000 bp divides by zero (`combat.rs:72-93`).
  - `effective_stats` (`ruleset.rs:91-135`) sums forms and Synergy.
  - `BEETLES_PER_PLAYER` is a structural `const`, enforced at
    `summon.rs:64`.
  - `MatchState::hash()` is FNV over postcard of the state: a new
    serialized field moves every hash golden.

## Rules

Canon: GAME-DESIGN §10.6 (BR-AG01–AG11), §10.7 (BR-X01–X11), §10.8 (the
catalog: ids, names, tags, effects) and §10.9 (gates). Related: BR-E08
(eligible kill), BR-E09 (no Augment touches regen, Interest or the Shop),
BR-K07 (HP fraction on max change), BR-M09 (Collapse is never reduced,
never a kill), BR-U08 (summons), BR-AR10 (Arcanas never touch
Strongholds). Decisions: D-34, D-38.

Values below are the §10.8 starting values written as integers. A value
that is not in §10.8 is marked **(spec)**: it is this phase's reading of
the text, and the reviewer copies it into §10.8 in the commit that
implements it.

## Shared interaction rules

Stated once; the per-Augment tables list only exceptions.

| With | Rule |
|------|------|
| Arcana | Not implemented before P33. When it exists, its bp joins the same Σbp of the same attribute (BR-X01). Arcanas never modify Strongholds (BR-AR10), so Stronghold Augments never stack with an Arcana |
| Troop Evolution | Augments apply to every tier. "Evolved" means `tier ≥ 1`. Evolution bp, Synergy bp and Augment bp add, then scale once (BR-X01). Evolution cost changes come only from `field_promotion` |
| `StrongholdKind`, `Fortification` | Stronghold Augment bp adds to kind bp and Fortification bp on the canonical base, then scales once (BR-X01, BR-X10) |
| Synergy | Same Σbp. Plants regeneration is not "healing" for heal bonuses, Grievous Wounds or Triage unless the row says so. Synergy counts never change because of an Augment |
| Summons | Only Augments naming Beetlings apply to them (BR-AG10). A summon's death is never an eligible kill (BR-E08), so it never feeds Bounty, Mend or Fury |
| Collapse | Not damage: no reduction, shield, lifesteal, Second Wind, Fury or Mend reacts to it (BR-M09) |
| Order | Conditional effects read the state when they apply (BR-X11): position and HP-fraction conditions at hit resolution, Synergy-count conditions after the Synergy update; hit-time effects (situational damage, lifesteal, heal cut, Second Wind) resolve inside the hit, in the existing hit order |
| Caps | DR Σ ≤ 2 000 bp; lifesteal Σ ≤ 1 200 bp; status duration Σ ≤ +3 000 bp; move Σ ≤ +5 000 bp; attack speed Σ ≤ +10 000 bp (BR-X02–X07). Each cap is applied in exactly one function |

## Catalog revision (owner's list → §10.8)

Why the list changed (D-38): equal effects at a higher rarity are one
decision, not two; values below a kill breakpoint are invisible; the game
lacked counters to healing stalls, Tank walls and burst; P16-C measured
Machines at 84.8 % faction-majority score while Humans (35.6 %) and Plants
(21.0 %) trailed, and found Evolutions not worth their Energy in 70 % of
duels.

| # | Owner's Augment | Result | §10.8 | Change and reason |
|---|-----------------|--------|-------|-------------------|
| 1 | Passos Leves | replaced | 0 `forced_march` | +5 % move on all units is invisible and speeds the trickle that loses to banking; a 5-s +40 % burst for new units lets reinforcements join the front instead |
| 2 | Mira Firme | changed | 3 `steady_aim` | +6 % → +8 % Ranged damage (Archer 45 → 48 crosses more breakpoints) |
| 3 | Escudos Reforçados | changed | 4 `reinforced_shields` | +7 % → +8 % Tank HP, one value step for all Bronze specialists |
| 4 | Primeiros Socorros | changed | 5 `first_aid` | +8 % → +10 % direct and splash heal (Bloomkeeper 40 → 44) |
| 5 | Concentração Arcana | replaced | 1 `field_kit` | +6 % of 20–30 base Control damage is 1–2 HP; a Universal +4 % HP keeps Bronze useful for any army |
| 6 | Engrenagens Otimizadas | changed | 8 `tuned_gears` | +6 % → +8 % Machines attack speed; kept as Machines' only Bronze |
| 7 | Seiva Resistente | changed | 9 `hardy_sap` | +6 % → +8 % Plants HP (Plants trail at 21.0 %) |
| 8 | Disciplina Militar | changed | 6 `drilled_ranks` | +5 % → +8 % Humans damage |
| 9 | Ossos Frenéticos | changed | 7 `frenzied_bones` | +5 % → +8 % Undead attack speed |
| 10 | Sentinela | changed | 2 `sentry` | +4 % tower damage is 2.6 damage and crosses no kill breakpoint; +10 % (65 → 71) turns Sawbot and Bloomkeeper into 4-shot kills |
| 11 | Saque Organizado | changed | 10 `plunder` | Cap 0.45 per window instead of an open cap; shares the 1-Energy ceiling with Treasury (BR-E08) |
| 12 | Recuperação de Pedra | kept | 11 `stone_mend` | 50 HP per eligible kill, cap 150 HP per 30 s; Collapse still outgrows it (BR-M10) |
| 13 | Passo de Escudo | changed | 19 `shield_step` | +10 % / +4 % → +12 % move / +5 % HP; the Tank line keeps one card per rarity up to Gold |
| 14 | Artilharia de Apoio | changed | 15 `siege_arrows` | +10 % → +15 % Ranged vs Strongholds; absorbs 39 (same effect) |
| 15 | Colmeia Blindada | changed | 16 `armored_brood` | +20 % → +25 % Beetling HP (90 → 112: survives one more Archer arrow pair) |
| 16 | Marcha Mecânica | replaced | 12 `field_promotion` | Machines need no more help; Evolutions do: −1 Energy per Evolution |
| 17 | Retaguarda Protegida | changed | 14 `shield_wall` | −6 % → −8 % damage taken near a Rampart; radius fixed at 1.5 tiles |
| 18 | Cura de Emergência | changed | 17 `triage` | +20 % → +25 % direct heal below 40 % HP |
| 19 | Treinamento de Elite | changed | 13 `veterans` | +8 % HP → +8 % HP and +4 % damage; Silver step of the Evolution line (33 is the Diamond step, distinct size) |
| 20 | Maldição Profunda | changed | 18 `lingering_hex` | +10 % → +15 % slow and root duration; absorbs 30 |
| 21 | Vampirismo de Guerra | kept | 20 `war_leech` | 8 % lifesteal on primary hits on conventional units (BR-X06) |
| 22 | Conquistadores | kept | 21 `conquerors` | +12 % vs Strongholds, all conventional units |
| 23 | Fúria da Torre | changed | 22 `tower_fury` | Stacks share one 12-s timer refreshed by each kill (simpler, same ceiling) |
| 24 | Defesa Entrincheirada | changed | 23 `entrenched` | "Nearby" fixed at 5 tiles from the own Stronghold |
| 25 | Instinto de Caça | kept | 27 `hunters_instinct` | DPS +10 % attack speed, +8 % move |
| 26 | Cura Superior | replaced | 24 `grievous_wounds` | A third heal-percentage card added no decision; healing stalls had no counter |
| 27 | Colmeia Furiosa | merged | 39 `hive_awakening` | Beetling damage moves into the Diamond Nester card; two Beetling cards remain (16, 39) |
| 28 | Máquinas de Guerra | replaced | 25 `giant_slayer` | Machines lead the field; Tank walls (Rampart, Bone Brute) needed a counter |
| 29 | Vanguarda de Ferro | changed | 28 `iron_vanguard` | +12 % HP → +15 % HP, +10 % move (absorbs 35 one rarity lower) |
| 30 | Controle Prolongado | merged | 18 `lingering_hex` | Duplicate of 20; Gold Control slot becomes 29 `spellweaver` (+20 % attack speed: more status applications without longer locks) |
| 31 | Último Bastião | changed | 30 `last_bastion` | Shield 400 HP → 12 % of max HP, so it scales with kind and Walls |
| 32 | Fúria da Desvantagem | kept | 31 `desperate_assault` | Fractions compared in `i64` cross-multiplication |
| 33 | Legado Evoluído | kept | 32 `legacy` | Diamond step of the Evolution line |
| 34 | Sangue da Conquista | replaced | 33 `second_wind` | Duplicate of 21 at 12 %; burst (Ballista, War Engines splash, Siege Walker) lacked a counter |
| 35 | Vanguarda Suprema | merged | 28 `iron_vanguard` | Third Tank-HP card; merged into Gold |
| 36 | Colheita Amplificada | kept | 38 `bountiful_harvest` | Plants capstone |
| 37 | Legião de Ferro | changed | 37 `iron_legion` | +18 % damage → +15 % damage, +10 % attack speed; Machines keep a capstone, smaller |
| 38 | Despertar do Ninho | changed | 39 `hive_awakening` | +25 % damage and +1 summon limit, kept; absorbs 27 |
| 39 | Chuva de Cerco | merged | 15 `siege_arrows` | Third "vs Stronghold" card; replaced at Diamond by 35 `royal_banner` (Humans had nothing above Bronze) |
| 40 | Convergência de Guerra | kept | 34 `convergence` | Two Factions with Synergy level ≥ 1 |
| — | (new) | added | 26 `executioner` | Finishing damage; rewards focus and Control |
| — | (new) | added | 36 `deathless_legion` | Undead had nothing above Bronze |

Count check: 40 owner entries → 30 kept or changed, 6 replaced (1, 5, 16,
26, 28, 34), 4 merged (27, 30, 35, 39). The four freed slots hold
`spellweaver`, `royal_banner`, `executioner` and `deathless_legion`, so
§10.8 holds 40 distinct effects.

## Technical decisions

- **One enum, one hook per kind.** New variants of `AugmentEffect`:
  `DeployHaste { move_bp, ticks }`, `KillMend { hp, cap, window }`,
  `DamageTakenAura { anchor, radius, bp, below_bp }` (`anchor` is
  `OwnRampart` or `OwnStronghold`; `below_bp` = 0 means always),
  `HealBonus { bp, below_bp }`, `StatusDuration { bp }`,
  `TowerFury { bp, max_stacks, ticks }`, `HealCut { bp, ticks }`,
  `Executioner { below_bp, bp }`, `SecondWind`,
  `Convergence { hp_bp, dmg_bp }`, `SummonLimit { add }`; and a
  `Filter::Beetlings` for `UnitStat`. No trait, no callback list, no
  generic condition language.
- **Per-player sums, computed once.** On purchase, static effects fold
  into the player's existing per-player Augment sums struct from P16-F5
  (bp per filter and attribute). Conditional effects are never folded: position and HP-fraction
  conditions (`entrenched`, `desperate_assault`, the Banner aura,
  `executioner`) are read at hit resolution from that tick's state, as
  P16-F3/F5 do; `convergence` reads the Synergy levels after the Synergy
  update of the same tick (BR-X11). No per-tick allocation.
- **New per-unit state** (serialized, moves hashes): `heal_cut_until:
  Tick` (0 = none), `second_wind_used: bool`, `haste_until: Tick`.
  New per-player state: `fury_stacks: u8`, `fury_until: Tick`,
  `mend_window_start: Tick`, `mend_in_window: u16`. Smallest integer types
  that hold the bound; the step-1 golden procedure below applies.
- **Summon limit** becomes `summon_limit(player) = BEETLES_PER_PLAYER +
  augment add`; `BEETLES_PER_PLAYER` stays the structural constant; the
  per-Nester limits are untouched.
- **Effective damage for lifesteal and kill credit** is `min(dmg, hp
  before)`; the `Hit` event amount is unchanged (wire stays the same).
- **Stronghold kill credit** for `tower_fury`: inside `stronghold_fire`,
  a hit that moves a conventional unit's HP from > 0 to ≤ 0 counts;
  `remove_dead` stays the only place units are removed.

## Read first (only these)

- `AGENTS.md`
- `docs/GAME-DESIGN.md` §4, §5, §10 (all of it); `docs/DECISIONS.md`
  D-33, D-34, D-37, D-38
- `docs/phases/P16F5-augments-core.md` (including its report);
  `docs/phases/P16C-balance-lab.md` steps 4, 7, 8, 10
- Core: `augment.rs`, `content.rs`, `ruleset.rs`, `state.rs`, `sim.rs`,
  `combat.rs`, `hits.rs`, `targeting.rs`, `synergy.rs`, `summon.rs`,
  `intent.rs`, `bot.rs`, `tests/contract_lab.rs`,
  `tests/contract_determinism.rs`
- Lab: `crates/tbg-balance/src/{policy,knobs,search,gate,metrics}.rs`,
  `tests/contract.rs`

## Files

| File | Change |
|------|--------|
| `crates/tbg-core/src/augment.rs` | new variants, hooks, per-player sums |
| `crates/tbg-core/src/content.rs` | `AUGMENTS` entries 0–39 complete |
| `crates/tbg-core/src/ruleset.rs` | new numbers as `Ruleset` fields; caps applied in `effective_stats` |
| `crates/tbg-core/src/state.rs` | new unit and player fields above |
| `crates/tbg-core/src/hits.rs` | situational damage, DR, lifesteal, heal cut, Second Wind, heal bonus |
| `crates/tbg-core/src/combat.rs` | DR auras, Fury credit in `stronghold_fire`, status duration |
| `crates/tbg-core/src/summon.rs` | per-player summon limit |
| `crates/tbg-core/src/intent.rs` | deploy haste stamp on purchase |
| `crates/tbg-core/tests/augments_catalog.rs` | new: per-Augment tests named below |
| `crates/tbg-balance/src/{knobs,policy,search,gate}.rs` | knobs, triple search, gate rows |
| `docs/media/p16f6/*.txt` | evidence summaries (small text only) |
| `docs/GAME-DESIGN.md` §10.8 | **(spec)** values and tuned numbers, in the commit that changes them |

## Do NOT

- Renumber, reorder, or reuse an `AUGMENTS` index (serialization
  contract). Append only in index order.
- Add a generic effect engine, trait objects, closures in tables, an event
  bus, or a dependency.
- Let any Augment change slow strength, base regen, Interest, Shop odds or
  Synergy counts (BR-X03, BR-E09).
- Turn canon Augment rounds on, touch the protocol, the server, or the
  client (P16-F7).
- Use confirmation seeds for tuning; change thresholds to pass a gate.
- Change mechanics during tuning (step 9 varies numbers only).

## Steps (in order; check and commit each)

### 1. Baseline and golden procedure

Run `gate` on the P16-F5 canon (Augments forced on via the lab flag) and
`examples/army_load.rs`; save `docs/media/p16f6/baseline-*.txt`. Add all
new state fields with their zero values and no reader. Prove behavior is
unchanged: same winner, reason, end tick, purchase sequence for the golden
seeds and identical `tbg-balance run` output to the baseline; only then
refreeze the hash goldens.
**Check:** `cargo test --workspace`; diff of run outputs empty.
**Done when:** `test(p16f6): new augment state, behavior unchanged
(BR-AG08)` — the commit message names the rule, per the contract-file rule.

### 2. Stat-family Augments (S complexity)

Implement `Filter::Beetlings`, and table entries 5–9, 16, 19, 27–29,
35–39 that are pure `UnitStat`/`HealBonus` without condition, plus
`SummonLimit` (39). Tests: `br_x01_bp_sum_truncates_once`,
`br_ag10_only_beetling_augments_reach_summons`,
`hive_awakening_raises_player_limit_not_nester_limit`,
`br_x02_attack_speed_cap_holds_with_every_source`.
**Check:** `cargo test -p tbg-core --test augments_catalog`.
**Done when:** `feat(p16f6): stat and summon augments`.

### 3. Deploy and status Augments

`DeployHaste` (0): purchase stamps `haste_until = tick + 100`; speed
+4 000 bp while `tick < haste_until`. `StatusDuration` (18): ticks =
`floor(base × (10 000 + Σ) / 10 000)`, Σ ≤ 3 000, root still bounded by any
P32 root contract. `spellweaver` (29) is `UnitStat` on Control.
Tests: `forced_march_only_units_bought_after_purchase`,
`br_x07_status_duration_cap`, `lingering_hex_never_changes_slow_strength`.
**Done when:** `feat(p16f6): deploy haste and status duration`.

### 4. Defensive Augments

`DamageTakenAura` (14, 23), `HealBonus` with `below_bp` (5, 17, 38),
`HealCut` (24), `SecondWind` (33). DR applies after the hit's damage
truncation, Σ ≤ 2 000, at least 1 damage remains (BR-X05). Heal cut
multiplies heal received by `(10 000 − 4 000) / 10 000` while `tick <
heal_cut_until`; a new hit sets `heal_cut_until = tick + 40` (refresh, no
stacking); it also reduces lifesteal and Plants regeneration received
**(spec)**. Second Wind: if a hit would bring a conventional unit with
`second_wind_used == false` to ≤ 0, set HP 1 and the flag; splash
included; summons and Collapse excluded.
Tests: `br_x05_damage_reduction_cap_and_floor`,
`shield_wall_needs_a_living_rampart_within_1500`,
`entrenched_only_below_half_and_within_5000`,
`grievous_wounds_refreshes_and_does_not_stack`,
`second_wind_once_per_unit_not_summons`,
`br_x06_lifesteal_reduced_by_heal_cut`.
**Done when:** `feat(p16f6): defensive augments`.

### 5. Offensive and Stronghold Augments

`Executioner` (26): +2 000 bp when target HP × 10 000 < 3 500 × max at
hit time (`i64`). `TowerFury` (22): a Stronghold kill sets `fury_stacks =
min(3, stacks + 1)`, `fury_until = tick + 240`; stacks → 0 when `tick ≥
fury_until`; Stronghold damage +500 bp per stack. `KillMend` (11): +50 HP
per eligible kill, at most 150 HP per 600-tick window starting at the
first mend, never above max (BR-X10). `Convergence` (34): active while at
least two Faction Synergy levels ≥ 1.
Tests: `executioner_threshold_is_exclusive`,
`tower_fury_counts_only_stronghold_kills_of_conventional_units`,
`br_x10_stone_mend_cap_and_max_hp`, `br_m09_collapse_feeds_no_mend_or_fury`,
`convergence_turns_off_when_a_synergy_drops`.
**Done when:** `feat(p16f6): offensive and stronghold augments`.

### 6. Catalog completeness contracts

Append to `crates/tbg-core/tests/contract_lab.rs` (append-only file):
`augment_table_has_forty_in_index_order`,
`every_rarity_has_a_universal_augment_left_for_three_rounds` (BR-AG04),
`no_augment_touches_regen_interest_or_shop` (BR-E09, scans `AUGMENTS`
effects), `every_cap_is_reachable_and_holds` (stack every source to its
maximum: one kind, two Fortification tiers, two Evolution tiers, two
Synergy levels, three Augments; assert each BR-X cap).
**Done when:** `test(p16f6): catalog contracts (BR-AG04, BR-E09, BR-X09)`.

### 7. Knobs and policies

Register every new `Ruleset` number as a knob or mark it untunable (C5).
Bounds: bp values ±50 % in steps of 100 bp; ticks ±25 % in whole ticks;
HP amounts ±50 % in steps of 10; Diamond costs 12–18, Gold 8–12, Silver
5–7, Bronze 2–4 Energy. `AugmentPolicy::Greedy` scores options by a fixed
table of tags against the current army (documented in `policy.rs`), so a
new Augment needs one table row (C5 forces it).
**Check:** C3 and C5 pass; `Force(id)` for each of the 40 plays to the end
without rejection (C11).
**Done when:** `feat(p16f6): augment knobs and policies`.

### 8. Measure: single, triple, adversarial

Discovery seeds `0..`, screening `200_000..`, confirmation `1_000_000..`,
never mixed (P16-C step 4). Commands are added to `tbg-balance`:
1. `augments --pairs 2000`: Δ for every id at its round (40 rows) against
   `bot`, `eco`, `rush`, each `faction-<f>`; pick and Forgo rates of
   `Greedy` per round.
2. `search --space augment-triples`: the P16-C (1+λ) evolution strategy
   over (triple × `StrongholdKind` × Fortification branch × policy),
   λ = 16, successive halving 256 → 1 024 → 4 096 pairs, 60 generations,
   fitness = score against the strategic pool.
3. `confirm`: best five participants on 20 000 confirmation pairs.
Save `augments-single.txt`, `augments-triples.txt`, `confirm.txt` in
`docs/media/p16f6/`. Budget: if any stage exceeds 30 minutes on the
development PC, split it by seed block; never lower a gate.
**Done when:** `docs(p16f6): augment measurements`.

### 9. Tune numbers only

From the gate failures, propose at most 8 number changes per proposal
(P16-C loss and rules). Re-run confirmation. Apply to `content.rs` and
§10.8 in the same commit, updating `contract_*` tests in that commit.
If numbers cannot fix a row, stop and ask the owner to remove or redesign
that Augment (record a DECISIONS entry); never add a mechanic here.
**Done when:** `balance(p16f6): <summary>`.

### 10. Owner checkpoint and report

Present §10.8 as tuned, the gate, the catalog risks, and the revision
table (O-8). Fill in the phase report.
**Done when:** `docs(p16f6): phase report`.

## Per-Augment analysis

Abbreviations: Δ = paired value against Forgo (pp); "all" = all
conventional units of the owner; window/tick integers as implemented.
"Shared" means no exception to the shared rules. Hypotheses are what the
lab must confirm or refute, not results.

### Bronze (3 Energy)

| # id | Hook & condition | Duration / limits | Exceptions | Counterplay | Exploit & guard | Cx | Hypothesis | Tests |
|------|------------------|-------------------|------------|-------------|-----------------|----|------------|-------|
| 0 `forced_march` | purchase: `haste_until = tick + 100`; move +4 000 bp | 5 s per unit; move cap 5 000 | applies only to units bought after the purchase | Ballista splash on the arriving group | stacking with Hunter's Instinct and Shield Step → move cap | M | Δ +0–3; helps banker waves arrive together | `forced_march_only_units_bought_after_purchase` |
| 1 `field_kit` ★ | stat: all, HP +400 bp | match | shared | none specific (generic) | none | S | Δ +1–3, safest pick | F5 tests |
| 2 `sentry` ★ | Stronghold dmg +1 000 bp | match | stacks with kind/Arms | outrange (Longshot), Tanks | none | S | Δ +1–3; better with Ballista | F5 tests |
| 3 `steady_aim` ★ | stat: Ranged dmg +800 | match | shared | Tanks screen, Giant Slayer n/a | none | S | Δ 0–3 if ≥ 2 Ranged | F5 tests |
| 4 `reinforced_shields` ★ | stat: Tank HP +800 | match | shared | Giant Slayer, Executioner | none | S | Δ 0–2 | F5 tests |
| 5 `first_aid` | heal bp +1 000 (direct and splash) | match | not Plants regen | Grievous Wounds | stall with Redoubt → Collapse ends it | S | Δ 0–2 with a Bloomkeeper, ≈ 0 without | `first_aid_excludes_plants_regen` |
| 6 `drilled_ranks` | stat: Humans dmg +800 | match | shared | anti-Humans: Ballista vs Archers | none | S | Δ 0–3 Humans armies | `faction_filter_matches_base_faction` |
| 7 `frenzied_bones` | stat: Undead atk +800 | match | shared | Root/slow | none | S | Δ 0–3 Undead armies | same |
| 8 `tuned_gears` | stat: Machines atk +800 | match | shared | Giant Slayer vs Rampart | Machines already strong: watch Δ | S | Δ 0–2 | same |
| 9 `hardy_sap` | stat: Plants HP +800 | match | shared | Grievous Wounds | none | S | Δ 0–3 Plants armies | same |

### Silver (6 Energy)

| # id | Hook & condition | Duration / limits | Exceptions | Counterplay | Exploit & guard | Cx | Hypothesis | Tests |
|------|------------------|-------------------|------------|-------------|-----------------|----|------------|-------|
| 10 `plunder` ★ | eligible kill: +90 sub-energy | cap 270 per window; player ceiling 600 (BR-E08) | stacks with Treasury under the ceiling | buy fewer, larger units | feeding cheap units: cap; summons never count | S | Δ +1–4; Treasury synergy capped | F5 tests |
| 11 `stone_mend` | eligible kill: Stronghold +50 HP | ≤ 150 HP per 600 ticks; ≤ max | Collapse feeds nothing | fewer, bigger units; push past the Stronghold's reach | stall with Redoubt + heal: Collapse grows past 5 HP/s within the first Siege steps (BR-M10) | S | Δ 0–2 | `br_x10_stone_mend_cap_and_max_hp` |
| 12 `field_promotion` ★ | Evolution cost −600, min 600 | match | cost shown by the view (F7) | rush before evolutions pay back | none: no refund path | S | Δ +1–4; Evolver policy gains most | F5 tests |
| 13 `veterans` ★ | stat: tier ≥ 1, HP +800, dmg +400 | match | Beetlings inherit tier but are summons: excluded | kill units before they evolve | none | S | Δ 0–3 | F5 tests |
| 14 `shield_wall` | aura: units within 1 500 milli-tiles of an own living Rampart take −800 | while condition | DR cap 2 000 shared with 23 | Giant Slayer, splash on the clump | many Ramparts: one aura per unit (no stacking per Rampart) | M | Δ 0–3 with Rampart, ≈ 0 without | `shield_wall_needs_a_living_rampart_within_1500` |
| 15 `siege_arrows` ★ | Ranged dmg vs Stronghold +1 500 | match | situational Σ (BR-X04) | kill Ranged before range | none | S | Δ 0–3 | F5 tests |
| 16 `armored_brood` | Beetlings HP +2 500 | match | summons only | splash, Ballista | none: summon limit unchanged | S | Δ ≈ 0 without Nester, +1–3 with | `br_ag10_only_beetling_augments_reach_summons` |
| 17 `triage` | heal +2 500 when target HP < 40 % of max | per heal | not Plants regen | burst damage, Grievous | stall → Collapse | S | Δ 0–2 with Support | `triage_threshold_is_exclusive` |
| 18 `lingering_hex` | own slow/root ticks +1 500 | match; Σ ≤ 3 000 | P32 root contract bounds root | DPS kill Control first | root lock: duration only, strength unchanged | S | Δ 0–2 with 2+ Control | `br_x07_status_duration_cap`, `lingering_hex_never_changes_slow_strength` |
| 19 `shield_step` | stat: Tank move +1 200, HP +500 | match | move cap | Root | none | S | Δ 0–2 | `br_x03_move_cap` |

### Gold (10 Energy)

| # id | Hook & condition | Duration / limits | Exceptions | Counterplay | Exploit & guard | Cx | Hypothesis | Tests |
|------|------------------|-------------------|------------|-------------|-----------------|----|------------|-------|
| 20 `war_leech` ★ | primary hit on enemy conventional unit: heal `effective × 800 / 10 000` | match; Σ ≤ 1 200 | reduced by Grievous Wounds | Grievous Wounds; burst | Beetling farming: summons excluded; overkill excluded | M | Δ +2–6 | F5 tests, `br_x06_lifesteal_reduced_by_heal_cut` |
| 21 `conquerors` ★ | all dmg vs Stronghold +1 200 | match | situational | defensive Tanks hold the line | none | S | Δ +2–6 | F5 tests |
| 22 `tower_fury` | Stronghold kill of conventional unit: +1 stack | 240 ticks after last kill; ≤ 3 | Collapse feeds nothing | stay out of range; feed nothing | summons excluded so Beetlings never feed it | M | Δ +1–4 defense-heavy | `tower_fury_counts_only_stronghold_kills_of_conventional_units` |
| 23 `entrenched` | own Stronghold HP × 10 000 < 5 000 × max: units within 5 000 milli-tiles take −1 000 | while condition | DR cap with 14 | finish the Stronghold with Ranged out of 5 tiles; Collapse | deliberate HP loss: costs the Stronghold, Collapse is symmetric | M | Δ 0–3; comeback aid | `entrenched_only_below_half_and_within_5000` |
| 24 `grievous_wounds` | own hit: `heal_cut_until = tick + 40`; heal received × 6 000 / 10 000 | refresh, no stacking | also cuts lifesteal and Plants regen **(spec)** | none needed: it is the counter | none | M | Δ ≈ 0 vs non-healers, +3–8 vs Plants/leech | `grievous_wounds_refreshes_and_does_not_stack` |
| 25 `giant_slayer` ★ | dmg vs target base Archetype Tank +1 500 | match | Strongholds are not Tanks | fewer Tanks | none | S | Δ +1–5 vs Tank metas | F5 tests |
| 26 `executioner` | dmg +2 000 if target HP × 10 000 < 3 500 × max | per hit | units only | high-HP Tanks with heals | Strongholds excluded (no anti-Stronghold finisher) | S | Δ +1–4 | `executioner_threshold_is_exclusive` |
| 27 `hunters_instinct` | stat: DPS atk +1 000, move +800 | match | MIN_INTERVAL floor | Tanks, slow | Beetlings are DPS but summons: excluded | S | Δ 0–3 | `hunters_instinct_skips_beetlings` |
| 28 `iron_vanguard` | stat: Tank HP +1 500, move +1 000 | match | move cap | Giant Slayer | none | S | Δ +1–4 | `br_x03_move_cap` |
| 29 `spellweaver` | stat: Control atk +2 000 | match | statuses unchanged | DPS focus | root uptime ↑: P32 root contract | S | Δ 0–3 | `spellweaver_root_uptime_within_contract` |

### Diamond (15 Energy)

| # id | Hook & condition | Duration / limits | Exceptions | Counterplay | Exploit & guard | Cx | Hypothesis | Tests |
|------|------------------|-------------------|------------|-------------|-----------------|----|------------|-------|
| 30 `last_bastion` ★ | own Stronghold first below 25 %: shield `max × 1 200 / 10 000` | 300 ticks; once | Collapse ignores it (BR-M09) | burst before or after | none: once per match | M | Δ +1–4 | F5 tests |
| 31 `desperate_assault` ★ | `hp_own × max_enemy < hp_enemy × max_own` (`i64`): dmg vs Stronghold +1 500 | while condition | situational | keep the HP lead small; defend | deliberate HP loss: costs real HP; boosts only Stronghold damage | M | Δ +2–6; raises comebacks | F5 tests |
| 32 `legacy` ★ | stat: tier ≥ 1, HP +1 500, dmg +1 000 | match | stacks with 13 | kill before evolution | none | S | Δ +3–8 Evolver | F5 tests |
| 33 `second_wind` | lethal hit, flag unused: HP = 1 | once per conventional unit | splash included; summons, Collapse excluded | splash, Executioner, focus fire | lifesteal/heal after survival is normal; flag survives Evolution | M | Δ +3–8; counters Ballista/War Engines | `second_wind_once_per_unit_not_summons` |
| 34 `convergence` | ≥ 2 Factions with Synergy ≥ 1: all HP +800, dmg +800 | while condition | Synergy counts unchanged | kill one Faction's pair | none | M | Δ ≈ 0 mono-faction, +2–6 mixed | `convergence_turns_off_when_a_synergy_drops` |
| 35 `royal_banner` | stat: Humans HP +1 200, dmg +1 000 | match | stacks with Humans Synergy +40 % HP | splash vs Archers | Humans HP Σ up to +5 200 with Synergy level 2 and Field Kit: Diamond, one per match | S | Δ +2–6 Humans | `faction_filter_matches_base_faction` |
| 36 `deathless_legion` | stat: Undead atk +1 500, HP +1 000 | match | atk cap with Synergy +40 % | Root/slow | atk Σ ≤ 10 000 | S | Δ +2–6 Undead | `br_x02_attack_speed_cap_holds_with_every_source` |
| 37 `iron_legion` | stat: Machines dmg +1 500, atk +1 000 | match | shared | Giant Slayer, Grievous n/a | Machines lead: gate watches this row first | S | Δ +2–6; risk of dominance | same |
| 38 `bountiful_harvest` | stat: Plants HP +1 000; heal +1 500 | match | heal not Plants regen | Grievous Wounds | stall → Collapse | S | Δ +2–6 Plants | `first_aid_excludes_plants_regen` |
| 39 `hive_awakening` | Beetlings dmg +2 500; player summon limit +1 | match | per-Nester limit unchanged | splash, Ballista | entity growth: +1 per player only | S | Δ ≈ 0 without Nester, +2–5 with | `hive_awakening_raises_player_limit_not_nester_limit` |

## Catalog risks

- **Redundant:** 13/32 (Evolution line) and 4/19/28 (Tank line) are
  intentional ladders of different size; if Δ of the lower card is within
  1 pp of 0, merge before tuning more.
- **Dominant candidates:** `second_wind` against Ballista and War Engines
  splash builds; `iron_legion` on the leading Faction; `war_leech` +
  `bountiful_harvest` + Redoubt (healing stall — Grievous Wounds counters,
  Collapse ends it, BR-M10).
- **Useless without a condition:** `armored_brood`, `hive_awakening` (no
  Nester), `shield_wall` (no Rampart), `convergence` (one Faction),
  `first_aid`/`triage` (no Support). Guard: BR-AG04 Universal slot and
  Forgo +2; gate requires Bronze/Silver Δ ≥ 0 only for the policies that
  can use them.
- **Too situational:** `entrenched`, `desperate_assault`, `last_bastion`
  trigger only when behind. `desperate_assault` cannot be farmed by losing
  HP on purpose: it compares fractions, boosts only Stronghold damage, and
  Collapse is symmetric.
- **Entity and CPU growth:** `hive_awakening` +1 Beetling per player;
  auras scan Ramparts per unit (O(n × ramparts)); measure in Performance.

## Acceptance tests

All §10.9 Augment rows on the full 40 on confirmation seeds, plus:

| Row | Target |
|-----|--------|
| Greedy pick rate of any Augment when offered | ≤ 60 % |
| Greedy Forgo rate per round | 5–50 % |
| Seat bias, bot vs bot with Augments forced on | 50 % ± 1.5 pp |
| Every named test in the tables above | passes |
| Step mean at 100 units/side | ≤ 1.2 × P16-F5 measurement |
| Per-tick allocation added | none (code review + `army_load` allocation count unchanged) |

## Verification

```bash
cargo fmt --all -- --check
cargo clippy --workspace -- -D warnings
cargo test --workspace
cargo check -p tbg-client -p tbg-core --target wasm32-unknown-unknown
cargo test -p tbg-server --test lab_equivalence -- --ignored
cargo run --release -p tbg-core --example army_load
cargo run --release -p tbg-balance -- augments --pairs 2000 --threads 16
cargo run --release -p tbg-balance -- gate --pairs 20000 --threads 16
```

## Performance

Report step mean / p99 at 10, 40 and 100 units per side before and after
(same table as P16-B), allocations per tick, and the size of `MatchState`
postcard bytes. Views and the wire do not change in this phase.

## Acceptance checklist

- [ ] All 40 `AUGMENTS` entries implemented in index order; no renumbering
- [ ] Every BR-X cap enforced in one function and tested at its maximum
- [ ] Golden procedure followed; hash goldens refrozen once, with reason
- [ ] Knobs registered (C5); C3, C10, C11 pass
- [ ] Single, triple and adversarial measurements saved in `docs/media/p16f6/`
- [ ] Gate passes on confirmation seeds, or owner decision recorded
- [ ] §10.8 updated with tuned values and **(spec)** readings
- [ ] Owner checkpoint (O-8) answered; report filled in

## Risks and what to do

| Problem | Action |
|---------|--------|
| An effect needs a mechanic the enum lacks | Stop; propose the smallest variant to the reviewer; never generalize |
| Golden hashes move in a step meant to be neutral | The refactor is wrong; fix before continuing |
| Search finds a triple > 65 % | Tune its numbers; if impossible, ask the owner to cut one of the three |
| Greedy policy favors one card regardless of army | Fix the policy table first (it measures the bot), then judge the card |
| Lab cost too high | Split by seed block; fewer generations with the same confirmation size |

## Reviewer checklist

- Re-run `augments` and `gate` on confirmation seeds; compare to the report.
- Every effect touches only its hook; no Augment code outside
  `augment.rs`, `hits.rs`, `combat.rs`, `summon.rs`, `intent.rs`,
  `ruleset.rs`.
- Collapse, summons, and Plants regen exceptions tested.
- `contract_*` diffs tied to a canon commit; confirmation seeds absent
  from search manifests.

## Owner checkpoint

- O-8: approve the revised catalog and the tuned numbers (or remove cards).
- Confirm the **(spec)** readings (Grievous Wounds also cuts lifesteal and
  Plants regen; mend window starts at the first mend).

## Phase report

Owner revision 2026-10-29: see the [combined report](P16F1-economy-and-siege.md#phase-report). The original checklist below is kept for the gated work still open.

_Not executed._ Fill in:

- Starting state (with references) and baseline files
- Golden procedure evidence
- Δ table for 40 Augments; pick and Forgo rates per round
- Triple search: best participants, confirmation intervals
- Gate before / after; proposal and applied numbers
- Performance table
- Owner decisions; deviations; blocked items; limitations (bots are not humans)
