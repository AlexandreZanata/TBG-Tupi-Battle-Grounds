# Automated player-safe delivery — work report, 2026-11-02

Branch `codex/automated-player-safe-delivery`, based on main `70827a7`, with PR #42
(automation plan, LF-normalized native inputs) and PR #46 (backend packaging
interpreter, pinned `rust-toolchain.toml`) merged in normally. Inventory and gap
diagnosis: [DEPLOY-automation-inventory-20261102](DEPLOY-automation-inventory-20261102.md).
Level: work front plus scoped production actions authorized by the owner's request.

## State per family (2026-11-02 01:00 UTC)

Implemented, published, verified, applied and timer are separate states.

| Family | Implemented | Published (main) | Verified on host | Applied by controller | Timer |
|--------|-------------|------------------|------------------|-----------------------|-------|
| Browser game | yes | `web-70827a7…` | yes, tokenless | yes, 00:46:42Z | **active** since 00:47:50Z |
| Static landing/admin | yes | `static-70827a7…` | yes, tokenless | admin yes; landing restored to a hand publish (see below) | off: needs `--adopt` after #45 |
| Native catalog | yes (metadata) | `v0.1.2-rc.1` (branch commit, before this pipeline) | by the publication front, 00:12 | by the publication front | off: enable after this PR's pagination fix is installed |
| Backend | slot mode bootstrapped; routine A/B coordinator pending | none yet: first candidate after #46 merges | — | — | off; slot a runs the 8be248e binary since 10:31:34Z (see below) |

## Production actions and evidence

- **Dry verification, tokenless.** Static and web candidates for `70827a7` passed
  attestation, digests and contract checks with an empty `GH_CONFIG_DIR` and no
  token, 7 s each. Nothing was written.
- **Static apply, 00:45:31Z.** Run under the unit's own sandbox (`systemd-run` with
  `ProtectSystem=strict` and the same `ReadWritePaths`). Result: healthy.
  **Incident:** the landing had been hand-published from PR #45's branch at
  00:27Z, after the inventory and before this apply. The automatic promotion
  replaced it with main's landing. That reverted #45's scroll fix publicly for
  36 s; the release was atomically restored at 00:46:14Z and logged in
  `activations.log`. The admin site keeps `70827a7`, which has identical source.
  **Fix:** drift detection now refuses to overwrite any pointer changed outside
  the controller without `--adopt`.
- **Web apply, 00:46:42Z.**
  - Pointer moved to `releases/70827a7…`.
  - New JS and 27.9 MB WASM return 200 with immutable caching; the entry is
    `no-cache`.
  - The four probed assets of the previous release `71eb818` still return 200 for
    open tabs.
  - Backend PID 5283 is unchanged and public API health is 200.
- **Web timer, 00:47:50Z.** The flag was created and the timer enabled. Its first
  real sandboxed run reported "already handled" and exited with success.
- **Operator mistake, no impact.** `--build-info` was run against the deployed
  binary, which predates that flag. It tried to start a second server and exited
  on the bind error. No binary is executed on the host anymore.

## Slot bootstrap, 2026-11-02 10:31Z (owner-approved quiet window)

Preflight required 0 connections, matches, queue, pending results and ranked
inflight, twice (before and right after the pre-deploy backup
`predeploy-20261102T103132.sqlite3`, integrity `ok`). The web timer was stopped
and the shared lock held for the duration, then restored.

- 10:31:33.6 single process (pid 5283) stopped; 10:31:34.3 slot `a` ready
  (generation `bootstrap-8be248e`, same binary sha256 `e7daa9c8…`); promoted and
  ranked authority taken in the same second; admitting at 10:31:35.3.
- Local probe of Caddy `/health` every 250 ms across the switch: 22 samples,
  0 non-200 (the stop-to-ready gap was 0.23 s). Public `/health` 200 on api and
  app hosts; `/api/admission` returns `/ws/slot-a`; a real browser guest queued
  and cancelled on slot a, leaving 0 connections.
- Not an interruption-free claim: a connection opened in that 0.23 s would have
  failed. The unit is enabled for reboot; the single unit is disabled.
- Run through a detached transient unit; `deploy/slot-bootstrap.sh` is the
  generalized, reviewed form of that one-time script (not itself run).
- **Finding:** `cron` is not installed on the host, so the nightly
  `/etc/cron.d/tbg-backup` never ran (only the 2026-11-01 18:40 copy exists).
  Not changed here; needs an owner decision (systemd timer or install cron).

## Implementation in this branch

- **Controllers.**
  - Shared non-blocking lock with deferral (exit 0, nothing latched).
  - A durable operator hold after a failed rollback, so the timer never loops
    stop/drain/start.
  - The timer can never restart the single game process (`--maintenance-window`).
  - Rejected candidates are latched; network errors remain retryable.
  - Conditional, quota-logged API reads over the newest 100 releases.
  - The native tag is revalidated by name; a deleted tag fails closed.
  - Browser/static drift detection with `--adopt`.
  - The landing contract gate, plus waiting (not latching) for a coordinated server
    contract.
- **CI.**
  - Family relevance is measured from the last published candidate.
  - Filter gaps are closed: toolchain, embedded patch notes, site-manifest sources.
- **Native.**
  - Version gate in pull requests.
  - `release-main` tags a new version once from a main commit, using the job token.
  - Main push builds are promoted byte for byte with suite evidence reuse.
  - Draft-first, reconciling publication.
  - `0.1.2-rc.2` is the first version for this pipeline.

## Zero-interruption backend: findings and plan

Read-only analysis of the P66/P71 code, all claims with file references in the
work log:

1. **Bootstrap needed one restart (done above).**
   - `tbg-server@.service` declares `Conflicts=tbg-server.service`.
   - The running single process never reads the slot directory, never records
     account ownership and keeps answering admission.
   - The proxy moves to slot b only when 8080 stops answering.

   The switch to slots therefore requires the current process to exit once.
   Scheduled when it holds zero connections, matches and pending results (it held
   0 connections at 00:44Z), no player is interrupted. Requests arriving in the few
   seconds before slot a is ready can still fail. **This is the one owner decision
   needed.**
2. **Gaps before routine A→B promotions are truly uninterrupted:**
   - ranked authority moves up to ~5 s after the admission switch;
   - idle clients on a draining slot get `ServerFull` instead of re-admission
     (client change);
   - queued, room and rematch waiters on A are not told to move;
   - open connections are not part of the retire gate;
   - a just-started slot answers `unavailable` until its first directory read;
   - `/api/live` and `/api/online` are per process;
   - any Caddy reload closes WebSockets (no `stream_close_delay`).

   These are server, client and coordinator changes. They will be rehearsed with
   two isolated generations and synthetic accounts before any canary. Until then
   the backend has no automatic activation.

## Validation

Local, no compilation: `test-auto-deploy.py` (20), `test-web-deploy.py` (14),
`test-static-deploy.py` (11), `test-native-catalog.py` (7),
`test-native-release-policy.py` (12), `test-ci-required-checks.py` (11),
`test-native-inputs.py`, `check-native-inputs.py`, `ci-test-lanes.py`,
`check-native-version.py --base origin/main`, `cargo metadata --locked --offline`
and `git diff --check`.

Deferred: everything that only runs on GitHub after merge, namely `release-main`,
tag dispatch, draft-first publication, the first main backend candidate and the
second controlled merge. Also deferred: the A/B coordinator and its rehearsal,
the backend first apply and timer, the static adoption and timer, and the native
timer.
