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
are pinned. A newer main commit supersedes an older pending publication; each
family measures relevance from its **last published candidate** through the compare
API (`scripts/release_scope.py`), so a change skipped by a superseded run is still
published by the next one.

Three source filters avoid redeploying unrelated products:

- `backend-release.yml`: Rust server/core dependencies and runtime inputs, packaged
  as `server-<full SHA>`. Activation is an A/B promotion through the two release
  slots (`deploy/auto/slots.py`); the timer never stops a running game process. A
  host still running the single process needs the one-time `deploy/slot-bootstrap.sh`
  (or an attended `--maintenance-window` restart) before automation can act.
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
cannot replace them. All payload controllers and the native catalog share one
non-blocking host lock and durable transition journals. A busy lock, a draining or
unreachable server, or a candidate waiting for a coordinated server contract is a
**deferral**: exit 0, nothing latched, the next run retries. Forged or malformed
candidates are latched as `rejected`; failures needing a person stay visible.
A browser/static pointer changed outside the controller (for example a hand-published
landing) is never overwritten: the run is refused until an operator reviews it and
reruns with `--adopt`. A landing whose `config/release.json` names another server
contract waits, so downloads are never advertised for a server that is not live. Browser/static updates switch symlinks after staging; a failed public
health probe restores previous pointers/receipts. No DB or game-process action is
part of a frontend promotion. An interrupted transition is recovered before another
applied update. Old tabs keep loading their source-qualified assets.

Backend automation accepts only the exact live MatchContract and database schema.
Contract/schema changes stop for coordinated client/migration review. On a slot host the
candidate must be what its attested manifest says (the binary describes itself as `nobody`),
is installed into the idle slot, checked, promoted when nobody waits in the lobby, confirmed
through the public proxy (admission, health and a WebSocket upgrade to its own `/ws/slot-x`
route) and the previous slot is retired only after the new one is seen admitting and the old
one's own gate passes (no running match, pending result or ranked authority). The
journal `slot-transition.json` makes every phase resumable and every resume reads the
directory first; a candidate that does not serve is rolled back by moving admissions to the
previous slot, which is still running.
Details, the phases and the operator commands are in
[deploy/auto/README](../deploy/auto/README.md); what a player can notice is bounded there
and measured in the [delivery report](phases/DELIVERY-automated-player-safe.md).
On a single-process host the timer refuses; the drain-and-restart path needs
`--maintenance-window` and is not zero downtime (it pauses matchmaking; this is not HA).
It never rolls back player data, shortens a drain or kills active matches. If a rollback
or cleanup fails, a durable operator hold (`/var/lib/tbg-deploy/operator-hold.json`)
stops every further backend action instead of repeating a loop each timer run; clear it
with `--release-hold` after review.

Initial domain/proxy setup is separate from a normal deployment. Caddy reloads can
close WebSockets, so the bootstrap verifies zero connections, matches and pending
results, backs up the real config, preserves API/auth/private routes, validates the
candidate and only then reloads. Later frontend updates do not reload it. Preserve
Cloudflare strict proxy trust and TLS; do not weaken SSL mode to hide a handshake error.

## Secrets and custody

No secret is needed for this delivery and none is stored in the repository, GitHub or the
host's controller files. GitHub builds use the job's ephemeral token with OIDC build
attestations; only the `release-main` job holds `contents: write` and `actions: write`, to tag
a version and start its publication. The host reads public release assets over HTTPS and
verifies attestations with an empty `GH_CONFIG_DIR` and no token. Its one private file,
`/etc/tbg/auto-deploy.json` (mode 0600), holds public URLs and the deployment floor SHA. The
owner's key-only SSH access stays the maintenance channel; its alias lives in the owner's
private configuration. Native updater (TUF) keys and any Windows Authenticode certificate are
not on GitHub or the host (Windows packages stay unsigned beta); if they are ever created they
stay offline with the owner. Nobody pastes a token or key into chat, a file under version
control, a log or a screenshot; a step that would need one is an owner action.

## Android extension point (documented only)

Mobile is a later maturity stage (owner rule, AGENTS.md), so nothing Android is built, published
or advertised. A future Android family plugs into the same shape without changing the other
families: (1) a separate attested workflow publishing an immutable `android-<full SHA>` release
whose manifest binds the source commit, the exact MatchContract and each package's digest (the
`tbg-native-release.json` pattern); (2) a catalog entry beside `linux` and `windows` in the
public `/downloads.json`, `unavailable` until a complete verified release exists, produced by a
host controller that keeps the same gates (attestation from main, contract equal to the live
server, atomic replacement, never a partial set); (3) the landing offering only formats whose
catalog entry is `available`, chosen by platform with a manual list as the fallback. Store
distribution and signing stay owner decisions with their own custody; none is implied here.

## Activation gates and operating commands

The future end-to-end activation and native publication plan is
[MERGE-TO-DEPLOY](plans/MERGE-TO-DEPLOY.md). It is a plan, not an activation receipt.

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
5. Enable only the observed family. Keep native TUF signing disabled until its
   separate owner/custody/security gate passes. The backend timer is enabled only
   after a promotion through the slots was observed end to end.

On the official host, after the reviewed installation and private config:

```bash
# Each command verifies a real artifact; omit --apply for the dry check.
GH_CONFIG_DIR=/var/lib/tbg-deploy/gh python3 /opt/tbg/deploy/pull.py
GH_CONFIG_DIR=/var/lib/tbg-deploy/gh python3 /opt/tbg/deploy/web_pull.py
GH_CONFIG_DIR=/var/lib/tbg-deploy/gh python3 /opt/tbg/deploy/static_pull.py
# Apply/observe one candidate per family, using --apply --recheck; a first
# adoption of a hand-published entry also needs --adopt.
# Only afterward create its flag and enable its timer:
touch /etc/tbg/web-deploy.enabled
systemctl enable --now tbg-web-deploy.timer
# Status, latches and quota:
systemctl list-timers --all 'tbg-*'
journalctl -u tbg-web-deploy.service --since -1h   # github-api lines log remaining quota
cat /var/lib/tbg-deploy/web-checked.json /var/lib/tbg-deploy/web-current.json
# Retry a latched candidate after fixing its cause, or stop a family:
GH_CONFIG_DIR=/var/lib/tbg-deploy/gh python3 /opt/tbg/deploy/web_pull.py --apply --recheck
systemctl disable --now tbg-web-deploy.timer && rm /etc/tbg/web-deploy.enabled
# Release a reviewed backend operator hold:
GH_CONFIG_DIR=/var/lib/tbg-deploy/gh python3 /opt/tbg/deploy/pull.py --release-hold
```

Rollback of a browser or static family is a pointer switch to a retained release,
never a database restore: an interrupted promotion is restored from its journal on
the next `--apply`; a reviewed manual rollback switches `current` atomically to the
previous `releases/<id>` and records it in `activations.log` (the receipt then
differs and the next candidate needs `--adopt`).

Backend names are `auto-deploy.enabled` / `tbg-auto-deploy.timer`; static names are
`static-deploy.enabled` / `tbg-static-deploy.timer`. Stop future polling by disabling
its timer; let a running deployment finish. All timers check about every ten minutes
with jitter. API reads are conditional (ETag): an unchanged answer is a 304, which
does not spend GitHub's anonymous quota, and every call logs the remaining quota.
Discovery reads the newest 100 releases; the active native tag is revalidated by
name, and a deleted one turns both downloads pending. Exhausted quota or network
errors fail visibly and promote nothing; do not add a personal token.

Inspect systemd journals plus `/var/lib/tbg-deploy/{checked,web-checked,static-checked}.json`,
current receipts and transition files. Never delete a journal to make a failure vanish.
Backup/release cleanup remains operator-reviewed. [SCALING](SCALING.md) describes
measurement and upgrade boundaries.

## Native packages and truthful availability

`native-release.yml` builds the Linux tarball/AppImage and Windows x64 MSVC
ZIP/per-user installer from one SHA, dependency lock, art inventory and feature set.
Windows uses static CRT, SDK 10.0.26100.0 and Inno 6.7.1. The workflow provisions
the official Inno distribution from `deploy/native/windows/inno-toolchain.json`,
verifies its SHA-256 before running it in a private portable tools directory,
and checks the real `PREPROCVER` during fixture and production compilation.
`ISCC.exe` PE version resources may be `0.0.0.0` and are not version authority.
After DevShell, the reviewed rustup proxy and exact Rust version are restored;
the runner records the actual compiler and Inno provenance/hashes. Build jobs have
read-only permissions.

**Version policy.** A pull request that changes shipped desktop inputs
(`scripts/native_release_policy.py`) must bump `crates/tbg-client/Cargo.toml` and
title `deploy/native/RELEASE-NOTES.md` with that version; CI fails early otherwise.
Workflow/test-only, docs-only and static-only merges publish nothing native. After
the merge, both main platform builds run once; `release-main` tags the new version
from a commit still on main and dispatches the tag-ref publication with the job
token. That run promotes the main run's exact artifacts, reuses the successful
`ci.yml` main push run of the same commit as suite evidence, and publishes
draft-first. A version that already names another commit with changed shipped
inputs fails as contention; retries upload only missing files and never replace an
asset. An exact-source manual dispatch remains available for an explicitly
approved beta. Never use PR merge artifacts or overwrite an existing tag's assets.

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
