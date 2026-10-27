# Infrastructure — TBG 2.0

> Hosting, persistence, and deployment plan for the reboot, adapted from
> legacy `archive/docs/architecture/infrastructure-overview.md`.
> Single-server principle kept; Nakama/PostgreSQL stack retired.

## Topology (target)

```
Browser (Bevy WASM + assets, static)
   │  HTTPS (pages + assets) · WSS (game)
   ▼
Single VPS
 ├─ Reverse proxy + TLS (Nginx or Caddy + Let's Encrypt)
 ├─ tbg-server (native binary, Axum + Tokio, WebSocket)
 ├─ SQLite database file (persistent volume, backed up)
 └─ Static file service (WASM bundle, GLBs, content JSON)
       — served from the same VPS initially; CDN when metrics demand it
```

Legacy mapping: Nakama → `tbg-server`; PostgreSQL → SQLite;
Unity clients → browser clients. Uptime monitoring, firewall posture,
secret handling, and backup discipline carry over unchanged.

## Stack

| Layer | Choice | Notes |
|-------|--------|-------|
| Game server | `tbg-server` (Rust, Axum + Tokio) | Authoritative fixed-tick simulation via `tbg-core` |
| Persistence | SQLite (`rusqlite`, bundled) | Accounts, inventory, match records, rankings; zero-ops at initial scale |
| Proxy / TLS | Nginx or Caddy + Let's Encrypt | Required for secure WebSockets and store/browser policy |
| Static hosting | Same VPS initially → CDN for WASM/assets on growth | WASM + GLB are cache-friendly immutable artifacts |
| Monitoring | Uptime checks + server metrics endpoint | Alert to maintainer channel; Prometheus/Grafana optional |
| Backups | Scheduled SQLite snapshots to external object storage; restores tested | Backup without tested restore is not a backup |
| Orchestration | Docker Compose on one node | No Kubernetes at this scale |
| CI/CD | Hosted CI (build, fmt, check, test) + SSH deploy | ~10-line deploy script is sufficient |

## Persistence notes (SQLite)

- Single-writer model fits the authoritative server; WAL mode for
  read concurrency.
- Keep a migration story from day one of Phase 04 (versioned schema
  migrations, even if hand-rolled SQL files in `deploy/`).
- Move to a managed database only when concurrent-user metrics demand
  it — that decision gets its own `docs/decisions/` record.

## Security essentials

| Control | Requirement |
|---------|-------------|
| Firewall | Open 80/443 only; game/admin ports on internal network |
| Secrets | Environment only, never committed; `.env` gitignored |
| Updates | Unattended security upgrades on the VPS |
| Rate limiting | Proxy-level limits on login/matchmaking endpoints |
| Validation | All IAP-equivalent grants (when they exist) validated server-side |
| Dependency audit | `cargo audit`-class review before beta; license check on every new crate |

## Scaling evolution (unchanged principle, new components)

| Phase | Topology |
|-------|----------|
| Now → public beta | Everything on one VPS, Compose, single `tbg-server` node |
| Growth | SQLite → managed DB; static assets → CDN |
| More growth | Multi-node `tbg-server` + load balancer; sticky matches |
| Advanced | Multi-region, edge-cached assets |

Invest only when CCU and latency metrics demand it.

## `deploy/` contents (Phase 00 scaffolding)

- `docker-compose.yml` skeleton (proxy + server + volume comments).
- `vps-setup.md` checklist (firewall, TLS, backups, monitoring).
- No production secrets, no live hostnames, no real credentials.
