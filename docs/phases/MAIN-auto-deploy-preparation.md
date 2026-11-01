# Main-based automatic delivery preparation — 2026-11-01

Owner request: prepare professional automatic game releases from GitHub main and
explain Docker/Kubernetes tradeoffs. Branch `phase/main-auto-deploy`, based on the
verified deployment `7f93e23`. Detailed operating procedure:
[deploy/auto/README.md](../../deploy/auto/README.md).

## Implemented preparation

- CI-success/main-push gated backend release workflow; exact source checkout,
  pinned actions/compiler, separate publication permissions, keyless GitHub OIDC
  provenance, immutable prerelease per source SHA. No VPS secrets/SSH in CI.
- Backend-only source filter: landing/admin/game-client-only edits do not restart
  the server. Existing protected main `check` and CI lanes remain intact.
- Offline server `--build-info`, with actual compiler target and the current
  database schema metadata. No game/wire/schema migration or auth lifetime change.
- Strict package builder and host pull/activation controller, disabled installer,
  opt-in systemd timer. Signature/source/contract/schema guards, unprivileged
  candidate inspection, backup, unlimited match drain, immutable binary pointers,
  fsynced recovery journal and binary/receipt rollback without database restoration.
- Linux updater signing, the web game and independent static frontend delivery
  remain separate releases. No native update trust root or signing gate is bypassed.

## Verification

Level: scoped work-front/consolidation checks, no full local suite or load test.
`python3 scripts/test-auto-deploy.py`: 10 passed (archive/provenance/compatibility
policy, package round trip, source scope, rollback and interruption recovery with
preserved player rows). `test-ci-required-checks.py`: 9 passed;
`test-ci-test-lanes.py`: 13 passed. Source formatting, shell syntax and mandatory
patch-note/source identity check pass. The initial policy test exposed missing
workflow pin comments/cancellation expression; both now match the existing policy.
One local indentation error introduced during journal refactoring was corrected.

Completed under the shared host lock, max two jobs (after waiting for the other front):
`cargo test --offline --locked -p tbg-server --test build_info --test schema_upgrade
-- --test-threads=1`: **5 passed** (offline identity 1, schema upgrades 4). No schema migration changed. The rollback fixture also runs activation with the service's 0077 umask and verifies release directories remain executable by the unprivileged service.

## Not activated / required acceptance

No production service, SSH policy, DNS, database or release pointer was changed.
No timer or persistent runner was installed/enabled on the VPS. Main remains
protocol 30 while the running integration is 31; a reviewed merge is prerequisite.
The current local/stock Debian gh CLI cannot verify these attestations. The first
real CI-produced package, current official verifier, verification with no token,
reviewed host installation and an observed first activation remain required.
Unit tests mock signature-command execution and service actions; they do not prove
an actual Sigstore verification, power-loss event, production drain or rollback.
Two-slot migration/canary, native TUF O02/public bootstrap, Windows and capacity
acceptance remain deferred. Existing P71 look/balance reservations remain intact.
