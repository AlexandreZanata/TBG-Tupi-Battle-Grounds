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

Status: Linux-first candidate **0.1.0-p68.1 published** with directed auth,
real-provider/browser and packaged runtime evidence below. Windows acceptance
remains pending. No real-user migration or installed application change occurred.

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
   implemented; synthetic coordinator tests and real Linux provider/browser
   journeys passed with the explicit transport/platform limits below.
4. Shared desktop routing/art and machine-checkable UI contract: implemented;
   source routing gate and comparator negative cases passed.
5. Focused consolidation, Linux reference/package and immutable GitHub
   prerelease: complete for the Linux candidate scope, with deferred gates below.
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

The following chronological ledger records directed acceptance evidence.
It does not mark all A01–A14/W01–W18 accepted: Windows, public HTTPS, full
matched visual and remaining device scenarios are explicitly deferred below.
No deferred platform check is a pass.

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

### Packaged practice journey correction

The first packaged AppImage lobby capture passed with boot_ready=true,
scene_waiting=false and zero pending/failed images or text. The Arena capture
then asserted default MatchInfo Forest before a match existed: consolidation
had changed Train here to open the Practice configuration screen, while the
old capture helper never pressed that screen's Play action. Investigation also
found its Catalog retained the startup Arena, so the configuration label could
show Forest after choosing Storm elsewhere. Opening the shared catalog now
synchronizes the saved Arena; the helper verifies that label, sends the normal
Practice action and waits for InMatch before checking the actual Arena.
This is presentation/integration glue, not a rule or balance change. The
failed assertion is not reported as a successful gameplay observation. The
source and package are rebuilt after this correction; no published artifact
or existing release is overwritten.


## Published Linux candidate and final consolidation

Release: [v0.1.0-p68.1](https://github.com/AlexandreZanata/TBG-Tupi-Battle-Grounds/releases/tag/v0.1.0-p68.1).
Draft implementation PR: [#32](https://github.com/AlexandreZanata/TBG-Tupi-Battle-Grounds/pull/32).
The annotated tag resolves to code source
`e59be5e910bf406222538105ed0a7af9ffce4a6c`. Later report-only commits do
not relabel or rebuild these immutable artifacts. Protocol 30, schema 9,
Ruleset 2 / `3d56156049e626c5`, 46 active/51 presentation kinds are unchanged
from the implemented candidate identity. Existing releases were not replaced;
this prerelease is not promoted to latest and does not trigger P66's native
publisher. Main was not merged, CI was not awaited, production was not deployed
and the installed native.3/data were preserved.

### Final commands and results

- Scoped `cargo clippy --offline --locked -p tbg-client --bin tbg-client
  --features ui-acceptance -- -D warnings`: passed after the Practice setup
  correction (1.31 s). A subsequent release batch received SIGTERM (exit 143)
  without a compiler diagnostic; no old binary was relabeled. One fresh batch
  passed: `cargo build --offline --locked --release -p tbg-client --bin
  tbg-client --features ui-acceptance` (3m07s), then `trunk build --release
  --offline --skip-version-check` (Rust compilation 2m07s plus wasm-opt).
  Both used source SHA above, shared Cargo target/cache, the host lock,
  CARGO_BUILD_JOBS=2 and BINARYEN_CORES=2. No concurrent Rust validation ran.
- `python3 scripts/package-native.py --platform linux --binary
  .local/release-p68-candidate/tbg-client`, followed by
  `bash scripts/package-appimage.sh <cached-official-linuxdeploy>` under the
  same lock: passed. Cached linuxdeploy SHA-256:
  `8aea8da0f7f7039d2a2cecb14657d752a222a5e1d3825caeef186c82f751cdd1`.
  Only owned unpublished staging was moved aside; shared Cargo caches and
  another front's artifacts were not deleted. Package identities, all 352
  portable and 358 AppDir manifest entries, post-patching executable hashes
  and safe tar paths verified. No byte-for-byte rebuild claim is made.
- `python3 scripts/check-native-auth.py --client
  target/native/AppDir/usr/bin/tbg-client --server
  .local/release-p68/tbg-server-test --report
  .local/release-p68-candidate/packaged-native-auth.json`: passed against the
  real Secret Service with synthetic account/data only. Secure persistence,
  same-family restore, explicit migration, process exclusion, token-free
  ordinary metadata, offline pending logout, restart/revocation and synthetic
  vault cleanup were all true. The final unbundled executable passed too.
  The separate unavailable-provider result above remains the negative case;
  no plaintext/mock provider fallback was introduced.
- Actual Chrome 156.0.8078.79 with its sandbox, isolated profile and the auth
  module on an owned same-origin fixture: protected HttpOnly/Secure/Strict
  cookies were observed through the browser cookie API, not document.cookie;
  persistent/session selection, same-family restoration, simultaneous Web
  Locks tabs, absolute expiry, native-route/CSRF rejection, ordinary storage
  and offline/restart logout passed. The first harness query filtered secure
  cookies with an HTTP URL and hid them; inspecting the unfiltered cookie API
  corrected that harness failure. HTTP loopback's secure-context exception
  does **not** establish public HTTPS acceptance. Synthetic Node tests remain
  distinct evidence from these real-cookie observations.
- `node .local/browser-game-smoke.cjs` on the final Trunk distribution:
  real game lobby and Practice through the visible control observed in pt-BR
  at 1440x900; page_errors=[]; sandbox enabled. This is a focused browser
  journey, not a complete game-screen gallery or cross-platform comparison.
- `python3 .local/capture-packaged-linux.py`: final AppImage outside the
  checkout, fresh temporary preferences, X11/Xvfb 1440x900, pt-BR. All five
  Arena training actions existed, Storm remained selected in configuration
  and actual Practice started in Storm. Capture reported boot_ready=true,
  scene_waiting=false, pending_images=0, pending_texts=0, failed_images=0.
  Screenshot visually inspected. The earlier packaged lobby screenshot was
  also inspected; its demo did not populate real match purchases/stats.
- `python3 .local/capture-offline-linux.py`: final AppImage outside the
  checkout with **no backend listening**, fresh data and an owned reserved
  loopback port. The same visible Practice route started Storm successfully.
  No production connection or installed application data was used.
- Missing-assets negative case: final release executable copied alone to an
  unrelated temporary directory exited 2 before graphics with `TBG: Game
  assets are missing. Extract the complete package and keep assets beside
  the executable.` It did not borrow checkout assets.
- Final source routing, patch-note and diff checks passed. Directed case totals
  are 89 Rust scenarios, 3 synthetic browser-coordinator scenarios and 2 strict
  comparator negative scenarios; reruns are not additional coverage. Earlier
  exact target commands and corrected failures remain in this ledger. No full
  workspace suite, broad benchmark, balance gate or extensive gallery ran.
- Normal branch push, new annotated tag push and `gh release create
  v0.1.0-p68.1 ... --verify-tag --prerelease --latest=false` succeeded.
  `gh release download v0.1.0-p68.1 --repo
  AlexandreZanata/TBG-Tupi-Battle-Grounds --dir
  .local/release-p68-downloaded`, then `sha256sum -c SHA256SUMS`: every
  published asset passed download-back verification. Tag dereferencing also
  matched the exact source SHA. No release upload was inferred from local
  files alone.

### Environment and artifacts

Pop!_OS 24.04 x86_64; Linux 7.2.6-76070206-generic; Rust/Cargo 1.99.0;
release profile with `ui-acceptance`; NVIDIA GeForce RTX 4060 Laptop GPU,
NVIDIA 595.104.02, Vulkan; X11/Xvfb for isolated captures. Xvfb emitted
XSETTINGS/zero-millimeter-display warnings, without capture/asset failure.
The ELF requires **glibc 2.39 or newer**. AppImage does not bundle glibc or
GPU drivers; older Linux distributions are not claimed compatible. Both
packages are unsigned candidates. Windows support/signing remain undecided.

- AppImage: 31,734,264 bytes; SHA-256
  `055569484d5bb961a0b352ace53ca45921599eae81102857cb624c5ec6a3178b`.
- Portable tar.gz: 37,590,537 bytes; SHA-256
  `af72529e33eac0fafd719c66de812a0b997c6646fd4b79a7fa45f0fe267ea6fa`.
- Assets SHA-256: `330bf402c958b5d3024f1814fdbfa9e38132ea4615e3328c3b6eb9a65b6b401f`.
- Cargo.lock SHA-256: `229debac9048d089ed9c47f8b3ffad4994713fd07da0584fdd7d967718662f19`.
- Release attachments include separate portable/AppImage build-info and full
  manifests, individual digests, provenance.json and SHA256SUMS. ELF hashes
  differ after linuxdeploy's RPATH patch and are explicitly distinguished.
- Owned local evidence: `.local/release-p68-candidate/` (native/bundled auth,
  browser game, Storm online-test/offline screenshots, missing-assets report,
  manifest verification, release metadata); real-cookie report remains
  `.local/release-p68-final/browser-auth.json`. No account keys/tokens, test
  databases, profiles, build outputs or screenshots were committed to Git.

### Remaining acceptance and next delivery

A01–A06/A09–A13 have the directed unit/store/HTTP/socket/client evidence above;
this is not an assertion that every manual/device/long-running variant has
been observed. A07 has actual Linux available/unavailable provider evidence;
Windows Credential Manager denial/Local persistence and Wine are pending.
A08 has real loopback cookie/session/multi-tab evidence and negative route
checks; blocked-cookie policies and actual public HTTPS remain unobserved.
A14's common code/i18n/source identity and strict contract are implemented;
exact Linux/Windows packaged pixels and both-language device journeys remain
pending. Natural-expiry/controller authority is tested in focused server
scenarios; the full actual cross-platform two-client match/reconnect journey
is not claimed from those tests.

W01/W13 and Linux portions of W03/W11/W12/W14 have the concrete evidence
above. No Windows W02–W12 claim follows: MSVC PE/ZIP/installer, clean-user
install/upgrade/uninstall, Wine prefix, actual Windows GPU/DPI/audio/input/
clipboard/TLS, authoritative mixed-platform match and full matched capture
matrix need the subsequent Windows delivery. W07/W08 remain unverified and
blocked by **any** visible difference; no masks/tolerance/driver waiver.
Signing/support/CRT/promotion decisions, public HTTPS and owner visual sign-off
remain explicit gates. The PR stays Draft for these outstanding P68 items.
No full-suite/benchmark/balance command is scheduled automatically.

## Consolidation update (2026-11-01)

The P66 front had independently used schema 9 and its own resume vault
writer. The consolidated branch keeps this front's coordinator as the single
vault owner, moves to server schema 10 for both feature sets, keeps protocol
30 and routes auth requests to the release slot that holds the socket (D-99).
Its refusal of legacy WebSocket sign-in also required adapting P66's
release-slot tests. See [CONSOLIDATION-20261101](CONSOLIDATION-20261101.md).
