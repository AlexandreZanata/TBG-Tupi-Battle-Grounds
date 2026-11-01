# Native cross-platform release and OS-aware downloads (2026-11-01)

Owner scope: finish Linux/Windows packages, run Windows inside the owner's local
Docker/KVM VM, compare with Linux, publish matching GitHub releases and update the
landing download choices automatically. This work extends PR #40; mobile/macOS are
not native release targets.

## Current work-front state

- CI canonical failures were corrected separately in D-101 / commit `70dc1c6`;
  all 20 tests across the four failing targets passed under the shared lock/jobs=2.
- Candidate version is `0.1.2-rc.1`, protocol 31, ruleset 6 / 097385fee9a10b0f.
  No gameplay source or number changes are introduced by native packaging.
- MSVC build embeds the existing multi-resolution icon, numeric/string versions
  and an asInvoker manifest. Winit owns DPI configuration. Static CRT avoids a
  silent elevated runtime installer. The portable ZIP and per-user Inno installer
  share the exact file inventory. Inno 6.7.1's official license permits commercial
  use without a required purchase; retain its copyright in the generated installer.
- Windows runner is explicitly windows-2022, SDK 10.0.26100.0, Rust 1.99.0,
  Inno 6.7.1, with actual tool identities recorded. Windows compilation uses two
  Cargo jobs. Build/package checks inspect x64 GUI PE/resources/runtime imports.
- Matching native artifacts are built on both OSes from one source and feature set.
  Publication is an explicit tag dispatch after runtime inspection, gated by all
  required CI lanes. Promotion may reuse only successful same-repository,
  exact-source native dispatch artifacts, preserving the exact tested bytes.
  A PR merge artifact or another source/workflow cannot be substituted.
- Windows publisher signing is not configured. Packages remain clearly labeled
  unsigned beta candidates, not a signed, SmartScreen-trusted or pixel-certified
  stable release. GitHub provenance and SHA-256 are separate from Authenticode.

## Validation so far

Native work-front source: affected formatting and scoped client Clippy with
`-D warnings` passed (6.07 s), shared lock and two jobs, existing cache. Native
workflow actionlint and nine required-CI policy scenarios passed. No full local
workspace suite or unrelated gallery was run. Windows compilation, installer,
exact downloaded artifact identity and runtime results must be added below;
source inspection is not a pass for these gates.

## Reused Windows test environment

The existing stopped `tbg-win11` VM was inventoried and started without recreation.
Image is official dockur v6.06, digest
`16fd7c39a3e48bb946d29c59dc6015942e5b900a1fa5d7e5552053bfc48ee2ea`.
It uses KVM, four vCPUs, 6 GiB RAM, a sparse 64 GiB image on `/data` and loopback-only
VNC/RDP ports. Its existing interactive job agent and earlier evidence are retained.
No unrelated container, image, account, installed game or host display driver changes.

The prior report concerns an unofficial GNU/protocol-30 package and does not validate
this new MSVC/protocol-31 candidate. It reports DX12/WARP software rendering,
black engine screenshots despite visible GDI captures, slow loading and no audio
device. These are known environment/acceptance issues to investigate rather than
hide by disabling game quality. A running VM is not real hardware GPU acceptance.

## DevShell compatibility correction

The first Windows job rejected `-DevCmdArguments` on `Launch-VsDevShell.ps1` before
compilation. That flag belongs to the underlying `Enter-VsDevShell` cmdlet. The
helper now loads Microsoft's installed DevShell module directly, checks that the
cmdlet supports the parameter, selects x64 plus SDK 10.0.26100.0 and verifies the
resulting SDK/architecture. It does not silently fall back to another SDK. This
fix is source-reviewed; the next Windows run must provide actual environment/build
evidence. Native artifact publication remains gated, not claimed successful.

## Landing and metadata work-front checks

The landing now has Windows/Linux cards, OS recommendation and keyboard-accessible
format dialogs, translated in English/pt-BR, using the existing gold/navy design.
Download eligibility requires a complete, original-repository, hash/size-bound
package pair and the live server contract. The metadata-only controller shares
the existing lock/verifier; native tag attestations do not weaken the main-only
server/web/static policy. Initial route adoption still uses the quiet-window guard.

Checks on this working diff: six `test-native-catalog.py` scenarios passed;
`node scripts/test-native-downloads.cjs` passed; existing auto/web/static controller
tests passed (10/12/9); `python3 web/scripts/check.py` passed (120 landing files).
Chrome checked the pending catalog and a local-only complete fixture, Windows
recommendation, both platform dialogs, both languages, checksums, Escape/focus
return and zero page/CSP/console errors. Screenshots were inspected; fixture URLs
are not published. No Cargo compilation was needed for this front.

The downloaded Linux Actions artifact from run `41348898691` passed its outer
SHA-256 and reports source `9fac2d611fff87b99dfaa38bff91898c2aed7dab`, client
0.1.2-rc.1, release/ui-acceptance, protocol 31 and w31-r6-097385fee9a10b0f.
Windows environment setup passed in that run; actual Windows build/runtime and
installer evidence remain pending, distinct from this source/landing check.

The Linux candidate rendered the lobby on the host's NVIDIA RTX 4060/Vulkan with
boot_ready=true, no pending/failed images and exit 0. The packaged README still
contained old protocol-30 prose, discovered during package inspection. Packaging
now renders its version/source/contract from the verified executable metadata and
explains both OS vaults. Final publication must rebuild/retest this updated source;
the `9fac2d6` package is inspection evidence only, not the final download.

The Windows lane now runs scoped platform regressions in the release profile,
sharing optimized dependencies with the actual executable instead of compiling
the entire Bevy graph in debug and release on a cold runner. Tests remain enabled.
Trusted manual branch builds may save their Windows dependency cache; PRs and
release tags cannot write it. Existing caches and active jobs were not removed.

## Official landing adoption

Landing `20261101T210337Z-e1017fc55ed0` is active. Both platform cards/dialogs
were checked in public Chrome (English/pt-BR, pending state plus a browser-local
fixture only, zero console/CSP errors). `/downloads.json` returns HTTPS 200 with
`application/json` and `no-cache, no-transform`, currently both platforms pending.
The new controller is installed with its timer disabled until an actual attested
complete release is observed. Backend/web/static timers remain disabled.

The owner explicitly authorized interruption of their single test tab for initial
proxy adoption. A one-off guarded maintenance operation required at most that one
connection and zero matches/ranked/results, backed up the TBG include, compared
all other directives, validated Caddy and reloaded it. The main Caddyfile and game
server PID stayed identical. Normal zero-connection guards remain unchanged.
No game executable, player DB, native signing keys or unrelated service changed.
