# Infrastructure Overview

> Technical architecture for TBG: live PvP, authoritative server, F2P cosmetics — on a **single 16 GB VPS**, solo developer.  
> **Related:** [ADR-001](../adr/ADR-001-real-time-competitive-design.md), [ADR-002](../adr/ADR-002-nakama-self-hosted-infrastructure.md)

**Reality check:** Option A (real-time + authoritative + IAP) is the heaviest engineering path. This document prioritizes **minimal daily ops** so time remains for design, balance, and art.

---

## 1. Architecture diagram

```
┌─────────────────────────────────────────────────────────────────┐
│ CLIENT (Unity — iOS / Android)                                  │
│ - Rendering, input, local prediction                              │
│ - Nakama client SDK                                             │
└────────────────────────────┬────────────────────────────────────┘
                             │ HTTPS / WSS (TLS)
                             ▼
┌─────────────────────────────────────────────────────────────────┐
│ VPS (16 GB) — Hostinger                                         │
│                                                                 │
│  ┌─────────────┐     ┌─────────────────────────────────────┐   │
│  │ Nginx       │────▶│ Nakama (game server)                │   │
│  │ reverse     │     │ - Auth, matchmaking                 │   │
│  │ proxy + TLS │     │ - Authoritative match logic         │   │
│  └─────────────┘     │ - Leaderboards, storage             │   │
│                      └──────────────┬──────────────────────┘   │
│                                     │                            │
│                                     ▼                            │
│                      ┌─────────────────────────────────────┐   │
│                      │ PostgreSQL                          │   │
│                      │ - Accounts, inventory, matches      │   │
│                      └─────────────────────────────────────┘   │
│                                                                 │
│  IAP receipt validation (Nakama module or companion service)  │
│  Monitoring: Uptime Kuma (+ Prometheus/Grafana optional)      │
│                                                                 │
│  Orchestration: Docker Compose                                  │
└────────────────────────────┬────────────────────────────────────┘
                             │
                             ▼
┌─────────────────────────────────────────────────────────────────┐
│ External: Apple App Store / Google Play (IAP receipt APIs)      │
└─────────────────────────────────────────────────────────────────┘
```

---

## 2. Why Nakama

Solo developers should **not build netcode and game-server plumbing from scratch** — months of invisible engineering work.

**Nakama** (open source, Heroic Labs) provides:

| Capability | Benefit |
|------------|---------|
| Player authentication | Email, device, social |
| Matchmaking | Session creation |
| Authoritative match modules | Go, Lua, or **TypeScript** (recommended for solo maintainability) |
| Leaderboards | Ranking |
| Player storage | Inventory, progress |
| Social (optional) | Chat, friends — later |

Runs entirely on your VPS — **no license fee**, no third-party CCU billing.

Custom code scope: **TBG-specific rules only** — energy, shop rotation, evolution, synergies, combat resolution.

### Why not Photon Fusion

Photon is faster to integrate in some cases but is **hosted and paid per CCU**. It does not run on the owned VPS. For a small initial base, self-hosted Nakama on an already-paid VPS is more sustainable financially.

---

## 3. Stack components

### 3.1 Game server — Nakama

- Docker container.
- Match logic in **TypeScript module** (truth source); Unity sends intentions, receives validated state.
- Alternative: Go for higher performance if profiling demands it later.

### 3.2 Database — PostgreSQL

- Nakama's native persistence layer.
- Separate container, persistent Docker volume.
- Sufficient for initial scale on same VPS — no managed DB required at launch.

### 3.3 Reverse proxy and TLS — Nginx + Certbot

- Nginx routes HTTPS/WSS to Nakama.
- Let's Encrypt via Certbot — required for store policies and secure WebSockets.

### 3.4 In-app purchases

- Prefer **Nakama native Apple/Google receipt validation**.
- Flow: client purchases → store receipt → client sends to Nakama → Nakama validates with store → cosmetic granted in PostgreSQL inventory.

### 3.5 Monitoring

| Tool | Purpose |
|------|---------|
| **Uptime Kuma** | Service up/down alerts (Telegram, Discord, email) |
| **Prometheus + Grafana** (optional) | CCU, latency, CPU/RAM — Nakama exports metrics |

### 3.6 Backups

- Daily automated `pg_dump` to external storage (Backblaze B2, S3-compatible bucket).
- **Test restores** periodically — backup without tested restore is not backup.

---

## 4. Docker Compose layout

Conceptual skeleton (not production-final):

```yaml
services:
  nginx:
    image: nginx:latest
    ports: ["80:80", "443:443"]
    depends_on: [nakama]

  certbot:
    image: certbot/certbot
    # automatic certificate renewal

  postgres:
    image: postgres:15
    volumes: ["pgdata:/var/lib/postgresql/data"]
    # credentials via .env — never hardcoded

  nakama:
    image: heroiclabs/nakama:latest
    volumes: ["./game-modules:/nakama/data/modules"]
    depends_on: [postgres]
    ports: ["7349:7349", "7350:7350", "7351:7351"]

  uptime-kuma:
    image: louislam/uptime-kuma
    ports: ["3001:3001"]

volumes:
  pgdata:
```

### Why Docker Compose, not Kubernetes

Kubernetes adds operational complexity with no benefit on a **single VPS** and solo maintainer. Compose delivers isolation, reproducibility, and simple deploy with minimal overhead.

---

## 5. VPS capacity (16 GB RAM)

Approximate allocation:

| Service | RAM |
|---------|-----|
| PostgreSQL | 1–2 GB |
| Nakama | 1–3 GB (scales with concurrent matches) |
| Nginx | ~100 MB |
| Uptime Kuma | ~100 MB |
| OS + headroom | ~2 GB |
| **Available for match peaks** | **~8–10 GB** |

Supports **hundreds to low thousands of concurrent players** depending on match logic weight. Scale path documented in section 8.

---

## 6. Deploy pipeline (solo CI/CD)

```
1. Develop match module locally (TypeScript)
        ↓
2. Test with local Docker Compose
        ↓
3. Push to GitHub (main)
        ↓
4. GitHub Actions — build + basic automated tests
        ↓
5. Deploy via SSH: docker compose pull && docker compose up -d
```

A ~10-line SSH deploy script is sufficient for single-VPS phase — no elaborate pipeline required at launch.

---

## 7. Security essentials

| Control | Requirement |
|---------|-------------|
| Firewall (UFW) | Open only 80, 443; Nakama/Postgres on internal Docker network |
| Secrets | `.env` only — never in version control |
| OS updates | Enable unattended security upgrades (Ubuntu) |
| Rate limiting | Nginx on public endpoints (login, matchmaking) |
| Backups | Automated + restoration tested |
| IAP | Server-side receipt validation only — never trust client alone |

---

## 8. Stack summary

| Layer | Choice |
|-------|--------|
| Client | Unity (C#) + Nakama client SDK |
| Game server | Nakama (self-hosted, Docker, TypeScript match module) |
| Database | PostgreSQL (container, persistent volume) |
| Proxy / TLS | Nginx + Certbot |
| IAP | Nakama native validation (Apple / Google) |
| Monitoring | Uptime Kuma (+ Prometheus/Grafana if needed) |
| Backup | Cron `pg_dump` → external object storage |
| Orchestration | Docker Compose |
| CI/CD | GitHub Actions + SSH deploy |
| Hosting | 16 GB VPS (Hostinger) — initial phase |

---

## 9. Scaling evolution

| Phase | Topology |
|-------|----------|
| **Now** | Everything on one VPS, Docker Compose, single Nakama node |
| **Growth** | Move PostgreSQL to dedicated server or managed service |
| **More growth** | Nakama cluster + load balancer |
| **Advanced** | Multi-region nodes, CDN for game assets |

Only invest in later phases when CCU and latency metrics demand it.

---

## 10. Next steps

- [ ] Provision VPS: Docker, Docker Compose, UFW
- [ ] Local Nakama + PostgreSQL — validate match module
- [ ] Implement match rules: energy, shop, evolution (TypeScript)
- [ ] Domain + Nginx + Certbot on VPS
- [ ] PostgreSQL backup automation before public launch
- [ ] Uptime Kuma → personal alert channel
