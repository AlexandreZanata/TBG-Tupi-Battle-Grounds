# P68 execution: Linux first and a shared desktop UI contract

## Owner scope and status

2026-10-31: execute the P68 handoff **Linux first**, publish a Linux GitHub
release and make exact Linux/Windows frontend equivalence an enforceable
contract. Persistent authentication remains part of P68-A; Windows binaries,
installer, Wine and actual Windows acceptance follow the initial Linux
delivery. Public Linux prerelease publication is expressly authorized by
the direct request, superseding the attached handoff's default prohibition
on automatic public promotion. No production deployment, main merge,
certificate purchase or change to another front is authorized here.

Status: transactional storage and auth/wire integration implemented; directed
auth consolidation in progress. No new release, real-user vault migration or
installed application change yet.

## Step 0: reviewed source and ownership

- Requested frozen source: `55510c5363e3f1e555fa5133208a5faf20c5825b`.
- Planning descendant: `21551deb603042caf2dabf95bf3a4cd0d365c487`.
- Source branch advanced to `e26d4d496edd9d6ef9b2866160cd30367dbcd448`:
  only the scoped replay fixture and CI efficiency changes since the frozen
  source. Their targeted validation is in `CI-EFFICIENCY.md`.
- Preserve the already published native.3 UI/audit delivery at
  `014b0e0c046b38c551ee24534ee00709cde0e493` by integrating that reviewed
  descendant. This includes patch notes/news, the approved proportional
  audit criterion and CI corrections; gameplay numbers remain unchanged.
  Both append-only decision records are retained during the document merge.
- Chosen base: P68 planning plus that native.3 descendant, recorded by the
  normal merge commit containing this report. The separate source manifest
  `planning/P68-LINUX-SOURCE-MANIFEST.csv` hashes the audited inputs.
- `main` remains `9008f117c4289122ddff978c5d25720d2bd79687`; inherited
  consolidation commits will remain visible in the PR instead of pretending
  this is a small main-only diff.
- Isolated implementation branch: `codex/p68-windows-native-release`.
  Existing worktrees, dirty owner policy files, services and installed
  native.3 are preserved. P66 remains a separate front: no edits to its
  worktree, updater or native release workflow.
- Actual input identities: native.3, protocol 29, RulesetVersion 2, ruleset
  source hash `3d56156049e626c5`, schema 8, 46 active champion kinds and 51
  presentation kinds. Choose the next available auth wire/schema on these
  inputs, not the planning examples or unmerged P67 work.

## Decisions and delivery boundaries

- O03 recommendation: use a fresh explicitly labeled Linux P68 prerelease,
  never replace native.2/native.3 tags or assets. Final identity and source
  are recorded with the actual package after implementation.
- O01/O02/O04/O05 Windows support, CRT/installer, signing and device gates
  remain pending the Windows delivery. No unsupported Windows claim follows
  from enabling common desktop presentation.
- The Linux reference's visual quality is preserved. W07/W08 cannot pass
  from shared code alone: same-source packaged Windows captures and actual
  device evidence remain required. No default pixel/driver waiver.
- P68-A changes the auth wire. A new Linux candidate must not claim the
  current protocol-29 production server is compatible. Test a separate
  synthetic backend; production activation remains a distinct decision.

## Ordered implementation ledger

1. Base/audit and current policy: complete; source inventories retained.
2. P68-A canonical wire, transactional store, bounded auth service and
   socket authority: implemented; targeted server/protocol scenarios passed.
3. Native vaults, browser coordinator, shared auth UX and legacy migration:
   implemented; synthetic coordinator tests passed; real provider/browser
   journeys remain pending before the artifact evidence is recorded.
4. Shared desktop routing/art and machine-checkable UI contract: implemented;
   source routing gate and comparator negative cases passed.
5. Focused consolidation, Linux reference/package and immutable GitHub
   prerelease: pending.
6. Windows/Wine/signing and exact cross-platform visual acceptance: deferred
   to the requested subsequent Windows stage, never represented as passing.

## Validation ledger

Level: minimum work front, then one directed consolidation. The owner
request authorizes the Linux package and required auth/UI journeys, not a
workspace suite, balance gate or benchmark. Every local Rust compilation,
test and package uses the shared nonblocking host lock and two Cargo jobs.
Exit 75 is recorded as deferred; no bypass is introduced by this phase.

Executed: read current AGENTS/VALIDATION; inspect planning/read-first inputs,
source manifests and current auth boundaries; fetch the four explicit refs;
check ancestry/remote release identities; hash baseline source inputs;
resolve the additive decision-record conflict preserving both sides.

A01–A14 and W01–W18: implementation acceptance pending except the recorded
source/ownership inventory portion of W01. Commands, outcomes, artifacts,
secure-store modes, exact failures and remaining gates will be appended as
they actually execute. No deferred platform check is a pass.

### P68-A durable storage front

Added schema 9 after checking the integrated schema 8 and the unmerged
P67 source, which reserves no implemented schema 9 migration. The bounded
existing writer now owns hash-only family creation, atomic refresh history,
confirmation, scoped revocation and indexed cleanup (256 rows total per
turn). Registration, ranked baseline and initial family share a transaction;
existing registration reuses that same helper. No production database is
opened or migrated by this front.

Minimum check: `cargo check --offline --locked -p tbg-server --lib`, through
the shared lock/two-job helper, passed (0.92 s final check). Initial compile
reported `u64: ToSql/FromSql` and `usize: ToSql`; auth storage now uses
checked nonnegative SQLite i64 timestamps and a checked bounded batch limit.
Formatting applied to the affected package. The new named store regression
scenarios in `tests/auth_sessions.rs` are queued for the single targeted
consolidation, not claimed as executed yet. HTTP/WS/vault wiring is next;
this storage step alone does not enable or establish complete P68 auth.

### P68-A coordinated wire and authority consolidation

Implemented protocol 30 (new appended tags only), hash-only schema 9 families,
900-second Access grants, absolute 30-day families, bounded 30-second retry
caches with monotonic deadlines, separate IP/family budgets, registration
retry proof, indexed bounded cleanup and fail-closed capacity handling.
Browser/native routes enforce strict DTOs, 4 KiB/5-second bounds, no-store,
exact configured Origin/custom headers, duplicate-header rejection, cookie
isolation and Secure/HttpOnly/SameSite=Strict host cookies. Browser cookie
persistence choice survives rotation/restart. Logout-all never clears an
unrelated account's cookie. Revocation uses one account/family barrier before
acknowledgment, including naturally expired admitted-match leases.

Sockets authenticate account/family before attaching a resume handle. Passive
same-family restore cannot displace an attached controller. Explicit claim
alone authorizes takeover. Renewal uses the current socket; natural Access
expiry preserves only the admitted match lease. Queued intentions/surrender
carry the server connection ID and are rejected before state/stats/replay when
that controller has been replaced or revoked. Bots now issue over the native
HTTP channel and keep credentials in process memory. No combat/rating value
or deterministic simulation state changed.

Directed consolidation executed on this worktree diff (shared lock, two jobs,
serial test execution):

- `cargo test --offline --locked -p tbg-server --test auth_sessions --test auth_http --test accounts --test contract_registration --test arcanas --test ranked --test leaderboard --test contract -- --test-threads=1`:
  first sandbox execution could not bind loopback (`Operation not permitted`).
  Repeated with the normal host authorization for synthetic 127.0.0.1 servers;
  accounts 12 and Arcanas 3 passed. New HTTP tests exposed duplicate Origin
  acceptance and a test-only IP-budget collision. Reject ambiguous duplicate
  auth headers; isolate the family-cap test with a larger *fixture* IP budget,
  leaving production defaults unchanged.
- Remaining HTTP/store/registration/queue/ranking targets after those fixes:
  22 passed (HTTP 4, store 3, queue contract 3, registration 2, leaderboard 4,
  ranked 6). Leaderboard target took 67.38 s; no rerun or benchmark followed.
- `cargo test --offline --locked -p tbg-server --test auth_websocket -- --test-threads=1`:
  3 passed, including same-family passive conflict, natural-expiry/same-socket
  renewal and logout-all of an expired admitted-match lease (0.08 s).
- `cargo test --offline --locked -p tbg-core --test contract_protocol -- --test-threads=1`:
  14 passed, old tags retained and new credential/debug bounds covered.
- `cargo test --offline --locked -p tbg-server --lib match_task::tests -- --test-threads=1`:
  4 passed, including stale/revoked controller frames before state/metrics.
- `cargo test --offline --locked -p tbg-server --lib lobby::tests -- --test-threads=1`:
  16 passed; synthetic ranked fixtures now include a valid auth binding.

Client auth/net targeted tests also passed (3 + 6); native and WASM checks
passed. Initial client integration exceeded Bevy's plugin tuple size; the
additional plugin is in the second existing tuple. The deprecated Interaction
approach was replaced with the project's existing hit-test pattern before
acceptance. Browser check warnings and pre-existing test-only `Layout::clear`
warning are recorded for scoped cleanup. Real vault/browser journeys,
migration cancellation/offline cases, package evidence and Windows visual
gates remain separate acceptance work. These checks are not a full suite.

### Native/browser coordination and shared UI front

The client now uses one epoch-checked coordinator for HTTP auth and socket
identity. Native refresh uses the explicit real keyring providers, a per-origin
OS process lock independent of TBG_DATA_DIR, and nonsecret atomic metadata.
There is no plaintext token/AccountKey fallback. Temporary sessions are visible.
Browser credentials use one Web Locks coordinator, nonsecret operation markers,
strict bounded fetch/no redirects and an Access-only fallback when coordination
or cookies are unavailable. Public DOM status/remember controls never receive
Access, Refresh or recovery keys. Explicit key export uses Blob URLs on web;
native recovery export reports a write failure before backup acknowledgment.

Logout/account switches invalidate the epoch before work completes. Late
successes retain revocation proof and schedule cleanup instead of publishing
identity. Native cancellation stores pending proof only in the real vault;
unavailable secure storage leaves an explicit nonsecret recovery marker.
Offline logout remains pending. Legacy deletion requires current-family secure
confirmation and explicit backup acknowledgment, and compare-and-remove never
erases a replaced key. Ordinary account updates no longer rebuild the header
on every frame. Public grant metadata includes persistence choice and server
time; restored session cookies never silently become persistent cookies.

Linux presentation guards were reviewed by concern (HUD/details/countdown,
menus/header/catalogs/ranking, loading/GPU readiness, lobby demo and capture)
and moved to the target-side `tbg_desktop` selector for Linux and Windows.
Actual vaults/Unix file permissions remain OS-specific. Native art inventory
uses the target OS, sorted paths and normalized separators. Source check
`python3 scripts/check-desktop-ui.py` passed; it is explicitly not visual proof.
The comparator requires the complete same-input matrix and rejects missing
frames, source/assets/settings differences and every decoded pixel mismatch.
Two directed comparator negative-case tests passed using existing Pillow 13.0.0
in the project's logo-tools environment. System Python lacked Pillow; no
system package install was attempted. The small CI gate pins Pillow 13.0.0.

`node crates/tbg-client/tests/auth-session.test.cjs`: 3 synthetic coordinator
cases passed (no Web Locks => credentials omitted; late cancelled login;
serialized lost-reply retry). These mocks are not real-cookie evidence.
`cargo test ... -p tbg-client --bin tbg-client auth:: -- --test-threads=1`:
4 passed, including a completed-but-cancelled login never publishing identity
and retaining revocation proof. Scoped client Clippy with `ui-acceptance` and
server `--lib` passed with `-D warnings`; initial new lints were corrected
(constant-size chunk decoding, retry argument grouping, nested migration
condition). Native and WASM checks passed; a browser bridge method was corrected
to Bevy 0.20's `insert_non_send`. The existing test-only Layout::clear warning
remains recorded; default binary Clippy passes without suppressing it.

Additional HTTP/socket edge cases passed: concurrent same-operation retries,
unrelated cookie preservation during manual recovery, wrong-family/guest
resume refusal and ordinary logout refusing an admitted match. Latest focused
HTTP/socket run: 6 + 4 passed. No full workspace suite, balance gate, benchmark,
Windows build, Wine, installation or production rollout was run.

### Candidate preparation

Fresh identity: `0.1.0-p68.1`, protocol 30, schema 9, Ruleset 2 /
3d56156049e626c5. The `v0.1.0-p68.1` candidate does not match the existing
native-tag publisher trigger; P66's workflow/updater and existing releases
are untouched. Package staging now refuses reuse/overwrite, records assets,
Cargo.lock, executable and file manifests, and includes exact provider/D-Bus
license texts. Release-mode assets must be beside the executable; source-tree
fallback is restricted to debug builds. `--version` and `--build-info` run
before graphics and expose no credentials. Opt-in `ui-acceptance` enables
packaged captures and strictly loopback/fresh-directory synthetic auth probes.
`check-native-auth.py` starts/stops only its own backend process, validates
restart/offline logout and reports real vault availability without logging
credentials. The package/runtime runs and their hashes are still pending.

Static checks passed: affected Cargo formatting, `git diff --check`, Python
syntax for package/comparison/journey scripts, shell syntax for AppImage,
JS module syntax, patch-note gate (3 entries; same ruleset fingerprint).
The candidate's public patch note explicitly preserves game numbers and states
production incompatibility and the remaining Windows visual gates.

### First real Linux provider evidence and final connection boundary correction

At source `a5b10c5e14b5c34ef865c951bb68fcf19818e541`, the Linux release
build passed in 3m24s; the Trunk release build passed in 2m12s. Headless identity
reported protocol 30, the exact source SHA, x86_64-unknown-linux-gnu, release
profile and `ui-acceptance`. Core protocol contracts passed again (14 tests).

`python3 scripts/check-native-auth.py --client <built-client> --server <owned-test-server> --report <owned-output>`
passed against the **real Secret Service**: secure write/readback, same-family
restore/rotation, explicit legacy migration, per-origin process exclusion,
no tokens in ordinary auth metadata, offline logout marker, backend restart,
revocation and removal of the synthetic vault entry. A separate run with
`DBUS_SESSION_BUS_ADDRESS=unix:path=/tmp/tbg-p68-unavailable-secret-service`
made only that subprocess's real provider unavailable: secure persistence and
migration remained false, the pre-existing synthetic key was preserved,
logout stayed pending offline and remote revocation completed on recovery.
No real-user account, installed app data or unrelated vault entry was used.

Final connection review adds origin binding to the permitted family-bound
resume handle, including guests, and holds/reissues an expired mode request
only after renewed authorization. Acknowledged invitations remain probes and
are not recreated. Duplicate refresh requests coalesce; a server losing its
Access index can try the existing durable family before manual recovery, while
revocation/invalid renewal stops new authorization. Explicit Continue here
survives a simultaneous refresh completion. `cargo test ... -p tbg-client
--bin tbg-client net:: -- --test-threads=1`: 8 passed, including the two new
boundary cases. Scoped binary Clippy passed. A fresh final-source Linux/WASM
build follows these source changes; the first binary is not the published
artifact and is not silently relabeled.
