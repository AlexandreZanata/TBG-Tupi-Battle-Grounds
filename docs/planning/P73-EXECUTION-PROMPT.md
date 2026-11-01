# P73 execution prompt — global active players and direct challenges

Copy this prompt to the executor. Keep code, comments, docs and commits in
English; communicate with the owner in Portuguese.

---

Execute **P73 — Global active players, direct challenges and rematches**
in `docs/phases/P73-active-players-and-direct-challenges.md`. Read its
Read first list and `docs/planning/P73-PLAN-MANIFEST.json` before editing.

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
participant card, not the public directory. No new persistent history DB.

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

Work in phase order: compatibility/UI freeze -> accurate paged global
presence -> directed invitation lifecycle -> rematch integration -> shared
localized UI -> one targeted consolidation and evidence report. Write the
D01–D16 regressions and actual two-client/slot scenarios from the manifest.

Follow the owner's minimal local validation policy. Format affected crates
and run at most one cheap cached focused check per relevant front. Queue
targeted tests for one consolidation; no automatic workspace suite, release
build, benchmark or VPS run. Every compiling command uses the common
`flock -n -E 75 "/tmp/tbg-validation-$(id -u).lock"` and
`CARGO_BUILD_JOBS=2`; busy locks/cold caches are deferred without polling,
bypass or cache deletion. Native success does not prove web compatibility.

Finish `docs/planning/P73-EXECUTION-REPORT.md` with actual commits, wire
indices, test commands/results, screenshots of populated/paged/search/error
and invite/rematch states, privacy/race evidence, global coverage, deferred
checks and pending owner look/fun approval. Fixtures must be labeled and
must never be represented as production users. Do not call incomplete
global coverage or unrun tests verified.

Commit logical changes locally and stop for review. Current AGENTS.md
forbids pushing; no push, merge or deployment is authorized by this prompt.
If the owner later explicitly authorizes publication, open a PR targeting
`main` with the implementation/report/evidence and no automatic merge.

---
