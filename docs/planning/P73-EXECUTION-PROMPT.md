# P73 execution prompt — players, challenges, rematches and match history

Copy this prompt to the executor. Keep code, comments, docs and commits in
English; communicate with the owner in Portuguese.

---

Execute **P73 — Active players, challenges, rematches and match history**
in `docs/phases/P73-active-players-and-direct-challenges.md`. Read its
Read first list and `docs/planning/P73-PLAN-MANIFEST.json` before editing.
The owner's required history extension is
`docs/planning/P73-MATCH-HISTORY.md`, with the conceptual schema contract
in `docs/planning/P73-HISTORY-SCHEMA.sql`. Execute it as part of P73.

Create an isolated `phase/p73-active-players-challenges` branch from the
reviewed current integration base, including these planning documents.
The audit reference is `e2a18af02d809b5926b752ff50e8116fe6d9e63d`.
Coordinate with P72's shared Bevy browser/native interface before touching
common UI files. Never restore a removed HTML lobby, downgrade the roster,
reset an active worktree or overwrite another front's work.

Implement a real Active players section reached from the header's online
entry and alongside Live battles. Show **all online accounts**, with global
name search, accurate status/availability and real pagination beyond the
first 50. Guests remain counted but not publicly enumerated, as required
by the existing privacy rule. Preserve Watch and shareable Quick Challenge.

Each available player has **Challenge**. The sender explicitly sends a
targeted in-game invite; the actual recipient may Accept/Decline, and the
sender may Cancel. No automatic joining, queue/room eviction, combat
interruption or public invitation tokens. Reuse the existing authoritative
private Challenge pairing, casual rules, reconnect and result flow.

Use bounded lobby-owned directed records, random notification IDs bound to
both controllers, private WebSocket state, server-side expiry and limits.
Recheck target/source identity, connected state, true Idle state, auth,
release admission and compatibility on accept. Replays, double-clicks,
crossed invitations, disconnects and takeovers must not produce two games
or authorize another player. Preserve P71's token-auth continuity.

Add **Request rematch** for the actual latest opponent with a valid
server-issued ticket, both on Result and Active players. Extend offers to
completed online PvP Casual/Ranked/private-room games as specified. Reuse
existing both-consent RequestRematch/RematchState and the current TTL. A
rematch is a fresh private Challenge with no rating effect, even after
Ranked. Opening the players page must not accidentally send Cancel and
erase the ticket. A guest recent opponent can appear only in the private
participant card, not the public directory. Invitation records themselves
remain ephemeral and separate from the history persistence described below.

Add **Match history**, showing the player's online results and their own
available statistics, including Ranked, Casual, Challenge/rematch and
private-room PvP while account-bound. This owner extension authorizes the
history sub-spec's additive database/migration; presence/invitations remain
ephemeral.

The player chooses **Public / Only me**, persisted on the account and
private by default. Enforce that choice on every new public history route
and the old profile's `recent` field. Use server authorization, no-store
responses, per-entry ownership, current privacy revision and conditional
updates to prevent late public saves from overwriting a newer private
choice. Public views contain only the opted-in player's allowed summary;
redact private-game/guest opponent identity and never dump the other seat's
private data. Ship guards across public-serving slots before enabling the
feature, and preserve guards/settings/data during rollback.

Freeze authoritative terminal statistics, reuse server validated purchase/
Evolution/Fortification and final-state data, and send the same personal
summary to Result/reconnect and storage. Current client `damage_dealt` is
enemy-Stronghold hit amounts, not effective total damage; do not claim it
as such. Unsupported damage/healing/kill fields and absent legacy metrics
are null/unavailable, never invented zeros.

Use existing SQLite/store/journal patterns for bounded post-combat
persistence, idempotent source keys and atomic participant links. Handle
retry/crash/duplicate/conflict, P66 pending-slot handoff and pending versus
Stored feedback without blocking combat or altering Elo. Backfill only
verified account-bound Ranked records with honest partial metrics; do not
guess ownership for old non-ranked logs. Keep account privacy independent
of frozen results. Allocate the next free schema version with other fronts.

Implement indexed keyset list/detail APIs, slot-aware authenticated owner
routing through the current auth coordinator, public projections and the
shared localized history/settings UI. History is viewing only: no replay,
export, deletion, editing or Challenge/Rematch action on an old history row.
No automatic retention purge is authorized. See the sub-spec for limits,
DTOs, metrics, privacy, legacy data, rollout and H01–H18 acceptance.

Treat **global presence** honestly. The existing API is process-local and
the proxy does not aggregate it. In single-server mode reuse its snapshot;
in fixed release-slot mode add the phase's bounded background aggregation,
using existing release directory and `ureq`, without SQL per HTTP request
or a new service/Redis. Internal snapshots are loopback-only and explicitly
blocked by the public proxy. Deduplicate account names/generations and
report incomplete/stale coverage. Do not migrate an ongoing match or pair
sessions across incompatible/draining processes just to make an action work.

Implement the phase's Proposed wire/types/defaults and freeze actual
append-only discriminants and next free protocol version at S0. Document
product values in GAME-DESIGN in the same relevant commit. Do not assume
protocol 31 is unused. Keep combat, rating calculation and auth architecture
unchanged; extend only the listed invitation/rematch/presence contracts.

Build the UI on the same Bevy shell for native Linux, Windows and web.
Complete pt-BR/en and existing locale coverage, keyboard/focus behavior,
self/busy/disabled reasons, incoming/outgoing state, opt-out, 429 retry,
loading/empty/error/stale/partial states. Errors must not become fake zero
players. Respect per-session invitation preferences, bounded inbox/outbox,
IP/account/session throttles and TTL. Keep private details and credentials
out of API rows, URLs, logs and screenshots. No giant central module.

Work in phase order: compatibility/UI/schema freeze -> accurate paged global
presence -> directed invitations/rematch -> history's H0–H4 persistence,
privacy and API work -> shared localized UI -> one targeted consolidation
and evidence report. Write D01–D16 and H01–H18 regressions and actual
two-client/slot scenarios from the manifest. Validate the conceptual SQL
only on disposable fixtures during planning; no real DB migration now.

Follow the owner's minimal local validation policy. Format affected crates
and run at most one cheap cached focused check per relevant front. Queue
targeted tests for one consolidation; no automatic workspace suite, release
build, benchmark or VPS run. Every compiling command uses the common
`flock -n -E 75 "/tmp/tbg-validation-$(id -u).lock"` and
`CARGO_BUILD_JOBS=2`; busy locks/cold caches are deferred without polling,
bypass or cache deletion. Native success does not prove web compatibility.

Finish `docs/planning/P73-EXECUTION-REPORT.md` with actual commits, wire
indices, test commands/results, screenshots of populated/paged/search/error
and invite/rematch/history/visibility states, persistence/crash/migration
and privacy/race evidence, global coverage, deferred
checks and pending owner look/fun approval. Fixtures must be labeled and
must never be represented as production users. Do not call incomplete
global coverage or unrun tests verified.

Commit logical changes locally and stop for review. Current AGENTS.md
forbids pushing; no push, merge or deployment is authorized by this prompt.
If the owner later explicitly authorizes publication, open a PR targeting
`main` with the implementation/report/evidence and no automatic merge.

---
