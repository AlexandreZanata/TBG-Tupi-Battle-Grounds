# ADR-002: Nakama Self-Hosted on Single VPS

**Status:** Accepted  
**Date:** 2026-06-30  
**Deciders:** Engineering (solo dev)

## Context

[ADR-001](ADR-001-real-time-competitive-design.md) requires an authoritative game server, persistent accounts, matchmaking, and IAP validation. The project runs as a **solo developer** with a **single 16 GB VPS** (Hostinger) and a small initial player base.

Building custom netcode and game-server infrastructure from scratch would consume months without player-visible differentiation.

## Decision

Use **Nakama** (Heroic Labs, open source) **self-hosted via Docker Compose** on the existing VPS as the game server platform. Write match logic as a **custom Nakama module in TypeScript**.

Supporting stack on the same VPS:

- **PostgreSQL** — persistence (accounts, inventory, matches, leaderboards)
- **Nginx + Certbot** — reverse proxy, TLS/WSS
- **Uptime Kuma** — uptime alerts (Telegram/Discord/email)
- **Cron backup** — `pg_dump` to external object storage (e.g. Backblaze B2)

Orchestration: **Docker Compose** (not Kubernetes).

## Consequences

### Positive

- No license cost; runs on already-paid VPS.
- Built-in auth, matchmaking, leaderboards, storage, IAP receipt hooks.
- Solo-friendly ops: single `docker compose` workflow for deploy and restart.
- 16 GB RAM sufficient for hundreds to low thousands of concurrent users at initial scale.

### Negative

- DevOps responsibility falls entirely on the solo developer (updates, backups, monitoring).
- TypeScript module performance adequate for MVP; Go may be needed at extreme scale.
- Single VPS is a single point of failure until horizontal scaling (documented evolution path).

## Alternatives considered

| Option | Rejected because |
|--------|------------------|
| Custom Node/C# game server | Months of netcode work; no player-facing benefit |
| Photon Fusion | Paid per CCU; hosted externally; does not use owned VPS |
| Firebase/Supabase only | Insufficient for authoritative real-time match simulation |
| Kubernetes on one VPS | Operational overkill for solo dev and single node |

## Implementation notes

- Match module is **source of truth** for energy, shop rotation, evolution, synergies, and combat.
- Unity client sends intentions; server returns validated state.
- IAP: Nakama native Apple/Google receipt validation preferred over separate microservice.
- Secrets in `.env` only — never committed.
- PostgreSQL not exposed publicly; firewall (UFW) on ports 80/443 only.

Full topology and deploy pipeline: [infrastructure-overview.md](../architecture/infrastructure-overview.md).

## Follow-up actions

- [ ] Provision VPS: Docker, Docker Compose, UFW
- [ ] Run Nakama + PostgreSQL locally first; validate match module
- [ ] Configure domain, Nginx, Certbot before public beta
- [ ] Automated PostgreSQL backup before any real players
- [ ] Uptime Kuma alert to personal Telegram/Discord
