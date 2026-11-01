# Main-based backend delivery (prepared, not enabled)

The owner requests automatic releases from GitHub main. Keep the existing single
Debian VPS, Rust process, systemd and Caddy. GitHub Actions builds; the VPS pulls
attested public artifacts over HTTPS. GitHub receives no SSH access, VPS password,
origin address, personal key or long-lived production token. The owner's existing
key-only SSH access remains the maintenance/recovery channel.

## Exact scope

This first increment automates **compatible backend updates**. It uses the
existing single-process drain fallback, not P66 two-slot canaries. During a drain
new matches pause; existing matches and result/reconnect windows finish before
replacement. It is not zero-downtime admission or live simulation migration.

A changed wire/gameplay contract, database version or an already enabled two-slot
runtime blocks automatic activation. The controller records operator_required
and does not keep retrying the same candidate. Coordinate client distribution,
review migrations or use the reviewed P66 rollout before rechecking. The process
never restores a database backup during rollback, truncates a match timeout or
kills a busy previous generation to finish a deploy.

Web/game bundles, static landing/admin and Linux client distribution are separate
artifacts. Frontend-only merges do not redeploy this backend. The existing native
release workflow builds archives/AppImages; the TUF updater's O02 signing custody
and public bootstrap gates remain open. This backend's GitHub OIDC provenance is
not authorization to activate native signing or administrative player permissions.

## Build and trust chain

1. Protected main merge runs the existing `ci` workflow. `check` still requires
   all six lanes and main protection remains strict with administrator enforcement.
2. `backend-release.yml` accepts only successful **push** CI runs from this repo's
   main and the matching default-branch SHA. PRs/forks cannot enter its build.
   Frontend/docs-only commits are skipped by an explicit source-scope guard.
3. Ubuntu 22.04 produces a release backend with a glibc 2.35 baseline, compatible
   with Debian 13. The pinned compiler matches CI. `--build-info` exits before
   loading runtime config/binding and identifies source, target, contract/schema.
4. Package only two executables plus their hash-bound manifest. No configuration,
   database, signing private key, SSH material or source tree is in the archive.
5. Pinned official GitHub provenance action attests the package with OIDC. A
   separate read-artifact/write-release job publishes `server-<full SHA>` once.
   It refuses replacement and skips a build already superseded before publishing.
   Backend candidates are prereleases and do not become the native latest release.
6. The host checks the public repository's numeric ID, release names, HTTPS URLs,
   a verified source ancestry floor and GitHub provenance bound to this exact repo,
   workflow, main ref, source digest and GitHub-hosted builder. Hashes alone do
   not authenticate a release. Archive extraction allows only three regular files,
   refusing traversal, duplicate members, links and size/hash mismatches.

Repository is public, so the timer uses no GitHub credential. Five-minute polling
normally uses 24 public API reads/hour; source comparisons add requests only for
new candidates. A private repository or renamed/transferred repo requires a new
review of access/trust, not a silent token fallback.

## Activation and recovery

The reviewed controller is installed once by the owner's computer. Artifacts
cannot replace privileged controller scripts, systemd units or proxy settings.
The timer and CLI serialize through the host state lock. Do not run the legacy
manual deploy or a provisioning job while this controller is enabled.

For each eligible candidate: verify before extraction; compare live contract and
read-only schema; inspect build identity as `nobody`; preserve previous binaries;
persist a transition journal and fsync files/pointers; drain via systemd; run the
existing integrity-checked predeploy backup; switch executable pointers; start;
check local admission/store availability and public HTTPS health. Failure restores
the prior binary/receipt after another safe drain. New player data stays intact.
Interrupted transitions are recovered on the next applied invocation. An unhealthy
rollback leaves its journal for operator recovery instead of claiming success.

Monitoring: `journalctl -u tbg-auto-deploy.service`, `/var/lib/tbg-deploy/checked.json`,
`current.json` and `transition.json`. GitHub has the immutable source, CI run and
provenance. Operator status/metrics stay loopback-only; Caddy is untouched.

## Before activation

**Current prerequisite:** main was `027fd61` when preparation began and advanced
to `e2a18af` (P71 match continuity, still protocol 30) during review;
the deployed backend is source `8be248e` (protocol 31, ruleset 6, schema 10).
Merge the deployed integration and this preparation into main through its required
checks first, using a regular merge commit so the deployed SHA remains an ancestor.
Squashing/rebasing requires a separately reviewed deployment-floor reconciliation. Never enable a blind main pull that would downgrade the live game.
The `minimum_source` must be the full last verified runtime SHA, not an arbitrary
old commit, a branch name or a placeholder.

The local CLI is 2.45 and Debian's default gh package is 2.46; both predate needed
attestation support. Install a current GitHub CLI using its official Linux package
instructions: https://github.com/cli/cli/blob/trunk/docs/install_linux.md. Confirm
`gh attestation verify --help` includes `--bundle`, `--source-ref`, `--source-digest`,
`--signer-workflow` and `--deny-self-hosted-runners`. Bundle verification must work
with an empty GH_CONFIG_DIR and no GH_TOKEN; do not copy the owner's gh login.

From a reviewed checkout on the owner's computer, after main CI publishes its
first candidate:

```bash
# SSH connection details remain in the computer's private config.
tar -cf - deploy/auto | ssh -o BatchMode=yes tupibg-vps \
  'mkdir -p /root/tbg-auto-review; tar -xf - -C /root/tbg-auto-review'
ssh tupibg-vps 'bash /root/tbg-auto-review/deploy/auto/install.sh'
```

The installer does not enable a timer, create an activation flag or restart the
game. It refuses replacement while a controller/timer is running. On the host,
copy config.example.json to `/etc/tbg/auto-deploy.json` with mode 0600 and replace
its source placeholder with the verified deployed full SHA. No origin IP or SSH
credential is required in that file. Then:

```bash
# Verify a real main package and compatibility, without activating it.
sudo env GH_CONFIG_DIR=/var/lib/tbg-deploy/gh \
  python3 /opt/tbg/deploy/pull.py
# After reviewing that first verified candidate, apply and observe it once.
sudo env GH_CONFIG_DIR=/var/lib/tbg-deploy/gh \
  python3 /opt/tbg/deploy/pull.py --apply --recheck
# Only after the real first delivery passes, enable future compatible releases.
sudo touch /etc/tbg/auto-deploy.enabled
sudo systemctl enable --now tbg-auto-deploy.timer
```

`Waiting for the first...` is not verified delivery. Missing signature, failed CI,
source rollback, contract/schema mismatch or an unavailable verifier is not a pass.
Stop future polling with `systemctl disable --now tbg-auto-deploy.timer`; let an
already active deploy finish, and never delete its transition or ranked journals.
Retain release directories/backups until an operator reviews safe disk cleanup.

## Docker and Kubernetes decision

Docker can later standardize packaging if several runtime services or developer
machines need it. Current game code is one Rust executable with bundled SQLite;
landing/admin are static foundations. systemd already restarts the process, Caddy
owns TLS/proxying and CI builds the artifact. Containerization is optional for this
workflow and would not by itself increase player capacity or shorten network RTT.
Docker Compose is a reasonable next packaging option for one host if a real need
appears. Kubernetes is deferred until there are several hosts/services and a
measured scheduling/availability need: a cluster placed on one VPS keeps that VPS
as the same failure boundary and does not migrate in-memory matches.

References checked 2026-11-01:
- https://docs.github.com/en/actions/reference/workflows-and-actions/events-that-trigger-workflows#workflow_run
- https://cli.github.com/manual/gh_attestation_verify
- https://docs.docker.com/compose/how-tos/production/
- https://kubernetes.io/docs/setup/production-environment/
