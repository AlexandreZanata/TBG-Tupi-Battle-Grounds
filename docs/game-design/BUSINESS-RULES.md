# TBG Business Rules

> Numbered, testable rules. Each rule carries a status:
> **Confirmed** (legacy, unchanged) · **Adapted** (legacy, changed for 2.0) ·
> **Proposed** (new, awaiting validation) · **Retired** (legacy tech decision).
> Sources in [TRACEABILITY.md](TRACEABILITY.md). No invented balance numbers.

## Match

| ID | Rule | Status |
|----|------|--------|
| BR-M01 | A match is a live session between exactly two players, ending in victory, defeat, or forfeit | Confirmed |
| BR-M02 | There are no sub-rounds or preparation phases; combat and shopping overlap in real time | Confirmed |
| BR-M03 | Disconnection is handled by a reconnect window, then timeout/forfeit under documented rules | Adapted (legacy required documented policy; exact policy still to be written in Phase 04) |

## BattleLine

| ID | Rule | Status |
|----|------|--------|
| BR-B01 | Combat resolves on a single horizontal lane; units have no lateral movement | Confirmed |
| BR-B02 | Deployment zones are per-player; a unit enters the lane only through a validated purchase-and-deploy action | Adapted (legacy implied; deployment validation now an explicit server duty) |

## Units and combat

| ID | Rule | Status |
|----|------|--------|
| BR-U01 | Once deployed, a unit moves and fights automatically with no player control | Confirmed |
| BR-U02 | Every unit has exactly one `Archetype`, one `Faction`, stat block, and energy cost | Confirmed |
| BR-U03 | Every unit occupies a `SlotCount` of one or two; larger units must be visually and mechanically consistent with their slot count | Confirmed |
| BR-U04 | Combat outcomes (damage, deaths, match result) are computed by the authoritative simulation only | Confirmed |
| BR-U05 | Cosmetic items never change stats, hitboxes, or behavior | Confirmed |

## Energy

| ID | Rule | Status |
|----|------|--------|
| BR-E01 | `Energy` is the only in-match spendable resource; it is distinct from any metagame or premium currency | Confirmed |
| BR-E02 | `Energy` regenerates over time according to server-owned rules | Confirmed |
| BR-E03 | Purchases and evolutions that exceed available `Energy` are rejected by the server | Adapted (legacy: client predicts, server reconciles on rejection) |

## Shop

| ID | Rule | Status |
|----|------|--------|
| BR-S01 | Each player is offered a rotating shop of exactly three unit options per cycle | Confirmed |
| BR-S02 | Shop rotation timers are synchronized by the authoritative server so both players share fair timing | Confirmed |
| BR-S03 | The shop offers units only; it never sells power, stats, or competitive advantage directly | Confirmed |

## Evolution

| ID | Rule | Status |
|----|------|--------|
| BR-V01 | A deployed unit may be upgraded along a branching choice tree (a choice at one tier conditions the next) | Confirmed |
| BR-V02 | Evolution is an in-match decision paid with `Energy` (or otherwise gated by in-match rules), never by out-of-match purchases | Adapted (legacy glossary defines evolution as in-match; payment source confirmed here as in-match-gated) |
| BR-V03 | Evolution never applies to cosmetic-only items | Confirmed |

## Synergy

| ID | Rule | Status |
|----|------|--------|
| BR-Y01 | Fielding units of the same `Faction` (or a defined cross-faction combo) together grants a bonus effect | Confirmed |
| BR-Y02 | Synergy rules are data, owned by the simulation, and identical for both players in a match | Adapted (legacy: match module owns synergy truth; 2.0: `tbg-core` owns it, server runs it) |

## Player intentions and authority

| ID | Rule | Status |
|----|------|--------|
| BR-A01 | The client sends `PlayerIntention`s (purchase, evolve, reposition); nothing the client sends is a state change until validated | Confirmed |
| BR-A02 | The server validates every intention against energy, shop, evolution, and match state before applying it | Confirmed |
| BR-A03 | The client may predict for responsiveness but must reconcile to authoritative state on rejection | Confirmed |
| BR-A04 | Client-reported damage, balances, and results are never trusted | Confirmed |

## Monetization and inventory

| ID | Rule | Status |
|----|------|--------|
| BR-C01 | Monetization is free-to-play with cosmetic items only (skins, VFX, emotes, board themes) | Confirmed |
| BR-C02 | Persistent cosmetics are bound to a player account inventory | Confirmed |
| BR-C03 | Store purchases are granted only after server-side receipt validation | Adapted (legacy named StoreKit/Play Billing via Nakama; 2.0 defers provider integration but keeps the validation principle) |

## Retired technology-specific decisions (kept out of the rules above)

| ID | Retired decision | Replacement direction |
|----|------------------|-----------------------|
| RT-01 | Nakama as game server, TypeScript match module (ADR-002) | Rust + Axum + Tokio authoritative server (`tbg-server`) |
| RT-02 | Unity client + Nakama client SDK (ADR-001 stack) | Bevy + Rust/WASM browser client (`tbg-client`) |
| RT-03 | PostgreSQL on VPS as the only persistence story | SQLite for initial persistence (`tbg-server`); managed DB only if CCU demands it |
| RT-04 | Per-character Blender/MB-Lab/Mixamo rigging pipeline (ADRs 003–006) | Modular/shared-rig procedural asset factory (`tools/asset-factory/`); see `docs/architecture/ASSET-PIPELINE.md` |
| RT-05 | Native iOS/Android distribution via stores | Browser-first delivery; native wrappers deferred |
