# Handoff — automatic player-safe delivery for TBG (2026-11-02, about 12:50 UTC)

For the next agent (any model). The owner writes Portuguese: **talk to them in Portuguese; keep code,
comments, repo docs and commits in English** (AGENTS.md rule 7). Read `AGENTS.md` first, then this file,
then `docs/phases/DELIVERY-automated-player-safe.md` (the evidence report) and `deploy/auto/README.md`
(the runbook). Everything below was verified from the repository, GitHub or the host unless marked
"not observed".

## 1. The task

Make delivery automatic from `main` and prove it for real, without interrupting players:

- server changes → a compatible backend promoted on the official VPS;
- web client → `app.tupibg.com`;
- static → landing/www and the permitted admin;
- eligible native changes → the same Linux and Windows version on GitHub, with the landing offering the
  compatible formats automatically; Android only as a documented extension point;
- routine compatible transitions must not interrupt matches, reconnections, results, sessions or admission
  (Ranked included); never claim "zero downtime" for anything that restarts the only process.

The owner's last instruction (paraphrased from Portuguese): wait for the CI result, then *really*
implement the three families that were still "not automatic" — static (landing/admin), native catalog and
backend — so the process is truly automated. Everything that remains needs the owner's merge (below).

### Owner rules that bind you (verbatim essentials, Portuguese)

- "Este pedido autoriza implementação, commits, push normal da branch, abertura ou atualização de PR para
  main e aplicação na VPS oficial do fluxo revisado e validado. **Não faça merge por mim, force-push, rebase
  de branch compartilhada, reescrita de histórico ou substituição de releases publicadas.**"
- Only the official VPS, via the existing private SSH alias `tupibg-vps` (72.60.147.2). **Do not touch** the
  old `tbg.abastevo.com.br` VPS or `/home/iiii/I9-PROJETOS/VPS-HOSTINGER-I9-32GB`; no other projects,
  containers, ports or services.
- Never print tokens, keys, emails, accounts, private addresses or databases. No secret is needed; do not ask
  for a PAT or an SSH key for GitHub Actions; never ask the owner to paste a secret in chat/files/logs.
- Do not build Rust on the VPS that serves players. No routine Caddy reload or global restart. No destructive
  tests on the VPS; use synthetic accounts for anything that changes ratings; do not pollute the real ladder.
- "Não fabrique recibo para destravar timer. Ative cada família somente após verificar e aplicar seu primeiro
  artefato main real com atestação e observar sucesso." Then prove a **second controlled merge** going through
  the automation with no manual deploy command (ask the owner to make that concrete merge).
- Keep native packages unsigned beta; never move private signing keys to GitHub/the VPS. Package publication
  is not automatic installation on players' PCs.
- Never change competitive rules or numbers. Desktop only (mobile is later). Final state must keep
  **implemented / published / verified / applied / timer-active** as separate facts.
- Validation policy (AGENTS.md + `docs/VALIDATION.md`): minimum checks per front, **every Rust compile under the
  shared lock with at most 2 jobs**, never poll or bypass a busy lock, no full suite unless the owner asks:
  `flock -n -E 75 /tmp/tbg-validation-$(id -u).lock env CARGO_BUILD_JOBS=2 CARGO_TARGET_DIR=<own dir> cargo …`.
  Exit 75 means busy: do other work and retry later.

## 2. Where everything is

| Thing | Value |
|-------|-------|
| Repo | `AlexandreZanata/TBG-Tupi-Battle-Grounds`; main checkout `/home/iiii/PESSOAL-PROJETOS-ALEXANDRE/TBG-Tupi-Battle-Grounds` (other fronts' uncommitted work lives there: do not touch) |
| **The PR** | **#49** `codex/automated-player-safe-delivery` (ready for review, not draft). It contains #42 and #46 by normal merges and supersedes the closed #47 |
| Worktrees (mine) | `/data/tbg-automated-delivery` (the PR branch), `/data/tbg-ab-delivery` (stacked local branch `codex/zero-interruption-backend-delivery`, same commits; remove with `git worktree remove` when done), `/data/tbg-ab-old` (detached at `8be248e`, used to build the live generation for rehearsals) |
| Cargo targets (mine) | `/data/tbg-ab-target` (server, debug+release), `/data/tbg-ab-target-old` (`8be248e`), `/data/tbg-ab-target-client` (native client tests/clippy) |
| Native client builds need | `PKG_CONFIG_PATH=/home/iiii/PESSOAL-PROJETOS-ALEXANDRE/TBG-Tupi-Battle-Grounds/.local/native-deps/root/usr/lib/x86_64-linux-gnu/pkgconfig` |
| Rehearsal outputs | `/data/tbg-ab-rehearsal/run1..run7` (run7 = final) |
| Root disk | `/` is full (~100%): write only small files there; build and log on `/data` |

Other PRs (not mine unless said): **#42** native downloads reproducible (publication front; included in #49),
**#46** backend packaging fix (included in #49), **#45** landing scroll fix (separate; the live landing is a
hand-publish of its branch), **#44** Draft consolidation of P73 + lab polish onto main (protocol 32, D-103; earlier
part of this session; the landing `release.json` still says w31 so its `quality` lane fails: owner decision, my
recommendation is a coordinated w32 release after automation is active; #41 and #43 close as superseded after
#44 merges), **#48** someone else's champion expansion (Draft).

Main is protected: PR + green `check`, merge commit only (squash/rebase would break the deployment-floor
ancestry: the deployed source `8be248e` must stay an ancestor of main).

Commits on the PR branch after `70827a7` (the last five are mine for the slot work): `1e56dd1` slot coordinator,
`222d2f5` server supports, `582cb6a` client re-admission, `07db0ff` rehearsal harness, `1cf7747` docs/D-104,
`7a524c2` second review round, `dcedd69` final numbers and this handoff (earlier: #42/#46 merges, controllers,
CI and native publication work, `8a00f22`, `14f6c2c` slot-a switch). CI on `1cf7747`: **all `ci` lanes and both
native builds passed**. CI on `dcedd69` was **not observed** (it started right before this handoff):
run `gh pr checks 49`.

## 3. What is live on the VPS right now (2026-11-02 ~12:50Z)

| Family | Implemented | Published | Verified | Applied | Timer |
|--------|-------------|-----------|----------|---------|-------|
| Web | yes | `web-70827a7…` | tokenless attestation on host | yes, 00:46Z, by the controller | **active** (flag `/etc/tbg/web-deploy.enabled`), runs every 10 min |
| Static | yes (hardened) | `static-70827a7…` | yes | admin yes; **landing is a hand publish of #45's branch** (`cb68ad4`) so the controller refuses to overwrite it without `--adopt` | off |
| Native catalog | yes | `v0.1.2-rc.1` (catalog on `/downloads.json`, Linux+Windows `available`) | by the publication front | by the publication front | off |
| Backend | yes (A/B slot coordinator) | none yet (first `server-<sha>` comes from the merge of #49) | host adapter and preflight verified read-only; idle-slot start/retire verified under the unit sandbox | slot mode switched **by hand** once (10:31Z) | off |

Host facts: Debian 13, Python 3.13, Caddy 2.11, gh 2.103. The game runs as slot `a` (unit
`tbg-server@a`, port 8080, generation `bootstrap-8be248e`, binary = the `8be248e` build, contract
`w31-r6-097385fee9a10b0f`, schema 10); slot `b` (port 8082) is retired/idle. Caddy routes `/ws/slot-a|b` and
the shared `lb_policy first` for `/health` and `/api/*`; releases never reload Caddy. Controllers installed in
`/opt/tbg/deploy` are still the **old** versions (before the hardening) until you install the PR's.
State: `/var/lib/tbg-deploy` (lock, receipts, journals, `operator-hold.json`), `/etc/tbg/*.enabled` flags,
private config `/etc/tbg/auto-deploy.json` (0600, do not print). `cron` is **not installed**, so the nightly
backup in `/etc/cron.d/tbg-backup` never runs (owner decision pending: systemd timer or install cron).
Leftovers I created and that are harmless: `/root/tbg-slot-bootstrap.sh`, `/root/tbg-ab-review/` (review copy
of the controllers, not installed), `/var/lib/tbg-deploy/slot-bootstrap-*.log`, `/opt/tbg/releases/rehearsal-8be248e/`,
`/etc/tbg/slot-{a,b}.env`, `/opt/tbg/slots/{a,b}`, backup `/var/backups/tbg/predeploy-20261102T103132.sqlite3`.

Incident to remember: the first static apply overwrote the landing that another front had hand-published
(36 s, restored atomically); that is why drift detection and `--adopt` exist.

## 4. What was built (all in PR #49)

- **Controllers (`deploy/auto/`)**: shared non-blocking lock with deferral (exit 0), operator hold after a
  failed rollback, rejected-candidate latch, conditional quota-logged GitHub reads (`github.py`), drift
  detection + `--adopt`, native tag revalidation by name, landing waits for its server contract. New:
  `slots.py` (coordinator), `slot_host.py` (machine adapter), `status.py` (one screen for all families).
- **Backend A/B promotion (D-104)**: before any download the host gate (one fresh active slot, free idle slot,
  memory/disk, empty lobby ≤ 90 s); then manifest vs live contract/schema (permanent mismatch → latched
  `rejected`), the binary describes itself as `nobody` (`--build-info`), install into the idle slot, ready/identity/
  store check, last lobby look, `promote --expect`, ranked authority, admission + health + a **WebSocket upgrade
  through the public proxy** ×3, receipt, then the old slot drains and retires only if the new one is seen
  admitting and its own retire-check passes (idle sockets get a 120 s grace). Failure to serve → rollback to the
  still-running old slot. Journal `/var/lib/tbg-deploy/slot-transition.json`; every resume reads the directory first.
  Exceptions: `Deferred` (wait), `Rejected` (latch), `OperatorRequired` (hold), `Busy` (retry next run).
- **Server (no wire/schema change)**: `lobby_waiting` in `/metrics`; a draining slot refuses stranded waiters with
  `ServerFull`; ranked authority requested every second; a just-started slot answers admission from the DB; the
  process exits non-zero when its server task stops (so `Restart=on-failure` revives a slot).
- **Client**: re-admission (3×/30 s) on `ServerFull` from a draining slot (`net/socket.rs`). Native version
  `0.1.2-rc.2` with release notes.
- **CI/native**: relevance measured from the last published candidate (`scripts/release_scope.py`), PR version gate
  (`check-native-version.py`), `release-main` job (tags a new native version once and dispatches the exact-byte
  publication, draft-first), promotion guard, toolchain pinned 1.99.0, Python 3.11 for backend packaging.
- **Scripts**: `deploy/release-slot.sh` (runuser instead of sudo, enable/disable with start/retire),
  `deploy/slot-bootstrap.sh` (one-time single→slot switch; already done by hand), `scripts/rehearse-slot-release.py`
  (local two-generation rehearsal with bots), `tbg-bots --surrender-s/--seed`.
- **Docs**: D-104, `docs/RELEASES.md` (families, secret custody, Android extension point), `docs/DEVELOPMENT.md`,
  `docs/SCALING.md`, `deploy/auto/README.md` (phases, operator commands), the delivery report.

## 5. Evidence you can rely on

- Local rehearsal (run7, final code): real processes + real coordinator + synthetic bots across 2 releases, a lone
  waiter and a rollback drill: 60 matches (24 ranked) completed, 0 failed, 2541 admission probes at 10 Hz with
  0 failures, 0 health failures. Loopback only, bots not browsers — mechanism evidence, not a promise.
- On the host (12:05Z): idle slot b started/retired through the scripts under the unit's sandbox; a WebSocket
  upgrade through Cloudflare+Caddy to `/ws/slot-a|b` returned 101; `runuser -u nobody` works under the sandbox;
  `systemctl enable/disable` of slot units works; coordinator preflight passes read-only.
- Two independent read-only reviews of the coordinator; all findings fixed (see the report). 51 slot tests including
  every crash point of six outcomes; mutation checks confirm tests fail when a guard is removed.
- Local checks run: Python suites (slot 51, auto 21, web 14, static 11, native catalog 7, native policy 12, CI guards
  11), shellcheck, fmt, clippy (server and client, CI commands), server lib 61, `release_slots` 6, `health` 2,
  `online` 32, client `p66_` 6.
- Not run: full workspace suite, browser, Windows beyond CI, anything that only exists after the merge.

## 6. What remains — in this order

1. **Owner merges #49 with a merge commit.** (I could not and must not merge.) Check main afterwards:
   `gh run list --branch main --limit 10`, `gh release list --limit 10`. Expected without any command: `ci` green;
   `backend-release` → `server-<sha>`; `web-release` → `web-<sha>`; native `release-main` → tag `v0.1.2-rc.2` →
   dispatch → draft-first publication (public when complete). **None of this has ever run on main** — watch the
   first `release-main` and the tag-ref publication run closely (token permissions, `ci_proven` output, draft→public).
2. **Install the new controllers** (the installer refuses while any timer runs):
   `tar -cf - deploy/auto deploy/release-slot.sh deploy/slot-bootstrap.sh | ssh -o BatchMode=yes tupibg-vps 'rm -rf /root/tbg-auto-review && mkdir -p /root/tbg-auto-review && tar -xf - -C /root/tbg-auto-review'`
   (from a checkout of merged main), then on the host: `systemctl disable --now tbg-web-deploy.timer`, wait for
   `tbg-web-deploy.service` to be inactive, `bash /root/tbg-auto-review/deploy/auto/install.sh`,
   `systemctl enable --now tbg-web-deploy.timer`. Check `/opt/tbg/deploy/*.py` includes `slot_host.py`, `slots.py`, `status.py`.
3. **Backend**: dry run (`pull.py`, no `--apply`) under the unit's sandbox via `systemd-run` (same flags as
   `deploy/auto/tbg-auto-deploy.service`: `ProtectSystem=strict`, its `ReadWritePaths`, `NoNewPrivileges`,
   `PrivateTmp`, `ProtectHome`, `Nice=15`, `IOSchedulingClass=idle`, `GH_CONFIG_DIR=/var/lib/tbg-deploy/gh`) → expect
   "verified … mode slots … activated false". Then apply once: `touch /etc/tbg/auto-deploy.enabled` and
   `systemctl start tbg-auto-deploy.service` (runs `pull.py --apply` under the real sandbox without enabling the
   timer), watch `python3 /opt/tbg/deploy/status.py`, `journalctl -u tbg-auto-deploy.service`, the journal, public
   `/health`, `/api/admission`. The old slot may defer for a while (idle grace, matches). Only after one promotion
   completed and was observed: `systemctl enable --now tbg-auto-deploy.timer`. If something stops, do **not**
   work around it: read `status.py`, the journal and `operator-hold.json`.
4. **Native catalog**: when `v0.1.2-rc.2` is public, dry run `native_pull.py`, then enable
   `/etc/tbg/native-catalog.enabled` and `tbg-native-catalog.timer`. It only refreshes `/downloads.json` atomically
   and only when the server contract matches.
5. **Static**: after #45 is merged (owner) and a `static-<sha>` candidate exists: `static_pull.py --apply --adopt`
   once (adopts the hand-published landing), then enable `/etc/tbg/static-deploy.enabled` and the timer.
6. **Web** is already automatic: confirm it applied the new web candidate (the client re-admission change) without a command.
7. **Second controlled merge**: ask the owner to merge one concrete small change that triggers a candidate
   (e.g. a web or static input) and capture that every family involved updated itself with no manual command.
8. **Final report to the owner (Portuguese)** with per-family state (implemented/published/verified/applied/timer),
   evidence, what stayed deferred; update `docs/phases/DELIVERY-automated-player-safe.md` with real timestamps and push to the PR branch.

Open owner decisions: nightly backup (`cron` missing on the host); #44 / w32 coordinated release timing; whether
to keep the old `tbg-server.service` unit installed (disabled) or remove it.

## 7. Known limits (do not oversell)

- The first promotion overlaps the `8be248e` generation: no lobby gauge or waiter refusal, no cross-slot
  revocation propagation, so a waiting player can strand until the 120 s idle grace and a session revoked on the new
  slot stays valid on the old one until it drains.
- Older tabs still show "server full" after a promotion until reloaded; rematch offers, spectators and per-process
  listings (`/api/live`, `/api/online`) do not follow a promotion.
- Verification goes through Cloudflare at ~1.5 req/s for ≤ 60 s; a block would cause a safe false rollback.
- Held controllers and stuck drains are visible only in `status.py` / failed unit state; nothing pages anyone.
- The brief window of the first single→slot switch was 0.23 s with no failed local health sample, not "zero downtime".

## 8. Gotchas learned the hard way

- `gh pr edit` fails on this repo (classic Projects GraphQL): use
  `gh api -X PATCH repos/AlexandreZanata/TBG-Tupi-Battle-Grounds/pulls/N -f title=… -F body=@file`.
- Never run `tbg-server --build-info` from a binary that predates the flag (the `8be248e` one starts a server).
- Never `pkill -f tbg-server` or by name: other fronts run servers and cargo here; stop your own processes by PID/exact path.
- Backend packaging needs Python ≥ 3.11 (`hashlib.file_digest`); the toolchain is pinned in `rust-toolchain.toml`.
- Adding an `Origin` header to the WebSocket probe is unnecessary; the server accepts a native-style upgrade.
- Caddy adds duplicate `Cache-Control` headers unless deferred (`header { … defer }`) — solved in PR #44's Caddy work.
- The shared validation lock may be held by another front for tens of minutes: do not poll it.
- `tbg-bots` derives account names from its seed: use `--seed` or ranked bots run out of names after 10 matches.

## 9. Useful commands

```bash
# status of everything on the host (after install; before it, use the copy in /root/tbg-ab-review)
ssh -o BatchMode=yes tupibg-vps 'python3 /opt/tbg/deploy/status.py'
ssh -o BatchMode=yes tupibg-vps 'sudo -u tbg /opt/tbg/bin/tbg-release-admin --db /var/lib/tbg/tbg.sqlite3 status'
# local checks (no compile)
cd /data/tbg-automated-delivery && for t in test-slot-deploy test-auto-deploy test-web-deploy test-static-deploy test-native-catalog test-native-release-policy test-ci-required-checks; do python3 scripts/$t.py 2>&1 | tail -3; done
# rehearsal (needs the release binaries built under the lock)
python3 scripts/rehearse-slot-release.py --old-bin /data/tbg-ab-target-old/release/tbg-server --new-bin /data/tbg-ab-target/release/tbg-server --admin-bin /data/tbg-ab-target/release/tbg-release-admin --bots-bin /data/tbg-ab-target/release/tbg-bots --out /data/tbg-ab-rehearsal/runN --rollback-drill --ranked 2
# server tests under the lock
flock -n -E 75 /tmp/tbg-validation-$(id -u).lock env CARGO_BUILD_JOBS=2 CARGO_TARGET_DIR=/data/tbg-ab-target cargo test -p tbg-server --lib -- --test-threads=1
```
