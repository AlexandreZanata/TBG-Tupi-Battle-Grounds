# Traceability — Business Rules to Legacy Sources

> Every preserved rule traces to a concrete legacy path. "Adapted" marks
> rules whose technology-specific wording changed for 2.0; the gameplay
> concept is unchanged.

## Sources

| ID | Legacy source path | Description |
|----|--------------------|-------------|
| S1 | `archive/docs/adr/ADR-001-real-time-competitive-design.md` | Competitive pillars: live PvP, continuous shop, cosmetics-only, authoritative server |
| S2 | `archive/docs/adr/ADR-002-nakama-self-hosted-infrastructure.md` | Self-hosted authoritative server, single-VPS model, server-side validation |
| S3 | `archive/docs/architecture/infrastructure-overview.md` | Topology, tick/intention flow, security essentials |
| S4 | `archive/docs/GLOSSARY.md` | Ubiquitous language: all domain terms, enums, MVP roster of 12 |
| S5 | `archive/content/characters/definitions/_schema.example.json` + `_schema.mesh3d.example.json` | Unit shape: archetype, faction, slot-relevant proportions, evolution stage |
| S6 | `archive/content/characters/behaviors/*.json` | Per-archetype presentation profiles (retired as canon, kept as naming reference) |

## Rule-to-source map

| Rule(s) | Source(s) | Status note |
|---------|-----------|-------------|
| BR-M01, BR-M02 | S1 (match mode, shop pillars) | Confirmed |
| BR-M03 | S1 (disconnect policy follow-up), S3 §match lifecycle | Adapted: policy still to be written |
| BR-B01 | S4 (`BattleLine`), S1 | Confirmed |
| BR-B02 | S2/S3 (intentions → validated state) | Adapted: deployment validation made explicit |
| BR-U01, BR-U04 | S1 (authoritative server), S2 (match module is truth) | Confirmed |
| BR-U02, BR-U03 | S4 (`Unit`, `Archetype`, `Faction`, `Slot`), S5 | Confirmed |
| BR-U05, BR-C01 | S1 (cosmetics-only monetization) | Confirmed |
| BR-E01, BR-E02 | S4 (`Energy`), S2 (energy owned by match module) | Confirmed |
| BR-E03 | S1 (prediction with server reconciliation on rejection) | Adapted |
| BR-S01, BR-S02 | S4 (`Shop`), S1 (fair rotation timers) | Confirmed |
| BR-S03 | S1 (no pay-to-win) | Confirmed |
| BR-V01 | S4 (`Evolution`) | Confirmed |
| BR-V02, BR-V03 | S4 + S1 (in-match vs account progression split) | Adapted/confirmed as noted |
| BR-Y01 | S4 (`Synergy`), S2 (synergies in match module) | Confirmed |
| BR-Y02 | S2 (single truth source) | Adapted: owner is now `tbg-core` data + `tbg-server` runtime |
| BR-A01–BR-A04 | S1 (intentions; tick 10–20 proposed), S2/S3 (validate everything; never trust client) | Confirmed; tick rate stays proposed |
| BR-C02, BR-C03 | S1 (inventory, server-side receipt validation), S2 (IAP flow) | BR-C03 adapted: provider integration deferred |
| RT-01–RT-05 | S1–S3 stack sections, ADRs 003–007 pipeline history | Retired with replacement direction |

## Explicitly not established (no legacy source found)

Shop reroll/weighting/duplicates · energy capacity and regen numbers ·
slot cap per player · combat formulas and ranges · evolution timing and
gating details · synergy thresholds and bonus values · reconnect window
length and forfeit policy · tick rate final value · starter-8 roster list.

These must be proposed and playtested in Phases 02–05, never back-filled
as "legacy canon".
