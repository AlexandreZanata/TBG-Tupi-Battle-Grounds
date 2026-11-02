# Native beta publication and one-off landing download refresh

Status: **published and public landing downloads verified**. Release
[v0.1.2-rc.1](https://github.com/AlexandreZanata/TBG-Tupi-Battle-Grounds/releases/tag/v0.1.2-rc.1)
contains both tested platforms. The official catalog and both existing download
dialogs are live. This is an unsigned beta with full visual acceptance pending.

Owner scope (2026-11-01): save the future automation plan and make the new
Linux/Windows release available through the official landing now. Automatic
backend/browser/static/native-catalog timers remain disabled in this delivery.

The future plan is [MERGE-TO-DEPLOY](../plans/MERGE-TO-DEPLOY.md), proposed in
[PR #42](https://github.com/AlexandreZanata/TBG-Tupi-Battle-Grounds/pull/42).
It does not implement or activate future merge-based native publication.

## Release identity and trust boundary

PR #40 was merged normally into main as
`70827a74e7ac66faaf36b23fd204e5b0d9ce1ff1`. Client version is `0.1.2-rc.1`,
protocol 31, contract `w31-r6-097385fee9a10b0f`. The official server uses the same
contract and schema 10; its process and data are outside this metadata-only update.
The old public `0.1.1-rc.1` Linux/unofficial Windows packages are incompatible and
must not be substituted for this version.

Initial native build run
[41361235748](https://github.com/AlexandreZanata/TBG-Tupi-Battle-Grounds/actions/runs/41361235748)
was rejected before publication because Windows checkout changed hashed bytes.
The final tag uses corrected source `ebae3645ee8875ac61a511c7dce85df0dfddd3dc`,
with the exact-source manual-build and tag-promotion evidence recorded below.
No PR merge artifact was relabeled, provenance fabricated or public asset replaced.

## Initial attempt: superseded, not final acceptance

Linux artifacts downloaded from the manual run passed both format checksums,
exact-source CLI identity and all **355** file-manifest entries. AppImage
extract-and-run reports the same source/contract. A packaged Linux Lobby capture
at 1280x720 rendered successfully with NVIDIA RTX 4060 / Vulkan 595.104.02,
`boot_ready=true`, no waiting scene, no pending/failed images or texts. The test used
an isolated Xvfb display and data directory with an unreachable loopback server;
no real account or installed app was used.

The first display attempt was started while extraction was still completing and
returned 127. It is not runtime evidence. The completed-package check and subsequent
capture above passed. Artifacts and command outputs are retained privately under
`/data/tbg-native-0.1.2-rc.1`, outside Git and Cargo caches.

The initial Windows pair was not accepted. Its later rendering-environment
pilot does not certify the final corrected packages. No complete pixel,
GPU/audio/DPI or multiplayer matrix is inferred from a build or one screenshot.
Existing Linux playtest evidence
for another source is not the final package's acceptance.

## Production scope

The reviewed installed native catalog scripts match this source. The official
host has current `gh` attestation verification tooling and a verified ancestry
floor. After the complete compatible attested native publication, we invoked only
`native_pull.py` once and verified `/downloads.json` plus both existing platform
dialogs. The old catalog was backed up before its atomic replacement; the game
process, TLS/proxy configuration, site asset pointers and all timers/flags stayed
unchanged. No database mutation was part of this metadata update.

This publication does not install a native package on the VPS, activate TUF,
restart the backend, deploy landing/admin HTML or enable general automatic rollout.
Only the existing download metadata changed.

## Owner-requested consolidation handoff

A read-only inventory confirmed that PR #22 head `1d27ae9`, PR #26 head `3a95248`
and PR #39 head `2000407` are ancestors of main `70827a7`, with zero unmerged
three-point diff paths. Their old stacked bases/descriptions must not cause the
integrated roster, native UI or public-site work to be reapplied or downgraded.

PR #41 head `bf6da86` is not an ancestor and has 96 changed paths. A merge-tree
simulation (no worktree/ref changes) identified conflicts in `protocol.rs`,
`contract_protocol.rs`, `DECISIONS.md`, `GAME-DESIGN.md` and `ROADMAP.md`.
Its P73 D-101 identifier collides with main's D-101 deterministic-reference
reconciliation; main also has D-102. Allocate a currently free decision ID for
P73 and preserve current protocol bytes, all competing requirements and tests.
History's optional schema-11 migration/privacy rollout must remain separately
gated; historical tests/captures before P73's final edits are not final acceptance.

A complete owner-facing handoff prompt includes these findings, current CI and
Windows fixes, native publication/source freeze, visual correspondence, deployment
families, disabled timers and first-candidate gates. No PR was closed or merged
by this handoff inventory; the other agent must reconfirm remote state first.

## Blocking initial pair rejected: Windows checkout conversion

The first complete pair from main/manual run 41361235748 was correctly rejected
by the existing native manifest validator before publication. Linux Cargo.lock
SHA-256 was `b7f7662c4d00f01a6a5bb9d48029a5ec02957db52f0d5df5cfedd9ad2218046d`;
Windows reported `cac230345f32cf51d7632e63b61ae30a6d0b670c4af551f9ebcb3d622d420dab`,
exactly the hash of that file converted to CRLF. The Windows catalog contained
1454 CRLF endings; Linux had zero. Windows therefore also reported the wrong
contract `w31-r6-0ffb33fcd2a7364e` and another asset digest. Those files must not be
promoted, and their Linux-only runtime evidence does not certify the final pair.

Added repository UTF-8/LF checkout attributes and a native-input check before
either OS compiles. Binary assets remain verbatim and the read-only historical
archive is excluded. Raw package identity/promotion checks remain unchanged.
No gameplay rule, number or wire version changes: Windows produces the existing
Linux contract.
Three tests reproduce autocrlf without attributes, prove canonical text/binary/
historical-fixture bytes with attributes, and reject CRLF/non-UTF-8 inputs.

Local targeted checks passed: 3 input scenarios, the live source-input guard,
4 native-promotion scenarios, 7 package/workflow scenarios, 9 required-check
scenarios, native/CI actionlint and diff checks. No Rust compilation or broad
local suite was needed. Commands: `python3 scripts/test-native-inputs.py`,
`python3 scripts/check-native-inputs.py`, `python3 scripts/test-native-artifacts.py`,
`python3 scripts/test-native-package.py`, `python3 scripts/test-ci-required-checks.py`
and `git diff --check`. A new exact-source manual native run built the corrected
branch (main plus documentation/checkout fix); native beta publication is already
allowed independently of a main merge, with the same tag/provenance and
compatibility gates. Automatic main deployment remains only planned.

## Corrected candidate: exact-source acceptance

Corrected source: `ebae3645ee8875ac61a511c7dce85df0dfddd3dc` (main plus the
future plan and canonical-checkout fix). Manual build run
[41363044535](https://github.com/AlexandreZanata/TBG-Tupi-Battle-Grounds/actions/runs/41363044535)
passed the input preflight and completed successfully on both operating systems.
Linux's two download checksums and all 355 file-manifest entries passed.
Both the extracted executable and AppImage extract-and-run report this source,
version `0.1.2-rc.1`, protocol 31 and the existing canonical server contract.

The corrected Linux package rendered a 1280x720 Lobby in an isolated Xvfb display
on NVIDIA RTX 4060 / Vulkan 595.104.02. Capture exited 0 with `boot_ready=true`,
`scene_waiting=false`, no pending images/text and no failed images. The screen was
visually inspected. The process used a separate muted data directory and refused
loopback endpoint; no installed version, real account or production match changed.
Evidence is under `/data/tbg-native-0.1.2-rc.1-ebae364`, outside Git/build caches.

A separate offline Windows rendering-environment pilot on the rejected first
source proved that the existing Windows 11 VM can render via DirectX 12 WARP.
It is explicitly **not** acceptance of the corrected candidate. The VM has no
hardware graphics passthrough or audio output. Corrected Windows inventory,
runtime/installer checks and complete-pair verification subsequently passed as
recorded below.

### Corrected Windows and complete pair: passed

Both native jobs in run 41363044535 completed successfully. Hosted Linux native
tests: **180 passed, 0 failed, 2 ignored**; the ignored cases were not executed.
Hosted Windows platform tests: **4 passed, 0 failed**, release build and Inno
packaging successful. Windows recorded the real reviewed Inno 6.7.1 engine.
Its log explicitly reported no cache found: regression compilation took 11m38s,
and the application build took 5m05s. This cold corrected-lock run does not prove
warm-cache Windows/Linux timing equivalence; existing compiler/cache optimizations
were preserved and Windows saved its completed dependency cache afterward.

The production manifest validator accepted all four corrected formats together:
same source, dependency lock, assets (`08a8aaac40a992c38b2a19bc31b43ed74c38aba0193d0aedc9c5dc2553a7d086`),
features and canonical MatchContract. Extracted inventories passed **355 Linux**
and **354 Windows** entries (Linux additionally packages its installer script).
Private output: `/data/tbg-native-0.1.2-rc.1-ebae364/package-verification.json`.

The exact Windows ZIP and installer were exercised on the existing Windows 11
Pro 25H2 VM, without replacing the owner's installed application or profile:

- ZIP SHA-256, every manifest entry and binary digest matched inside Windows.
  `--version`, `--build-info` and `--contract` reported the exact source/version,
  MSVC target and canonical contract; embedded product was TBG / 0.1.2-rc.1.
- A lone executable refused missing assets with exit 2 in 0.8 seconds.
- Both real Windows Credential Manager probes wrote/read/deleted randomized
  synthetic entries. Existing target count stayed 2; zero TBG/synthetic entries
  remained. Existing credentials were not modified.
- Owned per-user installation verified every installed file and identity from
  an unrelated working directory. An exclusive executable handle caused a
  reinstall refusal; a subsequent reinstall and uninstall passed. A synthetic
  data sentinel outside the installation directory was retained. This checks
  that boundary, not every possible existing real profile or UAC environment.
- Final GUI capture exited 0 after 195.3 seconds at 1280x720, with no panics/errors,
  `boot_ready=true`, no waiting scene or pending/failed resources. The Lobby was
  visually inspected. It used DirectX 12 Microsoft Basic Render Driver (WARP)
  and an isolated muted/offline data directory; the VM has no audio output.

Private host commands were the package verifier, AppImage identity probe, isolated
Xvfb launch and three `run-job.sh` jobs (`ebae364-windows-package`,
`ebae364-windows-installer`, `ebae364-windows-lobby`). Windows outputs are under
`/data/tbg-windows-vm/shared/out/native-ebae364`. No local Rust build, target
directory bypass, game deployment or local application reinstall was performed.

### Immutable tag publication: passed

Created unused tag `v0.1.2-rc.1` at corrected source `ebae364`, without replacing
any ref or public artifact. Tag dispatch
[41365150621](https://github.com/AlexandreZanata/TBG-Tupi-Battle-Grounds/actions/runs/41365150621)
promotes the exact successful manual run 41363044535. Its source-promotion check
passed. All six required CI lanes (quality, clippy, test, balance, wasm and
desktop-ui), the fail-closed aggregator and publication completed successfully
on this exact tag SHA. No gate was skipped to reuse tested bytes. Release
published at `2026-11-02T00:11:11Z`; the remote workflow took approximately seven
minutes including its required CI. No local full Rust suite was run.


## Public delivery: passed

The published manifest exactly matches the locally accepted complete pair. Public
GitHub asset metadata, SHA-256 sidecars, sizes and unauthenticated HTTPS HEAD 200
were verified for all four formats. The provenance bundle is published beside the
manifest. Windows remains explicitly unsigned; no publisher reputation is claimed.

- Linux tarball: `6855ce887d1a6fc5ef22d3f6b4db7230345e5006e21411fc7fd824b2817c612a`.
- Linux AppImage: `93ac1da20670c8ad28b81219eb9e8719136b97e86fbd610a093eac495fcfb9fb`.
- Windows ZIP: `be12ba79b066016b4552bcfe201395f3679b5461fcc8b97560a7d7e1ee7e61a3`.
- Windows installer: `9e96935dd039f40bed34634dfd5ff708a76c76aee413e01774539989d5b86fbf`.

On the official VPS, the installed native controller verified the real tag/source/
workflow/hosted-builder attestation with `gh`, no personal token, the adopted source
floor, all published digests and the live MatchContract. It printed:
`Published compatible native download catalog: v0.1.2-rc.1`.
A private backup of the old catalog/receipts and protected-state snapshot is retained
under `/var/lib/tbg-deploy/native-catalog-backup-0.1.2-rc.1-20261102T001242Z`.

The one-off invocation was equivalent to:

```bash
env -u GH_TOKEN -u GITHUB_TOKEN GH_CONFIG_DIR=/var/lib/tbg-deploy/gh \
  python3 /opt/tbg/deploy/native_pull.py
```

Before/after guards confirmed the same `tbg-server.service` PID **5283**, kernel
executable `/opt/tbg/bin/tbg-server`, Caddy configuration hash and all three site
pointers. All four deployment timers remain disabled and their flags absent.
No backend restart, proxy reload, database change, native TUF activation or owner's
local app reinstall occurred. The retired test VPS was untouched.

[Public metadata](https://tupibg.com/downloads.json) reports both platforms available
at 0.1.2-rc.1 with protocol 31 and the canonical server contract. The actual public
landing was checked in an isolated browser: Windows and Linux cards are available,
Linux is recommended for the detected system, both dialogs open with their two
correct public download links, version/requirements/checksum controls and beta/
visual-pending notices. Portuguese and English journeys passed. No console errors
were observed. Screenshots are retained outside Git in the owner's artifact folder.

## Explicitly deferred acceptance

This delivery proves package integrity/identity, the scoped final runtime/installer
checks, official publication and current public download journeys. It does not
complete P68 professional release or the core visual/motion matrix:

- Real Windows hardware GPU, audio output, DPI/monitor matrix and publisher signing
  were not available in the software-rendered VM; retain their P68 cases as pending.
- Exact Linux/Windows/browser pixel and motion parity, every screen/Arena/champion,
  fullscreen and complete native match/auth/reconnect scenarios on these final
  packages were not repeated. Follow `docs/ui/DESKTOP-UI-CONTRACT.md`,
  `docs/phases/P68-execution.md` and the current native acceptance report for the
  scoped follow-up. Shared source and these Lobby captures are not parity proof.
- Final-package native HTTPS authentication can reuse the existing focused harness
  `python3 scripts/check-native-auth.py --client <final-package>/tbg-client` against
  its documented isolated fixture; it was not executed for these final packages.
- Native signing/TUF custody, managed update installation and main-based automatic
  deployment remain separately gated. The future plan is not an activation receipt.
- No local full suite, balance search/benchmark or extensive gallery was run. The
  existing hosted release CI did run its mandatory workspace/balance lanes once.

Two complete Portuguese handoff prompts were provided outside Git: consolidation
of remaining PRs without losing integrated work, and real automated multi-family
rollout with measured zero deploy-induced disruption. The latter explicitly
requires closing the single-process/Ranked/admission/bootstrap gaps before any
zero-downtime claim; it does not enable those mechanisms in this delivery.
