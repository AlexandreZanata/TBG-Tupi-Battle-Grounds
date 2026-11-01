# Publishing the static web frontends

`publish.sh` (local) and `remote.sh` (host, run through `sudo`) publish the landing and the
admin placeholder **without touching the game backend**: no `deploy/deploy.sh --provision`,
no server restart, no release-slot change, no proxy rule for the API/app hosts.

What the scripts may write on the host, and nothing else:

| Path | Purpose |
|------|---------|
| `/srv/tbg/landing/releases/<id>/`, `/srv/tbg/admin/releases/<id>/` | Immutable releases (root-owned, world-readable) |
| `/srv/tbg/{landing,admin}/current` | Symlink to the active release, swapped atomically (`rename(2)`) |
| `/srv/tbg/{landing,admin}/activations.log` | Activation history used by rollback (not served) |
| `/etc/caddy/tbg-web/sites.caddy` | The rendered [Caddyfile.web.template](Caddyfile.web.template) |
| `/etc/caddy/Caddyfile` | Only appends `import tbg-web/*.caddy` when missing |
| `/var/backups/tbg-web/` (mode 700) | Private copies of the Caddy configuration before every check/change |
| `~/tbg-web-upload/` (mode 700) | Upload staging for the SSH user |

Never touched: `/srv/tbg/web`, `/srv/tbg/updates`, `/opt/tbg`, `/var/lib/tbg`,
`/var/backups/tbg`, SSH, firewall, DNS. Uploads go into fresh directories; nothing uses
`rsync --delete`. A local private copy of the live Caddyfile is also kept in
`${XDG_STATE_HOME:-~/.local/state}/tbg-web-deploy/` (mode 700).

Host selection: `--host <ssh alias>`, `TBG_WEB_SSH`, or the ignored `deploy/.env.deploy`.
Domains: `TBG_SITE_DOMAIN` (default `tupibg.com`, `www.` redirects to it) and
`TBG_ADMIN_DOMAIN` (default `admin.<site>`). Commands that change what the public hosts
serve require `--confirm`, given only after the owner's review.

## First publication (after owner review)

```bash
web/deploy/publish.sh check                                    # local, no network
web/deploy/publish.sh --host <alias> upload landing             # prints the release id
web/deploy/publish.sh --host <alias> upload admin
web/deploy/publish.sh --host <alias> --confirm activate landing <release-id>
web/deploy/publish.sh --host <alias> --confirm activate admin <release-id>
web/deploy/publish.sh --host <alias> caddy-check                # backup + validate, applies nothing
web/deploy/publish.sh --host <alias> --confirm caddy-apply      # validate, install, graceful reload
```

`upload` refuses uncommitted changes under `web/`, the patch catalog or the reused art, runs
`check.py`, stages with `stage.py` and installs a release named `<UTC time>-<commit>`.
`caddy-apply` validates the candidate (live file + import line + include) with
`caddy validate --adapter caddyfile`, installs it, validates the installed file again and
reloads; if the reload fails it restores the backup and reloads again. On the first reload
Caddy requests certificates for the three new hosts with the same ACME flow as the API/app
hosts; Cloudflare stays on Full (strict).

Verify afterwards:

```bash
curl -sI https://tupibg.com/ | head -5                      # 200, CSP and HSTS headers
curl -sI 'https://www.tupibg.com/x?y=1' | grep -i location   # https://tupibg.com/x?y=1
curl -sI https://admin.tupibg.com/ | grep -i -E 'x-robots|content-security'
curl -s https://api.tupibg.com/health                       # still ok
curl -s -o /dev/null -w '%{http_code}\n' https://tupibg.com/api/release   # 404 (no proxy here)
```

## Later releases and rollback

- New content: `upload` then `activate … --confirm`. The previous release stays on disk.
- Content rollback: `--confirm rollback landing` (or `admin`) re-points `current` to the
  previously activated release; no file is deleted.
- Remove the web hosts entirely: `--confirm caddy-disable` takes the include out and reloads;
  the API/app blocks never change. The Caddy backups in `/var/backups/tbg-web/` restore any
  earlier state by hand if ever needed.
- `status` lists releases, active links and the include state.

## Interaction with backend provisioning

`deploy/Caddyfile.domains.template` now ends with `import tbg-web/*.caddy`, so a future
`deploy.sh --provision` keeps serving the web hosts; on a fresh host the empty glob is only
a warning. A host provisioned from an older template loses the import line; the next
`caddy-check`/`caddy-apply` adds it back.

## Validation record (2026-11-01, not a publication)

- Offline with Caddy 2.10.2 (Docker, no network): the repository template plus include, the
  template without include, and the live file plus import line plus include all validate.
  Adapting the live file and the candidate to JSON shows identical API/app routes (after
  normalizing generated group ids), identical server settings (Cloudflare `trusted_proxies`,
  `client_ip_headers`) and a 404 handler scoped to the landing host only.
- On the VPS (Caddy 2.11.2) `caddy-check` validated the candidate. Afterwards the live
  Caddyfile hash was unchanged, no include was installed, `/srv/tbg` was untouched, Caddy and
  the game server stayed active and the API answered `ok`. Nothing was activated or reloaded.
