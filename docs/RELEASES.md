# Official release and deployment guide

Official public origins are `tupibg.com` (landing), `www.tupibg.com` (redirect),
`app.tupibg.com` (browser game), `api.tupibg.com` (game API) and `admin.tupibg.com`
(public preparation page). Connection addresses, SSH pins/keys, databases and
provider credentials stay outside Git. Never deploy to the retired test VPS.

## One reviewed main, independent payloads

Main CI keeps its existing six required lanes and fail-closed `check` aggregator.
The owner reviews and merges the PR manually. Publication workflows accept only a
successful main **push** CI run from this repository and the exact current main SHA;
PRs, forks, stale runs and failed checks cannot publish. Actions and compiler/tools
are pinned. A newer main commit supersedes an older pending publication.

Three source filters avoid redeploying unrelated products:

- `backend-release.yml`: Rust server/core dependencies and runtime inputs, packaged
  as `server-<full SHA>`. A compatible update drains new admission, finishes active
  matches/results/reconnect windows, backs up, switches binaries and checks health.
- `web-release.yml`: the shared Bevy browser client, core contract and its assets,
  packaged as `web-<full SHA>`. Trunk uses `/releases/<SHA>/`; the package sets the
  document base before any relative asset request. All old assets are retained.
  Frontend `no-transform` response headers prevent CDN script injection from changing
  the hash-bound HTML; health requests identify the TBG product explicitly.
- `static-release.yml`: landing, safe public admin, shared art/catalog dependencies,
  packaged as `static-<full SHA>`. No Rust game rebuild or backend restart is needed
  for a static-only change. The fictional admin preview is rejected by the packager
  and host policy. Real privileged admin requires a separate authenticated design.

Every archive has a source/hash manifest and GitHub OIDC build provenance. The VPS
pulls public artifacts over HTTPS and verifies repository identity, source ancestry,
workflow/main/builder identity, hashes, paths and expansion limits before promotion.
GitHub receives no VPS SSH credential. These internal prereleases do not become the
latest native release. Source filters are an optimization, not a way to skip required CI.

## Protect players and rollback

The reviewed controller and proxy configuration are maintenance-installed; packages
cannot replace them. All three controllers share a host lock and durable transition
journals. Browser/static updates switch symlinks after staging; a failed public
health probe restores previous pointers/receipts. No DB or game-process action is
part of a frontend promotion. An interrupted transition is recovered before another
applied update. Old tabs keep loading their source-qualified assets.

Backend automation accepts only the exact live MatchContract and database schema,
and only the single-process mode. Contract/schema changes stop for coordinated
client/migration review. It never rolls back player data, shortens the drain or kills
active matches. If rollback itself fails, operator intervention remains visible.
There is a temporary matchmaking pause during a backend drain; this is not HA.

Initial domain/proxy setup is separate from a normal deployment. Caddy reloads can
close WebSockets, so the bootstrap verifies zero connections, matches and pending
results, backs up the real config, preserves API/auth/private routes, validates the
candidate and only then reloads. Later frontend updates do not reload it. Preserve
Cloudflare strict proxy trust and TLS; do not weaken SSL mode to hide a handshake error.

## Activation gates and operating commands

See [deploy/auto/README](../deploy/auto/README.md) for installation, attestation and
backend details. The installer deliberately leaves all timers disabled. Before enabling:

1. Merge normally into main, preserving the last verified deployed source as an
   ancestor. Use its full SHA as the private config's `minimum_source`.
2. Install a current GitHub CLI with the attestation flags listed in the runbook.
   Verify a real main package with an empty GH configuration and no personal token.
3. Adopt the domain configuration and immutable directories once, in a verified
   quiet window; preserve current releases and legacy assets.
4. Dry-verify, apply and observe the first real attested candidate for each family.
   A message saying “Waiting for the first candidate” is not acceptance.
5. Enable only the observed family. Keep native TUF signing and dual-slot rollout
   disabled until their separate owner/custody/security gates pass.

On the official host, after the reviewed installation and private config:

```bash
# Each command verifies a real artifact; omit --apply for the dry check.
GH_CONFIG_DIR=/var/lib/tbg-deploy/gh python3 /opt/tbg/deploy/pull.py
GH_CONFIG_DIR=/var/lib/tbg-deploy/gh python3 /opt/tbg/deploy/web_pull.py
GH_CONFIG_DIR=/var/lib/tbg-deploy/gh python3 /opt/tbg/deploy/static_pull.py
# Apply/observe one candidate per family, using --apply --recheck.
# Only afterward create its flag and enable its timer:
touch /etc/tbg/web-deploy.enabled
systemctl enable --now tbg-web-deploy.timer
```

Backend names are `auto-deploy.enabled` / `tbg-auto-deploy.timer`; static names are
`static-deploy.enabled` / `tbg-static-deploy.timer`. Stop future polling by disabling
its timer; let a running deployment finish. All timers check about every ten minutes
with jitter: three families normally use at most 36 public API reads/hour, leaving
room below GitHub's unauthenticated quota for new-candidate checks. Other traffic
sharing the same origin IP and bursts can still exhaust it; fail closed and inspect
the HTTP error rather than adding a personal token. The current catalog scans the
latest 30 repository releases; if a family falls outside it, an operator must review
pagination rather than infer that it has no newer version.

Inspect systemd journals plus `/var/lib/tbg-deploy/{checked,web-checked,static-checked}.json`,
current receipts and transition files. Never delete a journal to make a failure vanish.
Backup/release cleanup remains operator-reviewed. [SCALING](SCALING.md) describes
measurement and upgrade boundaries.

## Native packages and truthful availability

The existing native workflow packages Linux/Windows separately. Linux validation
cannot prove Windows execution or visual parity. Native updater signing custody,
TUF bootstrap and per-platform acceptance are separate from GitHub OIDC backend/web
provenance. Keep landing `web/landing/config/release.json` links disabled unless the
actual public client matches the current server contract and has been verified.
Never relabel an old protocol-30 AppImage as compatible with protocol 31.

Email registration/recovery is still an offline P67 foundation, and the public admin
is a preparation page. Do not advertise email delivery, functioning administrative
permissions, accepted competitive balance or mobile availability from this merge.

## TLS incident, 2026-11-01

The initial origin configured only API/app hosts. Direct SNI for root/www/admin
returned an internal TLS alert because their host entries/certificates were absent.
API/app already used publicly trusted Let's Encrypt certificates; no evidence of a
self-signed certificate caused this incident. Added static hosts and obtained real
Let's Encrypt certificates, preserving the API routes and Rust process. Root and
admin now return HTTPS 200; www redirects to root. No Cloudflare SSL downgrade or
DNS change was made. Private logs/config backups remain off Git.

Technical references: [Cloudflare 525](https://developers.cloudflare.com/support/troubleshooting/http-status-codes/cloudflare-5xx-errors/error-525/),
[Caddy WebSocket lifecycle](https://caddyserver.com/docs/caddyfile/directives/reverse_proxy),
[GitHub attestation verification](https://cli.github.com/manual/gh_attestation_verify).
