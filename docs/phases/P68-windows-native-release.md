# P68 — Native Windows release and Wine acceptance

**Status:** planning only, 2026-10-31. No Windows executable, installer,
Wine installation or runtime acceptance is delivered by this document.

**Requested base:** `codex/consolidated-linux-release`, frozen at
`55510c5363e3f1e555fa5133208a5faf20c5825b`.
**Planning branch:** `codex/p68-windows-native-release-plan`.
**Implementation branch:** `codex/p68-windows-native-release`.
**Decision:** D-96. **Architecture:** §24. No gameplay changes.

## Read first

- Root `AGENTS.md` and the owner's current minimal-validation policy. Use
  `docs/VALIDATION.md` where present; it was untracked on the inspected host,
  not part of the frozen base. Do not copy unrelated dirty files into P68.
- This phase, [acceptance plan](../planning/P68-WINDOWS-ACCEPTANCE.md) and
  [platform-gate manifest](../planning/P68-PLATFORM-GATES.csv).
- `docs/phases/CONSOLIDATED-LINUX-RELEASE.md`, `NATIVE-client.md` and the
  native distribution sections of `docs/ARCHITECTURE.md` / `DECISIONS.md`.
- Only the source, packaging and workflow files identified below, followed
  by their direct dependencies as necessary. Do not inspect `archive/`.

## 1. Outcome and boundaries

Deliver a real Windows x86_64 Bevy application, a complete portable ZIP and
a per-user installer, using the same reviewed game content, interface and
authoritative multiplayer contract as native Linux. A player needs neither
Rust nor the source checkout. Running a browser wrapper or renaming a Linux
binary does not meet this requirement.

Then install Wine on the owner's Linux host from a reviewed official
distribution source, install the exact Windows candidate in an isolated
prefix and compare it with the Linux reference. Windows CI, Wine and a real
Windows GPU session provide different evidence; none substitutes for all
three. Pending real-Windows acceptance means a candidate, not a certified
public Windows release.

The scope includes necessary shared-desktop UI corrections, packaging,
installer lifecycle, provenance, diagnostics and focused validation. It
excludes a new engine, combat/balance changes, new protocol features,
production deployment, a new updater service, paid infrastructure, macOS,
ARM64, 32-bit Windows and mobile work. P65–P67 are reserved plans on other
branches, not prerequisites or implemented features of this base. P66's
future updater must not be claimed as delivered by an installer upgrade.

## 2. Audited baseline and release blockers

At the frozen base, the client is `0.1.0-native.2`, protocol is **29**, and
the active roster has **46** champion kinds. Treat these as provenance, not
values to hardcode in new release scripts. Extract identity from the actual
reviewed source/binary. Reconcile any intervening changes explicitly.

| Finding | Evidence / affected files | Required treatment |
|---|---|---|
| New native interface is often Linux-only | `src/ui/mod.rs`, `native_battle`, `native_loading`, `native_ranking`, `ui/actions.rs`, `sync/messages.rs`, `local.rs`, shell/HUD files in the manifest | Share desktop behavior before calling Windows equivalent; retain browser routing |
| Native image manifest is generated only for Linux | `crates/tbg-client/build.rs::native_art_manifest` tests the target triple for `linux` | Generate the same deterministic art inventory on Windows; use target configuration, not the build host |
| Window background differs by OS | `crates/tbg-client/src/main.rs` selects the dark native ClearColor only for Linux | Preserve the approved native presentation on both desktop OSes |
| Platform services need actual Windows coverage | `src/platform/{mod,storage,http,window}.rs`, `config.rs` and native Cargo dependencies | Check Windows paths, atomic preference replacement, clipboard, links, TLS and graphics/audio; retain Unix-only permission code |
| Packager executes its input on the build host | `scripts/package-native.py` calls the binary's `--version` directly | Package primarily on Windows; a Linux cross-build must use an explicit runner or verified metadata path, never assert it ran natively |
| Reused staging can retain obsolete files | Same script uses existing directories and merged asset copying | Fresh owned staging or fail if dirty; deterministic inventory and no stale assets |
| Only Linux is built/published | `.github/workflows/native-release.yml` | Add Windows build/package evidence and a coordinated publication gate |
| Existing-release branch silently exits successfully | Same workflow's `gh release view` / `exit 0` path | Check every required asset and digest; missing Windows assets cannot count as a successful Windows release |
| Assets can fall back to the source checkout | `src/platform/window.rs::asset_path` | Development fallback may remain explicit; release mode reports missing bundled assets clearly |
| Existing icon is not proof of PE resources | `assets/branding/tbg.ico`, `build.rs` | Verify executable/taskbar/installer icons, version metadata and manifest in the final PE |
| Toolchain and some release tools float | `rust-toolchain.toml`, native workflow | Record/pin a working release toolchain, SDK, installer tool and action revisions; preserve Cargo.lock |

The CSV records individual OS conditions with source hashes. It is an audit
snapshot, not a patch and not a list of conditions that must all be deleted.
Also inspect native/WASM routing, multiline conditions, generated code,
Cargo target dependencies, build scripts and installer scripts. A simple
text substitution of `linux` with `not(wasm32)` is unsafe.

Host inspection found Pop!_OS 24.04 / Ubuntu noble, x86_64; `wine`, `wine64`
and `wineserver` were absent. Existing signed distribution repositories
advertised Wine `9.0~repack-4build3`. Installed Vulkan libraries are not proof
of a working graphics driver. Re-probe during execution; no host package
was installed in this planning task.

## 3. Engineering approach

### 3.1 Share the desktop product, isolate OS services

Classify each manifest row and its negative branch: shared desktop UI,
browser alternative, OS integration or capture/development tooling. Move
native HUD/loading/ranking, Augment actions, shell panels, art loading,
training preview and result presentation to the shared native path where
appropriate. Inspect paired imports, resources, plugin registration and
system parameters together so the Windows path is not only syntactically
enabled but complete.

Keep platform implementations under the existing `platform/` module.
Do not introduce a trait/framework for each service or split the game into
Linux and Windows clients. Preserve the existing browser HTML path and
`?ui=bevy` routing. Leave Linux-specific file modes and host tooling specific.
Update compile-time generated asset paths with forward-slash logical names.
Use target-side configuration such as `CARGO_CFG_TARGET_OS` for build-script
decisions; `cfg!(windows)` inside build.rs describes the build host.

Validate native art and content identities, not just a visible lobby. Include
46-champion coverage and all 51 presentation kinds, including Beetling,
Crate and the three drill ranks, plus the existing forms, abilities and Factions;
use a representative visual set plus targeted coverage of distinct rig,
effect and evolution paths. Do not repeat a full gallery on each work front.

### 3.2 Build Windows on Windows first

Use `x86_64-pc-windows-msvc` on a pinned supported GitHub Windows runner
image, with `--locked` and at most two Cargo jobs. Check the repository's
Rust minimum (`1.97` on this base) against the selected installed toolchain.
Record exact rustc, target, linker, Windows SDK and runner image versions.
An unavailable toolchain is a blocker, not permission to upgrade the engine
or regenerate Cargo.lock. Scope Cargo feature changes to actual Windows
compatibility; audit X11/Wayland/ALSA-related transitive target selection.

Linux cross-compilation with `cargo-xwin` is optional development tooling,
not the primary release path. Adopt it only after reviewing its Microsoft
SDK/CRT license requirements and pinned tool inputs. Do not switch to GNU,
download DLLs from third-party sites or change gameplay merely to make a
cross-build or Wine launch pass.

Before bundling, inspect PE architecture, subsystem, DLL imports, resources
and graphics backend support. Resolve the MSVC CRT policy at O02: evaluate
static CRT for a self-contained per-user candidate; otherwise use a licensed,
tested app-local/runtime deployment. A central redistributable can require
elevation and must not silently defeat a per-user installation promise.
Static linkage still requires an import audit and future security rebuilds.
Test on a clean Windows account without developer tooling or a preinstalled
VC runtime supplied incidentally by the build environment.

Retain the release GUI subsystem. Machine-readable build identity should
run before graphics/window creation and return a meaningful exit code;
add a small `--build-info-json` diagnostic if redirected `--version` is not
reliable for the GUI executable. Do not remove quiet normal startup just to
make packaging stdout easier. Public errors must be useful when no console
is visible: write redacted logs and present a minimal native error on fatal
startup/graphics/asset failure without relying on Bevy having initialized.

### 3.3 Complete artifacts and safe installation

Use the existing packager for a portable ZIP containing executable,
`assets/`, required licensed runtime files, notices, license, Windows usage
instructions and build identity. Avoid a second independent asset-copy
implementation. Validate asset paths, forbidden traversal/symlinks and
case-colliding filenames; Windows must not silently discard a Linux asset.
The runtime reads bundled assets beside the executable regardless of CWD.
All mutable preferences/logs remain in the current platform user-data
abstraction, never the install tree. Test Unicode usernames and spaces.

Add a small Inno Setup definition under `deploy/native/windows/`, with
pinned build tooling and documented license. Recommended defaults:
`PrivilegesRequired=lowest`, a stable application identity, user-local
Programs directory, user Start Menu shortcut, optional desktop shortcut and
an uninstall entry. No firewall exception, autostart, service or registry
association is needed. Use the same underlying package inventory as the ZIP.
Embed a multi-size TBG icon, truthful product/publisher/version fields and
an `asInvoker` manifest. Inspect Bevy/winit's DPI handling before adding an
application manifest; avoid conflicting declarations.
Map the prerelease SemVer to valid numeric PE FileVersion/ProductVersion
fields while retaining the full display version/source in string metadata.
Verify the runtime window/taskbar icon as well as Explorer resources; an
embedded icon alone is not proof of both.

Installer reinstall/upgrade must preserve the existing account key,
preferences and logs; compare data-directory identity between versions.
If the app is running, request a normal close or cancel safely: never kill
an online match or force a reboot. Installation failure must not report
success, leave half-replaced executable/assets or remove the previous
working installation. Stage replacements and handle locked files explicitly.
Uninstall removes only owned application files/shortcuts by default, keeps
user data, and offers any data purge only as a separate explicit choice.
Test rollback/reinstall with synthetic data, not the owner's real account.

### 3.4 Provenance, signing and publication

Required build metadata: semantic version, full source SHA, protocol,
target/architecture, toolchain/build-tool identities, bundled asset manifest
and executable digest. Produce a file inventory with SHA-256 values and
final ZIP/installer checksums; reject source/binary/protocol/tag mismatch.
Do not overwrite `0.1.0-native.2` release assets or move its tag. Choose a
fresh candidate identity after O03; use one reviewed source SHA for the
corresponding Linux and Windows comparison builds after portability fixes.
The requested commit is the starting point, not the eventual modified SHA.
The base's Linux release report identifies published native.2 as source
`1815eb70c691e36b39b6995e6f62e8e135334e3e`, which differs from the requested
planning base. Preserve that installed image; do not present it as a new
same-SHA comparison build.

For public professional distribution, gate publication on the owner's
code-signing decision. Sign the final executable first, then derive its
digest and package it; sign the installer, timestamp with a trusted service,
verify signatures and compute final artifact checksums afterwards. Sign
the generated uninstaller with the supported installer signing mechanism too.
Use SHA-256 file/timestamp digests and verify the application signing policy,
including a valid trusted timestamp, rather than a driver-signing default.
Protect signing credentials in an approved secret store; never expose them to
untrusted PR jobs. Record signer/timestamp verification. A SHA-256 file is
integrity evidence, not proof of publisher authenticity; Authenticode does
not promise SmartScreen reputation or a warning-free download.

Unsigned builds are allowed as clearly labeled internal prerelease
candidates for implementation and Wine testing. No certificate purchase,
new paid service or public signed-release claim is authorized by this plan.
If final signing changes previously tested bytes, repeat the affected
packaged smoke tests with those exact signed bytes before publication.

Separate CI build/package, artifact validation, optional protected signing,
and release promotion. PR jobs have read-only permissions, no release
secrets; pin actions/tools, key caches by target/toolchain/lockfile, set
timeouts and publish job-scoped write permission only. Preserve Linux jobs.
Build/upload artifacts without requiring release credentials. Upload final
artifacts/provenance, then test the downloaded artifact, not a local binary.
Prevent concurrent tag publication. An existing immutable release must be
verified against the expected full artifact set or fail visibly; do not
silently skip Windows or replace artifacts with a different digest.

Do not automatically publish on a tag before W01–W16 and owner look/signing
gates are satisfied. Attach a release manifest to the PR and use a protected,
explicit promotion. Keep large binaries and test recordings out of Git.

### 3.5 Wine and real Windows are separate acceptance environments

Follow the [acceptance plan](../planning/P68-WINDOWS-ACCEPTANCE.md). Install
Wine only when execution is explicitly requested. Inspect signed package
sources/candidates and simulate the package transaction first. Prefer the
existing distribution repository on the inspected host; a WineHQ repository
requires accessible current official instructions and a matching supported
distribution. Do not mix Ubuntu release names, install arbitrary PPAs,
upgrade the whole OS, disable security tools or run Wine as root.

Use an owned disposable prefix and synthetic user data. Wine is not a
security sandbox: execute only the project's verified artifact. Preserve
the installed Linux game, `~/.wine`, account files, other Wine processes and
host shortcuts. Record Wine version, prefix architecture/Windows mode,
graphics backend/driver and exact artifact digest. Do not fake a passed
Windows test by launching the Linux executable from inside the prefix.

Use the default supported renderer first. Any process-local alternate
backend experiment needs verified configuration for this exact Bevy/wgpu
version and a separate result. Never require disabling security protections
or silently stripping shadows/effects/audio to make Wine appear equivalent.
Wine translation overhead is recorded separately from native Windows
performance. A headless or software-rendering smoke test proves neither
playability nor visual parity. Schedule an actual Windows GPU desktop for
shell, DPI, drivers, TLS, clipboard and installer acceptance.

### 3.6 File ownership and allowed tooling

Keep changes in `crates/tbg-client/` platform/UI/build configuration,
`scripts/package-native.py`, `.github/workflows/native-release.yml`, native
release instructions and phase documentation. Add installer/resources under
`deploy/native/windows/` (`tbg.iss`, `tbg.rc`, and `app.manifest` only if a
manifest override is needed). Compile PE resources with the selected
Windows SDK resource compiler on the primary Windows build. A small
`scripts/package-windows.ps1` may orchestrate the existing packager and
installer compiler; it must not become another independent content pipeline.

If a pre-render Windows error dialog needs direct bindings, the already
locked `windows-sys` 0.61.2 is permitted as a Windows-only direct dependency
with only the required API features and a Cargo.toml reason. Keep that code
in a small `platform/windows.rs`, not in game/UI orchestration. Do not add
a dialog framework, async runtime, webview or platform abstraction layer.
Use the Python standard library for packaging contract tests at
`scripts/tests/test_package_native.py`; run the named checks at consolidation.
Any different dependency or tool choice needs a concrete reason in Deviations.

Core/server/balance code is not expected to change. A genuine compatibility
defect there must be reported and reviewed before widening P68. Existing
append-only contract tests and historical decision records retain their rules.

## 4. Ordered implementation steps

1. **Freeze and re-audit.** Create an isolated branch from the requested
   base; capture dirty state elsewhere without copying it. Record package,
   wire, roster, Linux reference and source ancestry. Refresh the manifest,
   classify every condition, resolve O01–O03 as needed. Commit the report.
2. **Desktop parity.** Correct shared/native UI routing and art generation,
   preserve Unix/browser services, cover missing-assets and Windows storage
   replacement semantics. Write narrow regressions; minimum front checks
   only. No combat rule or contract-fixture rewrite. Commit.
3. **Windows identity/runtime.** Establish MSVC target, CRT/import policy,
   PE metadata, diagnostic identity/error behavior and complete assets.
   Add only necessary packaging/resource dependencies with rationale. Commit.
4. **ZIP and installer.** Reuse the packager, add clean staging/inventory,
   safe per-user install/upgrade/uninstall and Windows instructions. Write
   focused packaging failure-path checks. Commit.
5. **CI and candidate.** Produce an immutable Windows candidate and Linux
   comparison artifact from the same reviewed revision, recording actual
   toolchain/signature status and hashes. Fix publication's false-success
   path; keep public promotion gated. Commit.
6. **One acceptance consolidation.** Run scoped checks once; obtain Linux
   reference captures, install Wine, test the downloaded ZIP and installer,
   then test the same candidate on actual Windows. Group journeys to reuse
   evidence. Record all unavailable environments as deferred, never passed.
7. **Delivery.** Address concrete defects, repeat only affected checks,
   update the report/roadmap and open a PR to `main` when the execution
   prompt authorizes publication. No automatic merge, VPS deploy or release
   promotion. Provide artifact links, acceptance status and remaining gates.

## 5. Validation policy and completion

This planning delivery runs documentation/link/CSV checks and
`git diff --check` only. It authorizes no local compilation or installation.
During implementation use minimum work-front checks; queue relevant tests
for one consolidation. Copying an acceptance list does not authorize full
workspace tests, balance runs, repeated galleries or capacity benchmarks.

An owner request to execute the attached prompt specifically requests the
Windows release packaging, one Linux reference package, Wine installation
and the focused acceptance journeys. It does not request the full suite.
On Linux every compilation/packaging validation takes the shared nonblocking
`/tmp/tbg-validation-$(id -u).lock` with `CARGO_BUILD_JOBS=2`. Exit 75 means
deferred; no wait/poll/cache deletion or alternate worktree to bypass it.
On a separate Windows runner use the same two-job cap and a nonblocking
host-wide lock equivalent, documented in the execution report; a local
Windows build without that agreed equivalent remains deferred. CI's
isolated runner is not a reason to bypass contention on the Linux host.

See W01–W18 in the acceptance plan. **Implementation done** means code,
focused regression coverage, docs and explicit evidence/deferred checks.
**Windows release accepted** additionally needs packaged Wine and real
Windows GPU acceptance, reference parity and owner visual sign-off.
**Public release ready** additionally needs signing/support/promotion gates.
These statuses must not be collapsed into a single green checkbox.

Write `docs/phases/P68-execution.md` with base/final SHA, decisions, changed
files, commands and exit status, artifact hashes, OS/tool versions, manifest
dispositions, screenshots/videos, installer lifecycle evidence, performance
observations, exact failures/deferred checks and their next action. Redact
account keys, tokens, usernames in public logs and machine-specific secrets.

## 6. Owner decisions and practical limits

- **O01 — Support:** recommend Windows 11 x64 as the primary supported
  desktop; document the tested Windows build, GPU/driver and runtime API
  floor. Windows 10 is a separate explicit support decision with its own
  evidence. The Wine prefix's emulated version is not a Windows support claim.
  Distinguish the runtime API floor and installer minimum-version check from
  the supported consumer OS list; select a legitimate Wine mode compatible
  with those checks. Do not patch out OS checks to manufacture acceptance.
- **O02 — Runtime/installer:** choose the audited CRT approach and confirm
  per-user installation. A required elevated runtime installation needs a
  visible exception; never hide it in the launcher.
- **O03 — Release identity/ancestry:** select a fresh candidate version and
  reviewed source SHA; check whether the requested consolidation is already
  in `main` before presenting a PR as a Windows-only change. List inherited
  commits if not. Do not merge unrelated consolidation merely to hide a diff.
- **O04 — Signing:** identify the publisher/certificate or approve unsigned
  internal testing. Any purchase/service/license acceptance requiring owner
  authority remains pending. Do all independent work first.
- **O05 — Test devices:** obtain an available actual Windows GPU session
  and subjective Linux/Windows/Wine visual approval. If unavailable, finish
  the candidate and document this remaining acceptance gate honestly.

## Planning report

Inspected the requested branch, platform conditions, build script, package
script, native workflow and read-only host package availability. Produced
this executable spec, an acceptance matrix, a hashed gate inventory and an
execution prompt. No source, server, installed game or host packages changed.
Documentation verification and publication evidence are recorded in the
planning commit; all build/runtime/signing acceptance remains future work.

Planning checks: `git diff --check` passed; a Python standard-library check
validated the P68 relative links and all 234 CSV entries against the exact
source lines and SHA-256 hashes in 37 files, with no format errors. No
Cargo, Trunk, package build, installer, Wine or game test ran.
