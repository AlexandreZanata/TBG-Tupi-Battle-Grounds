# Future plan: main merge to safe deployment and desktop publication

Status: **planned, not activated**. Owner request: document the future automation;
for now, publish the compatible Linux/Windows beta and refresh the landing's
existing download catalog manually. This plan does not enable host timers or
change native updater/signing custody.

Read first: `AGENTS.md`, `docs/VALIDATION.md`, `docs/RELEASES.md`,
`docs/DEVELOPMENT.md`, `docs/SCALING.md`, `deploy/auto/README.md`,
`docs/ui/DESKTOP-UI-CONTRACT.md` and the current native execution report.
Use English for implementation/docs and the existing i18n for product text.

## Outcome and scope

A normal merge into protected main runs required CI. Relevant successful builds
produce immutable, source-bound and attested artifacts. The official TBG host
verifies and applies compatible server/browser/public-static updates. A new
reviewed desktop version publishes a complete Linux/Windows release; the landing
then advertises only compatible, verified downloads.

Scope: authoritative server, desktop browser game, public landing/admin static
sites, Linux/Windows packages and their download metadata. Mobile and macOS stay
out. Real administrative authorization, email-delivery foundations and P66 native
TUF keys/launcher/dual-slot rollout do not become complete through this automation.
Linux remains the visual reference under the core three-platform contract.

## Existing baseline to preserve

- PR #40 was merged normally as `70827a74e7ac66faaf36b23fd204e5b0d9ce1ff1`.
  The host's verified ancestry floor is
  `8be248e025ed8f56290d94604da75536726d765b`. Re-inventory these at execution time.
- Backend, browser and static workflows already consume successful current-main
  push CI and publish separate attested candidates. Source filters avoid backend
  replacement for frontend/documentation changes.
- Native builds produce Linux tarball/AppImage and Windows MSVC ZIP/installer.
  Public publication currently requires an explicit tag dispatch. Promotion is
  limited to a successful exact-source manual native run; PR artifacts cannot be
  relabeled as a release. Required CI still runs on the tag's exact source.
- Four host controllers and timers exist, but their activation flags/timers were
  disabled when this plan was written. The native controller changes metadata
  only. Do not fabricate first-delivery receipts or enable every timer together.
- The website already reads `/downloads.json`. A complete, attested release whose
  contract matches the running server enables the existing platform dialogs.
  Missing, changed, revoked or incompatible assets remain unavailable.

## 1. Confirm publication policy

Recommendation for owner approval before implementation: deploy relevant compatible
server/site changes after each successful merge, but publish a desktop release
**only when the client version advances**. Documentation-only merges must not
create desktop versions. Use the next unused SemVer/tag; never move a published
tag, overwrite assets or silently reuse a version for different bytes.

Keep unsigned beta labels until Windows publisher signing has its separate
approval. Decide how an emergency revocation affects an already downloaded
client; changing download links does not update installed applications.
A protocol/schema transition needs an explicitly coordinated rollout rather than
an automatic compatibility override.

## 2. Connect exact-source native build and promotion

Extend the existing native workflow rather than creating a second release system.
Build each OS once on the reviewed hosted image/toolchain. Retain MSVC, static CRT,
SDK selection, digest-pinned Inno, shared features/assets and source identity.

For efficient main-based promotion, explicitly review allowing a successful native
**main push** build as well as a manual build. Require the same repository/workflow,
exact source SHA, main ref for push events, successful required CI and complete
artifact identities. Reject PR merge refs, forks, failed/cancelled/stale runs,
mixed sources, mismatched contracts and a main that advanced during publication.
Do not simply remove the current manual-event condition.

Create a version tag once, attest the exact tested bytes and publish all four
formats, SHA-256 sidecars, manifest and provenance together. Add tests for the
new event/ref boundary and immutable-version behavior. A publish retry must
reconcile the recorded release; it must not turn a partial public upload into
success or replace assets. Keep build read permissions separate from publication.

## 3. Validate the package, not only its compilation

Complete the initial Linux/Windows acceptance on the exact release artifacts:
identity/contract, complete asset inventory, launch, critical navigation,
account persistence using the real OS vault, practice, per-user installer,
locked-app refusal, reinstall, uninstall and retained user data. Use synthetic
accounts/local backends for mutations. Preserve the owner's installed versions,
profiles and running sessions.

Establish scoped repeatable release smoke checks before removing manual observation
from later releases. Keep the visual contract's Windows/browser/motion coverage
honest: shared source, a CLI probe or one screenshot does not certify parity.
Record actual GPU/audio/DPI limits and pending cases; do not weaken them silently.

## 4. Bootstrap the official VPS once

After a real successful main build, refresh the reviewed controller scripts while
all relevant timers are stopped and no deployment is running. Re-check actual
systemd/kernel executable paths, source floor, contract/schema, immutable roots,
proxy routes and the attestation verifier. No legacy-host work is in scope.

For each family separately:

1. Dry-verify a real attested main candidate with the existing CLI and empty
   deployment GH configuration; no personal token or GitHub-to-VPS SSH secret.
2. Apply that candidate with the documented `--apply --recheck` path. Preserve
   player data, prior binary/site versions and current browser asset URLs.
3. Observe local/public health, source identity and its durable receipt.
4. Only then create that family's enabled flag and enable its timer.

Use `pull.py` for the server, `web_pull.py` for the browser and `static_pull.py`
for landing/admin. Run `native_pull.py` only after the complete native release
passes verification; observe both platform downloads before enabling its timer.
The detailed commands and rollback rules remain in `docs/RELEASES.md`.

Do not combine this with unrelated proxy/OS/container changes. Backend replacement
uses the existing unbounded drain: allow matches and result/reconnect windows to
finish; pause new admission as specified. No forced game termination, database
rollback over new player data or automatic incompatible schema migration.

## 5. Prove the automatic path and document operations

Exercise one controlled change per relevant family and observe the sequence:
main CI, relevant publication, host verification, compatible activation, public
health and durable source receipt. Confirm that frontend-only changes do not
restart the server, existing tabs keep their immutable assets, and native links
change without a game restart. Expect approximately ten-minute polling plus CI,
packaging and any player-safe drain; do not promise instantaneous deployment.

Include negative cases: failed/missing CI lane, wrong workflow/ref/source,
modified archive/asset, unpublished format, stale main, old source floor,
incompatible contract/schema, unavailable verifier, occupied host lock,
interrupted transition and unhealthy candidate. Fail closed and record an
operator-required state. Preserve journals/backups for recovery.

Record the exact enable/disable/rollback commands and responsible release identity
in the official guide. Keep P66 key custody disabled unless separately approved.

## Validation and completion

Follow the current minimum-per-front policy. At consolidation, run only the union
of affected guards, including `test-ci-required-checks.py`, relevant release-scope
checks, `test-native-package.py`, `test-native-artifacts.py`,
`test-native-catalog.py`, `test-native-downloads.cjs` and affected controller tests.
Run the actual Windows tools/installer smoke and scoped final-platform acceptance.
All local Rust/Trunk compilations use the shared nonblocking lock and two Cargo
jobs. No automatic full workspace suite, balance gate or extensive gallery.

Done means observed delivery from a real main merge, a complete versioned native
publication, both public download journeys working, four individually verified
controllers and truthful runtime/visual evidence. Preparation or “waiting for the
first candidate” is not completion. List every unexecuted acceptance case and its
reason, rather than treating a deferred check as passed.
