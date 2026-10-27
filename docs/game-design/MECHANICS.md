# TBG Mechanics

> How a match plays, derived from legacy design. Mechanics describe
> player-visible behavior; numeric balance is explicitly **not** set here.
> Rule IDs refer to [BUSINESS-RULES.md](BUSINESS-RULES.md).

## Match lifecycle

1. Two players are paired into a live `Match` (BR-M01).
2. Both boards evolve in real time; there is no waiting on the opponent
   and no separate preparation phase (BR-M02).
3. The match ends in victory, defeat, or forfeit. Disconnects get a
   reconnect window first; the exact window and forfeit policy are a
   Phase 04 deliverable (BR-M03, proposed detail).

## Energy economy

- `Energy` regenerates over time on server-owned rules (BR-E02).
- Spending paths: purchasing units from the shop, triggering evolutions
  (BR-E01, BR-V02).
- Overspend attempts are rejected by the server; the client reconciles
  its prediction to the rejection (BR-E03, BR-A03).
- Balance numbers (capacity, regen rate, costs): **not established**.
  No legacy source fixes them; they will be set by playtesting in Phase 03.

## Shop cycle

- Each player sees three unit options per rotation cycle (BR-S01).
- Rotation timers are server-synchronized so neither player gains a
  timing advantage (BR-S02).
- Buying reserves/assigns the unit to deployment; deployment itself is a
  validated `PlayerIntention` (BR-B02, BR-A01).
- Reroll mechanics, offer weighting, and duplicate handling: **not
  established** — no legacy source defines them. Proposed during Phase 03.

## Deployment and the BattleLine

- Units deploy into the player's zone of the single lane (BR-B01, BR-B02).
- Each unit occupies 1 or 2 slots (`SlotCount`); slot limits per player
  cap board presence (BR-U03).
- The per-player slot cap value is **not established**.

## Automatic combat

- After deployment, units advance and fight without player input (BR-U01).
- Targeting, damage, and death are resolved by the authoritative
  simulation (BR-U04).
- The browser client interpolates positions and plays effects from
  authoritative snapshots; it never decides outcomes (BR-A04).
- Combat formulas, ranges, and archetype interactions: **not established**.

## Evolution

- An eligible deployed unit offers a branching upgrade choice
  (BR-V01) — e.g. two tier-1 options leading to dependent tier-2 options.
- Evolution timing (mid-combat, instant vs. channeled) and gating beyond
  in-match resources: **not established**.

## Synergies

- Units sharing a `Faction` (or a defined combo) grant bonus effects
  while jointly active (BR-Y01).
- Synergy definitions are simulation-owned data, identical for both
  players (BR-Y02).
- Which bonuses, thresholds (e.g. counts), and any cross-faction combos:
  **not established**.

## Player intentions (client ↔ server)

- The client only ever submits intentions: purchase, evolve, reposition
  (BR-A01). Purchases against an empty/insufficient-energy or rotated
  shop are rejected, not queued (BR-E03).
- Intention set for Phase 01–03 local play is limited to what the bot
  match supports; the full networked intention protocol arrives in
  Phase 04 (`tbg-protocol`).

## Monetization boundary

- Nothing in the mechanics above may be gated by out-of-match purchases
  (BR-C01, BR-V03, BR-U05). Cosmetics (BR-C01) attach to presentation
  only and are invisible to the simulation.
