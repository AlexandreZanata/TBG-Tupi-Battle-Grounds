# P16-B — Open army, five-offer Shop, Nester and Rampart

**Depends on:** P16-A merged, and the desktop battle HUD (D-29) merged.
**Runs before:** P16 (P16 now depends on P16-B reviewed).
**Branch:** `phase/p16b-open-army`.
**Difficulty:** high — game rules, protocol, balance, and performance change
together. Owner decisions: D-32.

## Goal

1. **No army cap.** A player keeps as many conventional units as their
   `Energy` can buy. `SLOT_CAP`, `SlotCount`, and every capacity check are
   removed — not replaced by another number.
2. **Five Shop offers** instead of three.
3. **Two new units:** **Nester** (owner name *Ninhador*), a summoner, and
   **Rampart** (owner name *Baluarte*), a guard tank.
4. Large battles stay **fair, readable, deterministic, and fast**: balance
   re-checked with evidence, performance measured from 10 to 100 units per
   side.

Everything here is **Proposed** until the harness evidence and the owner's
playtest confirm it. This phase replaces every earlier instruction about the
eight-slot cap or a two-Rampart limit.

## Rules and values to write into GAME-DESIGN (step 1, all **Proposed**)

**Removed rules** (rows stay in the rule table marked *Removed — D-32*):
BR-U03 (`SlotCount`), BR-U06 (`SLOT_CAP`). The glossary loses `SlotCount`;
the "slot cap reached" rejection disappears from §4.

**Changed:** BR-S01 — each player sees exactly **five** unit offers; §5
"Shop offers / full rotation" = **5 / every 10**.

**New rules:**

| ID | Rule |
|----|------|
| BR-U07 | There is no limit on how many conventional units a player may own; only `Energy` gates purchases |
| BR-U08 | A *summon* is a temporary unit created by a unit's ability: it costs no `Energy`, is never sold, never evolves, never summons, does not count for `Synergy` or the army count, and disappears when its lifetime ends |
| BR-U09 | *Guard*: an enemy unit or `Stronghold` whose attack range reaches a living Rampart must target the nearest such Rampart |
| BR-U10 | A server-protection entity limit, if ever needed, is documented as server protection with measured evidence (ARCHITECTURE), never as an army rule |

**New units** (stats in the §5 units table format):

| Unit | Faction | Archetype | Cost | HP | Dmg | Every (s) | Range | Speed | Special |
|------|---------|-----------|------|----|-----|-----------|-------|-------|---------|
| Nester | Plants | Support | 4 | 300 | 15 | 1.5 | 2.5 | 0.8 | Hatch: every 4.0 s hatches a Beetling at its position; ≤ 2 alive per Nester, ≤ 6 Beetlings per player |
| Rampart | Machines | Tank | 5 | 1500 | 35 | 1.5 | 0.8 | 0.5 | Guard (BR-U09) |
| *Beetling (summon)* | Plants | — | — | 90 | 18 | 0.8 | 0.6 | 1.4 | lifetime 10 s |

| Unit | Branch A | Branch B |
|------|----------|----------|
| Nester | Brood: ≤ 3 alive per Nester → Hive: hatch every 3.0 s | Venom: Beetling hits slow 30 % 1.0 s → Blight: Beetlings +50 % HP and +50 % dmg |
| Rampart | Bastion +50 % HP → Citadel +50 % HP, hits slow 30 % 1.5 s | Spiked +60 % dmg → Siegebreaker +40 % dmg, splash 1.0 |

The per-player Beetling limit (6) holds at every tier; when a Nester dies its
Beetlings live out their lifetime. These limits belong to the Hatch ability
(BR-U08), not to the army.

## Read first (only these)

- `AGENTS.md`
- `docs/GAME-DESIGN.md` §2–§6, `docs/DECISIONS.md` D-29, D-31, D-32
- `docs/ARCHITECTURE.md` §4, §5, §10
- `docs/ART-DIRECTION.md` §5, §8
- Core: `content.rs`, `types.rs`, `state.rs`, `intent.rs`, `shop.rs`, `sim.rs`,
  `combat.rs`, `hits.rs`, `targeting.rs`, `synergy.rs`, `view.rs`,
  `protocol.rs`, `bot.rs`, `examples/balance.rs`, `tests/contract_*.rs`
- Server: `src/ws.rs` (size limits), `src/match_task.rs`
- Client: `src/ui/hud*.rs`, `src/ui/battle_hud.rs`, `src/ui/evolve.rs`,
  `src/ui/input.rs`, `src/visuals.rs`, `src/art/roster.rs`, `src/art/roster/`

## Do NOT

- Introduce any replacement army cap, per-type unit limit, or hidden limit
  in client, server, or bots.
- Change Energy start/cap/regen, Stronghold HP/attack, match timings, Elo,
  matchmaking, or evolution costs. Balance changes come only from step 7's
  evidence and stay **Proposed**.
- Break determinism: no floats in simulation state, no `HashMap` iteration,
  summon order by `UnitId`.
- Keep dead code: if `SlotCount` has no functional use, it is deleted.

## Steps

### 1. Canon first
Write the rules and values above into GAME-DESIGN (Proposed), remove
`SlotCount` from the glossary, update §4, and update BR-AR09's mention of
`SlotCount`. Measure the **baseline before any code change**: run
`examples/balance.rs` and save the table (per-unit purchase share and win
contribution, faction win rates, median match length, decisive/overtime/draw
share, comebacks). It is the "before" for step 7.

### 2. Remove the army cap
Delete `SLOT_CAP`, `SlotCount`, `slots_used`, `Rejection::SlotCapReached`,
the capacity check in `intent.rs`, the bot's slot conditions, and the
`slots` column of the content table. The client's large body for Bone Brute,
Siege Walker, and Rampart comes from a client-side table per `UnitKind`
(`art/roster.rs`), not from game data.

### 3. Five offers
`OFFERS = 5`; the Shop module, view, and bot read the constant (no literal
3 anywhere). Rotation timing unchanged; randomness still seeded server-side.

### 4. Summons and Guard in the simulation
- `Unit` gains `summoner: Option<UnitId>` and `expires_at: Option<Tick>`;
  Beetlings are a `UnitKind` excluded from the Shop pool.
- Hatch: per-Nester timer in ticks; limits checked at hatch time against
  the per-Nester and per-player counts; summons never hatch.
- Synergy and the army count skip summons.
- Guard in `targeting.rs`: an attacker with a Rampart in range picks the
  nearest Rampart in range; otherwise the existing rule. Stronghold too.
- Every new number lives in `content.rs` tables in ticks/centi-units.

### 5. Protocol v7
Bump `PROTOCOL_VERSION` to 7: remove `slots_used` and `SlotCapReached`;
offers become five; unit views carry the summon flag so the client can count
the army and draw Beetlings. Check that the largest view (100 units per side
+ 12 Beetlings) fits the server's 64 KiB write cap and the client decoder
with margin; record the byte size.

### 6. Client
- HUD: `Army N` = own living conventional units; `Brood M` shown beside it
  only while M > 0. The Energy panel stays aligned with the five Shop cards
  (desktop battle HUD and the Bevy HUD).
- Shop cards: five, portraits via the D-29 portrait tool (Nester, Rampart).
- Selection in crowds: a tap selects the nearest own unit; repeated taps on
  the same spot cycle through overlapping own units; the selected unit is
  highlighted so Evolution targets are unambiguous.
- Readability: a deterministic visual-only lateral spread across the lane
  width per `UnitId` (the simulation stays one-dimensional, BR-B01).
- Art: Nester (hive-druid with a seed-pod nest on the back), Beetling
  (small armored beetle, team-colored shell), Rampart (clockwork wall-walker
  with a riveted tower shield), each with base + four Evolution forms per
  ART-DIRECTION §5 (scale +8 %/+16 %, large-surface color, new silhouette
  piece, tier-2 artifact with glow). Final-quality polish comes in their
  own character phases later; here they must read clearly in the lineup.

### 7. Balance with evidence
Re-run `examples/balance.rs` with the same seeds (after = no cap, five
offers, ten units). Add two adversarial policies to the harness only:
*heavy spam* (always buy the most expensive affordable unit) and *cheap
spam* (always the cheapest). Report before/after per unit, especially Bone
Brute, Siege Walker, and Rampart (they lost their two-slot cost), plus
Synergy activation rates (2 and 4), Plants regeneration on big armies, swarm
and turtle outcomes, match length, comebacks, and how often Evolutions are
bought. Propose a number change only where the evidence shows a dominant or
dead choice against GAME-DESIGN §6 targets; mark it Proposed with the
evidence in the report. Do not touch Synergy thresholds without evidence.
Arcanas (P32+, not implemented): list the interactions to re-check there.

### 8. Performance of large armies
Measure 10, 20, 40, 60, 100 conventional units per side (+ max Beetlings):
- simulation step mean / p95 / p99 (release, a core example or bench);
- server CPU and RSS per match, view bytes per tick, KB/s per player
  (bots with the spam policies through `tbg-bots`);
- client FPS desktop 1920×1080 and phone 390×844 (`?stress=N`, extend it
  to 100 per side).
Fix hot spots proportionally: targeting without O(n²) rescans where it
matters (e.g. units kept sorted by position), splash without full scans,
no per-tick clones or allocations, Synergy recomputed only when the roster
changes, pooled visuals. A server-protection limit (BR-U10) only with
measured evidence and a separate ARCHITECTURE entry.

### 9. Tests (rewrite the old slot tests; cite the rule IDs)
Update the contract guards deliberately in the same commit as the canon
change (`contract_constants.rs`: `OFFERS == 5`, no `SLOT_CAP`;
`contract_roster.rs`: no slots, ten units, the Beetling row). New tests:
1. A player owns more than eight living units (BR-U07).
2. Purchases succeed whenever Energy suffices, at any army size.
3. No rejection depends on the number of conventional units.
4. Bone Brute, Siege Walker, Rampart buy with no capacity condition.
5. The army count shown matches living conventional units (client test).
6. Synergy counts are correct with large armies and skip summons.
7. Hatch respects per-Nester, per-player, lifetime, and no-recursion limits.
8. Guard holds with 100 units per side (BR-U09).
9. Determinism: two runs with 100 units per side produce identical hashes.
10. Client and server stay in sync (an online test with a large army).
11. The step-8 performance numbers hold (release test or measured example).
12. A guard that greps the workspace: no `SLOT_CAP`, `SlotCount`,
    `slots_used`, or `SlotCapReached` remains.

### 10. Fun check
Watch five bot matches and two human matches with big armies; report
overlap, readability, match length, spam dominance, Evolution relevance,
cheap vs expensive balance, and snowballing (comebacks ≥ 20 %). Propose
fixes that keep large armies possible.

## Acceptance checklist

- [ ] All verification commands from `AGENTS.md` pass; `trunk build --release` passes
- [ ] No army cap anywhere in game logic, bots, or UI; the HUD shows `Army N`
- [ ] Five distinct offers; Nester, Beetling, Rampart integrated with art and portraits
- [ ] A two-player online match with large armies, including a reconnect
- [ ] Before/after balance table and adversarial policies reported; changes Proposed with evidence
- [ ] Performance table for 10/20/40/60/100 units per side (sim, server, bandwidth, FPS)
- [ ] The 12 tests above pass; contract guards updated
- [ ] GAME-DESIGN, ARCHITECTURE §10, and the roadmap updated

## Phase report

### Status (2026-10-29, reviewer: algorithms and tests)

Done on `phase/p16b-open-army`: steps 1–5, the step-6 HUD count and five
cards (placeholder figures only), steps 7–8 measured, step 9 complete.
**Still open:** step 6 art (Nester, Beetling, Rampart final figures and
portraits, tap cycling, visual lateral spread), client FPS at 100 units a
side, step 10 fun check, the two-client match with a reconnect, and the
owner's decision on the balance finding below.

### Owner decision 2026-10-29

The owner chose to ship all of P16-B to the test VPS (five offers, no army
cap, Nester and Rampart) before the art and the balance decision. Nester,
Rampart, and the Beetling use recolored placeholder figures (honey hive
druid, iron-and-brass wall, dark beetle shell) and Shop portraits captured
with the D-29 gallery tool; final art and the spam-balance decision remain
open. All values stay **Proposed**.

### What changed

- `SLOT_CAP`, `SlotCount`, `slots_used`, and the cap rejection are deleted;
  Energy is the only purchase gate (BR-U07). The large body of Bone Brute,
  Siege Walker, and Rampart comes from `art::roster::large` (visual only).
- Five distinct offers: partial Fisher-Yates on draw, refills never
  duplicate another visible offer (BR-S01).
- `summon.rs`: Hatch timer, per-Nester and per-player limits with a
  fixed-size counter (no allocation), lifetime expiry; summons never sell,
  evolve (`Rejection::NotEvolvable`), hatch, or count for Synergy.
- Guard in `targeting.rs` (one pass computes nearest enemy and nearest
  Rampart in range) and in `stronghold_fire`.
- Large-army safety found while reading the code: Synergy counts were `u8`
  (overflow past 255 units of one faction), splash and splash heal cloned
  the whole unit list per hit, `effective_stats` allocated per unit per
  tick. All three fixed.
- Bot: buys while Energy allows; prefers evolving from army 6 (a play
  style, never a limit); never tries to evolve summons.
- Protocol v7; determinism goldens recomputed (seed 1 now ends on time).

### Tests (`cargo test --workspace`: 107 passed, 0 failed)

| # | Test | Where |
|---|------|-------|
| 1–3 | `br_u07_army_grows_past_eight_while_energy_lasts` (40 units; only Energy rejects) | core `open_army.rs` |
| 4 | `br_u07_former_two_slot_units_have_no_capacity_condition` (12 each) | core |
| 5 | `army_label_counts_living_units_and_summons_apart` | client `ui/army_tests.rs` |
| 6 | `br_y01_synergy_with_large_armies_skips_summons` (300 units) | core |
| 7 | `br_u08_hatch_respects_its_own_limits`, `br_u08_beetlings_take_the_nesters_form` | core |
| 8 | `br_u09_guard_holds_with_one_hundred_units_per_side`, `br_u09_guard_also_binds_the_stronghold` | core |
| 9 | `determinism_with_large_armies` (100+ units a side) | core |
| 10 | `br_u07_online_army_stays_in_sync` (real WebSocket match: army rebuilt from events equals every server view; only Energy refuses) | server `online.rs` |
| 11 | `large_battle_views_fit_the_wire` (< 32 KiB) + `examples/army_load.rs` | core |
| 12 | `no_slot_rule_survives_in_the_code` (scans every `.rs` in `crates/`) | core |

Plus `br_s01_five_distinct_offers_after_draws_and_refills` and the updated
contract guards (`contract_constants`, `contract_roster`,
`contract_protocol`, `contract_determinism`). Test 10 does not require
more than eight living units online: over the wire Energy cannot be
forced, and natural play rarely keeps nine alive against the bot; tests 1,
4, and 9 prove the uncapped army in the simulation itself.

### Balance: before vs after (2000 bot matches per row, same seeds)

"Before" is commit `121ca8c` (eight-slot cap, three offers, eight units),
run with the new harness metrics; spam policies respected the old cap.
Files: `docs/media/p16b/balance-{before,after}-{bot,heavy,cheap}.txt`.

| Metric | Before | After |
|--------|--------|-------|
| Median duration (bot vs bot) | 149 s | 138 s |
| Ended by Stronghold / time | 70.7 % / 29.4 % | 82.6 % / 17.4 % |
| Comebacks (behind at 2:00, then won) | 6.0 % | 2.8 % |
| Evolutions per match | 7.7 | 7.4 |
| Max army (one side) | 8 | 21 |
| **Heavy spam vs bot (seat A wins)** | **42.1 %** | **60.5 %** |
| Cheap spam vs bot | 29.3 % | 41.5 % |
| Siege Walker win when bought | 59.5 % | 66.5 % |
| Bone Brute win when bought | 45.9 % | 44.9 % |
| Rampart / Nester win when bought | — | 46.7 % / 43.3 % |

**Finding (owner decision needed).** Without the cap, buying every
affordable unit beats the bot's buy-and-evolve mix 60–62 % of the time,
even at the bot's own decision pace (61.8 %). Cause: with the cap,
Evolution was where Energy went once the army was full; without it, a new
unit gives about twice the stats per Energy of an Evolution (a Shieldbearer
Evolution adds 325 HP for 3 Energy; another Shieldbearer adds 650 HP and 30
damage). Cheaper Evolutions do not fix it: costs 2/3 → heavy spam 59.7 %;
2/4 → 58.9 %. Comebacks also fell (6.0 → 2.8 %, both far from the ≥ 20 %
target). Bone Brute did **not** become dominant without its two-slot cost;
Siege Walker was already strong when bought (59.5 %) and rose to 66.5 %.

Candidate corrections, none applied (numbers stay as before, all values
**Proposed**):
1. Stronger Evolutions (e.g. tier-1 forms worth a second unit's stats),
   re-run the heavy-spam check.
2. A per-purchase cost that grows with the living army — economy, not a
   cap: nothing is ever refused, buying just gets dearer.
3. Accept for now and judge with human playtests (the bot is a weak
   opponent; humans may out-play spam).
4. Siege Walker: watch, possibly −10 % damage, after 1–3.

### Performance (`examples/army_load.rs`, release, dev PC)

| units/side | live units | step mean µs | p95 | p99 | max | view bytes avg / max | KB/s per player |
|---|---|---|---|---|---|---|---|
| 10 | 20 | 0.9 | 1.4 | 2.0 | 5.4 | 298 / 393 | 3.0 |
| 20 | 40 | 2.9 | 3.9 | 5.0 | 8.4 | 566 / 895 | 5.7 |
| 40 | 81 | 9.0 | 10.6 | 11.6 | 24.3 | 1137 / 1715 | 11.4 |
| 60 | 122 | 18.1 | 20.5 | 22.2 | 35.8 | 1711 / 2604 | 17.1 |
| 100 | 203 | 45.4 | 50.7 | 53.5 | 57.2 | 2857 / 5492 | 28.6 |

- Simulation: targeting is still O(n²) per tick, but 200 units cost ~45 µs
  per step (~0.9 ms CPU per match-second): not the bottleneck. Keeping
  the live-position semantics (units move during the tick) was preferred
  over a per-tick sorted snapshot that would change behavior.
- Server memory: `Unit` state for 200 units is ~12 KB per match.
- **Bandwidth is the real cost**: full views at 10 Hz exceed the 6 KB/s
  budget from about 20 units a side (28.6 KB/s at 100). Typical bot
  matches peak at 21–40 living units in total (≈ 3–6 KB/s). The
  proportional fix is delta views (only changed units), listed in
  ARCHITECTURE §12 "Later"; recommended for P30-B. No unit limit was added
  (BR-U10 not used).
- Client FPS with 100 units a side: not measured yet (open).

### Deviations

- Beetling archetype: the table said "—"; the code needs one, `Dps` chosen.
- The Beetling keeps its Nester's tier and branch, so Venom/Blight are the
  Beetling's own forms (no new modifier fields needed).
- A Nester whose Beetlings are capped stays ready and hatches as soon as
  one expires or dies.
- Test 10 checks sync and refusals online; "more than eight alive" is
  proven in the simulation tests (see above).
