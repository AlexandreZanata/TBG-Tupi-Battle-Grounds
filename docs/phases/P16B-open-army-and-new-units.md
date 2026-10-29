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

_Executor fills this in: baseline and after tables, performance table,
proposed balance changes with evidence, fun notes, deviations._
