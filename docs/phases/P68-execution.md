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

Status: Step 0/base audit complete; authentication implementation pending.
No new build, release, vault migration or installed application change yet.

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
