# deploy/

Deployment artifacts for the single-VPS model
(see `docs/architecture/INFRASTRUCTURE.md`).

- `docker-compose.yml` — proxy + `tbg-server` + volume skeleton (Phase 06
  hardens it; Phase 00 keeps it structural, no live hostnames).
- `vps-setup.md` — provisioning checklist: firewall, TLS, backups,
  monitoring.

Rules: no production secrets, hostnames, or credentials in version
control. Ever.
