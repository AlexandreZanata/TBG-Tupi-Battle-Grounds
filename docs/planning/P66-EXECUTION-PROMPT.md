# P66 — Ready-to-send executor prompt

```text
Implement P66 — native Linux/Windows updater and match-preserving releases
for Tupi Battle Grounds. Deliver pertinent code/tests/docs in a PR to main.
Communicate in Portuguese; code/comments/docs/commits remain English.

Repository: https://github.com/AlexandreZanata/TBG-Tupi-Battle-Grounds
Planning branch: codex/p66-native-updates-release-safety-plan
Read current AGENTS.md/VALIDATION policy, then:
- docs/phases/P66-native-updates-and-release-safety.md
- docs/planning/P66-RELEASE-CONTRACTS.md
- the phase read-first/decisions/canon documents.

Use current main and an isolated worktree on
codex/p66-native-updates-release-safety. Incorporate planning docs without
overwriting newer code. Audit the actual native Linux branch, install mode,
version/protocol, account/result schema and proxy before implementation.
Preserve active installations and unrelated fronts. Native snapshots and
planning branches are not automatically merged dependencies.

Deliver the P66 §8 milestones as separate branches and PRs, M1 first:
M1 Linux self-update and notes (contracts/trust/journal, stable launcher and
transactional updater, startup modal/idle guard/resume/post-update notes,
test-key static repository); M2 backend release safety (computed contract
identity, structured admission, durable results, truthful retirement) can
run in parallel; M3 two-slot rollout and scheduled contract switch after M2;
M4 Windows adapter on real Windows. Each milestone has its own exit gate.

Use a maintained TUF implementation; no custom crypto or unsigned checksum
trust. Stage full immutable releases, verify before execution, atomically
activate with recoverable journal, preserve user data and compatible fallback.
Support explicit install modes. Don't overwrite a running Windows executable
or system-managed package. After healthy activation show exact installed-build
notes, including after crash/reopen, until durable user acknowledgement.

No routine update modal, restart, heavy extraction or automatic bandwidth
usage during online play/reconnect. Protect the Update/Play race. Native
resume credentials require protected persistence and original-generation
reconnect before update; current in-memory session storage is insufficient.

Backend releases pin each match's generation/wire/rules/content contract.
Two processes need shared durable account ownership/fencing and idempotent
results, not separate process-local lobbies. Preserve reconnect, takeover,
rooms and results. Old servers retire only after all matches/result work and
protected delivery/ownership are settled; no 300/330-second forced cutoff.
Avoid routine Caddy reload and mutable/deleted assets for open browser tabs.
Rollback new admissions without killing matches on either generation.

Do not claim arbitrary crash failover, zero possible interruption or proven
thousands-player capacity. Measure overlap headroom and queue/tick/view
guardrails; defer expensive capacity runs until explicitly authorized.
Use safe drain with a declared admission pause until full coexistence passes.

Resolve O01-O05 when needed. Routine engineering choices are autonomous;
owner decisions on keys/certificates, install support, emergency policy,
capacity/retention or paid resources cannot be silently approved. Continue
independent work while needed answers are pending.

Follow minimal per-front validation and one deduplicated targeted
consolidation, shared nonblocking host lock and max two compile jobs.
Run U01-U22 in isolated synthetic fixtures as scope/cache permit. Record
actual commands and deferred gates honestly. No full suite, broad packaging,
large benchmark or production scenario is automatically authorized. Missing
Windows hardware/test evidence keeps Windows pending, not verified by Linux.

No real installation, OS changes, key generation/production signing, public
artifact release, infrastructure provisioning or deployment is authorized.
The actual game installation and production services must remain untouched.

Explicit GitHub authorization: normal push of each milestone branch and
open/update its PR with base main, overriding the general AGENTS no-push
rule for this delivery. No force-push, history rewrite, merge or deploy. Include
only pertinent code/tests/docs, never secrets, build outputs or other fronts.
If mandatory trust/recovery/compatibility/OS gates remain blocked, deliver a
Draft PR with affected activation disabled and precise pending work.
PR description covers behavior, migrations/admission, trust, recovery,
tests actually run, deferred gates, measurements and owner decisions.
Finish with PR URL, Linux/Windows/backend status and actual verification.
Start now at Step 0; do not stop after restating a plan.
```
