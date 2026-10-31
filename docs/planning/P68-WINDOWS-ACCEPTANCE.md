# P68 — Windows artifact, installation and parity acceptance

This is a future acceptance contract, not a test report. Read
[P68](../phases/P68-windows-native-release.md) for source identity, scope,
implementation order and owner gates. Every result is tied to the exact
downloaded artifact digest and source SHA. Do not check items as passed
from a source inspection or a different debug build.

## 1. Evidence matrix

| ID | Contract | Evidence required at consolidation |
|---|---|---|
| W01 | Correct base and content | Requested base SHA, final SHA, protocol/version/46-kind baseline reconciliation, reviewed OS-gate dispositions |
| W02 | Actual Windows binary | PE x86_64/MSVC identity, subsystem/import/resource inspection, pre-graphics identity command and source match |
| W03 | Complete independent package | Fresh staging, asset inventory, ZIP installed outside checkout with unrelated CWD; missing-assets negative case; no repository fallback |
| W04 | Per-user installation | Final installer on clean Windows user and Wine prefix; no admin prompt in the chosen supported runtime configuration; shortcuts/uninstall entry and icons |
| W05 | Upgrade/reinstall safety | Synthetic account/preferences retained, locked running app handled without killing it, interrupted/error path preserves a working install; no forced reboot |
| W06 | Portable/uninstall boundaries | Moved ZIP works, no writes to program files, uninstall keeps user data; optional purge never implicit; unrelated files untouched |
| W07 | Shared desktop interface | Screen matrix below on Linux reference, Windows GPU and Wine; classify every remaining OS gate |
| W08 | Graphics and animation | Arena/character assets, forms, VFX, camera, loading, training and full result cast; default renderer first, no silent feature removal |
| W09 | Platform integration | Audio/input/focus/DPI/clipboard/browser links/preferences/TLS/logging cases below; no key leakage |
| W10 | Authoritative online parity | Focused cross-platform match/reconnect/result scenario against the compatible test backend; rejection for mismatch; no protocol shortcut |
| W11 | Offline boundary | Launch/practice/training without network according to existing behavior; online errors recover without hang or false login |
| W12 | Compatibility observations | GPU/driver/backend, window/DPI/settings, load time, frame-time observations, memory/audio issues and compared reference conditions |
| W13 | Focused regression coverage | Exact scoped commands/results for changed platform, routing and packaging contracts; browser/WASM compilation once if shared guards changed |
| W14 | Immutable artifact identity | Download hashes, final executable and bundle manifest, source/tag match; deterministic inventories without claiming byte-reproducibility without evidence |
| W15 | Signing and safe CI | Explicit signed/unsigned state, signature/timestamp verification if signed, PR secret isolation, pinned tools and read-only build jobs |
| W16 | No false publication success | Candidate artifact set is complete; existing-release/missing-asset case fails visibly; promotion cannot race or silently skip Windows |
| W17 | Support and public promotion | O01/O03/O04/O05 resolved, known issues and minimum requirements stated, owner look approval, protected promotion decision |
| W18 | Delivery evidence | Report and PR link, separate passed/failed/deferred fields; installed Linux and existing Wine applications preserved |

W01–W16 are the candidate's technical acceptance set. W15 may explicitly
record an unsigned internal candidate; that is not a pass for public signed
distribution. W17 gates a public professional release. An unobserved real
Windows GPU journey leaves the relevant W04–W12 items deferred even when
Wine and Windows CI pass.

## 2. Environments and controlled comparison

Use one reviewed source SHA, matching assets/protocol and equivalent
settings for the Linux and Windows comparison builds. The frozen requested
base is the initial reference; capture any intended shared UI correction
separately so changes are reviewed instead of hidden in a new baseline.
Record OS build, Wine version where applicable, GPU/driver/backend, CPU/RAM,
monitor size, DPI scale, locale and graphics settings. Do not compare an old
Linux install against a new Windows candidate without disclosing the gap.

Minimum environments:

- **Windows CI:** clean MSVC runner for dependency resolution, headless
  identity, PE/import/package checks and scoped non-GPU regressions. Hosted
  Windows Server runners do not certify a consumer Windows GPU desktop.
- **Linux reference:** existing supported native renderer on the owner's
  host, launched from a separate comparison package/data directory. Keep
  the installed Linux application and account unchanged.
- **Wine:** downloaded Windows ZIP and installer on the same Linux GPU,
  isolated owned prefix and test data; inspect the default graphics path.
- **Actual Windows:** physical machine or GPU-capable VM running the
  intended supported Windows edition/build, without a developer SDK/CRT
  incidentally supplying package dependencies. Do not purchase a VM by default.

Suggested comparison configurations are test inputs, not new game rules:
the existing 1280×720 minimum and a common 1920×1080 desktop; Windows DPI
100%, 150% and 200% on available displays, including resize/fullscreen and
moving between monitors where hardware is available. Report unavailable
configurations rather than invent observations. Use the project's existing
desktop frame/memory budgets; collect representative frame-time and loading
observations once. Wine overhead is not a server capacity benchmark or
evidence of native Windows performance. Do not create new numeric pass
thresholds without an owner decision and canonical documentation.

## 3. Screen and journey coverage

Capture comparable stable states using the same arena, scene/seed, unit
kind/form, language, UI scale and window settings. Visually review layout,
readability, art completeness, input targets, clipping and animations.
Pixel equality across drivers is not required. Reuse a small coherent
capture set across journeys; all 46 champions and five auxiliary presentation
kinds need manifest/resolver coverage,
while expensive animation recordings focus on distinct art/rig paths and
observed failures. This does not authorize a repeated full gallery.

| Surface | Required parity/behavior |
|---|---|
| Startup and loading | Native background, logo, staged loading UI, useful failure for missing assets/unsupported graphics; no silent blank window |
| Lobby/header/footer | Navigation, guest/account states, game modes, profile card, visible creator attribution and links |
| Arcanas | Complete choices, details, point display and existing selection rules; readable pt-BR/English content |
| Strongholds | All available kinds, status/ability details, models and selection; no new tower mechanics |
| Arenas | Available/locked states, previews, selection and themed arena entry |
| Ranking/stats/profile | Native ranking presentation, scrolling, request loading/error/empty states and localized API-derived display data already present on the base |
| Troops | Existing catalog, lore/abilities/form stats, preview walk/attack/Victory motion, selection and return navigation |
| Training | Existing exact-team/three-champion selection, enemy choices, dummy and bot difficulties; reset/exit and offline behavior, without expanding P62 rules |
| Settings | Sound/volume, language, window/fullscreen, VSync, shadows/Low motion and persistence, including change during an active screen |
| Account flows | Create/sign in with name/key, visible text editing, copy/paste key/challenge link and return states; no invented recovery/payment module |
| Online entry | Existing casual/private/challenge/spectator flows, server unavailable, connection/loading/cancel/error and mismatched-version states |
| Battle HUD | Native top/bottom areas, BattleLine, Energy, Shop, affordability, reserve and any Reroll controls only in modes where this base enables them |
| Details/actions | Champion inspection, abilities/statuses, Evolution, Stronghold/Fortification, surrender, keyboard shortcuts and overlays |
| War Council | Paused state, Augment details, selection/confirmation, localized labels and resume according to the current authoritative rules |
| Result and celebration | Winner's living/dead cast facing the camera, individual existing Victory, pagination where applicable, result actions/rematch and clean exit |
| Reconnect/next match | UI reconstruction, late outcome/rating behavior, no duplicate results or stale training/result figures |

A known unfinished feature in the consolidated base remains a documented
baseline limitation. Windows must not silently lose an implemented Linux
surface or claim completion of another phase. New defects caused by P68 are
fixed here; unrelated feature expansion returns to its owning phase.

## 4. Native platform and installation checks

- **Paths/storage:** user name/install paths with spaces and Unicode;
  default data-directory identity, `TBG_DATA_DIR`, preference writes/restart,
  existing-file atomic replacement on Windows, read-only/corrupt preferences
  and safe error recovery. Unix 0600 is not a Windows ACL guarantee. Use the
  current Windows user's protected profile location; no broad Everyone ACL.
- **Input/clipboard:** keyboard text/backspace/select/paste, pt-BR accents,
  account keys without logging them, retained clipboard ownership, mouse
  hit testing after resize/DPI/focus changes, Alt+Enter/F11 and Alt+Tab.
- **Links/network:** public links open through the OS browser; validate
  supported URL schemes, no shell interpolation; HTTPS/WSS use normal
  certificate verification, handle timeout/offline/mismatch without freezing.
  No TLS bypass, extra firewall ports or relaxed message limits.
- **Audio/rendering:** menu/battle sound and volume persistence, unavailable
  audio device, textures/fonts/pt-BR glyphs, shadows/effects, focus loss,
  fullscreen restore, minimized/occluded networking and graceful exit.
- **Diagnostics:** version/source/protocol before rendering, PE details and
  redacted logs without a console, bounded existing log retention, actionable
  startup failure; no keys/tokens/personal paths in published evidence.
- **Lifecycle:** fresh install, extraction outside repo, shortcut launch
  from unrelated CWD, per-user reinstall/upgrade, locked files/install failure,
  uninstall/reinstall with retained synthetic data and optional explicit purge.
  No forced reboot, privilege escalation hidden in the launcher or app kill.
- **Distribution security:** final checksum/signature, notices/runtime
  redistribution rights and publisher accuracy. Observe normal Defender/
  SmartScreen behavior on real Windows; never instruct players to disable
  them. Fix/report false positives through normal vendor channels.

## 5. Wine installation and test runbook

The planner inspected only host availability. WineHQ's official wiki was
blocked by access protection during this research; no exact WineHQ version
or repository-install command was verified. The official Ubuntu noble
catalog and the host's existing signed repository metadata support the
distribution-package route. Revalidate this at execution time.

1. Inspect `/etc/os-release`, architecture/foreign architectures, package
   source signatures/candidates, available disk, GPU/driver and Vulkan/DX
   translation support. Record `command -v` and package versions. Do not
   assume a 32-bit Wine component or new architecture is needed for x64.
2. Simulate the proposed package transaction and review removals, conflicts,
   download size and repository origin. Install only the required Wine
   packages through the supported package manager when this execution scope
   is invoked. Avoid broad upgrades, mixed-release repositories, root Wine,
   unverified DLL downloads and `curl | shell` instructions.
3. Create an explicitly owned prefix under a P68 test directory, separate
   from `~/.wine`. Record its architecture and Wine Windows mode. Initialize
   it with the reviewed Wine tools; preserve existing prefixes/processes.
   Wine can access host files and is not a sandbox: run only the hash-checked
   project candidate and use a synthetic account.
4. Download the candidate ZIP/installer from the immutable CI/release
   artifact; verify hashes and signatures where applicable. List/inspect
   archive members before extraction. Execute its actual `.exe`, not an ELF
   wrapper and not an executable from `target/`.
5. Test portable launch, then installer/shortcuts, data persistence and
   uninstall. Use prefix-default Windows user data or translate an owned
   test path with `winepath -w` before giving `TBG_DATA_DIR` to the Windows
   process. A raw Linux path is not a valid Windows path contract. Keep
   credentials out of command lines and captured logs.
6. Run the grouped screen and online journeys with default renderer. Save
   screenshots, renderer/driver identity and classified errors. Alternate
   renderer tests, if necessary, are process-local diagnostic experiments
   using settings verified for this Bevy/wgpu revision; record separately.
7. Close the owned game/prefix normally and preserve evidence. Never use a
   global `wineserver -k`; do not delete arbitrary prefixes or host data.
   Cleanup is limited to owned disposable P68 files with no needed evidence.

If Wine cannot supply a required GPU backend, report the exact limitation
and proceed with independent real-Windows checks. Do not claim Wine parity,
disable visual features silently or change server combat to compensate.
If actual Windows access is absent, deliver the candidate with W04–W12
appropriately deferred; do not label Wine as real Windows validation.

## 6. Online scenario and reporting

Use a compatible existing test endpoint, synthetic accounts and the same
ruleset/protocol as the candidate. Against that backend, run one focused
Windows/Wine versus Linux (or compatible browser) match that includes the
affected controls, one reconnect and authoritative result/next-match
cleanup. Observe server rejects/version errors as needed without changing
the handshake. Reuse approved existing protocol/regression evidence when
unchanged. Respect the owner's VPS/deploy rules; this phase does not
authorize a production restart, new backend deploy or real ranked farming.
Browser compatibility of shared guard changes needs its own focused build
and observation, not an inference from a native binary.

The report separates: source inspection, Windows build, portable package,
installer, Wine GUI, actual Windows GPU, Linux reference, browser regression,
signing and publication. For each include passed/failed/deferred, exact
artifact/source, environment, command/scenario, evidence link and next action.
Do not hide a graphics failure behind a successful `--version` command.

## 7. Scoped command queue for execution

These commands are future consolidation targets, not planning checks. Run
compilation/tests only inside the Linux shared lock or the documented
Windows runner equivalent, with two Cargo jobs and serial test execution.
On Windows use the pinned MSVC environment and set `TBG_SOURCE_COMMIT` to
the reviewed full SHA in that shell before building.

```text
cargo fmt -p tbg-client -- --check
cargo check --locked -p tbg-client --bin tbg-client --target x86_64-pc-windows-msvc
cargo test --locked -p tbg-client --bin tbg-client --target x86_64-pc-windows-msvc platform:: -- --test-threads=1
cargo build --locked --release -p tbg-client --bin tbg-client --target x86_64-pc-windows-msvc
python scripts/package-native.py --platform windows --binary target/x86_64-pc-windows-msvc/release/tbg-client.exe
python -m unittest discover -s scripts/tests -p test_package_native.py
```

The `platform::` filter includes the existing private preference recovery
and WebSocket-origin tests. Extend the preference scenario to replace an
existing file and preserve synthetic keys across repeat writes; this is
Windows behavior, not an assumption based on Unix rename semantics. Queue
new headless identity, routing/manifest and packaging regression selectors
by their exact implemented names in the execution report. Cover wrong
source/protocol, stale staging, incomplete assets and unsupported runner
failures. Do not invent passing results for test files not yet written.

The corresponding Linux comparison build/package uses the existing native
commands under the same lock, once at consolidation. If shared guards
changed, queue `cargo check --locked -p tbg-client --bin tbg-client --target
wasm32-unknown-unknown` and one focused desktop browser observation. Defer a
cold extra build in front mode. Installer compilation, final signing and
Wine launch commands are recorded only after exact pinned tools, owned
paths and candidate version are selected; never substitute unreviewed
example commands for a tested runbook.

## Primary references consulted

These are implementation references, not a reason to install tools now.
Resolve and record exact current versions when execution begins.

- [Rust target linkage and CRT controls](https://doc.rust-lang.org/reference/linkage.html): inspect CRT policy explicitly rather than assuming a portable EXE.
- [cargo-xwin upstream](https://github.com/rust-cross/cargo-xwin): optional Linux-to-MSVC tooling; review SDK/CRT licenses before adoption.
- [Microsoft C++ deployment](https://learn.microsoft.com/en-us/cpp/windows/deployment-in-visual-cpp?view=msvc-170): central, local and static runtime deployment have different maintenance responsibilities.
- [GitHub runner selection](https://docs.github.com/en/actions/how-tos/write-workflows/choose-where-workflows-run/choose-the-runner-for-a-job): choose a supported explicit Windows runner label and record its image version.
- [Inno Setup privileges](https://jrsoftware.org/ishelp/topic_setup_privilegesrequired.htm) and [install modes](https://jrsoftware.org/ishelp/topic_admininstallmode.htm): use the non-administrative per-user mode deliberately.
- [Inno Setup license](https://jrsoftware.org/files/is/license.txt) and [commercial-license FAQ](https://jrsoftware.org/isorder.php): the inspected license permits commercial use under its conditions; the current FAQ describes a commercial purchase as requested rather than strictly required. Record the pinned tool's terms; no purchase is part of P68 planning.
- [Inno Setup command line](https://jrsoftware.org/ishelp/topic_setupcmdline.htm) and [file replacement](https://jrsoftware.org/ishelp/topic_filessection.htm): log test installs and handle locked files/restart behavior explicitly.
- [Microsoft SignTool](https://learn.microsoft.com/en-us/windows/win32/seccrypto/signtool) and [Authenticode timestamping](https://learn.microsoft.com/en-us/windows/win32/seccrypto/time-stamping-authenticode-signatures): sign/verify final bytes with explicit modern digests and trusted timestamping.
- [Ubuntu noble wine64 package](https://packages.ubuntu.com/noble/wine64): the inspected host's distribution family has an official package route; recheck architecture, candidates and dependencies locally.
- [WineHQ official wiki](https://gitlab.winehq.org/wine/wine/-/wikis/Download): access was blocked during planning; do not invent repository instructions or bypass that protection.
