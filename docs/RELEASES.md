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
cannot replace them. All payload controllers and the native catalog share a host lock and durable transition
journals. Browser/static updates switch symlinks after staging; a failed public
health probe restores previous pointers/receipts. No DB or game-process action is
part of a frontend promotion. An interrupted transition is recovered before another
applied update. Old tabs keep loading their source-qualified assets.

Backend automation accepts only the exact live MatchContract and database schema,
and only the single-process mode. Contract/schema changes stop for coordinated
client/migration review. Before any pause or DB operation, it requires the systemd
main process kernel executable to match the adopted binary path; post-start and
rollback perform the same check. An unused legacy install path cannot masquerade
as a successful update. It never rolls back player data, shortens the drain or kills
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
3. Adopt the domain configuration, actual systemd executable path and immutable
   directories once, in a verified
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
with jitter: the three payload families plus native metadata normally use at most 48 public API reads/hour, leaving
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

`native-release.yml` builds the Linux tarball/AppImage and Windows x64 MSVC
ZIP/per-user installer from one SHA, dependency lock, art inventory and feature set.
Windows uses static CRT, SDK 10.0.26100.0 and Inno 6.7.1; the runner records actual
tool identities. Build jobs have read-only permissions. Tag dispatch publication
requires all CI lanes. A successful exact-source native dispatch can be promoted
by its run ID without rebuilding the tested package bytes. Never use PR merge
artifacts or overwrite an existing tag's assets. Native versions do not need a main
merge to publish an explicitly approved beta; server/web/static promotion still does.

Before promotion, download the CI packages, verify checksums and package identity,
and run the intended Linux/Windows installation and runtime checks. Linux evidence
cannot certify Windows or exact pixel parity. Record those outcomes in
[the native report](phases/NATIVE-cross-platform-release.md). An unsigned Windows
candidate must remain labeled as such; GitHub provenance is not Authenticode or
SmartScreen trust. A beta is not a completed P68 professional-release acceptance.
Native TUF/update signing and key custody remain separate and disabled.

The landing recommends the detected Windows/Linux desktop OS, keeps both choices
available and opens the platform's format/requirements/SHA-256 dialog. It fetches
`/downloads.json` from the same origin; only the verified host catalog can enable
buttons. The authored config and local fallback keep both native platforms pending.
No GitHub token, API polling per visitor, inline script or analytics is introduced.

The metadata-only `native_pull.py` controller verifies the native manifest's GitHub
OIDC provenance against the exact native workflow, tag ref, source SHA and hosted
builder. It requires source ancestry, monotonic SemVer, both OSes/all four formats,
original download URLs and GitHub asset size/SHA-256 matching the signed manifest.
It compares the package contract with the running server and atomically publishes
small public JSON. A changed server contract or revoked cached asset hides links.
It never installs a native package, updates TUF, changes the DB or restarts the game.
Never relabel an old protocol-30 AppImage as compatible with protocol 31.

Install the reviewed controller with all timers disabled, adopt the landing's
`/downloads.json` route in a verified quiet window, and observe a real complete
attested release before enabling this separate timer:

```bash
GH_CONFIG_DIR=/var/lib/tbg-deploy/gh python3 /opt/tbg/deploy/native_pull.py
# Only after observing verified public metadata and all download assets:
touch /etc/tbg/native-catalog.enabled
systemctl enable --now tbg-native-catalog.timer
```

The public JSON is no-cache/no-transform. The ten-minute timer updates links after
subsequent compatible native releases, independent of landing deployment. All other
payload timers keep their own main/first-candidate gates. Metadata errors are
visible in `journalctl -u tbg-native-catalog.service`; inspect private
`native-current.json`/`native-checked.json` receipts without exposing host secrets.

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
