# VPS Setup Checklist (target state, Phase 06)

- [ ] Provision small VPS; enable unattended security upgrades.
- [ ] Firewall (UFW): open 80/443 only; game/admin ports internal.
- [ ] Reverse proxy + Let's Encrypt TLS (Nginx or Caddy).
- [ ] Docker Compose deploy: `docker compose up -d` from `deploy/`.
- [ ] SQLite volume on persistent storage; scheduled snapshots to
  external object storage; **test a restore before beta**.
- [ ] Uptime checks to the maintainer channel.
- [ ] Secrets via environment only; none in this repository.
