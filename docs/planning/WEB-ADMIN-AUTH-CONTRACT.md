# Web admin authentication and authorization contract — Proposed, inactive

Status: **Proposed, nothing implemented on the server.** Written with the web admin
foundation (2026-11-01, [phase report](../phases/WEB-landing-admin-foundation.md)). It
records what a real administrative console needs before `admin.tupibg.com` may show any data
or perform any action. Until then the public host serves only a static "in preparation"
page and the working console exists only as a local preview with labelled fixtures.

Related: D-89 (ranked corrections are dry-run CLI plans, *no web admin*), D-97/D-99 (player
Access/refresh, slot-routed auth), and the unmerged P65 planning branch
`codex/p65-community-admin-foundation-plan` (`docs/planning/P65-SECURITY-CONTRACTS.md`,
negative cases S01–S17), which this contract follows rather than repeats.

## 1. What exists today (verified in source, base `8be248e`)

| Area | Fact | Consequence for the console |
|------|------|-----------------------------|
| Identity | Only player `Account`s exist (`PlayerName` + `AccountKey`, D-97 Access/refresh). | No operator principal exists. A player account is never an administrator. |
| Routes | `crates/tbg-server/src/lib.rs` registers player auth, public reads (`/health`, `/api/version`, leaderboard, players, live, online, patch notes, admission), `/ws*`, loopback-only `/metrics` and `/api/release`. | There is no administrative HTTP route to call. |
| Browser auth | Refresh is a `Secure`/`HttpOnly`/`SameSite=Strict` host cookie on the app host with exact-origin checks (`TBG_AUTH_ORIGINS`). | The admin origin must not receive, share or widen that cookie or its CORS/origin rules. |
| Operations | `tbg-release-admin` and `tbg-ranked-admin` run as the `tbg` user on the host; `--actor` is a label, not authentication. | Release and ranked operations stay host CLIs; the console does not wrap them. |
| Proxy | `/api/release` answers 404 publicly; `/metrics` is loopback-only. | Never proxy either from any web host. |

## 2. Rules for every phase (non-negotiable)

1. No client-side authorization: hidden routes, disabled buttons and missing links protect
   nothing. The server decides every read and write per request.
2. No login form, password, token, JWT or secret in JavaScript, HTML, `localStorage` or
   `sessionStorage`. No hard-coded credentials anywhere.
3. Never reuse the player cookie, Access token or `AccountKey` as an admin credential, and
   never relax CORS, `SameSite`, `HttpOnly`, CSRF, TLS or Origin checks to connect origins.
4. No administrative route on the public router, the static fallback or a generic
   `/api/*` proxy; admin handlers live behind their own listener and explicit routes.
5. The console never touches the simulation, `Energy`, `Shop`, `Evolution`, `Synergy`,
   seats or outcomes. Balance numbers change only through D-33 evidence and a reviewed
   repository change; ranked corrections stay the D-89 CLI plan/apply flow.
6. Every privileged write needs a reason and produces an audit record in the same
   transaction; if the audit write fails, the action fails.

## 3. Proposed contract

**Principals.** `OperatorPrincipal` distinct from `PlayerPrincipal`, resolved only on the
server. Operators are bootstrapped, rotated and revoked through a local maintenance CLI on
the host (as in P65 §3.1): individual 32-byte OS-random credentials shown once, stored as
digests, compared in constant time. Human passwords or an identity provider with MFA are a
separate owner decision (O-W1) and would need a reviewed library, not ad-hoc hashing.

**Exposure, in two steps.**
- *Step A (first activation):* a dedicated loopback listener (for example
  `127.0.0.1:8090`) reached through an authenticated SSH tunnel; the public admin host keeps
  the static placeholder.
- *Step B (owner decision O-W2):* internet-facing console on `admin.tupibg.com` only with
  MFA, rate limits and an explicit route list proxied to the admin listener (for example
  `/admin-api/*`), never `/api/*` or `/ws`.

**Session.** Opaque CSPRNG token, server-side store of its digest, cookie
`__Host-tbg_admin` (`Secure; HttpOnly; SameSite=Strict; Path=/`, host-only on the admin
origin). Proposed lifetimes from P65: 15-minute idle, 8-hour absolute, re-authentication
within 5 minutes for sensitive actions, at most 3 sessions per operator. Rotation on login
and privilege change; logout, role change, credential rotation and emergency CLI revocation
invalidate live sessions on the server. Responses carrying session or private data use
`Cache-Control: no-store`.

**Requests.** Every state change requires an exact `Origin` match with the configured admin
origin, a session-bound CSRF token header and `Content-Type: application/json`; no CORS
headers are emitted. Bodies are size-limited and validated; unknown fields are rejected;
errors return stable codes and a request id, never stack or SQL traces.

**Authorization.** Default deny. Capabilities per action and object, no implicit hierarchy:

| Capability | Covers | Explicitly not |
|------------|--------|----------------|
| `content.draft` / `content.publish` | News and notice drafts, preview, publish/withdraw | Patch notes (stay repository-reviewed), gameplay values |
| `balance.view` | Read balance-lab reports and pending proposals | Any write of ruleset values |
| `moderation.review` / `moderation.restrict` | Reports, reversible community restrictions with reasons | Game bans, rating changes, match outcomes |
| `matches.read` | Sanitized match and player summaries | Account keys, tokens, IP addresses |
| `audit.read` | The audit trail | Editing or deleting audit entries |
| (CLI only) operator grants | Grant/revoke/rotate operators | Any web endpoint |

**Audit.** Append-only records with operator id, capability, action, object, reason,
before/after digests, time and request id. Retention is an owner/legal decision (O-W3).

## 4. Mapping to the UI foundation

The preview console (`web/admin/app`) already separates the layers this contract needs:
`assets/js/api/http.js` (same-origin, no credentials), `api/public-status.js` (read-only
`/health` and `/api/version`), `api/admin.js` (`ADMIN_AVAILABLE = false`; every request throws
`AdminUnavailable`) and `data/fixtures.js` (fictional records, all `Demo…`/`DEMO-…`). Gated
buttons explain why they are unavailable instead of pretending to work. Activating real
reads means implementing the server side above, then replacing fixture reads one area at a
time behind `api/admin.js`; no UI change may bypass that boundary.

## 5. Activation gates

1. Server implementation with route inventory and the P65 S01–S17 negative cases adapted to
   these capabilities, executed and reported (a 200 response or a screenshot is not
   evidence).
2. Owner decisions: O-W1 credential/MFA method, O-W2 exposure step, O-W3 retention, the
   named operator list, and legal review of the privacy text.
3. Separate deployment review of the admin listener and any proxy route; until then
   `web/deploy` publishes only the static placeholder for the admin host.
