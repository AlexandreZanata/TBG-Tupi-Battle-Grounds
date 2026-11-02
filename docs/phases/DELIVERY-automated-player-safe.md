# Automated player-safe delivery — work report, 2026-11-02

Branch `codex/automated-player-safe-delivery`, based on main `70827a7`, with PR #42
(automation plan, LF-normalized native inputs) and PR #46 (backend packaging
interpreter, pinned `rust-toolchain.toml`) merged in normally. Inventory and gap
diagnosis: [DEPLOY-automation-inventory-20261102](DEPLOY-automation-inventory-20261102.md).
Decision: D-104. Level: work front plus scoped production actions authorized by the
owner's request.

## State per family (2026-11-02 12:10 UTC)

Implemented, published, verified, applied and timer are separate states. Every
"not yet" below waits for one thing: this PR (and #45 for the landing) reaching main,
because the host controllers verify only artifacts attested from main.

| Family | Implemented | Published (main) | Verified on host | Applied by controller | Timer |
|--------|-------------|------------------|------------------|-----------------------|-------|
| Browser game | yes | `web-70827a7…` | yes, tokenless | yes, 00:46:42Z | **active** since 00:47:50Z |
| Static landing/admin | yes | `static-70827a7…` | yes, tokenless | admin yes; landing restored to a hand publish (see below) | off: needs `--adopt` after #45 |
| Native catalog | yes (metadata) | `v0.1.2-rc.1` (branch commit, before this pipeline) | by the publication front, 00:12 | by the publication front | off: enable after this PR's pagination fix is installed |
| Backend | yes: A/B slot coordinator, rehearsed locally and its host adapter exercised on the host (below) | none yet: the first `server-<sha>` is built by the merge of this PR | host adapter and preflight verified read-only and a sandboxed idle-slot start/retire on the real host | slot mode bootstrapped by hand at 10:31:34Z; no candidate promoted yet | off until the first promotion is observed |

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

## Coordinator exercised on the real host (12:05Z, no player impact)

Read-only and idle-slot checks, run from a transient unit with the controller's own
sandbox (`ProtectSystem=strict`, the same `ReadWritePaths`, `NoNewPrivileges`):

- `release-slot.sh start b …` with the current binary: slot `b` ready in 2 s, admitting
  nothing (public `/api/admission` kept answering slot `a`, `/health` 200).
- `slot_host.SystemHost`: directory, schema, `/api/release`, `/metrics`, store check,
  public admission and health all answer; `preflight_host`, `gate` and `preflight` pass.
- A WebSocket upgrade through Cloudflare and Caddy to `/ws/slot-b` and `/ws/slot-a` on both
  `api.tupibg.com` and `app.tupibg.com` returns 101, so the route the coordinator checks
  after a promotion works end to end.
- `release-slot.sh retire b`: retirement gate passed, unit stopped and disabled, row
  `retired`. Slot `a` stayed active; `/health` 200 and admission unchanged.
- `systemctl enable`/`disable` of a slot unit work under the sandbox (the manager makes the
  symlinks); `/etc/tbg` and `/opt/tbg/slots` are the extra writable paths the unit needed.

These are mechanics, not a release: no candidate exists yet, nothing was promoted and
nothing was installed into `/opt/tbg/deploy`.

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
- **Backend A/B promotion** (D-104). `slots.py` is the coordinator and `slot_host.py` the
  machine adapter; `status.py` shows every family on one screen; the installer places
  them with the fixed `release-slot.sh` and `slot-bootstrap.sh`. See the next section.
- **Native.**
  - Version gate in pull requests.
  - `release-main` tags a new version once from a main commit, using the job token.
  - Main push builds are promoted byte for byte with suite evidence reuse.
  - Draft-first, reconciling publication.
  - `0.1.2-rc.2` is the first version for this pipeline.

## Backend A/B promotion

### What it does

Before anything is downloaded: one fresh active slot, a free idle slot, memory and disk for
two generations, an empty lobby (up to 90 s). Then: manifest against the live contract and
schema, the binary describes itself as `nobody`, install into the idle slot, ready/identity/
store check, a last lobby look, `promote --expect`, ranked authority, admission + health + a
WebSocket upgrade through the public proxy (3 in a row), receipt, and only then the previous
slot drains and retires, provided the new one is still seen admitting. Failure to serve rolls
admissions back to the old slot; the old slot is never stopped while it holds a match, a
pending result or ranked authority. Every step is journaled and every resume reads the
directory before the journal. Phases, outcomes and commands: `deploy/auto/README.md`.

Supports without any wire or schema change: `lobby_waiting` in `/metrics`; a draining slot
refuses its stranded waiters with `ServerFull`; an admitting slot without ranked authority
asks every second; a slot that has just started answers admission from the database (the
rehearsal found it answering "unavailable" for one probe); a server whose task stops by itself
now exits non-zero, so the slot unit's `Restart=on-failure` revives it (it used to exit 0 and stay
down); a client asks admission again (3 times in 30 s) when a draining slot refuses new play.

### Local rehearsal (`scripts/rehearse-slot-release.py`)

Real processes, the real coordinator, a private database and synthetic accounts; nothing
touches production. The live generation (`8be248e`, no gauge, no waiter refusal) runs in
slot `a`; three casual and two ranked bot loops ask admission, connect to the answer and
play a bounded 20 s match, continuously, across: release 1 (`8be248e` → this branch), release
2 with a lone player queued for 30 s (the coordinator waited, then promoted), and a drill
where a candidate promotes but the public route never follows it (rolled back).

| Final run (coordinator after the second review) | Result |
|-------------------------------------------------|--------|
| Matches completed | 60 (40 on slot a, 20 on slot b), 24 of them ranked |
| Failed attempts (match, admission, registration) | 0 |
| Admission probes at 10 Hz through all three events | 2541, 0 failed; median 0.5 ms, max 80 ms |
| Health failures | 0 |
| Release 1 / release 2 / rollback drill | old slot retired after 46.6 s / 84.6 s (28 s of it waiting for the lobby) / rolled back after 8 s, the rejected slot retired 10 s later |

Earlier runs found, and the final run confirms the fixes of: the "unavailable" window of a
just-started slot, ranked bots running out of deterministic names after 10 matches
(`tbg-bots --seed`), and a lone waiter shifting every later bot pair (the harness now pauses
the other players while it waits).

What it does not prove: loopback only (no Cloudflare or Caddy), bots instead of browsers,
20 s matches, one binary standing in for the second release, older tabs, spectators and rematch
offers, and a real overlap at production load. It is evidence for the mechanism, not a
promise of zero interruption.

### Independent review and what changed

A read-only reviewer read the coordinator, the controller wiring, the scripts and the unit
sandbox. Its findings, all reproduced or confirmed in the code, are fixed:

- the old slot could retire while its successor had died: it now requires the successor to be
  seen admitting (30 s window) and otherwise rolls back to it;
- rollback was not idempotent (a crash after its commit, or an operator's manual rollback,
  ended in a hold, a rewritten receipt and blocked releases): resume now reads the directory
  and adopts it, and the receipt is written only for the live release;
- verification never exercised the promoted slot's own route: it now upgrades a WebSocket to
  `/ws/slot-x` through the public proxy;
- slot mode skipped the `--build-info` identity check: restored (as `nobody`, in a temporary
  directory);
- permanent refusals were retried forever with a 64 MB download each time: contract and
  schema changes are latched `rejected`, and the host gate runs before any download;
- a candidate that died while warming blocked every later release: teardown now drains and
  retires it, and the next release reclaims such a row;
- over-eager holds: only states that need a person hold the controller (`OperatorRequired`);
  a busy database is retried, a slow stop is a deferral, `cleanup_failed` can be cleared;
- a stale journal could stop another generation: the generation is compared first;
- the test double accepted what the real admin refuses: it now implements the rules of
  `release_admin.rs`, `retire_self` and heartbeat ageing, and every crash point (before and
  after each state-changing step) is replayed with a restart, checking one admitting slot, no
  slot stopped under load and a closed journal. Mutation checks confirm the tests fail when a
  guard is removed.

A second read-only review of the rework found, and these are fixed too: the installer did not
copy `slot_host.py`, which would have broken all four controllers at import after the documented
reinstall (a test now checks the installer against the imports of every controller); a rejected
slot that had exited cleanly was never retired and ended in a hold (it is retired when it lists no
work, and the server now exits non-zero when its task stops, so systemd restarts it); a manual
rollback landing inside the verification window was treated as a refused rollback; a busy database
during cleanup or rollback held the controller (it is now a retry, with 20 attempts for rollback);
a hanging `--build-info` escaped uncaught; a host without the directory table was treated as locked;
a drain that outlives a day now fails loudly instead of waiting silently. The crash replay now
covers six outcomes (verification fails, never ready, promotion refused, reply lost, successor dies
while the old slot drains) and the test host also flags stopping the last admitting slot.

Residual risks: the controller runs at `Nice=15` with idle I/O (the timing-critical steps are
light and retried); verification passes through Cloudflare at about 1.5 requests per second for
at most 60 s (a block would cause a safe false rollback); a held controller or a stuck drain only
shows up in `status.py` and the unit's failed state, nothing pages anyone; and the first promotion
leaves the `8be248e` generation (no lobby gauge or waiter refusal) for waiting players to strand
until the idle grace.

### Gaps from the 2026-11-02 analysis

| Gap | Status |
|-----|--------|
| Single process → slots needs one restart | done once at 0 connections (10:31Z) |
| Ranked authority moves up to ~5 s after the switch | requested every second and moved by the coordinator right after the promotion (same second in the rehearsal) |
| Idle clients on a draining slot get `ServerFull` | clients with this change ask admission again; older tabs show it until reloaded |
| Queued, room and challenge waiters are not told to move | counted and awaited, refused if one slips in (not on the generation running today, so the first promotion can strand such a player for up to the idle grace) |
| Open connections are not in the retire gate | 120 s grace on idle sockets, then they reconnect through admission |
| A just-started slot answers `unavailable` | fixed |
| `/api/live`, `/api/online` per process | open (cosmetic during an overlap) |
| Rematch offers and spectators do not follow a promotion | open; offers are awaited like waiters |
| Any Caddy reload closes WebSockets | releases never reload Caddy |

## Validation

Local, work-front level (no full suite, per AGENTS.md):

- Python: `test-slot-deploy.py` (51, including every crash point of six outcomes), `test-auto-deploy.py` (21),
  `test-web-deploy.py` (14), `test-static-deploy.py` (11), `test-native-catalog.py` (7),
  `test-native-release-policy.py` (12), `test-ci-required-checks.py` (11),
  `test-native-inputs.py`, `check-native-inputs.py`, `check-native-version.py --base origin/main`,
  `shellcheck` of the three scripts, `git diff --check`.
- Rust under the shared lock with two jobs: `cargo fmt --all -- --check`; `cargo clippy --locked
  -p tbg-server -- -D warnings` and `-p tbg-client` (the CI commands); `tbg-server` lib (61),
  `release_slots` (6, one ignored probe), `health` (2), `online` (32); `tbg-client` `p66_` (6).
- The local rehearsal above, and the host checks of the previous section.

Not run here: the full workspace suite, the WASM and Windows builds (CI), the browser, and
everything that only exists after merge.

## Deferred and owner actions

- **Merge this PR** (it contains #42 and #46). Then the automatic native `v0.1.2-rc.2`
  publication (`release-main`, tag dispatch, draft-first publication), the first web
  candidate and the first `server-<sha>` happen without a command; each is observed, not assumed.
- **Install the new controllers** (briefly stop the web timer; `deploy/auto/README.md`),
  verify the first backend candidate with a dry run, apply it once and watch the promotion,
  then enable the backend timer. Enable the native catalog timer after its dry run.
- **Static:** merge #45, let its static candidate publish, then `static_pull.py --apply --adopt`
  once and enable the timer.
- Owner decision: the nightly backup cron never ran because `cron` is not installed.
- Not done by design: native TUF signing, Windows signing, mobile, any change to rules.
