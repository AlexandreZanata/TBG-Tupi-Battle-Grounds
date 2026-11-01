# Native beta publication and one-off landing download refresh

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

Manual native build run
[41361235748](https://github.com/AlexandreZanata/TBG-Tupi-Battle-Grounds/actions/runs/41361235748)
builds both OSes from that exact main source. Use the existing tag-dispatch
publication and its exact-source manual-run promotion gate; never relabel a PR
merge artifact, manually fabricate provenance or change source during promotion.
Public packages remain unsigned beta candidates with visual acceptance pending.

## Verification in progress

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

Windows package identity, inventory, real-vault cleanup, GUI/installer checks,
publication provenance and public landing validation must be recorded before
claiming their outcomes. No complete pixel, GPU/audio/DPI or multiplayer matrix
is inferred from a build or one screenshot. Existing Linux playtest evidence
for another source is not the final package's acceptance.

## Production scope

The reviewed installed native catalog scripts match this source. The official
host has current `gh` attestation verification tooling and a verified ancestry
floor. After a complete compatible attested native publication, invoke only
`native_pull.py` once and verify `/downloads.json` plus both existing platform
dialogs. Preserve the old catalog until validated replacement, the game process,
TLS/proxy configuration, site asset pointers, databases and all timers/flags.

This publication does not install a native package on the VPS, activate TUF,
restart the backend, deploy landing/admin HTML or enable general automatic rollout.
Only the existing download metadata is eligible to change.

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

Add repository UTF-8/LF checkout attributes and a native-input check before either
OS compiles. Keep binary assets verbatim and the read-only historical archive
excluded. Retain all raw package identity/promotion checks. No gameplay rule,
number or wire version changes: Windows must produce the existing Linux contract.
Three tests reproduce autocrlf without attributes, prove canonical text/binary/
historical-fixture bytes with attributes, and reject CRLF/non-UTF-8 inputs.

Local targeted checks passed: 3 input scenarios, the live source-input guard,
4 native-promotion scenarios, 7 package/workflow scenarios, 9 required-check
scenarios, native/CI actionlint and diff checks. No Rust compilation or broad
local suite was needed. A new exact-source manual native run will build the
corrected branch (main plus documentation/checkout fix); native beta publication
is already allowed independently of a main merge, with the same tag/provenance
and compatibility gates. Automatic main deployment remains only planned.
