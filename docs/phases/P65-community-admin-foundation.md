# P65 — Secure news, community and administration foundation

> **Planning only — 2026-10-31.** Owner request: an initial news/community
> module and basic administration, moderation and issue resolution, before
> building the full in-game news page and community forum. No implementation,
> deployment, account provisioning or security certification is performed here.
> Product limits/policies are **Proposed** in GAME-DESIGN §18; security
> requirements below are release gates, not optional UI suggestions.

Planning branch: `codex/p65-community-admin-foundation-plan`.
Suggested execution branch: `codex/p65-community-admin-foundation`.
Parent: P64 plan `ed0eb0c1bf585365259bbfa80dd301c6f002aaf3`.

## 1. Read first and audit baseline

Read [AGENTS.md](../../AGENTS.md), current `docs/VALIDATION.md` if present,
[security contracts](../planning/P65-SECURITY-CONTRACTS.md),
[execution prompt](../planning/P65-EXECUTION-PROMPT.md), GAME-DESIGN §18,
ARCHITECTURE §21, DECISIONS D-84, and the relevant parts of P64 for i18n.
Read existing core account parsing, server Store/account/lobby/config/router
and deploy configuration only as needed. Do not touch `archive/`.

Main-derived source audited at `9008f117c4289122ddff978c5d25720d2bd79687`:

- `tbg-server/src/lib.rs`: one public Axum router with WS/read-only APIs
  and optional static fallback. Store failure currently allows the game to
  run with `store=None`; new authenticated writes must instead fail closed.
- `config.rs` derives Debug and `main.rs` logs Config. Never add plaintext
  credentials/session keys to that debug output or example env files.
- `store/accounts.rs`, `tbg-core/src/account.rs`, `lobby/accounts.rs`:
  random account keys, hashed verification and WS account sessions exist.
  Game account authentication is not administrator authorization.
- `store/schema.rs` is at v6 on this base, with one SQLite worker in
  `store/worker.rs`. P60/P61 and native fronts may change this. Reconcile
  the actual integration base instead of reserving global v7 here.
- `api.rs` currently serves public ranking/profile/live data. It has no
  authenticated community object-ownership contract or admin RBAC module.
- Client `index.html`, `assets/lobby/lobby.js` and `ui/shell/lobby.rs` use
  three catalog-backed static news entries, not a publishing service.
- `deploy/Caddyfile.template` proxies game/API routes and serves static files;
  `deploy/backup.sh` exists. Neither proves isolation/backups for a new DB.

This is a scoped source audit, not a penetration test or proof that current
deployment is secure. Freeze the real SHA/configuration at Step 0.

## 2. Smallest useful initial delivery

Deliver one server-side foundation and a small restricted web operator
console. No generic CMS, external SaaS, plugin marketplace or new backend
framework. Keep the authoritative match engine outside the module.

1. **News:** create/edit drafts, EN/pt-BR revisions, preview, publish,
   unpublish and list only approved published content through a public API.
2. **Community:** categories, topics and replies as plain text, authenticated
   ownership, premoderation, hide/restore/lock, reporting and scoped write
   restrictions. Implement/test APIs now; keep public authoring disabled
   by default until owner activation. Full in-game forum UI is a later phase.
3. **Issues/support:** private account-owned tickets and replies, staff
   assignment, internal notes, status/resolution and reopen/appeal workflow.
   The initial submission interface is a minimal authenticated web form or
   API test client; no full in-game support portal is required.
4. **Admin:** named operators, separate credentials/sessions, granular
   permissions, audit trail and a usable restricted console for the above.
5. **Operations:** feature flags, abuse limits, safe failure, backup/restore,
   revocation, retention and a focused security regression suite.

P65 does not build the polished news/forum navigation inside Bevy. Future
pages consume these versioned APIs and safe text components. Preserve the
existing lobby news as the offline/default fallback; do not silently point
every installed client at a new API before it is deployed and approved.

## 3. Trust and authentication boundary

### 3.1 Separate operator and player identities

Create server-resolved `PlayerPrincipal` and `OperatorPrincipal`. Never
accept account ID, role, ownership or staff status from JSON, URL/query,
player name, a client flag, WS seat, or a forwarded header from an untrusted
peer. An operator identity is not inferred from an account named admin.

**Recommended initial admin exposure:** dedicated loopback/private listener
and private origin, accessed through an authenticated SSH/VPN path with TLS
at the browser-facing endpoint. Do not mount admin handlers on the public
router, public static fallback or community origin. No public admin ingress
in this phase. Private transport complements, and never replaces, app auth.
Production activation requires an owner-reviewed reachable-origin/proxy
configuration. Internet-facing admin plus MFA/identity-provider integration
is a future explicit decision, not an accidental bind-address change.

Bootstrap named operators through a local maintenance CLI on the host.
Use individual OS-random 32-byte credentials, store SHA-256 digests only,
compare in constant time using the existing reviewed helper where suitable,
and disclose each credential only once through an explicit secure interactive
bootstrap channel. No shared global API key, default password, first-user
admin or self-service role grant. Never use game account keys for admin.
This hashing choice is for high-entropy random secrets, not human passwords.
If human passwords are introduced, redesign hashing/MFA with established
libraries rather than applying fast SHA-256 to passwords.

The CLI manages operator grants/revocation, rotates credentials and records
operator/maintenance identity in audit. It does not provide arbitrary SQL
or shell execution from the web console. Do not pass secrets in process
arguments, URLs, committed fixtures or debug-printable Config.

### 3.2 Player community session

Recommended MVP bridge: a same-origin TLS `POST /api/community/session`
accepts the existing account key in a size-limited body, verifies through
the established key parser/hash verifier, and issues an independent opaque
HTTP session. No new password database and no WS token in URLs. Share the
verification primitive, not the lobby takeover/login side effects: community
login must not terminate or replace the current game seat/session.

The browser sends the key only on explicit login, never each content request.
Do not add additional localStorage copies or telemetry of it. This inherits
the existing account-key compromise risk; new session revocation cannot
magically revoke a stolen permanent game key. Record that limitation and
do not claim account recovery or key rotation exists if it does not.
Native future pages need a reviewed authenticated transport/session adapter;
do not smuggle cookies through CLI arguments or store admin keys in the game.

Use separate host-only `__Host-tbg-community` and `__Host-tbg-admin` cookies
on their respective origins: Secure, HttpOnly, Path=/, no Domain, SameSite=Strict.
Store session-token hashes server-side, with server-enforced idle/absolute
expiry, revocation, rotation on login/credential changes and bounded counts.
No JWT framework is needed. Credentialless development exceptions are not
permitted; explicitly loopback-only local test configuration may terminate
TLS differently, but release cookie/origin checks remain mandatory.

Protect all cookie-authenticated mutations with a synchronizer CSRF token
bound to the session, exact configured Origin checks and JSON Content-Type.
Login/bootstrap-CSRF-token requests also reject foreign/missing unexpected
browser origins; define/test the unauthenticated login bootstrap flow.
No state changes on GET; SameSite alone is not the CSRF design. CORS denies
credentialed cross-origin access by default. Never trust arbitrary Host or
X-Forwarded-* values when deriving the expected origin.

### 3.3 Permissions and revocation

All handlers check role, action, object and state server-side on every
request. Default deny, including store/role/audit lookup failures. Read
revocation/restriction state from the authority per mutation; stale cached
claims must not permit a revoked operator to finish a new action.

| Principal | Allowed | Explicitly forbidden |
|---|---|---|
| Guest | Published news and approved public community reads | Writes, reports, tickets, drafts, admin |
| Player | Own pending submissions/edits, reports, own tickets; public reads | Other private tickets/reports/notes, self-approval, role changes |
| Editor | News drafts/revisions/preview | Publish without Publisher grant; moderation/private support |
| Publisher | Publish/unpublish validated news revisions | Role assignment, gameplay/rating edits |
| Moderator | Review/approve/hide/restore/lock content, reports, reversible community restrictions | Secrets, ticket bodies without Support grant, game bans/Elo changes |
| Support | Assigned/authorized support queue, ticket replies/notes/resolution | Credential recovery, arbitrary account data, moderation without grant |
| Owner operator | Explicit assigned app capabilities; operator grant/revoke through local maintenance | Raw SQL/shell/export-secrets from web; overriding server outcomes |

Operators can hold several grants; no implicit hierarchy gives Moderator
Publisher or Support rights. Global unauthenticated role listing does not
exist. Sensitive role/credential maintenance requires fresh local operator
verification; console publication/restriction actions require recent reauth.
Emergency CLI revocation invalidates current sessions, not just future logins.

## 4. Data and workflows

### 4.1 News, forum and localization

Use stable IDs separate from slugs and translated display names. News has
draft/revision/published/withdrawn states; published revision pointers change
atomically. Edits to published text create a new draft and never leak before
publication. Locale revisions are explicit; missing pt-BR returns a documented
fallback marker or unavailable state, not a fake Portuguese translation.
P64 translates UI/codes; authored community content is not auto-translated.

Forum topics/replies start pending. Only approved visible revisions appear
in public lists/detail/counts; pending/hidden text must not leak through
search, pagination, preview, cache or parent summaries. Editing an approved
post creates pending revision while retaining the last approved version.
Hidden authors may see their own submission status but cannot unhide it.
Locks prevent new replies. A reply cannot move to an unrelated/hidden topic.

All initial bodies are **plain text**. No HTML, Markdown rendering, embeds,
uploads, avatars, remote images, link preview fetch, webhooks or attachments.
Use textContent/text widgets in both operator and eventual game UIs, including
titles, filenames-as-text and report evidence. No active links from user text
by default. This eliminates a large initial markup/file/SSRF surface, but
does not replace authorization or input/output checks.

### 4.2 Reports, moderation and problem resolution

Reports reference existing visible objects or a supported match reference
the reporter is permitted to disclose; reasons use stable codes plus bounded
plain-text context. Deduplicate active reports by reporter/target; do not
publish reporter identity. Capture the referenced content revision privately
so later edits cannot erase evidence, subject to retention/access policy.

Moderation cases: open → triaged → actioned/dismissed → closed, with reason,
actor, timestamp and object version. Hide/restore is reversible, restrictions
are scoped to **community posting**, with reason and expiry; no automatic
punishment based on report count. Never implement a game ban, Elo reversal,
match deletion or recovery of secret keys as a generic moderation action.
Refer competitive disputes to P60 if its reviewed workflow is integrated.

Support tickets: open → in_progress → awaiting_player → resolved → closed;
owner can reopen for review. Separate public-to-owner replies from internal
staff notes in both schema and response projection. Assignment is explicit,
with allowed unassigned queue access. Reporter/owner cannot choose assignee,
priority, actor, resolution or internal visibility via mass assignment.
Provide resolution codes/explanation and appeal/reopen, not a button marked
“fix” that mutates arbitrary game data. Do not request account keys in ticket
text; warn users against sharing them and support restricted redaction.

### 4.3 Persistence and transaction boundary

Recommended: separate SQLite `community.sqlite3`, its own bounded worker,
schema history, indexes and backup. This prevents forum load from filling
the existing match-result writer queue; it is not host/process isolation.
Account IDs are verified against the existing account store via a narrow
bounded call; cross-database REFERENCES do not enforce those relationships.
Handle missing/deleted accounts explicitly and fail closed on auth failure.

Minimal tables grouped by responsibility: sessions/operators/grants;
news/revisions/locales; categories/topics/posts/revisions; reports/cases;
tickets/messages/assignments; community restrictions; audit records;
idempotency records. Avoid a generic unrestricted JSON admin-data table.
Use parameterized SQL, foreign keys within this DB and migration transactions.
Reserve version 1 only for this **new DB**; inspect main/P60/P61 schema before
any account-store change. No destructive migration or production execution.

Each privileged mutation and its audit event commit in one transaction.
Audit captures actor ID/action/object/revision/reason/result/request ID and
safe before/after metadata. It excludes cookies, keys, auth headers and full
ticket/message bodies. Audit failure aborts the action. Do not call an
application-append-only table tamper-proof: a compromised host/database owner
can alter it; backups/access control and later external audit can reduce risk.

Use expected revision/version for edits/transitions (409 conflict), and
bounded actor/action-scoped idempotency keys for retried creates/publish/
restrictions. Same key with a different request body conflicts. Recheck
authorization/object state in the write transaction to avoid TOCTOU races.
No unbounded reads, account scans or community work inside match tick loops.

## 5. API and operator console

Concrete route groups, with DTOs/auth policy finalized at Step 0:

- Public: `GET /api/news`, `GET /api/news/{id}`, and gated
  `GET /api/community/topics[/{id}]`/topic replies. Only published/approved
  projections; bounded cursor paging, explicit locale and revision.
- Player session: POST login/logout; GET session/CSRF bootstrap. Session,
  private content and error bodies use `Cache-Control: no-store`.
- Player writes: gated POST topics/replies, versioned own-edit route,
  POST reports, POST tickets, GET own ticket(s), POST own ticket replies/reopen.
- Private admin origin: session routes and capability-gated news draft/
  preview/publish/withdraw, moderation queue/cases/actions, ticket assignment/
  reply/internal-note/resolve and sanitized operational status.
- Maintenance CLI: bootstrap/rotate/revoke/grants, session revocation,
  migration/backup verification commands. No public role-creation endpoint.

All object handlers enforce eligibility; knowing an ID is not permission.
For private objects use a consistent not-found projection to avoid existence
leaks; reserve 401 for no session and 403 for known general capability denial.
Reject unknown mutable fields, unsupported methods/types, conflicting versions,
oversize or malformed payloads; use stable localized error codes/request IDs,
not SQL traces or reflected raw backend errors. P64 handles their presentation.

Do not apply the existing broad public-API cache policy to auth/admin data.
Published news caching varies by locale/revision and is invalidated on withdraw.
Initial forum/report/support reads are no-store to avoid stale moderation leaks.
Use JSON content type, nosniff, restrictive CSP, no framing and safe referrer
policy on operator pages. New admin assets are outside public static fallback.
No secrets enter public news payloads or client bundles.

The small console must actually support sign-in/logout, effective grants,
news draft/preview/publication, pending moderation, case reasons/actions,
ticket assignment/replies/private notes/resolution and audit inspection.
UI hiding is convenience, never enforcement. Keyboard/mouse and EN/pt-BR
labels are required; do not build a Bevy admin interface or full forum here.

## 6. Limits, failure handling and operations

Proposed starting limits, to confirm at O02 and test at boundaries:

- 64 KiB encoded request body cap; title 160 Unicode scalars and 640 UTF-8
  bytes; news text 16 KiB, post 8 KiB, ticket/report reply 4 KiB UTF-8.
- Pages default 20/max 50; no eager unbounded counts/history expansion.
- Community writes 5/min/account plus 20/hour/IP; reports/tickets 5/hour/
  account; login 5/min/IP plus 20/hour/principal. Apply limits before costly
  work and after identity resolution; no permanent victim-account lockout.
- Player sessions: 30-minute idle/24-hour absolute; operators 15-minute
  idle/8-hour absolute; sensitive admin action needs auth within 5 minutes.
- At most 3 active sessions/principal/type; role/key changes revoke all
  affected sessions. Sessions expire server-side and cleanup is bounded.
- New store queue 128 pending operations, bounded admission/deadlines and
  2-second DB busy deadline. Queue exhaustion returns retryable 503/Retry-After;
  no unlimited detached tasks or silent successful writes.
- Proposed retention: private closed-ticket/report bodies 30 days, security
  audit metadata 180 days, operator-accessible backups 30 days. Legal/product
  retention decisions remain O02; no jurisdictional compliance claim. Define
  tombstones/redaction and reviewed evidence holds with expiry, including
  deletion reapplication after backup restore. Do not launch collection
  before an approved retention/cleanup procedure exists.

Rate-limit maps and pending objects themselves need bounded storage and
expiry; otherwise rate limiting becomes a memory/disk exhaustion vector.
Per-account active-content quotas and a DB/disk-watermark write cutoff are
set from measured host capacity at Step 0, documented and tested before
activation. On quota/disk/audit/auth-store failure disable writes, preserve
existing published reads and game traffic, and expose sanitized local health.
Private endpoints remain unauthorized/unavailable, never anonymous fallback.

Feature flags default off: news publishing, public community writes and
private admin listener. Enable each deliberately after review. Independent
kill switches allow read-only mode, admin session revocation and community
write suspension without resetting ranked matches or removing evidence.
No module error may make the base game accept an unauthenticated admin call.

Back up the new DB consistently via SQLite backup/checkpoint-aware procedure;
do not copy a live main file while forgetting WAL. Test restore in an isolated
directory, account-reference consistency and reapplication of revocation/
redaction. Restoring any backup invalidates all restored operator/player
HTTP sessions before ingress is opened, including tokens minted after backup.
File/backup permissions are owner/service-only; secrets never enter git.
Document rotation/revocation/containment and audit of access to private cases.

## 7. Execution order and gates

**Step 0 — decisions/threat model.** Freeze actual base, routes, proxy trust,
account verification semantics, paths/versioning, quotas and dependencies.
Resolve O01 access/bootstrap, O02 limits/retention, O03 moderation/ownership.
Record endpoint × capability × object/state matrix before implementation.

**Step 1 — isolated persistence and fail-closed services.** Add new DB worker,
migrations/indexes, DTO validation, bounded queues, transactions, idempotency,
audit and feature flags. Test temporary DB upgrade/rollback/restart behavior.

**Step 2 — identities/sessions/policy.** Implement named operator CLI,
player-verification bridge without game takeover, separate cookies, CSRF,
expiry/revocation/reauth and every-request permissions. Prove negative cases
before creating privileged console actions. New dependency needs a one-line
Cargo reason; reuse current Axum/SQLite/getrandom/sha2 where appropriate.

**Step 3 — news and community primitives.** Implement revisioned text-only
publishing, public projections, pending topics/replies and ownership.
Public authoring remains off. No remote fetch/upload/HTML shortcuts.

**Step 4 — moderation and support.** Implement reports/cases, reversible
community-only restrictions, private tickets/notes/assignment/resolution and
reopen. Prove object privacy and actor-controlled-field rejection.

**Step 5 — minimal console and operations.** Build restricted functional web
console with safe rendering/i18n and usable actions. Add backup/restore,
retention cleanup, health, kill switch and incident runbooks. No production
bootstrap, secret rotation or live moderation in this implementation task.

**Step 6 — security consolidation and PR.** Run the focused abuse/permission/
transaction suite once on the integrated tree, review all routes, and record
outstanding risks/owner decisions. Publish code/tests/docs in a PR to main
under the explicit authorization in the executor prompt; no merge/deploy.
Incomplete security gates keep the PR Draft and feature flags off.

## 8. Verification and release criteria

Run the cases in the security companion: cross-role/object matrix, forged
identity, CSRF/login CSRF, XSS, SQL injection, leakage, replay, races,
revocation, exhaustion, DB failure, migration/restore and game isolation.
Required evidence is behavioral tests, not just linting or an admin screenshot.

Planned server integration targets to create: `community_auth`,
`community_policy`, `community_content`, `community_support` and
`community_recovery`. Use a separate focused JS test for console rendering/
CSRF/session expiry. Existing account/reconnect/result fixtures are narrowed
to the touched authentication boundaries. No live-server attacks or load
experiments against production are authorized.

Future consolidation example (targets do not exist yet):

```bash
flock -n -E 75 "/tmp/tbg-validation-$(id -u).lock" \
  env CARGO_BUILD_JOBS=2 cargo test -p tbg-server \
  --test community_auth --test community_policy --test community_content \
  --test community_support --test community_recovery
```

Per front follow the minimal policy: format and one cheap scoped cached
check if available; queue targeted tests for consolidation. Every compile
uses the shared nonblocking lock and at most two jobs. Busy lock/cold cache
means defer, not wait/bypass. No automatic full suite, balance gates,
benchmarks, broad browser galleries or packaging. Security-critical deferred
checks block release even if independent documentation is complete.

Done: usable backend/console, all required denial/recovery cases evidenced,
no gameplay authority change, reversible operations, private-content privacy,
tested backups, i18n labels and complete endpoint policy inventory. Owner
sign-off on access, moderation/retention and console flow precedes activation.
This is not a guarantee of a vulnerability-free future forum: new rich text,
uploads, native auth, public admin or remote integrations need a fresh review.

## 9. Report and owner decisions

- **O01 recommended:** private admin origin/listener, separate operator
  credentials and local CLI bootstrap. Select actual TLS/tunnel setup before
  activation; no paid identity service or public admin exposure by default.
- **O02:** approve timeouts/quotas/retention and capacity evidence, including
  redaction and backup-restore behavior. Published content and private
  evidence have different lifecycles; do not retain secrets forever.
- **O03 recommended:** premoderated text-only forum, anonymous public reads,
  account-only writes/reports/tickets, reversible community restrictions.
  Approve moderation reasons/appeal and staff access assignments.

Executor records: SHA, threat/policy matrix, decisions, migration versions,
dependencies, route/DTO inventory, actual tests/results, deferred gates,
restore/expiry/revocation evidence, limits, known residual risks and PR URL.
No full-suite or secure-production claim from this planning branch.
