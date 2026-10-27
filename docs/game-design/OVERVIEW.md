# TBG Game Design — Overview

> Product identity and design pillars for **Tupi Battle Grounds (TBG)**.
> Gameplay concepts are carried over from the legacy design; implementation
> technologies are not. See [TRACEABILITY.md](TRACEABILITY.md) for sources.

## Identity

**Tupi Battle Grounds** is an open-source, browser-first, real-time 1v1
strategy game featuring automatic single-lane combat, evolving units, and
faction synergies.

- No micromanagement: units move and fight automatically once deployed.
- Single lane: all combat resolves on one `BattleLine`.
- Depth comes from shop decisions, deployment timing, branching evolutions,
  and faction synergies — not from twitch control.

## Design pillars (confirmed legacy)

| # | Pillar | Meaning |
|---|--------|---------|
| 1 | Live 1v1 PvP | Both players share one match simultaneously; the board evolves in real time |
| 2 | Single lane | One `BattleLine`, no lateral movement |
| 3 | Automation | Unit movement and combat resolve without player input |
| 4 | Continuous shop | Unit offers rotate during combat; no isolated preparation phase |
| 5 | Branching evolution | Deployed units upgrade along a choice tree |
| 6 | Faction synergies | Fielding same-faction (or defined-combo) units grants bonuses |
| 7 | Authoritative simulation | The server owns truth for competitive outcomes |
| 8 | Cosmetics-only monetization | No pay-to-win; visual items never change stats |

Source: `archive/docs/adr/ADR-001-real-time-competitive-design.md`.

## Scope decisions (explicit, not silently reconciled)

| Decision | Status |
|----------|--------|
| Legacy MVP roster target of **12 unique units** | Confirmed legacy scope target (`MVPUnitRoster` in legacy glossary) |
| New proposal to begin 2.0 content work with **8 units** | **Proposed, awaiting validation** — a reboot scoping decision to reach a playable battle loop sooner; it does not change the 12-unit MVP target, only the order of delivery |
| Tick rate 10–20 ticks/second | Legacy proposal, never finalized — **proposed, awaiting technical spike** (see `docs/architecture/NETWORKING.md`) |
| Mobile-native clients (iOS/Android via Unity) | **Retired** — 2.0 is browser-first (desktop + mobile browsers); native apps deferred to post-beta if ever |

## What this document set covers

| Document | Contents |
|----------|----------|
| [BUSINESS-RULES.md](BUSINESS-RULES.md) | Numbered rules with status (confirmed / adapted / proposed / retired) |
| [GLOSSARY.md](GLOSSARY.md) | Ubiquitous language; code and docs must use these terms exactly |
| [MECHANICS.md](MECHANICS.md) | How a match plays: energy, shop, evolution, synergy, combat |
| [FACTIONS.md](FACTIONS.md) | The four factions and five archetypes |
| [TRACEABILITY.md](TRACEABILITY.md) | Every rule traced to its legacy source path |

## Non-goals for the design documents

- No numeric balance values (costs, damage, timers) are established here.
  Any numbers appearing in legacy JSON or examples are illustrative, not canon.
- No asset, UI, or control scheme is specified here.
- Technology choices (engine, server, database) live in `docs/architecture/`,
  not in game-design documents.
