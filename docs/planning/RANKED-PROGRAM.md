# Ranked integrity and statistics: implementation program

**Planning only, 2026-10-31.** No production code, deployed configuration,
database, protocol, balance value, or account was changed by this work.
The owner explicitly requested publication of this documentation branch;
that request overrides the attachments' generic no-push instruction for
this delivery only. It does not authorize a merge or deployment.

## Recommended execution order

1. Execute [P60](../phases/P60-ranked-integrity.md) steps 0–4: identity-safe,
   idempotent settlement; versioned authoritative records; shadow signals.
2. Execute [P61](../phases/P61-ranked-stats.md) E0/E1: detailed collection
   and an internal report using the same match identity and eligibility.
3. Consolidate the affected checks once. Observe shadow data and manually
   label possible false positives. Small samples may justify remaining in
   shadow indefinitely; elapsed days alone are not evidence.
4. Publish P61 E2/E3 only after privacy, definitions, recovery and desktop
   acceptance pass. An empty/insufficient-data page is an honest outcome.
5. Consider P60 enforcement separately. Recommend **no automatic Elo hold**
   at launch. A proposed pair ceiling is an explicit repeat-reward rule,
   never proof of fraud. Keep FIFO pairing unchanged.

P60 owns schema **v7**; P61 owns **v8**. Both attachments asked for v7;
implementing two unrelated v7 migrations would lose data or skip a migration.
P61 depends on the P60 record contract, not on activating P60 restrictions.
One executor at a time owns `match_task`, `store`, protocol and shared docs.
No agent delegation is required by this plan.

## Baseline and numbering

Audited GitHub `main` **9008f117c4289122ddff978c5d25720d2bd79687**,
confirmed through the GitHub connector on 2026-10-31. The original working
directory had unrelated native-client commits and uncommitted validation
documentation; they were not incorporated into this branch.

Main's roadmap ends at P57, but the existing roster front reserves P59.
Use P60/P61. Main's highest decision is D-59; local and fetched fronts
already use D-60 through D-69, including `origin/phase/system-champions-crowd-shop`.
Use D-70/D-71/D-72. These reservations were checked against local branch
refs and the connected GitHub branch inventory, not an assumed global lock.
Recheck before implementation/merge; renumber these new entries together
if another front has subsequently claimed them. Do not renumber old rules.

## Verified current behavior

Paths below are repository-relative; symbols are the stable evidence
anchors. These are source-inspection findings, not production observations.

| Area | Current implementation and consequence | Evidence: file and symbol |
|---|---|---|
| Account creation | Name plus generated 80-bit secret; no email/password/payment. SHA-256 of its encoded secret is stored. Accounts are cheap; IP throttling is not Sybil resistance. | `crates/tbg-server/src/lobby/accounts.rs`: `register`, `sha256`; `store/accounts.rs`: `register`; `crates/tbg-core/src/account.rs`: `encode_secret`, `format_key` |
| Registration limits | Default three successful registrations/IP/hour, ten logins/IP/minute; failed registration refunds its slot. Runtime overrides and process restarts limit the strength of this control. | `crates/tbg-server/src/config.rs`: `Config::default`, `from_env`; `lobby/account_done.rs`: `register_done`; `limits.rs`: registration windows |
| Authentication | `Login` verifies the key in the store; the lobby binds the returned account to the current idle connection. Takeover moves the old match seat and deletes the old session. | `lobby/accounts.rs`: `login`; `store/accounts.rs`: `login`, `hashes_equal`; `lobby/account_done.rs`: `login_done` |
| Resume | Session tokens are random 128-bit values. A resumed socket closes its predecessor; disconnect checks connection identity. It cannot select an arbitrary seat in an intention. | `lobby/sessions.rs`: `hello`, `disconnected`; `match_task/command.rs`: `command`; `ws` routing |
| Result delivery gap | `ranked_done` checks only that the token now has some account; it does not compare that account ID with the account settled. A delayed reply after logout/login could overwrite the wrong cached profile. Store rows still use the frozen original IDs. | `lobby/ranked.rs`: `ranked_done`; `match_task.rs`: `run`; `lobby/account_done.rs`: `login_done` |
| IP provenance | Only loopback peers may supply X-Forwarded-For; the last comma-separated address is used. Direct remote peers cannot spoof it. No CF-Connecting-IP trust or Cloudflare CIDR validation exists here. | `ws/admission.rs`: `client_ip`, `forwarded_ip_trusted_only_from_local_proxy` |
| Proxy uncertainty | Checked-in deployment uses Caddy on loopback; P47 records an external nginx cache observation. No checked-in nginx/Cloudflare chain proves that the last XFF element is the human's address. It may be an edge address. | `deploy/Caddyfile.template`; `deploy/deploy.sh`; `docs/phases/P47-live-spectate-and-presence.md`: audit A9 |
| Admission | Default socket cap is **8**, not the attachment's 16; API limit defaults to 120/IP/min. Actual production overrides were not inspected. | `config.rs`: `Config::default`; `ws/admission.rs`: `Slot::acquire`; `api.rs`: `limited` |
| Ranked pairing | Account-only, separate FIFO; first waiter is seat A, arriving player B. No rating band, bots, or tie-breaking clock: lobby command arrival order decides. In an otherwise empty queue two coordinated accounts can pair reliably. | `lobby/ranked.rs`: `ranked`; `lobby/matchmaking.rs`: `pair` |
| Private matches | Rooms are Casual/Private; challenges and rematches are Challenge/Private. Only `mode == Ranked` calls ranked settlement. Practice is client-local. Listing is visibility, not a sufficient authorization check for rating. | `lobby/matchmaking.rs`: `join_room`; `lobby/challenge.rs`, `lobby/rematch.rs`: pairing; `match_task.rs`: `run` |
| Elo | Start 1000, floor 100, K=40 for fewer than 20 ranked games, otherwise 24. Rust rounding is nearest with ties away from zero. Decisive raw deltas have minimum magnitude 1; applied ratings clamp to [100,65535]. Different Ks and boundaries make the update non-zero-sum. | `crates/tbg-core/src/rating.rs`: `expected`, `delta`, `apply` |
| Match endings | Stronghold destruction, including simultaneous destruction draw, or Forfeit only. Surrender in countdown/running/pause calls `forfeit`; expired disconnect also calls it. If both expire, seat iteration/order can decide the first forfeit. There is no gameplay timeout draw. | `crates/tbg-core/src/sim.rs`: `check_strongholds`, `forfeit`, `end`; `state.rs`: `EndReason`; `match_task.rs`: `run`; `match_task/command.rs`: `command` |
| Time | `tick` advances combat/countdown and freezes during Augment selection. `clock` advances selection steps too. Existing `duration_s` is wall time including countdown, pauses and scheduling delays. | `sim.rs`: `step`; `augment.rs`: `selection_step`; `match_task/report.rs`: `log_end` |
| Atomic rating | One transaction reads current database ratings/game counts, updates both accounts and inserts `ranked_matches`. Failure rolls back all three. `MatchEnd` was already sent; failure logs and sends no rating notification. There is no retry identity or durable result journal. | `store/ranked.rs`: `ranked_result`; `store/mod.rs`: `ranked_result`; `store/worker.rs`: `worker`; `match_task.rs`: `run` |
| Telemetry independence | `matches` insertion is a separate best-effort `try_send`; full queue can drop it without a log at the enqueue site. A database insert failure logs. No foreign key or shared match key joins it to `ranked_matches`. | `store/mod.rs`: `insert_match`; `store/matches.rs`: `insert`; `match_task.rs`: `run` |
| Available telemetry | Seat JSON persists purchase counts by kind, total Evolutions, surrender boolean, network/view counters. The final log also contains Stronghold kinds/tiers/branches and chosen Augments, but `matches::insert` discards those top-level keys. Arena and both Arcana presets are persisted. | `match_task/report.rs`: `SeatStats`, `log_end`; `match_task/views.rs`: `count_success`; `store/matches.rs`: `insert` |
| Missing analytics | No online ruleset/patch identity; no first-purchase time, per-kind historical Evolution, Augment offer history or Synergy history. Final survivors cannot reconstruct dead units' choices. | `store/schema.rs`: `migrate`; `match_task/report.rs`: `log_end`; `state.rs`: `PlayerState`, `Unit`; `augment.rs`: `open_round`, `close_round` |
| Catalog | 25 purchasable kinds in `FULL_POOL`, 27 `ALL_KINDS` including summons, four Factions, five Archetypes, four selectable Strongholds (Classic is lab-only), 40 Augments, 15 Arcana nodes, five Arenas. | `types.rs`: `FULL_POOL`, `ALL_KINDS`, enums, `StrongholdKind::SELECTABLE`; `content.rs`: Augment catalog; `arcana.rs`: `NODES` |
| Damage limitation | `Event::Hit` has target/amount, no attacker; poison, betrayal, shields, HP scaling and dead-unit removal prevent reliable per-seat actual damage from a final snapshot. Do not label terminal HP loss as damage dealt. | `state.rs`: `Event`; `hits.rs`; `afflictions.rs`; `sim.rs`: `step` |
| Ranking cache | Ordered by rating DESC, wins DESC, name ASC; accounts with at least one ranked game. Store rebuilds at startup and, when dirty, at most once/10 s. HTTP/WS readers use a snapshot. Profiles query SQL separately. | `store/leaderboard.rs`: `build`; `store/worker.rs`: `worker`; `api.rs`: `leaderboard`, `player` |
| Correction today | No invalidation, adjudication ledger or administrative correction API was found in store/CLI paths. Manual SQL plus snapshot refresh/restart is the current recovery option, with no built-in audit or safe replay. | `store/mod.rs`: public methods; `store/worker.rs`: `StoreCmd`; `crates/tbg-server/src/bin/` |
| Spectators | Public view hides Energy, Shop, RNG and pending picks. Public names/queued status make coordination easier but do not permit choosing a ranked opponent or sending a seat command. A second account can relay public information; this is not proof of collusion. | `crates/tbg-core/src/view/public.rs`: `public_view`; `lobby/presence.rs`; `lobby/spectate.rs`; `api.rs`: `live`, `online` |
| Versions | Wire v17, SQLite v6. Statistics JSON can evolve separately from postcard. | `protocol.rs`: `PROTOCOL_VERSION`; `store/schema.rs`: `migrate` |
| Deployment gap | Backup script uses SQLite `.backup`; deploy installs that script but does **not call it before restart/migration**. Automatic pre-migration backup must be made concrete before v7 ships. | `deploy/backup.sh`; `deploy/deploy.sh`; `deploy/README.md`: restore drill |
| Existing lab | Offline roster CSV and ruleset Debug-form fingerprint exist; they are not online data or a complete semantic ruleset version. No current spreadsheet workbook was found among tracked non-archive files. Do not infer access to an external spreadsheet. | `crates/tbg-balance/src/report.rs`: `fingerprint`, `roster_csv`; `docs/media/roster/baseline-roster.csv`; `docs/PREBALANCE.md` |

## Documentation discrepancies and required improvements

- GAME-DESIGN §9 BR-AR14 promises RulesetVersion in records; schema v6
  does not implement it. ARCHITECTURE §16 lists planned `ruleset_fp` and
  expanded seat records that are not the current schema.
- The attachment assumes timeout results and 16 default sockets/IP; code
  has neither. Historical §6 fun metrics still mention overtime, removed
  by BR-M07. This program does not restore that mechanic.
- GAME-DESIGN reuses BR-AR01–05 in §9 and §12. Cite section **and** ID in
  tests touching these until a separately coordinated glossary cleanup.
- Ranked queue wait metrics currently miss the ranked enqueue timestamp:
  `ranked()` does not populate `queued_at`, although `pair()` consumes it.
  Add a separate ranked timestamp/metric and cancellation cleanup; do not
  accidentally enable the casual queue's bot-fallback behavior.
- Harden delayed `RankedDone` by account ID plus settlement sequence;
  never refresh an account using a token alone. Read authoritative current
  profiles after replay/retry. This is a prerequisite, not an anti-fraud
  heuristic.
- Replace best-effort rating submission with a small idempotent durable
  final-result journal. Keep optional statistical persistence independent.
  A completed result cannot be recovered after process loss if every
  durable medium failed; expose that limitation instead of inventing data.
- Add coherent match identity, ruleset provenance and append-only
  adjudication before a public analytics dashboard. These give both
  requested features the same auditable source.

## Requirements that cannot all hold literally

**Zero collection cost versus historical metrics.** Exact historical
Synergy/offer/Evolution data cannot be inferred at match end. Recommended:
fixed-size server counters on successful commands and existing semantic
events, outside `MatchState::step`, with no new scan/log/I/O per tick.
Evaluation/SQL occurs only at completion. This is small nonzero bookkeeping,
not literally zero server-loop CPU. If literal zero bookkeeping is required,
ship only final-state metrics and mark history-based metrics unavailable.
Exact per-unit damage needs a separate measured instrumentation change;
it is not an MVP dependency and is nullable, never fabricated as zero.

**Exact counterfactual Elo versus no cascade.** Recomputing historical Elo
changes subsequent opponents' expected scores and can propagate through
the entire population. P60 deliberately freezes accepted deltas and
replays those entries for only affected accounts. That is an auditable
rating correction, not the rating history of an alternate universe.

**Perfect Sybil prevention versus open anonymous registration.** Cheap
accounts, open FIFO and unrestricted opponents make perfect prevention
impossible. Pair budgets bound one route; newcomer-source budgets bound
another; patient farms and mature colluders remain. No promise of perfect
integrity, automatic bans or identity inference belongs in the UI.

## Owner decisions before activation (planning can finish now)

| ID | Recommendation | Cost of choosing differently |
|---|---|---|
| O1 | Ship shadow first; keep pair/newcomer gates off until evidence. If enabling, trial **120 positive points/pair/7 days** plus **120 positive points from all immature opponents/7 days**, both proposed. | 40/7 freezes honest friends after only 2–3 wins. Higher ceilings allow more manipulation; any ceiling can inconvenience legitimate repeated opponents. |
| O2 | Review flags settle Elo normally; no automatic retention. | A hold delays honest rewards, needs a reviewer SLA and affects later Elo. Stage 3 is explicitly a separate activation decision/design gate. |
| O3 | Frozen-delta correction; no retroactive changes to pre-cutover matches. | Full counterfactual replay affects innocent later opponents and greatly increases operating complexity. |
| O4 | Public guests may see aggregates only after 50 exposed sides, 5 distinct accounts, 3 distinct pairs, and no account above 40% of exposure. | Lower limits are faster but noisier and easier to identify/manipulate; stricter limits may keep the page sparse for weeks. These are data-quality gates, not anonymity guarantees. |
| O5 | Accept bounded event bookkeeping; defer exact damage and public rating bands. | Literal zero bookkeeping removes historical offer/Synergy metrics; per-hit capture adds complexity before it informs a decision. |
| O6 | No IP-derived persistence in the MVP. Manual appeals use the repository issue route with only the private result receipt shared through an owner-approved private channel. | Any IP linkage needs verified proxy provenance, retention, privacy notice and separate approval; never publish keys or internal identifiers in an issue. |

Cache TTL and internal SQL/module layouts are engineering choices, not
questions for the owner. Reincidence may prioritize manual review; account
suspension remains a separate human decision, outside this implementation.

## Validation policy for both phases

Requirements coverage for the two attachments:

| Requirement group | Executable location / disposition |
|---|---|
| Audit of real authentication, IP, queue, endings, persistence, ranking and tests | This program's evidence table; P60 §§1/10 and P61 §§1/11 |
| Twelve attacks, legitimate friends and residual fraud | P60 §3, one fixture per A1–A12 |
| Pair/net/positive/gradual/maturity/season alternatives and real Elo examples | P60 §2, including unequal K and floor |
| Signals, summary, risk states, cost and privacy | P60 §§4–6; exact damage explicitly deferred because current events cannot attribute it |
| Review, invalidation, appeal, reversible rating correction | P60 §7; frozen-delta policy avoids rerating later opponents |
| Shadow, optional cap, pending rating, rollback, protocol | P60 §§4/8/9; automated holds explicitly rejected for MVP and gated as a later design |
| Catalog metrics, Evolution, offers, Synergy, Arena and general match data | P61 §§4/6, exact side/match/opportunity denominators |
| Bias, samples, uncertainty, account concentration and rating strata | P61 §§2/6/7; public bands deferred with an explicit growth threshold |
| Versioning, both migration requests, recovery and bounded aggregation | P60 §§5–7 and P61 §§3–5; single v7 owner followed by v8 |
| Public API/cache/privacy and desktop lobby/i18n | P61 §8; no production cache assumptions treated as verified |
| Online report -> D-33 evidence -> patch notes -> next patch | P61 §9 and `docs/patches/TEMPLATE.md` |
| Staged execution, tests, contracts and acceptance reports | P60 §§8–10, P61 §§10–11; implementation reports intentionally blank |
| Owner-only choices and project improvements | This program's decision/improvement sections; no production activation authorized |

The owner's 2026-10-31 policy overrides old acceptance blocks and the
attachments' six-job/private-target examples. Work-front docs: diff,
links/formats and `git diff --check` only. Rust work: affected-package
formatting plus at most one inexpensive cached scoped check; queue tests
for consolidation. Every compiling command uses:

```bash
flock -n -E 75 "/tmp/tbg-validation-$(id -u).lock" \
  env CARGO_BUILD_JOBS=2 cargo check -p tbg-server --lib
```

Exit 75, missing flock or cold cache: defer, do not poll/bypass/clean/use a
new target directory. One reviewer runs the union of the phase-specific
tests once at consolidation. No full suite, release build, lab gate,
benchmark campaign, production access or deployment is authorized by
this plan. A future full-suite request follows `docs/VALIDATION.md` once
that separate validation-policy front is merged. Phases specify focused
acceptance tests; these are planned work, not results from this delivery.

## Planning validation record

Validation level: documentation only. Source inspection, local Git/ref
inventory, official methodological references and arithmetic examples were
used. No Cargo, Trunk, browser build, balance lab or production database
was run. Implementation phase reports stay empty.

Completed documentation checks:

- `git diff --check`: exit 0, no whitespace errors.
- Python stdlib document inspection: 101 relative Markdown links resolve;
  code fences balance; both JSON examples parse; D-70–72 are unique;
  BR-RI01–12 and BR-ST01–12 are complete and nonduplicated in the new sections.
- Python `sqlite3` using **only a synthetic in-memory v6 schema**: the
  proposed v7/v8 SQL parses/applies, baseline copies preserve the fixture,
  schema reaches v8, foreign-key check is empty and integrity check says `ok`.
  The history SELECT prepares and both account arms use `ranked_side_history`
  in EXPLAIN QUERY PLAN. This validates the written SQL sketch, not future
  Rust transactions, durability, throughput or deployment compatibility.
- Python arithmetic: streak examples and unequal-K alternation reproduced;
  Wilson examples are 40.4–59.6% (50/100) and 46.2–72.4% (30/50).
- GitHub main rechecked before publication: still `9008f11`.

Deferred by scope: every implementation test/build/browser observation,
runtime measurement, shadow-data finding, owner activation decision,
production proxy/backup verification and D-33 balance experiment. None is
claimed passed by this documentation validation.
