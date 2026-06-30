# ADR-001: Real-Time Competitive Design (Option A)

**Status:** Accepted  
**Date:** 2026-06-30  
**Deciders:** Product / engineering (solo dev)

## Context

TBG (Tupi Battle Grounds) is an auto lane strategy game inspired by real-time competitive mobile titles. Before production code, four foundational product decisions must be locked because they directly shape netcode, backend architecture, monetization, and UI.

The alternative path (async PvP, turn-based shop, paid power, client-authoritative simulation) was rejected in favor of a live competitive model.

## Decision

Adopt **Option A** across all four pillars:

| Pillar | Decision |
|--------|----------|
| Match mode | **Live PvP** — both players in the same match simultaneously, board evolves in real time |
| Shop | **Continuous in-match shop** — unit offers rotate during combat; no isolated preparation phase |
| Monetization | **Free-to-play with cosmetics only** — no pay-to-win; skins, VFX, emotes, board themes |
| Multiplayer authority | **Authoritative game server** — server validates all gameplay outcomes; client predicts for responsiveness |

### Technical stack aligned with this decision

```
Client:           Unity (C#), 2D, Nakama client SDK
Game server:      Nakama (self-hosted) — match state, tick rate, authoritative simulation
Simulation:       Deterministic C# logic shared between client (prediction) and server (truth)
Accounts:         Persistent login (required for cosmetics and cross-device inventory)
Support backend:  Inventory, IAP receipt validation, leaderboards, profiles
IAP:              StoreKit (iOS) + Google Play Billing, validated server-side
Anti-cheat:       Server-side validation of every player action
CI/CD:            GitHub Actions + Unity Cloud Build (or equivalent)
Observability:    Game server metrics (Grafana/Prometheus or provider-native tooling)
```

See [ADR-002](ADR-002-nakama-self-hosted-infrastructure.md) and [infrastructure-overview.md](../architecture/infrastructure-overview.md) for hosting details.

## Consequences

### Positive

- Competitive integrity: authoritative server prevents client manipulation of energy, damage, and match outcome.
- Fair monetization positioning: cosmetics-only avoids pay-to-win backlash.
- Market alignment: live PvP matches player expectations for the genre (e.g. Clash Royale–style flow).

### Negative

- **High engineering scope** compared to async MVP — requires game server, matchmaking rooms, reconnection handling, and deterministic simulation.
- **Infrastructure cost and ops** — cannot rely on serverless-only backend; dedicated game server process per active match model.
- **UI pressure** — shop and evolution must be low-friction on mobile under time pressure.
- **Latency sensitivity** — netcode must handle interpolation, reconciliation, or lockstep with defined tick rate.

### Required capabilities (by pillar)

**Live PvP**

- Netcode with latency tolerance (interpolation, reconciliation, or lockstep).
- Authoritative match simulation (or server-validated client simulation).
- Match rooms with lifecycle: create → play → end → cleanup.
- Disconnect policy: reconnect, timeout, forfeit rules.

**Continuous shop**

- Mobile-first, always-visible shop UI with minimal tap depth.
- Per-player shop state synchronized via server tick (fair rotation timers).
- Client-side prediction for purchases with server reconciliation on rejection (e.g. insufficient energy).

**F2P cosmetics**

- Cosmetic catalog, purchase flow, and inventory delivery.
- Apple/Google IAP integration — all digital purchases through store billing APIs.
- Server-side receipt validation before granting items.
- Cosmetics strictly separated from balance — no stat, hitbox, or behavior changes from skins.

**Authoritative server**

- Game server maintaining match state for full match duration.
- Tick rate target: 10–20 ticks/second (to be finalized in technical spike).
- All client actions are **intentions**; server approves or rejects.

## Alternatives considered

| Option | Rejected because |
|--------|------------------|
| Async PvP | Does not match competitive real-time genre positioning |
| Turn-based / phased shop | Breaks continuous pressure design; different balance curve |
| Pay-to-win monetization | Conflicts with competitive fairness and store positioning |
| Client-authoritative simulation | Unacceptable cheat surface for ranked PvP |
| Photon Fusion (hosted) | Higher recurring CCU cost; does not run on owned VPS — see ADR-002 |

## Follow-up actions

- [ ] Define match tick rate and acceptable latency budget
- [ ] Design authentication flow (login required from first session for cosmetics)
- [ ] Map Apple/Google receipt validation endpoints before cosmetic store design
- [ ] Build isolated netcode spike (two real devices) before full game investment
- [ ] Document disconnect and forfeit rules in a use case under `docs/use-cases/`
