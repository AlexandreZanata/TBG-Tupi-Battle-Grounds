# P65 — Threat model, verification matrix and operating boundary

> Companion to [P65](../phases/P65-community-admin-foundation.md).
> Planning evidence only, not a security assessment of a running deployment.

## 1. Assets, attackers and boundaries

Protect game account keys, operator credentials, HTTP sessions, private
reports/tickets/notes, news publication integrity, moderation evidence,
availability of match/result processing and the meaning of ranked records.

Assume hostile guests, authenticated spammers, players targeting other
accounts, stolen/expired sessions, a compromised low-privilege operator,
untrusted content and repeated/reordered requests. Include accidental staff
actions, duplicate network retries, DB/disk failure and backup restore.
A root/host compromise defeats local DB/audit confidentiality; do not market
same-host SQLite separation as a sandbox or an immutable audit ledger.

Boundaries: public network → public read/player HTTP handlers; private
TLS/tunnel → operator listener; server identity resolution → permission
and object policy → bounded service → community DB transaction/audit.
Existing game account verification is a narrow dependency, not access to
match control. Community code never receives a mutable simulation handle.
Keep new credentials out of `Config` Debug and request/proxy logs.

## 2. Mandatory negative and recovery cases

| ID | Scenario | Required result |
|---|---|---|
| S01 | Guest/player/each operator role calls every privileged route directly, including alternate methods | Deny unless action, role, object and state all permit; hidden buttons are irrelevant |
| S02 | Change account/actor/role/owner/assignee/internal-note fields or another user's ticket/post/report ID | Ownership and visibility checked server-side; forged mutable fields rejected; private existence not leaked |
| S03 | Moderator attempts publish/role grant; Publisher attempts ticket access; Support attempts game ban/Elo update | Explicit denial; no implicit all-powerful staff flag or arbitrary command endpoint |
| S04 | Foreign Origin, missing/wrong CSRF, HTML form/simple content type, login CSRF, credentialed cross-origin fetch | No state change or session swap; exact-origin/CSRF/content-type rules hold including login/bootstrap/logout |
| S05 | Fixate/reuse a session; use old token after logout/role revoke/key rotation/idle/absolute expiry | Invalidated server-side; cookie flags and rotation verified; no resurrection on restart or restore |
| S06 | New community login for account currently playing a ranked match | Independent HTTP identity established; game seat/token and current match remain unchanged |
| S07 | Markup/script/event handlers, hostile URL text, Unicode/control characters, SQL syntax in every text field | Safe plain-text output in player and admin views, parameterized SQL, bounded/validated inputs; no fetch/eval/HTML parsing |
| S08 | Read pending/hidden/withdrawn content via lists, details, parent snippets, locale fallback, pagination, counts or cache | No unauthorized content leak; existing recipient copies of previously published content cannot be recalled and are not claimed revocable |
| S09 | Edit/approve/hide/restore/reply races, stale revision, duplicate retries, same idempotency key/different payload | Atomic valid transition or explicit conflict; one effect and audit event; no self-approval or posting to locked parent |
| S10 | Public/session/private DTOs and logs inspected with seeded secret canaries | No keys/cookies/internal notes/reporter IDs/SQL traces in public output or logs; safe stable error code/request ID only |
| S11 | Rate burst, large JSON, many IDs, many principals/IPs, queue full, request timeout/disconnect | Bounded memory/DB work, admission control and expiry; explicit 413/429/503 policy; no detached unbounded jobs |
| S12 | Audit insert failure, unavailable account/role DB, disk full, corrupt new DB or timeout | Privileged write fails closed and transaction rolls back; no falsely acknowledged action; match/result processing preserved |
| S13 | Existing/new DB migration twice, restart mid-action, isolated consistent backup/restore | Schema/idempotency correct, constraints/indexes valid; no gameplay DB damage; restored sessions invalidated before ingress |
| S14 | Reporter changes evidence; staff hides/restores; closed ticket expires; hold/redaction and restore cleanup run | Authorized revision evidence preserved only under policy; content/notes visibility separated; retention/tombstones survive restore |
| S15 | Disable feature flags and individual kill switches; spoof proxy headers or public Host/admin URL | Admin unreachable from public router/static fallback; revoked sessions unusable; disabled writes deny; trusted proxy rules cannot be bypassed |
| S16 | Moderator restriction expires/appeal accepted while another request is writing | Current restriction state governs transaction; community-only scope; no unintended match queue ban or account takeover |
| S17 | pt-BR/English console and future-consumer fixture, empty/loading/error/expired session | Localized UI/error labels, preserved content locale, safe text and accessible controls; authored user prose not auto-translated |

For S01–S03 enumerate a test row per registered route/action and role plus
owner/non-owner/assigned/unassigned object contexts. Name deliberate public
routes explicitly; default-deny middleware is not enough if a handler uses
an unsafe response projection. A route inventory missing a newly added
handler is a security gap, not a test pass.

Tests use temporary files, synthetic accounts and isolated HTTP listeners.
Use a controllable server clock in tests for expiry without long sleeps.
Test deadlines and bounded queue behavior deterministically instead of
launching a production stress test. Capture method/status/body, unchanged
DB state on denial and audit counts on success; never store actual secrets
as test snapshots. A 200 response and a screenshot are insufficient evidence.

## 3. Operator/problem-resolution runbooks to deliver

1. Bootstrap/rotate/revoke a named operator through local maintenance,
   verify all its sessions are invalidated and record a safe audit entry.
2. Draft/preview/publish/withdraw a news revision and confirm no draft leak.
3. Triage report, inspect permitted evidence, hide/restore/lock, apply/expire
   a reasoned community restriction, then handle appeal with an audit trail.
4. Assign ticket, distinguish owner reply from internal note, resolve/reopen,
   redact an inadvertently pasted credential without promising account-key
   recovery. Escalate real account compromise to the owner, not fake reset.
5. Enter read-only mode on abuse/disk pressure, preserve game/result traffic,
   revoke sessions, contain exposure and review logs without dumping secrets.
6. Consistent backup and isolated restore, invalidating all HTTP sessions,
   reapplying redactions/restrictions and verifying account references before
   any public/admin ingress. Do not automatically restore production data.

News/public community origin and private admin origin must be configured
explicitly. The operator web surface never becomes public merely because
the console was implemented or a generic proxy wildcard forwards `/api/*`.
Setup without approved TLS/private ingress stays disabled. SSH/VPN access
requires actual host credentials; assume neither that it exists nor that
it provides MFA without evidence.

## 4. Scope and future expansion gates

Deferred by design: full in-game news/forum UX, direct messages/chat,
attachments/media, rich text/Markdown, remote previews/webhooks, third-party
analytics/identity providers, public admin, automated gameplay punishments,
rating repairs, credential recovery and native authenticated authoring UI.
Each adds new trust/data paths and needs a scoped review before activation.
The foundation must still be executable/testable through its APIs and
restricted console; a collection of unused tables is not completion.

Keep optional public writes off until moderation staffing, queue limits,
retention and policy are approved. Internet-facing admin requires a separate
authentication/MFA and exposure decision. Third-party paid infrastructure
requires owner authorization; it is unnecessary for the proposed MVP.

## 5. Primary references checked during planning

Security direction follows primary OWASP guidance, reviewed 2026-10-31.
These support specific principles, not a claim of certification or a complete
ASVS audit. The concrete architecture/limits are this project's proposals.

- Authorize each request and default deny:
  [OWASP Authorization](https://cheatsheetseries.owasp.org/cheatsheets/Authorization_Cheat_Sheet.html).
- Opaque sessions, cookie protection and server-enforced expiry:
  [OWASP Session Management](https://cheatsheetseries.owasp.org/cheatsheets/Session_Management_Cheat_Sheet.html).
- Session-bound request tokens/origin validation alongside cookie policy:
  [OWASP CSRF Prevention](https://cheatsheetseries.owasp.org/cheatsheets/Cross-Site_Request_Forgery_Prevention_Cheat_Sheet.html).
- Ordinary text nodes for initial user-authored content:
  [OWASP XSS Prevention](https://cheatsheetseries.owasp.org/cheatsheets/Cross_Site_Scripting_Prevention_Cheat_Sheet.html).
- Audit useful security actions without logging authentication secrets:
  [OWASP Logging](https://cheatsheetseries.owasp.org/cheatsheets/Logging_Cheat_Sheet.html).
- TLS, bounded input and per-endpoint enforcement:
  [OWASP REST Security](https://cheatsheetseries.owasp.org/cheatsheets/REST_Security_Cheat_Sheet.html).

## 6. Planning report

Source inspection: router/config/logging, account parsing/hash verification,
WS/lobby sessions, existing SQLite schema/worker, static lobby news and
deploy proxy/backup paths. No live service, private player data or production
security tests were accessed. No implementation or infrastructure was changed.
Documentation checks and source links are verified before publication.
All S01–S17 executions, owner choices and release gates remain deferred to
the implementation; they cannot be reported as passed from this document.

Planning checks passed: `git diff --check`, four local document links,
balanced fenced blocks, shell-example syntax through `bash -n` without
execution, all S01–S17 cases and O01–O03 decisions present. Six primary OWASP
reference links were inventoried; guidance was browsed during planning.
The final staged diff is checked separately before publication. No Cargo,
security test, account creation, production probe or release was executed.
