# P65 — Ready-to-send implementation prompt

```text
Implement P65 — secure initial news, community and administration foundation
for Tupi Battle Grounds. At completion push the pertinent changes and open
or update a PR targeting main. Communicate with the owner in Portuguese.

Repository: https://github.com/AlexandreZanata/TBG-Tupi-Battle-Grounds
Planning branch: codex/p65-community-admin-foundation-plan
Read current AGENTS.md and docs/VALIDATION.md, then:
- docs/phases/P65-community-admin-foundation.md
- docs/planning/P65-SECURITY-CONTRACTS.md
- the read-first references and D-84/GAME-DESIGN §18.

Start from current main in an isolated worktree on
codex/p65-community-admin-foundation. Incorporate the needed planning
documents without overwriting newer work. Reconcile actual schema, account
auth, proxy/router and P60/P61/P64 integration before coding. Preserve all
existing work and do not merge unrelated fronts blindly.

Implement the steps in order: threat/policy/route inventory; separate bounded
persistence/audit; player/operator identities and secure sessions; text-only
news/revisioned premoderated community APIs; reports/private support cases;
minimal restricted operator console; recovery/security consolidation.
The polished in-game news page/full forum is a later phase.

Security is an acceptance requirement. Admin has separate named credentials,
least-privilege grants and a private listener/origin, never inferred from a
game account or client flag. Local CLI bootstrap/grant/revoke is required;
no shared default admin key, public admin ingress or secret in Config logs.
Player HTTP login verifies existing keys without taking over the game seat.
Enforce session revocation/expiry, cookie flags, CSRF/origin checks and
object ownership on every request. UI hiding is not authorization.

Use plain text only, no HTML/Markdown/uploads/remote fetch. Keep public
published data separate from drafts/reports/ticket bodies/internal notes.
Atomic state change plus audit, optimistic revision checks and idempotency
must hold under races/retries. Preserve match/rating processing; moderation
only restricts community writes and cannot change Elo, grant game bans,
reset account keys or run arbitrary SQL/shell.

Complete S01-S17 with synthetic isolated tests and retain explicit negative
permission cases per endpoint/role/object state. Deliver backup/restore,
retention/redaction, revocation and incident runbooks. Fail closed on DB,
audit, policy or auth failure; bound queues, input, pages and abuse state.
Keep public-write/admin activation flags off until required decisions and
security gates pass. Do not claim a future forum is vulnerability-free.

Resolve O01-O03 concretely with recommended defaults. Routine engineering
choices are autonomous; do not silently approve product/retention/exposure
decisions. Continue independent work while necessary owner input is pending.
No real credentials, production migrations, activation, live moderation,
host provisioning, paid service or deploy is authorized by this execution.

Follow minimal per-front validation and deduplicated focused consolidation,
shared nonblocking build lock and at most two jobs. Do not run the full
suite, broad benchmarks or production attacks. Deferred security checks
remain blockers, not passes. Code/comments/docs/commits are English;
EN/pt-BR product labels use the existing i18n system.

Explicit GitHub authorization: normal push of this branch and open/update
its PR with base main, overriding AGENTS.md's general no-push rule for this
delivery. No force-push, history rewrite, merge or deploy. Include only
pertinent code/tests/docs and never secrets/builds/other fronts' changes.
If mandatory security or owner decisions remain unresolved, deliver a Draft
PR with feature flags off and precise blockers. PR description records
architecture, threat/permission matrix, migrations, tests actually run,
deferred gates, residual risks and recovery/rollback. Finish with its URL,
delivered scope and actual validation status. Begin now with Step 0.
```
