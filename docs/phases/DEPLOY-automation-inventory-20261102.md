# Main-to-deploy automation: inventory and gap diagnosis — 2026-11-02

Status: **read-only inventory, implementation not started** (except the
prerequisite PR #46). Nothing was deployed, applied, enabled or migrated on
the official host. The active native publication (`v0.1.2-rc.1`) was not
touched. Plan of record: `docs/plans/MERGE-TO-DEPLOY.md` (PR #42).

Snapshot taken 2026-11-02 00:00–00:40 UTC from GitHub (read-only API) and the
official VPS `tupibg-vps` / 72.60.147.2 (read-only commands). The repository
snapshot is PR #42's head `ebae364` (main `70827a7` + publication commits).

## Live state per family

"Implemented" means code exists; "published" means a public attested release
exists for current main; "verified/applied" means a host controller did it;
"timer" is the systemd timer plus its `/etc/tbg/*.enabled` flag.

| Family | Implemented | Published for main `70827a7` | Verified / applied by controller | Timer | What is live, and how it got there |
|--------|-------------|------------------------------|----------------------------------|-------|------------------------------------|
| Backend server | yes (single-process only) | **no**: `backend-release` run 41361428350 failed packaging | no | off | `/opt/tbg/bin/tbg-server` installed manually 2026-11-01 18:31; protocol 31, contract `w31-r6-097385fee9a10b0f`, schema 10, single process, slots disabled |
| Browser game | yes | yes: `web-70827a7…` | no | off | manual bootstrap of `71eb818…` (`/srv/tbg/web/current`) |
| Static landing/admin | yes | yes: `static-70827a7…` | no | off | manual `web/deploy/publish.sh` releases (landing `c8fa110…`, admin `e4bdb38…`) |
| Native catalog | yes (metadata only) | `v0.1.2-rc.1` published 00:11, tag on `ebae364` (PR #42 head, not on main) | refreshed once by the publication front at 00:12 | off | `/downloads.json`: Linux and Windows 0.1.2-rc.1 `available`, server `w31` |

Installed controllers in `/opt/tbg/deploy` are byte-identical to the repository's
`deploy/auto/*.py`. The host floor (`minimum_source`) is `8be248e`. Host tools:
Debian 13, Python 3.13.5, Caddy 2.11.2, gh 2.103.3. No controller receipt
(`current.json`) exists for any family.

## Gaps, most severe first

### Blocking

1. **No backend candidate has ever been published.** `scripts/package-backend.py:26`
   uses `hashlib.file_digest` (Python ≥ 3.11) on ubuntu-22.04's system Python 3.10
   (`backend-release.yml:45`). CI tests the packager only on ubuntu-24.04.
   Fixed by PR #46 (interpreter set-up and guard); its merge should publish the
   first `server-<sha>` candidate.
2. **Native main promotion and versioning do not exist.**
   `scripts/check-native-promotion.py:13-15` accepts only an exact manual
   `workflow_dispatch` run; it has no test, no main/ancestry/age check, and nothing
   creates tags. The version comes only from `crates/tbg-client/Cargo.toml:6`, and
   `deploy/native/RELEASE-NOTES.md` is fixed to 0.1.2-rc.1. `v0.1.2-rc.1` was
   promoted from a branch commit. There is no version-advance detector.
3. **No automated zero-downtime path.** The P66 A/B building blocks exist
   (`tbg-server@.service`, `release-slot.sh`, slot routes, ownership epochs,
   admission handoff), but `deploy/auto/policy.py:61` refuses any slot and
   `activate.py` only stops and restarts `tbg-server.service`. Automated backend
   delivery today is: unbounded drain (no new matches), then a full API/WS outage
   through backup and restart. It must not be described as zero downtime.

### High

4. **Failed rollback loops in production.** Every `--apply` starts with
   `recover()` (`pull.py:61-62`). When rollback health fails (it requires the public
   URL through Cloudflare, `activate.py:76-90`), the journal stays and each 10-minute
   timer run repeats stop, drain and start with no circuit breaker.
5. **One fail-fast lock for all four controllers.** `LOCK_EX|LOCK_NB` on the shared
   state lock (`pull.py:117-120`, `web_pull.py:67-68`, `static_pull.py:64-65`,
   `native_pull.py:99-100`). A backend drain holds it indefinitely, so web, static
   and native updates and their recovery (including hiding incompatible downloads)
   are frozen for the whole drain.
6. **Relevant changes can be silently skipped.** Change filters compare only
   `HEAD^..HEAD`, and an outdated run exits 0 when main advanced
   (`backend-release.yml:103-107`). A backend change followed quickly by an unrelated
   merge never produces a candidate.
7. **Stuck Ranked settlement means an infinite drain** (`ranked/finalizer.rs:143-146`,
   `main.rs:66-81`), with `TimeoutStopSec=infinity`, which also holds the lock (5).

### Medium

8. Release discovery reads only `per_page=30` (`pull.py:55`, `native_pull.py:54`);
   once the active native tag falls out of that window revalidation stops silently.
   Worst-case unauthenticated reads (~72/h) exceed the 60/h limit.
9. Filter coverage: web misses `docs/patches/catalog.json`; static misses the branding
   PNG listed in both site manifests; native push paths omit `LICENSE`,
   `THIRD_PARTY_NOTICES.md` and `rust-toolchain.toml`.
10. Publish retries exit 1 when the release exists in every family; nothing
    reconciles a partial publication.
11. Tokenless `gh attestation verify --bundle` on the host is documented as pending
    and unproven.
12. The release image toolchain followed `stable` (`rust-toolchain.toml`), not the
    pinned 1.99.0; fixed by PR #46 with a guard.

### Low

Build jobs of backend/web/static hold `id-token`/`attestations: write` (native
builds do not); CI identity is matched by workflow name only; `static_pull.py:27`
points at `web-checked.json`; verify/download failures are not latched, so large
archives are re-downloaded every run; a leftover `.staging` directory blocks a SHA;
a briefly unreachable backend latches a web candidate as operator-required; the
native controller has no dry-run or journal.

## Preserved strengths

Exact attestation flags (`--signer-workflow`, `--source-ref refs/heads/main`,
`--source-digest`, `--deny-self-hosted-runners`), asset-URL prefix pinning, ELF and
manifest digest checks, ancestry floor, contract and schema refusal, atomic
`/downloads.json` replacement, immutable browser releases with old assets kept,
drains that never kill matches, pinned actions and fail-closed `check` aggregation.

## Merge-order and contract risks

- `v0.1.2-rc.1` (protocol 31) is tagged on PR #42's head. Merge #42 before anything
  else so main contains the published source.
- PR #44 moves main to protocol 32 / `w32-r6-097385fee9a10b0f`. Once that reaches
  the server, the 0.1.2-rc.1 downloads become incompatible, and every controller
  correctly refuses the cross-contract step. Activate the automation on a w31 main
  first, then ship #44 as one coordinated server + browser + native release.
- After #42 merges, main still says 0.1.2-rc.1, which must mean "already published";
  any native automation needs that rule before it runs on main.

## Staged plan (each stage observed before the next)

1. **Prerequisites** — PR #46. *Owner: merge #42, then #46.*
2. **Host bootstrap, one family at a time.** Dry-verify the real candidate with an
   empty GH configuration (proves tokenless verification), apply with
   `--apply --recheck`, observe receipts and public health, then create that
   family's flag and enable its timer. Order: static, web, native catalog, backend.
   The backend apply is a drain and restart and needs a quiet window.
3. **Controller hardening before the backend timer:** rollback circuit breaker and
   operator-required latch, per-family locks so a backend drain cannot freeze the
   others, a bounded drain report, latched verify failures, pagination and quota
   budget, and the small fixes above, with `scripts/test-auto-deploy.py` regressions.
4. **CI-side safety:** supersession-safe change detection (compare against the last
   published candidate, not `HEAD^`), native main-push promotion with
   version-advance detection, tag creation, per-version notes and tests, filter
   coverage and retry reconciliation.
5. **Zero-downtime backend:** automate the P66 A/B promotion (start B, canary,
   admission handoff, drain and retire A) after an isolated two-slot rehearsal with
   synthetic accounts. The one-time switch from single process to slots needs one
   planned restart.
6. **Coordinated protocol 32 release** for PR #44.

No secret is needed so far: GitHub-hosted builds use the ephemeral `GITHUB_TOKEN`
with OIDC attestation, the host pulls public releases over HTTPS, and the frontend
receives no credential.
