# P73 — Active players, challenges, rematches and match history

**Status:** planning only, 2026-11-01. No runtime feature implemented.
**Planning branch:** `codex/p73-active-players-challenges-plan`.
**Execution branch:** `phase/p73-active-players-challenges`.
**Audited base:** `origin/main` at
`e2a18af02d809b5926b752ff50e8116fe6d9e63d` (P71 token-auth continuity).
**UI integration:** coordinate with P72's shared Bevy web/native interface;
freeze its reviewed integration SHA before editing common shell files.
**Source/evidence scope:** [audit manifest and acceptance cases](../planning/P73-PLAN-MANIFEST.json).
**Handoff:** [copyable execution prompt](../planning/P73-EXECUTION-PROMPT.md).
**History extension:** [results, statistics, persistence and public/private
visibility](../planning/P73-MATCH-HISTORY.md), required by the owner on
2026-11-01. It is part of P73, not a separate phase.

## 1. Player outcome and scope

From the header's online-player entry, open an **Active players** section
alongside the existing Live battles page. Browse all online accounts across
the service, find a player, click **Challenge**, and send an in-game
invitation. The recipient can accept or decline. An accepted invitation
starts a private casual Challenge match through the existing authoritative
pairing path. Keep the existing shareable Quick Challenge link working.

After an online PvP match, either participant can request a rematch against
the same opponent from the result screen or this page while the server's
rematch offer remains valid. Both must agree. A rematch is a new private
Challenge with a fresh seed and **no rating change**, including when the
previous game was Ranked.

The same phase adds the player's paged **Match history**: results and
their own available match statistics, with an account setting choosing
**Public / Only me**. Default private; enforce the preference on the
server, including the legacy public profile's recent results. The history
sub-spec defines authoritative metrics, storage/recovery, API, privacy,
legacy data and H01–H18 acceptance. Viewing old rows is read-only; it does
not create a replay or a retroactive rematch invitation.

Global means the service's active presence, not a friends list, historical
registration database, country filter or the first 50 results. Retain the
existing privacy contract: **all online accounts are browseable; guest
sessions are counted but never publicly enumerated**. Guests retain their
existing casual/link-challenge permissions and may invite a listed account.
A guest recent opponent may be shown privately to the two participants.

The supplied screenshot identifies the current Live/Accounts surface and
visual style. Its zero counts alone do not establish a backend defect.
Never fill the page with fake players or present a request failure as zero.

## 2. Read first

- `AGENTS.md`, this phase, GAME-DESIGN BR-R08/09, BR-LV06 and the Proposed
  P73 section; ARCHITECTURE security, sessions and release/admission sections.
- The required [history sub-spec](../planning/P73-MATCH-HISTORY.md) and
  [conceptual history schema](../planning/P73-HISTORY-SCHEMA.sql); apply
  its privacy/recording contracts alongside the directory work.
- P39 Quick Challenge/rematch and P47 live/presence specs as historical
  context. Follow current code and owner validation policy, not their old
  full-suite command lists.
- `docs/phases/P71-match-continuity-and-launch-readiness.md` and current
  token-auth/release contracts. P72's
  `docs/phases/P72-web-native-interface.md` on its reviewed branch if not
  yet merged. Do not restore its removed HTML/CSS lobby.
- Core: `protocol.rs`, existing wire-contract tests. Simulation is read-only.
- Server: `live.rs`, `api.rs`, `lobby.rs`,
  `lobby/{presence,challenge,rematch,matchmaking,sessions,auth,ownership}.rs`,
  `ws/client_msg.rs`, `limits.rs`, `config.rs`, `release.rs`,
  `release/api.rs`, `deploy/Caddyfile.template`.
- Client: `ui/shell/{live,data}.rs`, `ui/{actions,result,desktop_actions}.rs`,
  `sync/messages.rs`, `link.rs`, existing account/network state and i18n.
- Existing focused tests: server `live_api.rs`, `contract_challenge.rs`,
  `online.rs`; core protocol contracts. Read additional files only when
  needed for the corresponding step. Never read/edit `archive/`.

Rust paths are relative to their crate's `src/`, unless stated otherwise.

## 3. Source audit: reuse and real gaps

1. `Lobby::publish_live` already builds an in-memory snapshot about once a
   second. HTTP reads the snapshot, not live actor state or SQL per request.
2. `/api/online` already supports offset/limit (default 50, maximum 100)
   and lists account name/rating/status, ordered by rating then name.
   The native shell requests the default first page; there is no complete
   browse/search flow. Pagination is an extension, not a new directory DB.
3. `presence_status` maps InRoom and InChallenge to Idle. **Idle currently
   does not prove availability**; sending a challenge must not evict those
   players or cancel their room/queue/watch state.
4. Quick Challenge uses a single-use random invite and
   `pair(..., MatchMode::Challenge, true, Listing::Private)`. Reuse pairing,
   simulation, reconnect, result and fresh-seed behavior. Do not publish
   bearer invite tokens in directory rows or reuse a link token as an
   unbound directed invitation.
5. Rematch tickets currently originate from rematchable Challenge pairs;
   public Casual and private-room pairing pass `rematchable: false`.
   Extending rematch offers to completed online PvP is a Proposed product
   change; this phase documents it rather than claiming it already exists.
6. Rematch is session-bound, expires after the existing configured 120 s,
   and is closed by current explicit Menu/Cancel flows. Opening Active
   players must not accidentally invoke Cancel and erase its valid offer.
7. `/api/online` is process-local. Caddy's shared API upstream chooses the
   first healthy release slot; it does **not** merge slot presence. Global
   counts cannot be claimed by simply relabeling the current response.
8. P71 revalidates token authorization at pairing and protects active-match
   continuity. Direct challenges must pass the same gate; expired tokens,
   revocation and explicit takeovers must never create a new match.
9. P72 is moving the browser to the same Bevy shell as native. Extend that
   canonical UI, not a new page framework or a replacement DOM lobby.
10. The public profile currently exposes only the last 10 Ranked results,
    not a full personal history. Generic match logs do not provide safe
    account ownership; P61 summaries are optional/Ranked-oriented.
11. Result counters are client-local; current `damage_dealt` measures
    reported enemy-Stronghold hit amounts, not effective total damage.
    Never persist that as total HP damage or fabricate missing statistics.
12. Persistent history/account privacy requires an additive schema and
    terminal recording. Presence and invitations still need no new durable
    database; isolate their ephemeral state from permanent result storage.

## 4. Product journeys

### Browse

The header entry opens the existing Live surface with clear **Live battles**
and **Active players** sections/tabs. Keep Watch/Copy watch link working.
Rows show account name, rating, localized activity and Challenge action.
Use the existing profile view when opening a player's name; its history
entry obeys that account's Public/Only me setting. Do not preload the match
history of every visible row.
Provide case-insensitive name search, availability filter and pagination/
Load more. Search/filter apply to the whole server snapshot before paging,
not only to the current 50 rows. Maintain keyboard focus while refreshing.

Distinguish loading, current data, empty results, unavailable API, stale
data and incomplete global coverage. Preserve the last good rows on error,
mark them stale and disable stale actions. Show update/retry feedback and
429 backoff without flickering or repeated full redraws. Do not label
connected sessions as a count of distinct real human beings.

### Challenge

- Clicking Challenge is the sender's explicit consent to play that target.
  Show pending status and Cancel; one outgoing invitation at a time.
- The recipient sees the server-derived inviter name and guest/account
  identity, with Accept/Decline and remaining time. No automatic acceptance.
- Incoming invites are non-blocking lobby notices; they never overlay or
  pause live combat. A busy player does not receive a new playable invite.
- Incoming/outgoing invitation state is private to its participants. A
  public row must never disclose a pending opponent, room or invitation.
- Repeated clicks and crossed invitations converge on one pending pair;
  a crossed Send does not implicitly accept. The recipient must Accept.
- Sending/receiving alone does not leave a queue, private room, spectator
  session, Quick Challenge or live game. For MVP both participants must be
  truly Idle to send/accept; show a reason otherwise. Leaving another
  activity remains an explicit existing user action.
- At acceptance recheck both controllers, connections, authorization,
  availability, preferences, expiry, release admission and compatibility.
  Consume once and pair at most once. The snapshot is never authority.

### Rematch

- Issue a bounded last-opponent ticket after each completed online PvP
  game: Challenge, public Casual, Ranked and human private-room matches.
  Practice/local bots and spectators never create a ticket.
- Reuse `RequestRematch`, `RematchOpen/State/Closed`, existing TTL, fresh
  seed and Challenge mode. One latest valid ticket per participant; a
  newer game or explicit decline supersedes old offers.
- Show **Request rematch** only for the ticket's actual opponent, plus
  waiting/accept/decline states. A client-supplied name cannot create a
  rematch relationship. Ordinary directory challenges are a different action.
- A private recent-opponent card can retain a guest opponent without
  enumerating guest sessions in the public directory. Expired offers fall
  back to an ordinary Challenge action for a visible eligible account.
- Navigating from Result to Active players is a view change, not a decline.
  Explicit Leave/Decline and joining another activity preserve their
  existing cleanup semantics.
- Reconnect within the existing retained session restores remaining TTL
  and acceptance state. No persistent invite history or guaranteed recovery
  after process restart, logout or an expired session is promised.

### Match history and account visibility

Browse the authenticated player's complete retained online history using
real keyset paging, then open a result/statistics detail. Public profiles
may display only the opted-in player's allowed projection. A Public/Only me
preference is saved on the account and applied to all existing/new entries
and to older profile APIs. Private-game opponent information is redacted
in public views, and missing legacy statistics are unavailable rather than
zero. No replay/export/edit/delete or old-match challenge action is added.
Use the full history sub-spec as the implementation contract.

## 5. Presence and global aggregation

### Public directory

Extend the existing online response additively with `invitable` and coarse
`busy` status where needed; do not expose private room/challenge details.
Keep name/rating/status and existing clients' fields. Self is rendered as
You with no Challenge action. Resolve identity using the existing immutable,
case-insensitive PlayerName; the server maps it to a current controller.
No new permanent public account/session ID is needed.

Suggested request: `GET /api/online?offset=0&limit=50&q=<name>&available=1`.
Bound/validate every query, using existing paging limits and account-name
length limits. Empty `q` means all; invalid filters do not execute a new DB
query. Sort deterministically after filtering. The response carries a
snapshot revision, freshness/completeness metadata, total filtered rows
and next-page information. The client resets/deduplicates loaded pages if
the revision changes; offset pagination under churn is not a frozen history.

Do not put viewer-specific rematch rights, inbox data or authentication in
the publicly cacheable response. Those come over the existing session's
private WebSocket state. Keep cache headers explicit on success and error;
incomplete/stale/personalized responses must not be cached as healthy data.
The header's `/api/live.online` counters and `/api/online` must read the
same aggregated presence index so they cannot silently disagree by slot.
Do not leak private match rows while updating those counters; existing
live-watch routing and match visibility keep their own contracts.

### Existing one-VPS release slots

- In single-process mode, reuse the actor snapshot directly.
- With multiple live P66 slots, aggregate their **public presence
  snapshots** in a small background task using the configured release
  directory. Snapshot reads remain outside combat, lobby hot paths and
  per-HTTP-request SQL. Reuse the existing server `ureq` dependency on a
  bounded blocking worker; never run blocking HTTP on a Tokio actor task.
- A proposed loopback-only `/api/internal/presence` exposes only that
  process's snapshot, generation, freshness and admission/contract metadata
  to the aggregator. Reuse the operator endpoint's direct-peer checks:
  loopback socket and no forwarded request. Explicitly deny this route in
  the public proxy before the general `/api/*` handler. Never return
  session tokens, account IDs, auth families or invitation IDs there.
- Fixed peers come from trusted server/operator configuration and known
  release slots, never a URL supplied by the browser or returned arbitrary
  endpoints. Bound timeout, response size and concurrency; no recursive
  aggregation (peer replies contain local data only). Disable redirects
  and environment-proxy routing for this loopback client; enforce the body
  limit while reading, not after allocating an unbounded response.
- Deduplicate accounts by canonical public name; prefer the live current
  generation/controller. Counts of connected sessions retain their
  documented meaning. Ignore retired generations. A missing expected live
  slot makes coverage incomplete; retain labeled last-good data only until
  freshness expires and never fabricate global zero.
- Direct invitations may start only inside one compatible admitting lobby
  ownership domain. A player on a draining/incompatible/unreachable slot
  stays visible but temporarily unavailable. Do not migrate a live match,
  bypass P71 ownership or pair sessions living in different processes.
  Reuse existing release/admission routing for an explicit idle-client
  transition when already supported; cross-slot matchmaking/migration is
  not silently invented in this phase.
- Verify real aggregation before calling multi-slot presence global. If
  older slots cannot expose the snapshot, mark partial coverage until they
  drain or are updated; a single-slot implementation is not proof of
  multi-slot acceptance. No extra Redis/service/cluster is needed for the
  current fixed-slot VPS. Multi-host/region discovery is a later measured
  need, not this MVP's prerequisite.

## 6. Directed invitation contract

Names below are the Proposed implementation contract; freeze actual wire
indices and the next unused protocol version at S0. Do not assume version
31 is free: parallel continuity work already discusses it. Append variants
only and update compatibility/contract consumers together.

```rust
pub type PlayerInviteId = [u8; 16]; // random notification ID, not a login/bearer token

// Append to ClientMsg:
ChallengePlayer { name: String },
AnswerPlayerChallenge { id: PlayerInviteId, accept: bool },
CancelPlayerChallenge { id: PlayerInviteId },
SetPlayerChallengesEnabled { enabled: bool },

// Append to ServerMsg:
PlayerChallengeState {
    enabled: bool,
    outgoing: Option<PlayerInviteNotice>,
    incoming: Vec<PlayerInviteNotice>, // server-bound, at most 5
},
PlayerChallengeResult {
    id: Option<PlayerInviteId>,
    outcome: PlayerInviteOutcome,
},

pub struct PlayerInviteNotice {
    pub id: PlayerInviteId,
    pub peer_name: String,
    pub peer_is_guest: bool,
    pub remaining_s: u16,
}
// Outcome enum: Unavailable, RateLimited, Declined, Cancelled, Expired.
// The existing authoritative MatchStart path proves successful pairing.
```

Keep a small lobby-owned `DirectedInvite` record with random ID, sender
and recipient controller tokens, monotonic created/deadline and normalized
pair key. These tokens never leave the server. Bind operations to the
connection: only the recipient may answer, only the sender may cancel,
and neither may supply an owner/seat/account identity. A guessed/stolen
notification ID never authorizes joining; an old P39 JoinChallenge token
cannot consume this record.

Lifecycle: Pending -> accepted/pair once, Declined, Cancelled, Expired or
Unavailable. Terminal states never revive. Invalid send/answer requests
must not evict someone else's room/queue or mutate an unrelated invite.
Only after both explicit consents and all gates may the existing private
Challenge pairing path run. Reuse its actual success/failure handling;
never display Match started merely because a local record was consumed.

Send canonical private state on changes and retained-session reconnect.
New matches/queue/room/watch/link-challenge activity, logout, revocation,
takeover, final session loss or release refusal cancel conflicting directed
records without changing an active game's outcome. Explicit auth takeover
invalidates directed invitations; passive token refresh never transfers or
implicitly accepts one. A retained same-session reconnect may recover the
remaining TTL, but no match starts while either party is disconnected.
Temporary socket detach blocks acceptance without extending the deadline;
final session cleanup, rather than a brief network interruption, removes
otherwise recoverable pending state.

## 7. Abuse protection, bounded cost and privacy

Proposed MVP values are documented in GAME-DESIGN in this planning commit:
direct invite TTL 60 s, 1 outgoing per controller, 5 incoming per recipient,
15 s sender/recipient-pair cooldown and 512 live directed records per
process. These are initial tunable limits, not measured capacity claims.
Reuse the existing invite IP limit (currently 30/min) and general WS rate/
size limits; apply limits to create/answer/cancel/preference actions and
per-account identity when present, per-guest session plus IP otherwise.
Invalid/missing targets must not bypass throttles or allocate unbounded keys.
Cooldown/deduplication indexes share bounded storage and TTL cleanup; a
stream of invented target names cannot grow an unlimited auxiliary map.

The recipient can turn off direct invitations in the shared UI. MVP scope
is a retained-session preference, not a new cross-device account blocking
system. Reapply the local preference after a fresh session before enabling
incoming notices; switching it off closes pending incoming records. Ordinary
Quick Challenge links remain explicit opt-in and continue to work.
For a new controller, keep direct-invite eligibility closed until the
client has explicitly reapplied its preference; a saved opt-out must not
briefly advertise the account as invitable during startup. A retained
same-session reconnect keeps the server's existing preference.

Enforce caps and TTL on the server before UI delivery. Slow/offline clients
do not block the actor; apply existing bounded channel behavior. Never
reserve a combat seat, start a simulation or perform a SQL write for an
unaccepted notification. Keep one aggregate metrics series for outcomes
and latency; avoid high-cardinality names/invite IDs and credential logs.

For slot aggregation, Proposed bounds: at most the two configured local
slots concurrently, 750 ms peer timeout, 2 MiB response limit and 15 s
freshness cutoff. Preserve the existing 5 s client refresh baseline, with
hidden-page pause/backoff and no overlapping fetches; recipient invitations
are pushed over the session WebSocket, not by polling every player.

No emails, keys, access/refresh/resume tokens, IPs, auth-family data, room
codes, private match IDs or spectator-private data belong in a directory,
notification, capture or URL. Dynamic names are text, never HTML markup.

## 8. File boundaries and execution sequence

### S0 — Freeze compatibility and UI ownership

Record actual base/P72 SHA, protocol/ruleset/auth version and release mode.
Identify the canonical Bevy Live surface and one reviewer for overlapping
P72 shell files. Freeze new wire discriminants and Proposed product defaults.
Preserve current AGENTS/validation policy even if older branch text differs.

### S1 — Accurate directory and availability

Extend `live.rs`, `lobby/presence.rs`, `api.rs` and shared DTO parsing;
separate presence status from invitable eligibility. Implement global
snapshot aggregation in a dedicated server presence module only when slot
mode is configured, using the existing release directory/HTTP dependency.
Add internal-route proxy denial. Presence reads remain snapshot-only;
the history extension has its own additive schema and bounded indexed
account-history reads.

### S2 — Directed invites

Use `lobby/player_challenges.rs` for record/authorization/lifecycle logic;
keep link challenges in their existing module. Register bounded actions in
protocol/WS dispatch and reuse limits, auth admission and pairing. Hook
cleanup, preference and retained-session reconnect without replacing the
auth coordinator. Write focused lifecycle/race/authorization regressions.

### S3 — Rematch in Result and Active players

Extend ticket eligibility at online PvP pairing/finish, enforce one latest
ticket per participant, and reuse rematch consent/TTL/seed/Challenge mode.
Expose only private ticket state to the two participants. Add a view-only
Result-to-players navigation that does not send Cancel. Protect legacy
decline/new-activity behavior and guests' private recent-opponent card.

### S4 — Shared frontend and localization

Implement the history sub-spec's H0–H4 backend/privacy work before enabling
history in the shared frontend; its H5/H6 work joins S4/S5 here. Allocate
one coordinated schema/wire version and do not run a second consolidation
campaign just because history is documented in a separate sub-spec.

Extend `ui/shell/live.rs` or split a cohesive `ui/shell/players.rs` module;
put invitation UI/state in its own client module. Reuse UiAction/network/
message routing, current dialogs, toasts and result controls. Avoid a god
file combining fetches, presence, invitations and matching logic.

Implement search, real paging, status/disabled reasons, self state,
incoming/outgoing cards, opt-out and rematch/private recent opponent.
Add Match history list/details and account Public/Only me control from the
history sub-spec. Result/reconnect and persisted details share one bounded
authoritative terminal summary; no separate client-inferred history totals.
Preserve visual parity with native Linux and web after P72; Windows uses
the same frontend assets/layout. Add all keys to existing language catalogs,
at least complete pt-BR/en and consistent existing locale coverage. No
hardcoded new English statuses or raw API messages in the Portuguese UI.
Keyboard controls, focus restoration and live notice announcements must
work without covering combat or repeatedly stealing focus on refresh.

### S5 — Targeted consolidation and review evidence

Run only the deduplicated affected checks/scenarios in section 9, with
the current shared host lock. Capture actual shared web/native screens:
populated list, second page/search, empty result, unavailable/partial state,
incoming invite, pending/cancel/decline, rematch and busy player.
Use explicit local fixtures/test identities where needed; label them as
fixtures and never claim they are production users. Preserve source
screenshots separately from compact review images; no secrets or raw
large captures/binaries in git.

Finish `docs/planning/P73-EXECUTION-REPORT.md` with source versions, wire
indices, files changed, commands/results, screenshots, exact deferred
checks, global coverage, history persistence/privacy/migration evidence
and owner look/fun review. No deploy, VPS mutation,
push, automatic merge or production readiness claim is authorized here.

## 9. Acceptance and validation

The machine-readable manifest defines D01–D16 and H01–H18 scenarios.
Required results:

- Every listed online account is reachable through real paging/search;
  no invisible 51st row, duplicate accounts or guest-session disclosure.
- Room/challenge/queue/watch/live-match state is not treated as invitable;
  both source and target are revalidated atomically at accept.
- Correct participant alone can answer/cancel; repeated, crossed, replayed
  and expired requests start zero or exactly one match, never two.
- Link challenges, private rooms, casual/ranked queues and P71 auth/resume
  remain intact. Direct games/rematches are private and never change Elo.
- Result and list rematch agree on the actual same-opponent ticket;
  navigation preserves consent state while explicit decline still closes it.
- Global slot aggregation deduplicates current generations, bounds errors,
  reports partial coverage honestly and never disrupts a draining match.
- pt-BR/en and shared desktop UI show real loading/error/empty states;
  the same frontend behavior is inspected on native Linux and web.
- History uses authoritative terminal data, persists idempotently through
  normal failures/restart and is linked only to frozen account participants.
- Public/Only me is a durable server-enforced choice; private history never
  leaks through old `recent`, detail/cursor, cache, proxy or old-slot routes.
- Legacy Ranked backfill is verified and resumable; unavailable stats are
  explicit, no non-ranked ownership is guessed, no automatic purge occurs.

**Planning now:** inspect docs/links, validate JSON/source paths and
`git diff --check`. No game build, online tests or deploy.

**Implementation fronts:** formatting and at most one cheap cached focused
check for the affected crate/target. Queue meaningful protocol, invitation,
presence, privacy and rematch regressions; don't run a suite per step.
All Rust/Trunk compilation uses:

```bash
flock -n -E 75 "/tmp/tbg-validation-$(id -u).lock" \
  env CARGO_BUILD_JOBS=2 cargo check -p tbg-server --lib
```

Choose the actual relevant package/target, not every crate by habit. Lock
busy/cold cache means deferred, without polling, bypass, cleaning or fresh
target directories. Preserve other fronts' builds and sessions.

**One targeted consolidation:** queue the existing narrow integration
targets and new tests, then run the deduplicated set once under that lock:

- `cargo test -p tbg-server --test live_api -- --test-threads=1`
- `cargo test -p tbg-server --test contract_challenge -- --test-threads=1`
- New Proposed `--test player_challenges` target for D03–D13.
- New Proposed `--test presence_global` target for D01/D02/D14/D15.
- New Proposed `--test player_match_history` and `--test history_privacy`
  targets for H01–H18, including reconnect, duplicates, migration and
  visibility races. Validate conceptual DDL in disposable SQLite only.
- Affected exact core protocol test targets and client parser/state tests,
  named in the executor's report after their actual paths are frozen.
- Focused two-client create/accept/decline/cancel/reconnect/rematch and
  busy-match observations on an isolated local server; one two-slot
  local scenario if global slot support is implemented.

Contract tests are append-only under AGENTS.md; do not weaken old assertions
or repurpose a previously private field to make a new feature pass. A
native check is not browser evidence. Missing multi-slot/web/device checks
remain deferred acceptance, not a pass. No workspace suite, clippy campaign,
release packaging, stress benchmark, balance lab, VPS test or deployment
without an explicit owner request for that scope.

## Planning report

Prepared from repository inspection and the supplied screenshot only.
No directory feature, invitation, rematch extension, snapshot federation,
history persistence, privacy feature, UI approval or live validation is
claimed as implemented by these docs.

Initial directory-only planning validation completed: `python3 -m json.tool
docs/planning/P73-PLAN-MANIFEST.json` parsed successfully. A one-off Python
audit checked 37 baseline source revisions/SHA-256 hashes, two new relative
links, D01–D16 uniqueness/pending status, BR-AP01–08 and the proposed client
wire names against this phase. `git diff --check` passed. Runtime scenarios,
Cargo/Trunk checks, browser/native captures, slot testing and VPS work were
not run because this delivery changes documentation only.

History extension planning validation: the expanded manifest has 68
verified baseline source paths/revisions/hashes, seven new relative links,
16 Proposed BR-AP rules and 34 D/H acceptance scenarios still pending
runtime execution. The conceptual SQL passed disposable in-memory SQLite
checks for default privacy, 65-match keyset traversal with time ties,
owner/public query predicates, conditional privacy revisions, constraints,
atomic rollback and the intended paging index. This is not a production
migration, API authorization, journal recovery or browser/native test.
