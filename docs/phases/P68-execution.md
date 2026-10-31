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
   socket authority: pending.
3. Native vaults, browser coordinator, shared auth UX and legacy migration:
   pending.
4. Shared desktop routing/art and machine-checkable UI contract: pending.
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
