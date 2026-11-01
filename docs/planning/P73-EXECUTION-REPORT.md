# P73 execution report

Status: implementation delivered for **Draft review**, not accepted or release-ready.
No deployment, VPS access, production database migration or production flag change.
All validation below is targeted. Deferred checks are not passes.

## Frozen integration and commits

- Initial GitHub `main` / fetched `origin/main`: `e2a18af02d809b5926b752ff50e8116fe6d9e63d`.
- Shared P72 interface: `2da0a35df87f29fc9cd346eaab9032b47d96af86`, integrated without rewriting its history.
- Planning: `8d7df1bb372bd0a5bba292c4feaffe9e96bfceb7`, merged contextually by `b48e689`.
- Authority, persistence, privacy, wire and regressions: `4e3ca16` (D-101).
- Shared Bevy frontend and explicit presentation fixtures: `2b50e68`.
- Evidence/report: `089c681`; normal merge of current main: `f49de90` (no source changes).
- Worktree: `/tmp/tbg-p73-implementation`; branch: `phase/p73-active-players-challenges`.
- No existing local P73 implementation branch/worktree or open P73 PR was found before work.
- The owner's original working tree and its uncommitted files were preserved.
- Main advanced to the reviewed P72 merge `0e23a1ac2f601b182bcfcc03b25acce76e69f962` during execution. Its source was already integrated through the P72 parent; it is merged normally before publication.

Base protocol **30**, schema **10**. This integration allocates protocol **31**,
additive schema **11**, personal-summary format **1**. D-100 was already used by
P71; the P73 decision is D-101. Authentication, simulation, ruleset and Elo
calculation remain the existing shared authorities.

## Implemented behavior

The existing Bevy Live surface now includes all online accounts, whole-set
search/availability filtering before real pagination, stable identity-bound
actions, status/rating, existing full-profile links and permission-aware history.
Guests count as connections and are never enumerated. Queues, rooms, spectating,
matches, detached controllers, disabled recipients and outstanding challenges
are distinguished from invitation eligibility. Loading, empty, failed,
last-good/stale and partial coverage have separate states. Live watch/share
controls are retained.

Single-process reads reuse the actor snapshot. Fixed release slots aggregate
bounded loopback snapshots in the existing release cadence, without SQL per
presence request. Account generations deduplicate, counters and rows come from
the same aggregate, and remote/incompatible/draining targets are unavailable.
There are at most two fixed peers, a 750 ms peer timeout, 2 MiB response cap and
15 s last-good window. Forwarded/public access to the internal route is denied;
the Caddy template blocks it explicitly. No match is migrated or distributed
matchmaker introduced.

Directed invitations are authenticated by the connection, random-ID located and
participant bound, with explicit accept/decline/cancel. Repeated/crossed sends
do not imply consent. Accept rechecks both participants and consumes one record
before existing private, unrated Challenge pairing. TTL 60 s, one outbox, five
inbox entries, 15 s pair cooldown, 512 process records and bounded session/account/IP
rate logs apply. Retained detach preserves only the original deadline; takeover,
revocation, logout and new activity clear invitations. Receive opt-out is
independent of history visibility; an opted-out sender can still explicitly send.

Latest-opponent server rematch tickets now cover all online PvP pairing modes.
Both parties must consent within the existing expiry. The new match is private
Challenge with a fresh seed and no rating settlement. Result-to-players/history
navigation does not send Cancel. No history row can manufacture a rematch.

A single authoritative personal summary is frozen at termination for Result,
CompletedResult replay and stored participant detail. It includes successful
purchases and champion distribution, evolution tiers, successful Evolutions and
fortifications, resolved Augments, terminal Stronghold values, combat ticks,
wall duration, mode/origin/arena/outcome/reason and source versions. The existing
collector is reused for all online modes. Missing metrics remain nullable;
there is no damage/healing/kill attribution fabricated from client hit counters.

The existing SQLite worker installs schema 11 only with explicit migration.
Immutable parent envelopes and independent participant IDs commit atomically;
replays verify canonical bytes, conflicting duplicates never overwrite. A separate
per-slot fsynced journal, 64-entry queue, 32 KiB envelope, 32 MiB directory and
bounded retries decouple history from combat and Elo. Pending/Stored/Delayed are
distinct, and Stored requires a successful database acknowledgment. Outstanding
or quarantined journal work blocks slot retirement. Pre-journal failures are
explicit coverage gaps, not a durability guarantee. No purge is implemented.

Verified Ranked account links backfill in bounded 100-row checkpoints. An
optional `history_expected` marker in the existing Ranked transaction prevents
partial backfill from preempting a modern pending complete envelope. The marker
is not part of Elo. Legacy journals default false, and late old-slot Ranked rows
are included incrementally. Old non-ranked names/IPs are never ownership evidence.

Owner/public list/detail APIs use indexed keyset paging (default 20, maximum 50),
subject/filter/privacy-revision-bound bounded cursors and one-side projections.
Private/guest opponent identities are redacted publicly. Account visibility is
private for both existing and new accounts, persisted with conditional If-Match
revisions. Stale public saves return 409. Legacy profile `recent` is guarded too.
Reads reuse the Access coordinator, Origin/channel/slot policy, enforce current
bindings at queued SQL execution, and never accept an expired match-seat lease.
Four actual queued reads maximum, 2 s HTTP timeout, 32 KiB detail cap and no-store
responses apply. Current effective Ranked deltas come from existing settlement
and correction data, with no second rating calculation.

Shared pt-BR/en views cover private/public owner settings, list/detail/back/filter,
public preview, pending save, unavailable/legacy metrics, 401/refresh, 409 and 429.
The two supported catalogs have identical keys/placeholders. Account epochs clear
personal UI state and discard stale requests. Existing keyboard actions retain
stable names/IDs, focus and scroll; presence updates do not open combat modals.
Native timestamps use the already locked Jiff library for the system zone;
browser timestamps use locale-aware numeric parts and explicit offsets.

## Real validation performed

Every Rust/Trunk compilation used the shared **nonblocking** host lock and at most
two Cargo jobs. Tests used one test thread. The existing common target cache was
reused; no fresh target directory, cargo clean, workspace suite, clippy, release,
balance or VPS validation was run.

The wrapper was:

```sh
flock -n -E 75 "/tmp/tbg-validation-$(id -u).lock" \
  env CARGO_BUILD_JOBS=2 \
  CARGO_TARGET_DIR=/home/iiii/PESSOAL-PROJETOS-ALEXANDRE/TBG-Tupi-Battle-Grounds/target \
  cargo <targeted arguments below>
```

Successful scoped runs before the final edits (66 distinct tests across these
scopes; reruns after relevant changes are not counted twice):

- `check --offline -p tbg-core --lib`: exit 0.
- `check --offline -p tbg-server --lib`: exit 0 after resolving implementation errors.
- `test --offline -p tbg-core --test contract_protocol --test contract_protocol_server -- --test-threads=1`: 15 + 2 passed, 0 failed. Existing discriminant/byte contracts remain; appended P73 variants round-trip.
- `test --offline -p tbg-server --lib lobby::player_challenges::tests -- --test-threads=1`: 5 passed, 0 failed at that revision.
- `test --offline -p tbg-server --lib match_task::summary::tests -- --test-threads=1`: 5 passed, 0 failed.
- `test --offline -p tbg-server --test live_api --test contract_challenge --test history_privacy --test player_challenges --test player_match_history --test presence_global -- --test-threads=1`: respectively 3, 2, 3, 2, 5, 4 passed, 0 failed.
- `test --offline -p tbg-server --test ranked_admin --test ranked_recovery --test ranked_stats --test ranked_stats_recovery -- --test-threads=1`: respectively 4, 4, 6, 2 passed, 0 failed. Existing non-history fixtures only supply the new journal hint as false.
- `test --offline -p tbg-client --bin tbg-client p73_ -- --test-threads=1`: 4 passed, 0 failed.
- `check --offline -p tbg-client --bin tbg-client --target wasm32-unknown-unknown`: exit 0 before final edits. This proves compilation at that revision, not browser operation.
- `build --offline -p tbg-client --bin tbg-client --features ui-acceptance`: exit 0 before final edits; executable used for native evidence below.

Native compilation initially reported missing ALSA pkg-config metadata. The
matching official `libasound2-dev_1.2.11-1ubuntu0.3_amd64.deb` was extracted only
under `/tmp/tbg-p73-native-deps`, pointing to the installed ALSA runtime. No system
package was installed. Native commands additionally set
`PKG_CONFIG_PATH=/tmp/tbg-p73-native-deps/usr/lib/x86_64-linux-gnu/pkgconfig`.
The existing unrelated `visuals/layout.rs::clear` dead-code warning remains.

Real network tests launch disposable servers and actual WebSocket/HTTP clients:
consent, private pairing, terminal result, identical persisted personal detail,
Saved acknowledgment, authenticated result reconnect and replay, latest rematch,
fresh seed, cross-user denial, expired Access denial, privacy revision conflict
and journal-directory failure with Delayed result were exercised. A separate
actual two-slot fixture uses fixed loopback peers, shared disposable SQLite,
aggregate counts/page, internal forwarded denial, drain and missing peer states.
The 137-account pre-pagination case and 65 tied-result keyset/index case use
explicit data fixtures; they are not claims of production capacity.

Final static checks on the committed source:

- `cargo fmt -p tbg-core -p tbg-client -p tbg-server --check`: exit 0; no compilation.
- `node --input-type=module --check < crates/tbg-client/assets/auth/history.js`: exit 0.
- Both JSON catalogs: 1543 keys match, placeholder parity passes.
- Installed additive SQL applied to a disposable in-memory predecessor: existing account default private/revision 0 and intended account/time index verified. Application migration/backfill is separately covered by the targeted tests above.
- Manifest JSON parsing and new Rust module responsibility comments checked.
- `git diff --check` / staged equivalent: exit 0. No archive edits, credentials, databases, raw screenshots or build products are committed. Seven compact review WebPs total approximately 400 KiB.

## Blocked and deferred validation

> blocked: The final targeted consolidation command returned **exit 75**, with
> no output: the shared validation lock was busy. No compilation/test from that
> batch ran. It was not retried, polled or bypassed.

Changes after the last successful relevant run include receive-opt-out versus
explicit-send eligibility, retention on a detached accept, pending-target gates,
backfill concurrent-writer conflict handling/no-op update avoidance, fresh-guest
probe and pending-send readiness, opt-out reapplication, countdown refresh,
fixture origin and pt-BR wording. A sixth directed-invitation regression was
written and remains unrun. Earlier passes do **not** verify these final changes.

Queue these exact scoped checks for the designated reviewer, once, when the host
lock is available (same wrapper above and native pkg-config setting where needed):

```sh
cargo test --offline -p tbg-server --lib lobby::player_challenges::tests -- --test-threads=1
cargo test --offline -p tbg-server --test player_challenges --test player_match_history -- --test-threads=1
cargo test --offline -p tbg-client --bin tbg-client p73_ -- --test-threads=1
cargo check --offline -p tbg-client --bin tbg-client --target wasm32-unknown-unknown
cargo build --offline -p tbg-client --bin tbg-client --features ui-acceptance
```

> blocked: `trunk build --features ui-acceptance` entered cold Bevy WASM codegen.
> Only this task's Cargo child was interrupted with SIGINT; no other build was
> stopped. The browser bundle and browser screenshots remain deferred under the
> cold-cache policy. A native build is not browser evidence.

Caddy is unavailable locally, so template syntax/runtime, mixed-version public
route fallback/encoded-path behavior and real proxy headers require a disposable
privacy-aware Caddy/slot setup. The latest allowlisted routing change received
static review only. Do not activate public history before this gate.

Additional remaining acceptance: visible browser/native keyboard and focus
interaction; search/page-two and full loading/error/stale/partial captures; real
GUI account preference changes across devices; every PvP source-mode matrix;
invite takeover/revocation/expiry races over real transports; process-crash and
OS disk-full fault boundaries; drain/handoff operations. Unit/API/socket coverage
below is partial evidence, not replacement for those scenarios. Owner look/fun
and desktop-device acceptance remain pending. No phone/touch validation was done.

## D01–D16 evidence ledger

All cases remain **partially evidenced / acceptance pending**, including final
lock-busy edits. This ledger supersedes historical `not_run` planning entries.

- **D01:** 137-row fixture verifies global search/filter before page; native connected directory capture. UI page-two/search interaction and changing-revision capture pending.
- **D02:** actor eligibility and busy/opt-out tests plus snapshot projection. Complete status matrix over transport pending; final readiness edits unrun.
- **D03:** real two-client consent, private Challenge result and one-shot pairing test passed before final edits.
- **D04:** expiry/decline/cancel actor tests passed; final detached accept retention revision unrun.
- **D05:** third-party/self protections in actor tests; copied-ID HTTP denial and connection-bound messages exercised. Full casing/takeover transport matrix pending.
- **D06:** crossed/repeated sends and replayed accept actor tests passed; final pending-target gate unrun.
- **D07:** busy/auth-expiry/opt-out/disconnect actor scenarios and release-gated snapshots covered. Full concurrent release/admission race pending.
- **D08:** retained-deadline actor test and real authenticated completed-result reconnect passed. Invite-specific passive refresh/takeover/revocation transport matrix pending.
- **D09:** existing `contract_challenge` and `live_api` targets passed. Complete room/queue/watch interaction matrix deferred.
- **D10:** real Challenge result -> two-party fresh-seed rematch passed. Casual/Ranked/private-room source-mode matrix pending.
- **D11:** view-only Result navigation inspected; native rematch/result fixtures captured. Interactive offer retention/decline focus check pending.
- **D12:** internal forwarded route denial tested; public data shapes inspected and privacy API tested. Actual proxy boundary validation pending.
- **D13:** bounded record/rate/expiry tests passed, including per-account replacement and disabling receive despite rate refusal. Cap-pressure/slow-peer transport matrix pending.
- **D14:** actual two-slot aggregation and stopped-peer partial/stale test passed; generation/dedup fixtures passed.
- **D15:** draining/incompatible rows unavailable in tests; no remote pairing/migration path added. Full ongoing-match mixed-generation acceptance pending.
- **D16:** shared native build, WASM check and catalog parity passed at earlier revision; seven native captures supplied. Final source/browser/keyboard/desktop acceptance pending.

## H01–H18 evidence ledger

All cases remain **partially evidenced / acceptance pending**.

- **H01:** real account Challenge and rematch persistence, frozen participant data and no pre-combat entry path inspected. Complete mode/guest/spectator/late-login matrix pending.
- **H02:** real and disposable-db private defaults, authenticated preference commit and revision tested. GUI multi-device/reconnect preference acceptance pending.
- **H03:** owner/private/public list/detail, cross-account entry denial, subject-bound cursor and legacy `recent` guard covered. Real mixed-version proxy gate pending.
- **H04:** five authoritative collector tests and identical result/detail transport summary passed. Additional combined dead-champion/resolved-Augment/fortification GUI scenario pending.
- **H05:** nullable legacy backfill verified; new UI uses no hit-counter damage attribution. Unknown-version presentation inspected, full visual state matrix pending.
- **H06:** real result -> Stored -> authenticated completed reconnect replay -> HTTP detail equality passed. Final client source requires recheck.
- **H07:** atomic failed-side rollback, canonical replay, stable participant IDs and conflict refusal passed in disposable DB tests.
- **H08:** after-commit journal replay and invalid/oversize retained file tested. Process-kill boundaries and every pre-journal failure not exhaustively injected; pre-journal coverage gaps are explicit.
- **H09:** actual journal-directory failure returns Delayed without changing result; retry/bounds/pending retirement code inspected. OS disk-full/read-only/queue-saturation/drain handoff pending.
- **H10:** additive v10 migration, verified partial Ranked backfill and late legacy versus modern expected envelope tests passed. Final concurrent-writer handling unrun.
- **H11:** 65 tied-time rows, head insertion, subject/filter cursor refusal and query-index plan passed.
- **H12:** public one-side projection, hidden private/guest opponent and absent other-side private stats tested.
- **H13:** stale public If-Match returns conflict after private revision; new public reads deny and no-store asserted. Mixed-slot/cache runtime acceptance pending.
- **H14:** real owner Access/slot, expired token refusal and existing auth binding reuse tested. Browser Origin/refresh/duplicate-header UI matrix pending.
- **H15:** application migration/backfill tested only on disposable DB; rollback routing is implemented but Caddy/mixed-version runtime not verified.
- **H16:** existing Ranked settlement/recovery/statistics targets passed; effective-delta read reuses ledger/corrections. Combined correction-and-history HTTP scenario pending.
- **H17:** shared owner/list/detail/settings and result native presentation fixtures inspected; client privacy-state regressions passed. Browser, real GUI saves and keyboard/state matrix pending.
- **H18:** bounded SQL/journal/cursor validation and oversized retained-journal tests passed; no automatic purge or payload/credential logging added. Full pressure/version/confidentiality fault matrix pending.

## Visual evidence and provenance

See [the compact capture gallery](evidence/p73/README.md). These are actual local
1440x900 pt-BR native screenshots of the shared Bevy interface, captured by the
existing `ui-acceptance` screenshot feature under Xvfb. They were visually inspected.
They use the last successfully built binary, **before the final edits** listed
above. They are not browser evidence, current-final-source verification, visual
approval or performance measurements.

`players-native` used 137 real local fixture-account connections to the disposable
server. Every other capture uses explicit synthetic presentation state through
`capture_panel=p73-*`, not actual authentication/history persistence. The Result
and owner-history screens label this locally; the rematch capture's fixture
provenance is explicit in its filename, log and gallery caption. API/socket tests,
not these fixtures, supply behavioral evidence. Fixture names are not real users.
No DB, Access token, account key or private journal is included in the PR.

Raw PNGs and logs remain in `/tmp/tbg-p73-visual-evidence/`. Screenshots were
lossily transcoded to small WebP files without changing content. The fixture
server owned by this task was stopped; the owner's other server/builds were left
running. Capture commands used the already-built executable, not another build:

```sh
env -u WAYLAND_DISPLAY TBG_DATA_DIR=/tmp/tbg-p73-visual-evidence/client \
  XDG_DATA_HOME=/tmp/tbg-p73-visual-evidence/xdg WINIT_UNIX_BACKEND=x11 \
  RUST_LOG=warn TBG_SERVER=ws://127.0.0.1:52205/ws TBG_PANEL=live TBG_LANG=pt-BR \
  TBG_CAPTURE=/tmp/tbg-p73-visual-evidence/<capture>.png \
  TBG_CAPTURE_PANEL=<p73-fixture-or-live> TBG_CAPTURE_SIZE=1440x900 \
  TBG_CAPTURE_AFTER=10 timeout 55s xvfb-run -a -s '-screen 0 1440x900x24' \
  /home/iiii/PESSOAL-PROJETOS-ALEXANDRE/TBG-Tupi-Battle-Grounds/target/debug/tbg-client
```

The initial native capture exposed a visible-but-unfocused window polling bug;
it was fixed to respect native visibility and received a focused regression
before the successful populated capture. The original loading capture remains
local for audit. Captures still show earlier wording/countdown behavior, explicitly
covered by the final deferred check list.

## Migration, rollout and rollback

All four flags default false: `TBG_HISTORY_MIGRATE`, `TBG_HISTORY_COLLECT`,
`TBG_HISTORY_API`, `TBG_HISTORY_PUBLIC`. No flag was changed outside disposable
fixtures. `TBG_PRESENCE_PORT_A/B` only identify fixed loopback peers; defaults
remain 8080/8082, and tests use isolated ports.

Before any separately approved deployment: install privacy-aware handlers/proxy
on all public paths, validate the mixed-slot barrier, then explicitly install
schema 11 on the shared DB with the existing serialized migration mechanism.
Preserve a backup through the owner's normal operational procedure. Collection,
owner API and public opt-in are separate gates; collection requires the additive
schema. Public profile/history routes are pinned to privacy-aware slot a with
no old-slot fallback; only explicitly safe APIs share failover. Named owner HTTP
continues to the token's slot. Until every serving owner-API slot is compatible,
keep public opt-in off. Protocol 31 clients/servers must be coordinated through
existing compatibility/admission routing; older wire peers are refused.

Rollback disables collection/API/public flags as needed, while retaining all
schema additions, preferences, guards and journal directories. Do not route
profiles to an unguarded older binary, reset Public/Only me revisions, drop
tables/columns, purge files or retire a slot with pending/quarantined work.
A compatible history recovery owner must settle or explicitly take responsibility
for retained journals before retirement. An old binary that cannot open schema
11 is not a safe serving rollback. No production rollback was performed or
verified by this work front.

## Deviations and review boundary

D-101 records the appended **adjacent** private PersonalMatchResult message
instead of extending existing MatchEnd bytes. CompletedResult retains and
replays that same summary, preserving old tag/shape contracts while bumping
the handshake to 31. The Jiff dependency was already transitively locked and has
a one-line native Cargo rationale. No new service, auth system or Elo settlement
was introduced. Extra Caddy fail-closed routes and the non-rating Ranked hint
close mixed-version privacy/backfill races within the authorized scope.

Implementation is substantive and committed, but the phase does not claim all
34 acceptance scenarios passed. The final lock and browser cache blockers are
real. Publish normally and open a **Draft** PR to main per the owner's scoped
authorization; do not merge, auto-merge, deploy or infer owner visual approval.
