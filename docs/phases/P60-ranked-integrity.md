# P60 — Ranked integrity and auditable settlement

**Status:** executable proposal; implementation has not started.
**Base:** main `9008f11`; reconcile newer main before execution.
**Branch for execution:** `phase/p60-ranked-integrity`.
**Decisions:** D-87/D-89. **Rules:** GAME-DESIGN §14, BR-RI01–12.
**Scope:** desktop browsers, server records and minimal local admin CLI.
No balance change, bots, rating-based matchmaking, new service or web admin.

## Read first

- `AGENTS.md`; [program, audit and validation policy](../planning/RANKED-PROGRAM.md).
- GAME-DESIGN §§2, 5 (Elo), 10.2/10.6, 13/14; DECISIONS D-18/D-19,
  D-33/D-41/D-59/D-87/D-89; ARCHITECTURE §§3/5/8/9/11 and ranked addendum.
- Core `rating.rs`, `state.rs`, `sim.rs`, `protocol.rs`; server
  `lobby/{ranked,matchmaking,sessions,accounts,account_done}.rs`,
  `match_task.rs`, `match_task/{command,report,views}.rs`, `store/*`,
  `ws/admission.rs`, `limits.rs`, `api.rs`, `config.rs`, `chan.rs`.
- Existing `tbg-core/tests/{rating,contract_protocol,contract_protocol_server}.rs`;
  `tbg-server/tests/{ranked,accounts,leaderboard,contract_challenge}.rs`;
  `deploy/{deploy.sh,backup.sh,README.md}`. Read P61's record contract only.

## 1. Audit and constraints

The program's **Verified current behavior** table is the baseline audit
for this phase, including file/function evidence for authentication, IP,
FIFO, result causes, transactions, failures, corrections, ranking and
spectators. Reverify changed symbols at step 0, not the entire repository.

Preserve BR-R01/R02: one open ranked FIFO, no Elo bands, random queue
delays, opponent selection, bots or penalties to queue priority. Integrity
runs after the authoritative outcome. Operational unavailability is an
explicit unavailable response, never a silent matchmaking delay.

## 2. Elo arithmetic and mechanism selection

For participant i, let `d_i = delta(R_i,R_j,games_i,score_i)` and
`b_i = apply(R_i,d_i)-R_i`. Eligibility selects **either `(b_a,b_b)`
unchanged or `(0,0)`**. Do not clamp one beneficiary, rescale Ks or force
zero sum. The baseline already permits inflation/deflation:

| Current-code example | A delta | B delta | Pair total |
|---|---:|---:|---:|
| New 1000 beats new 1000 | +20 | -20 | 0 |
| Established 1000 beats established 1000 | +12 | -12 | 0 |
| New 1000 beats established 1000 | +20 | -12 | +8 |
| Established 1000 beats new 1000 | +12 | -20 | -8 |
| Established 1000 beats established 100 at floor | +1 | 0 | +1 |
| Established 1400 beats established 1000 | +2 | -2 | 0 |

Numbers were calculated from `rating::{expected,delta,apply}`; the
arithmetic check reproduced Rust's ties-away-from-zero rounding in Python,
not a Rust build or lab simulation. The u16 ceiling has the corresponding
clipping effect. Draws can move rating and must use the same eligibility.

Starting both accounts at 1000 and repeatedly letting A win:

- New accounts: gains `20,18,16,14,13,11,10,9,9,8`; cumulative
  `20,38,54,68,81,92,102,111,120,128`.
- Established accounts: gains `12,11,10,10,9,9,8,7,7,7`; cumulative
  `12,23,33,43,52,61,69,76,83,90`.
- A 40-point hard budget rejects the third new-account win or fourth
  established-account win. **40/7 days is too restrictive for this community.**
- A candidate 120-point budget permits nine new-account wins in this
  streak. It is less restrictive, not established as fair by these examples.
- Alternating wins, established A/new B at 1000: A wins `(12,-20)`, B
  wins `(-13,+22)`; after two games `(999,1002)`, total +1. After four,
  `(998,1004)`, total +2. This is not evidence that both always profit.
  A ring can redistribute gains, exploit placements and recycle donors.

| Alternative | Benefit | Cost / strongest attack / small-community effect |
|---|---|---|
| Net directional 40/7 cap | Understandable net transfer | Losses replenish allowance; alternating wins, asymmetric Ks and floor frustrate net-transfer intuition; stops friends early |
| Sum of positive gains, whole-pair gate | Easy invariant and audit; losses never replenish allowance | Waiting out the rolling window and many donors remain; honest repeated winners receive zero after exhaustion |
| Gradual repeated-opponent weight | Softens the cliff | Another rounding/floor policy; easy farm across many opponents; partially changing only one delta is unacceptable |
| Repeat count alone, then both zero | Extremely cheap | Equal competitive back-and-forth loses rewards too; count can be exhausted deliberately |
| Gate only after combined suspicion | Fewer indiscriminate limits | Sophisticated fake play evades it; heuristics effectively punish false positives; use for review, not automatic gating |
| Separate skill rating and seasonal recognition | Could bound rewards without distorting Elo | New product/ranking model and reset policy; rank itself remains farmable; defer |
| Lower initial rating or change K | Reduces some new-account transfers | Changes established skill calibration, does not establish identity; outside this phase |
| Mature-opponent weight / global immature-source budget | Global budget bounds many disposable donors per recipient | Age/count farms can manufacture maturity; weights introduce new math; prefer a whole-pair global budget if owner enables any gate |

**Selected architecture:** keep Elo unchanged; shadow evidence first;
optional positive-gain pair/global-newcomer budgets with whole-pair zero;
combined signals only queue human review; append-only correction history.
Recommend leaving automatic gates off initially.

### Exact candidate eligibility (only enforce after O1 approval)

Store authoritative completion time `t` as UTC integer seconds. History
window is `(t - 604800, t]`, with only earlier committed match sequence IDs
included for ties. Clock rollback is handled by `t=max(system_utc,last_t)`
inside the writer; persist `last_t`. Seven days means seconds, not local
calendar weeks. There is no midnight reset. Expiry can still permit a new
burst; a rolling window reduces synchronized resets, not patient attacks.

For each account i and opponent j:
`pair_in(i,j)=sum(max(effective_delta_i,0))` over Applied, non-Invalidated
entries in the window, including Review entries that settled rating.
`new_in(i)` sums the same positive points from all opponents classified
immature at that match's settlement. Negative deltas never refund a budget.
Classification is immutable evidence, not inferred from today's profile.
Mature means account age >=7 days **and** >=20 previously Applied,
non-Invalidated results; NoChange games do not establish maturity.
Legacy ranked_games initializes this count at cutover; old games are not
retroactively investigated. K still reads ranked_games, but adoption of
enforcement explicitly makes NoChange results ineligible for competitive
game/W/L/D counters. This prevents burning placements or farming the wins
tie-break through zero-rating matches. Shadow retains today's counters;
the optional counter-policy amendment must accompany BR-RI adoption.

Compute both baseline applied deltas. If a positive `b_i` would make
`pair_in(i,j)+b_i > 120`, or an immature opponent would make
`new_in(i)+b_i > 120`, classify this settlement NoChange for **both**.
Exactly 120 is allowed. No partial award or remainder. A player at their
own cap can still lose normally if the other beneficiary has allowance;
a negative delta does not trigger that player's cap. Draw beneficiaries
are checked too. Opponent caps never alter who gets paired.

Other objective NoChange reason: a forfeit before entering Running, if
this proposed rule is enabled. It is not evidence of misconduct. In shadow,
all these conditions are recorded as `would_*`, but existing Elo still
applies. No automatic Invalidated status comes from a heuristic.
Non-ranked/private matches cannot call this ranked settlement path at all;
equal account IDs and contradictory identity/outcome envelopes are internal
errors to quarantine, not invented draws. Duplicate match keys return the
previous receipt without applying anything again.

## 3. Attack catalog and expected scenario tests

Every row becomes a named fixture in `ranked_integrity` tests; cite BR-RI
IDs in assertions. Signals are weak evidence, not proof of human intent.

| ID | Attack / attacker cost | Gain and observable evidence | Defense / residual risk / false positive |
|---|---|---|---|
| A1 | Immediate donor Surrender; two accounts, seconds | Baseline +20 or +12; repeat directional endings | Pre-Running objective zero if enabled, then budgets and review; genuine accidental queue entry remains valid/no misconduct finding |
| A2 | AFK/organized Stronghold loss or disconnect; match time | Same Elo without pressing Surrender; low participation + repeated losses | Outcome-independent budget; behavior plus relationship review. Deliberate plausible play remains undetectable; defensive/saving play alone never flags |
| A3 | N disposable donors; registrations/IPs/time | Pair budget alone permits roughly N times the cap | Global immature-source budget bounds recipient inflow; staged maturity circumvents it; new friends can also reach budget |
| A4 | Slow transfer across days | Window expiry and gradual positive inflow | Longitudinal CLI evidence; no lifetime automatic restriction. Waiting seven days remains possible; long-term friends look similar |
| A5 | Fresh 1000 donor repeatedly loses | Free initial rating; placement K | Maturity classified from server data, not client claim; newcomer source budget. Do not lower starting rating by stealth |
| A6 | Recycle weak donor through other accounts | +1 minimum, floor, unequal Ks and replenishment elsewhere | Positive-only sums, exact baseline drift logged; no net-budget refund. Mature rings remain possible |
| A7 | Alternate victories or a transfer ring | May mint total rating, not necessarily improve both actors | Both positive directions charged separately; review graph only in CLI. Legitimate close rivals must not be invalidated |
| A8 | Exhaust a rival's budget | Can remove their future rating opportunity at personal cost | No bans/queue exclusion; transparent NoChange, owner review of impact. Whole-pair caps inherently permit this griefing; shadow must quantify it |
| A9 | Synchronize queue join; two clients | Nearly certain pair when alone, probabilistic with others | No artificial wait; retrospective evidence only. Presence list reveals availability, not authority to choose an opponent |
| A10 | Same-account reconnect/takeover/late result | Attempt duplicate settlement or cached-profile corruption | Frozen account IDs, unique match key, connection checks, account+sequence delivery; ordinary multi-device use is valid |
| A11 | Discard accounts to erase history | Per-account risk resets | Recipient's global immature-source history survives donor replacement; recipient replacement sacrifices rank. Mature Sybils remain |
| A12 | Honest friends repeatedly play | Same patterns as farming in a two-player community | Repetition alone Normal; complete matches remain recorded; shared IP ignored. Optional caps may yield NoChange, never a fraud label; owner must accept this tradeoff |

## 4. Signals, costs and state machine

Post-match history is one indexed query with two account/time range arms
over `ranked_sides`, joined to status. No per-tick SQL or risk evaluation.
Return the last seven days for A and B, deduplicating matches for pair counts.
Abort history processing at 10001 rows; limit is 10000. On overflow/error,
record `history_unavailable`, apply ordinary Elo, do not guess a cap; alert
the operator. This fail-open limit is a known saturation attack, measured
before enforcement. Do not silently describe it as complete protection.

Active-community size U is distinct accounts with a completed ranked game
in the same seven days, from a cached periodic read-only snapshot (10-minute
refresh, not presence/socket count). It is diagnostic only. Repetition
ratio `pair_matches/max(1,min(games_a,games_b))` and expected random share
`1/max(1,U-1)` are reported together; FIFO is not uniformly random, so this
is not a p-value or a reason to withhold rating.

| Signal | Exact proposed definition / source | Cost, weakness and false positive |
|---|---|---|
| S1 relationship | At least 5 pair games/7 days and >=80% of donor's games against recipient; report U, 1-hour and 7-day counts | Same history read; friends in tiny communities often meet it; never sufficient |
| S2 directional transfer | Recipient positive net >=40 and donor lost >=80% of at least 5 pair games/7 days | Same rows; related to S1, so **not independent** evidence; skill gaps are legitimate |
| S3 behavior | In at least 3 losses to that recipient: zero successful purchases and zero Evolutions/Fortifications/applied Augments; or voluntary Surrender before 15 combat seconds | Fixed command counters + completion cause; saving/defending, connection trouble and lost positions can qualify; not sufficient alone |
| S4 newcomer concentration | >=80% of recipient's positive inflow from immature opponents, at least 5 source accounts/7 days | Same account-history rows; recruiting new friends looks similar; supporting evidence only |
| S5 sessions | Record takeover/reconnect counts, whether both frozen accounts are distinct | No extra query; takeover is expected, identity equality is a programming invariant failure, not a player risk score |
| S6 network | Not collected/persisted in MVP | Unverified proxy provenance; NAT families/cafes and mobile/VPN changes make inference unreliable |

Minimal automatic Review = **S1 AND S2 AND S3**. S1+S2 is one relationship
family; S3 provides a different behavior family. S4 alone or with S1
does not suffice. All thresholds are Proposed, versioned and shadow-tested.
Short match, Surrender, IP, age or repeat count alone must remain Normal.
Review is a match flag; never label an account a cheater publicly.

Two independent axes avoid conflating suspicion and settlement:

- `RiskState::{Normal, Review, Invalidated}`.
- `RatingDisposition::{Applied, NoChange}`; a zero-delta equal draw can
  still be Applied. Do not introduce a `Pending` implementation yet.

Normal -> Review by the documented combination; Normal/Review ->
Invalidated only by a human decision with evidence or an adjudicated
objective data-integrity violation. Review -> Normal and Invalidated ->
Normal/Review require a new audit entry; never overwrite an old decision.
Review settles current Elo but is excluded from balance statistics until
cleared. NoChange due to a budget is Normal and is also excluded from P61's
main cohort. Ranking shows applied committed rating, no projected points.

**Retention stage decision:** recommend not implementing automated holds.
A 24-hour, auto-release hold is a candidate only: calculate Elo on release
using then-current ratings, show a private pending receipt, exclude it from
ranking/statistics, release both sides together. That changes rating order,
needs rechecking budgets/reserving capacity and more than one lifecycle
history read; it cannot honestly fit the present one-query contract.
Stage 3 stays disabled and requires a separate owner-approved extension
covering expiry worker, crash recovery, cancellation, SLA and new contracts.
This is a deliberate non-goal, not an unfinished hidden implementation step.

If IP linkage is reconsidered, require verified proxy stripping/trusted
CIDRs first; use keyed HMAC-SHA256 with a random daily key held outside
the DB, retain pseudonyms <=7 days and destroy corresponding keys, store
no raw addresses, never compare across rotations or expose them publicly.
Publish purpose/retention in the notice and treat it as personal data;
this is an engineering proposal, not a claim of legal compliance.

## 5. Types and module boundaries

New pure `tbg-core/src/ranked_integrity.rs` contains only copied integer
facts and baseline-delta gating; it does not touch simulation state, clocks
or SQL. Server modules own collection, history loading and adjudication.
Use existing `serde_json` manually; no new production dependency.

```rust
// core/ranked_integrity.rs: no serialization into MatchView
pub enum RiskState { Normal, Review, Invalidated }
pub enum RatingDisposition { Applied, NoChange }
pub struct BudgetFacts {
    pub pair_positive: [u32; 2],
    pub immature_positive: [u32; 2],
    pub opponent_immature: [bool; 2],
}
pub struct BudgetPolicy { pub pair_limit: u32, pub immature_limit: u32 }
pub struct BudgetDecision { pub applied: [i32; 2], pub reason_bits: u32 }
pub fn gate(base: [i32; 2], facts: &BudgetFacts,
            policy: &BudgetPolicy) -> BudgetDecision;

// server/ranked/mod.rs: internal, never accepted from a client
pub type MatchKey = [u8; 16]; // OS-random at pairing; separate from LiveId
pub enum EndDetail { Stronghold, Surrender, Disconnect }
pub struct IntegritySeat {
    pub purchases: u32, pub evolutions: u32, pub fortifications: u32,
    pub augments_applied: u32, pub reconnects: u32,
    pub surrender_tick: Option<u32>,
    pub final_hp: i32, pub final_max_hp: i32,
    pub actual_damage_dealt: Option<u64>, pub actual_damage_taken: Option<u64>,
}
pub struct RankedFinal {
    pub key: MatchKey, pub accounts: [i64; 2],
    pub outcome: tbg_core::Outcome, pub end_detail: EndDetail,
    pub combat_ticks: u32, pub wall_ms: u64, pub reached_running: bool,
    pub ruleset_version: u32, pub ruleset_source_hash: u64,
    pub seats: [IntegritySeat; 2],
    pub factions: [[u32; 4]; 2],
    pub arcanas: [tbg_core::ArcanaPreset; 2],
    pub stats: Option<serde_json::Value>, // P61, size bounded
}
pub struct RankedReceipt {
    pub key: MatchKey, pub sequence: i64, pub accounts: [i64; 2],
    pub results: tbg_core::protocol::RatingResults,
    pub disposition: RatingDisposition, pub risk: RiskState,
    pub reason_bits: u32,
}
pub struct JournalEntry {
    pub completion_seq: i64, pub ended_unix: i64, pub result: RankedFinal,
}
// Store API, replacing the positional ranked_result arguments:
pub async fn settle_ranked(&self, result: RankedFinal)
    -> Result<RankedReceipt, tbg_core::protocol::AccountError>;
```

`combat_ticks = state.tick.saturating_sub(RUNNING_START)`; persist ticks
and derive seconds using canonical TICK_HZ, not accelerated test tick_ms.
`reached_running` is latched by PhaseChanged, not inferred from winner.
First terminal cause wins: latch Surrender only when it actually ends the
match; latch Disconnect only when `forfeit` changes an unfinished state.
Do not rewrite `EndReason` or add a fictitious Timeout variant. Augment
selection expiry is Forgo, not match timeout. Track the losing seat and
surrender timing in the bounded JSON envelope. Both expiry and Surrender
continue to store legacy `reason='Forfeit'` plus the new `end_detail`.
Actual damage fields are `None` in v1; final HP/max HP is terminal progress,
not cumulative damage, and must not enter S3 as damage evidence.

## 6. Migration v7 (single owner: P60)

Run these additive statements in the existing migration transaction after
the v6 block; insert schema_version=7 last. Application encoding validates
JSON fields/versions/lengths; SQL does not require optional JSON1 functions.
Old rows keep NULL keys and are excluded from new analyses. P61 owns v8.

```sql
CREATE TABLE ranked_rulesets (
  version INTEGER PRIMARY KEY,
  source_hash TEXT NOT NULL UNIQUE,
  catalog_json TEXT NOT NULL CHECK(length(catalog_json)<=65536)
);
ALTER TABLE ranked_matches ADD COLUMN match_key BLOB;
ALTER TABLE ranked_matches ADD COLUMN ruleset_version INTEGER REFERENCES ranked_rulesets(version);
ALTER TABLE ranked_matches ADD COLUMN ruleset_source_hash TEXT;
ALTER TABLE ranked_matches ADD COLUMN ended_unix INTEGER;
ALTER TABLE ranked_matches ADD COLUMN end_detail TEXT;
ALTER TABLE ranked_matches ADD COLUMN combat_ticks INTEGER;
CREATE UNIQUE INDEX ranked_match_key ON ranked_matches(match_key)
  WHERE match_key IS NOT NULL;

CREATE TABLE ranked_integrity (
  match_id INTEGER PRIMARY KEY REFERENCES ranked_matches(id),
  policy_version INTEGER NOT NULL,
  mode TEXT NOT NULL CHECK(mode IN ('shadow','enforce')),
  risk INTEGER NOT NULL CHECK(risk IN (0,1,2)),
  disposition INTEGER NOT NULL CHECK(disposition IN (0,1)),
  reason_bits INTEGER NOT NULL,
  facts_json TEXT NOT NULL CHECK(length(facts_json) <= 8192)
);
CREATE TABLE ranked_sides (
  match_id INTEGER NOT NULL REFERENCES ranked_matches(id),
  seat INTEGER NOT NULL CHECK(seat IN (0,1)),
  account_id INTEGER NOT NULL REFERENCES accounts(id),
  opponent_id INTEGER NOT NULL REFERENCES accounts(id),
  ended_unix INTEGER NOT NULL,
  opponent_mature INTEGER NOT NULL CHECK(opponent_mature IN (0,1)),
  original_delta INTEGER NOT NULL,
  effective_delta INTEGER NOT NULL,
  low_activity INTEGER NOT NULL CHECK(low_activity IN (0,1)),
  early_surrender INTEGER NOT NULL CHECK(early_surrender IN (0,1)),
  PRIMARY KEY(match_id, seat), CHECK(account_id <> opponent_id)
);
CREATE INDEX ranked_side_history ON ranked_sides(account_id, ended_unix, match_id);
CREATE TABLE ranked_baseline (
  account_id INTEGER PRIMARY KEY REFERENCES accounts(id),
  rating INTEGER NOT NULL, peak_rating INTEGER NOT NULL,
  ranked_games INTEGER NOT NULL, wins INTEGER NOT NULL,
  losses INTEGER NOT NULL, draws INTEGER NOT NULL,
  mature_games INTEGER NOT NULL
);
INSERT INTO ranked_baseline
  SELECT id,rating,peak_rating,ranked_games,wins,losses,draws,ranked_games FROM accounts;
ALTER TABLE accounts ADD COLUMN mature_games INTEGER NOT NULL DEFAULT 0;
UPDATE accounts SET mature_games=ranked_games;
CREATE TABLE ranked_meta (
  singleton INTEGER PRIMARY KEY CHECK(singleton=1),
  last_utc INTEGER NOT NULL, revision INTEGER NOT NULL
);
INSERT INTO ranked_meta VALUES(1,0,0);
CREATE TABLE ranked_decisions (
  id INTEGER PRIMARY KEY,
  operation_key TEXT NOT NULL UNIQUE,
  match_id INTEGER NOT NULL REFERENCES ranked_matches(id),
  decided_unix INTEGER NOT NULL, actor TEXT NOT NULL,
  action TEXT NOT NULL CHECK(action IN ('invalidate','clear','review','restore')),
  reason TEXT NOT NULL CHECK(length(reason) BETWEEN 1 AND 512),
  before_json TEXT NOT NULL CHECK(length(before_json)<=4096),
  after_json TEXT NOT NULL CHECK(length(after_json)<=4096)
);
CREATE TRIGGER ranked_decisions_no_update BEFORE UPDATE ON ranked_decisions
BEGIN SELECT RAISE(ABORT,'append-only decision log'); END;
CREATE TRIGGER ranked_decisions_no_delete BEFORE DELETE ON ranked_decisions
BEGIN SELECT RAISE(ABORT,'append-only decision log'); END;
INSERT INTO schema_version(version) VALUES(7);
```

`facts_json` freezes baseline raw/applied deltas, actual inputs, signal
values, shadow decision, policy hash, terminal summary and missing-data
flags. New registrations insert a 1000/zero baseline in their transaction.
Original before/after ranked columns and `original_delta` never change.
`effective_delta` and status are disposable projections of the original
record plus the latest append-only adjudication. Store revision increments
on every successful settlement/correction. CLI uses it as a write lease.
Register a ruleset catalog once at startup, outside per-match writes;
an existing version with a different hash/catalog is a startup error.
P61 §3 specifies its immutable dimensions and history decoder.

Normal settlement adds four authoritative rows (match, integrity, two
sides), updates two accounts and one metadata row: **7 row mutations** in
one transaction. No new SQL on pairing/ticks. History read budget is one
additional statement (two indexed range arms) beyond existing account
reads/idempotency lookup. A periodic active-account snapshot and offline
admin replay are separate bounded workloads, never hidden per-match scans.

Use this single history statement; both range arms must use
`ranked_side_history` in the scratch `EXPLAIN QUERY PLAN`. Compute signal
aggregates while streaming, deduplicate by match_id for pair counts, and
fetch each account's age/maturity with its existing rating read:

```sql
SELECT h.*, m.winner, m.combat_ticks, m.end_detail, i.risk, i.disposition
FROM (
  SELECT * FROM (
    SELECT * FROM ranked_sides
     WHERE account_id=:a AND ended_unix>:cutoff AND ended_unix<=:now
       AND match_id<:sequence
     ORDER BY ended_unix, match_id LIMIT 10001
  )
  UNION ALL
  SELECT * FROM (
    SELECT * FROM ranked_sides
     WHERE account_id=:b AND ended_unix>:cutoff AND ended_unix<=:now
       AND match_id<:sequence
     ORDER BY ended_unix, match_id LIMIT 10001
  )
) AS h
JOIN ranked_matches m ON m.id=h.match_id
JOIN ranked_integrity i ON i.match_id=h.match_id
ORDER BY h.ended_unix, h.match_id, h.seat LIMIT 10001;
```

The query returns historical Review/Invalidated/NoChange facts too, so the
caller can separate observation counts from budget-eligible points. Never
sum all returned deltas without the disposition/risk filter.
Each arm also stops at 10001 indexed rows, so an overflow cannot force
sorting an unbounded seven-day result set before the outer limit applies.

## 7. Durability, correction and privacy

Generate MatchKey at pairing. Before submitting a completed ranked result,
write its minimal authoritative envelope to a private `ranked-pending/`
directory beside the DB using a temp file, `sync_all`, atomic rename and
directory sync. Use `spawn_blocking` for file work and existing dependencies.
An optional P61 blob serialization failure omits only `stats`; it must
never prevent the minimal journal/settlement. Never log account keys/IPs.

Journal file cap 32 KiB including optional stats; total pending cap 32 MiB.
The one finalization worker assigns a monotonically increasing completion
sequence before journaling, taking its startup high-water mark from committed
IDs and outstanding journal filenames. A file is named by sequence plus
MatchKey; replay preserves this sequence, never random-key ordering. The
JournalEntry freezes completion UTC here, clamped to the last observed UTC
across committed records and pending files; retries never replace it with
the recovery date. Use completion_seq as the new ranked row ID
(starting above the existing maximum). Do not allow later results to settle
ahead of an authoritative pending entry; matches still play, receipts say
unavailable until ordered recovery. Optional P61-only retry does not stall
later authoritative settlements.
Retry worker: one worker, delays 1/5/30 seconds then keep file for operator;
startup drains sequence order and idempotent receipts before declaring ranked
storage healthy. Duplicate identical key returns committed receipt; same
key/different envelope is quarantined with an operational alert. Journal
delete occurs only after commit. Crash after commit/before delete is safe.
Account-only result notification carries current authoritative profile and
settlement sequence; older replies cannot regress a newer cached profile.
Refresh by account ID, including takeover, rather than copying to whatever
account now owns an old token.

If neither journal nor DB can persist, report settlement unavailable,
retain the bounded in-memory result while alive and stop new ranked
admission with an explicit unavailable error. Existing games finish;
Casual remains available. No phantom win/zero result or fabricated receipt.
This is not a guarantee against disk loss/power loss on all durable media.

CLI belongs in the existing server crate: `src/bin/tbg-ranked-admin.rs`
and `store/integrity_admin.rs`, using std argument parsing and rusqlite.
Commands: `list-review --db COPY`, `inspect --match HEX --db COPY`,
`plan --action invalidate|clear|review|restore --match HEX --reason TEXT
--actor NAME --db COPY --out PLAN`, `apply --plan PLAN --db DB`.
Read-only commands work on a SQLite backup. Apply requires a stopped
server, the same base revision and matching affected-record hashes,
a fresh backup and a unique operation key; repeated apply is a no-op.
Plans show old/new ratings, peak and record for both affected accounts.
Actor is an operator identifier, not a claim of cryptographic authentication.

Rebuild the two accounts from `ranked_baseline`, replaying original
**effective-at-first-settlement deltas** in match-id order. Invalidated
entries contribute zero and no competitive W/L/D/game/maturity count;
NoChange entries contribute zero and no competitive W/L/D/game/maturity
count; the raw match and result history remain visible as unrated.
Use `rating::apply` at each accepted event so floor/ceiling remain valid.
Recompute peak along the same path, never below baseline peak. Later
opponents' frozen deltas do not change; no recursive Elo recalculation.
Restore replays the same log with the original entry reinstated; repeated
clear/restore cannot credit twice. Floor clipping means a correction need
not equal simply subtracting a visible historical number. Show both the
frozen delta and resulting projection in the dry run. All projected updates
and decision insertion commit together. Restart rebuilds all snapshots;
P61 invalidates its cache by revision. No pre-v7 match is adjudicated here.

Appeal: show a per-match receipt to its two participants and neutral reason
code, not the other account's risk facts. The owner provides a contact link
before enforcement; no existing support service is assumed. Public GitHub
issues may request contact but must never contain credentials or private
audit evidence. Backup retention and operator permissions apply to these
account-linked records; do not claim IP-free data is anonymous.

## 8. Small implementation steps

0. **Reconcile.** Reserve phase/decision/schema/protocol IDs on integrated
   main. Record differences from the audit. Ensure the frontend plan still
   matches the browser lobby if the native/UI migration has landed.
1. **Identity and delivery.** Add MatchKey/frozen account delivery/sequence;
   defend equal IDs and duplicate results. Add targeted regressions for
   logout/login before delayed settlement and live takeover. Keep v17.
2. **v7 records and journal.** Implement the SQL, atomic finalization,
   baseline initialization and journal recovery. Preserve full original
   errors internally instead of logging only `Unavailable`. Add counters
   `ranked_settlement_failures`, `ranked_pending_files`, duplicate/mismatch
   keys, result-notification failures. Do not block the combat task on I/O.
3. **Shared collection/provenance.** Implement the P61 §3 ruleset stamp
   before accepting new records. Record terminal causes, clocks and fixed
   command counters, with unavailable damage explicit. No core-state/wire
   additions. Fix ranked queue-wait instrumentation independently of Casual.
4. **Shadow evaluator.** Implement pure gating, one-query history, combined
   signals and CLI read-only reports. `TBG_RANKED_INTEGRITY=shadow` default;
   `TBG_RANKED_PAIR_CAP=120`, `TBG_RANKED_NEW_SOURCE_CAP=120`,
   `TBG_RANKED_WINDOW_SECS=604800`; validate nonzero bounded integer config,
   snapshot config per match. Startup rejects invalid settings. Enforce is
   unavailable until compatible clients/receipt presentation land.
5. **Correction CLI.** Implement dry run, revision lease, append-only log,
   replay and restore. Prove idempotence and floor behavior on scratch DBs.
6. **Optional enforcement UI.** Append a neutral `RankedSettlement` server
   message containing MatchKey, sequence, disposition and public reason
   (`Normal`, `RepeatLimit`, `NewOpponentLimit`, `NotStarted`, `Unavailable`).
   Do not expose internal signal flags. Keep existing RatingChanged for
   applied profile refresh. Bump protocol from 17 to the next free version
   (18 only if still free), update version/append-index contracts in the
   same commit as D-87 and BR-RI adoption. Older clients fail the existing
   version handshake; ship client/server together before enforce. Provide
   en/pt-BR keys and browser result/history rendering; rule text explains
   limits, no accusation or thresholds for behavioral evasion. Translation
   is implementation work; this English-only specification defines keys
   `ranked.settlement.normal`, `.repeat_limit`, `.new_opponent_limit`,
   `.not_started`, `.unavailable`, `.receipt`, `.appeal`.
7. **Consolidate and report.** One reviewer runs the targeted union below,
   validates scratch migration/recovery and desktop receipts, then fills
   the report. Owner separately decides deployment and gate activation.

Each step is one logical commit; no production code belongs in this
planning branch. Append new tests without weakening old contracts. A test
whose contract intentionally changes (e.g. optional countdown NoChange)
must distinguish shadow defaults from enforce mode and cite the adopted
BR-RI decision; never delete the original BR-R04/BR-M06 coverage.

## 9. Rollout, budgets and advancement gates

**Stage 1 shadow:** v7 plus journal; no wire changes and no player-visible
risk status. Existing Elo identical. Observe at least 7 days AND 50 games
from at least 5 accounts; if not reached, continue shadow. Manually review
every flagged match and every would-limit legitimate repeated-pair case.
Document false positives as human labels with uncertainty, not a statistically
validated error rate from a few matches.

**Stage 2 optional gate:** compatible receipt UI/version deployed, all
focused tests pass, zero observed settlement mismatches/lost committed
results, and owner approves the measured restriction burden/values. No
advance merely because a timer expired. Keep the flag reversible to shadow;
do not retroactively reclassify old results when the flag changes.

**Stage 3 holds:** intentionally deferred under §4. No automatic activation.

Proposed budgets: integrity facts <=8 KiB/result; collector <=4 KiB/match;
history <=10000 rows streamed; one completion history query; <=5 ms p95
additional evaluator CPU and <=20 ms p95 storage/journal time on the
measured local device. These are acceptance targets, not measurements.
Measure end-to-end settlement separately from simulation step. Fixed-size
counts saturate with a completeness flag rather than wrapping. Open-ended
matches must not accumulate event logs/RTTs for this feature.

At 1000 results/day, 8 KiB facts is ~7.8 MiB/day before indexes/WAL, about
2.8 GiB/year. P61 adds its own bounded blob. Start with retention of raw
post-cutover ranked records until an explicit owner retention decision;
measure actual DB growth. Do not silently delete records required for
appeal/replay. With larger traffic, stream read-only history or maintain
auditable rolling counters only after the measured query budget fails.

Queue proof: identical arrival sequence pairs identical account pairs in
shadow/enforce; zero DB/risk work added to ranked admission. Measure ranked
queue wait separately (fix current instrumentation); local same-load p95
must not increase by more than 5 ms, and pair creation must have no new
await. Sparse real-player waits are reported with n, not attributed to code
from a handful of games. No large load campaign is part of work-front checks.

Before an authorized deploy: drain/stop server, invoke SQLite backup and
verify its integrity, start new binary on a scratch copy, test migration
v6->v7 and recovery. Add backup invocation to the real deployment script;
installing `backup.sh` is insufficient. Preserve backup/binary pair.
Rollback normally sets shadow and uses a v7-aware previous feature build.
An old v6 binary ignores integrity: never start it against enforced v7
records as a normal rollback. Restoring pre-migration backup after accepting
new matches loses data; require maintenance plus export/replay of new
records and explicit owner decision. No live rollback is executed here.

## 10. Validation and acceptance

Work-front: affected-package fmt and at most one warm cached scoped check
per the program. **Do not run these tests per step.** New named targets:

- `tbg-core/tests/contract_ranked_integrity.rs`: BR-RI02 binary choice,
  unequal K/floor/ceiling/draws, exact limit and expiry, positive-only
  allowance, deterministic permutations/seat swap. Exhaust bounded
  representative ratings/game-counts in a table, no random framework.
- `tbg-server/tests/ranked_integrity.rs`: A1–A12 fixtures; Normal for
  repetition/IP/short game alone; review combination; one exhausted loser
  does not suppress an eligible winner; Sybil source budget; history overflow.
- `tbg-server/tests/ranked_recovery.rs`: v6->v7 twice, constraints, original
  rows unchanged, transaction insert failure, queue full, journal failures,
  crash before/after commit, duplicate recovery, baseline initialization,
  identity switch/takeover, floor/ceiling invalidation and restore twice,
  wrong revision/admin rollback. Test rollback of account updates when the
  ranked insert fails, not merely mock an early return.
- `tbg-server/tests/contract_ranked_integrity.rs`: BR-RI03/06/09 authority,
  private isolation, immutable originals and append-only decisions. Couple
  new contracts to D-87/D-89 in the implementation commit.

Reviewer runs once under the shared lock, jobs=2, test threads=1:

```bash
flock -n -E 75 "/tmp/tbg-validation-$(id -u).lock" env CARGO_BUILD_JOBS=2 \
  cargo test -p tbg-core --test rating --test contract_ranked_integrity -- --test-threads=1
flock -n -E 75 "/tmp/tbg-validation-$(id -u).lock" env CARGO_BUILD_JOBS=2 \
  cargo test -p tbg-server --test ranked --test ranked_integrity --test ranked_recovery \
  --test contract_ranked_integrity --test accounts --test leaderboard --test contract_challenge -- --test-threads=1
```

When step 6 changes wire, also queue the two existing protocol contract
targets once and the affected browser build under the same lock. Inspect
two desktop clients for ordinary results, NoChange, reconnect and stale
version; no mobile/full suite/lab/production tests. Final acceptance needs
actual evidence for transaction safety, bounded counters/query plans,
correct terminal reason, privacy and unchanged FIFO. Deferred = unverified.

## Phase report (executor fills after implementation)

Branch `codex/p60-p61-ranked-implementation`, based on `origin/main`
`9008f11` (fetched 2026-10-31) plus the planning commit `501d80c`,
cherry-picked as `8086675`. Validation level: **targeted tests** (run on
the owner's request, 2026-10-31). The scoped clippy and the queued tests
pass. **P60 is incomplete:** steps 5–7 are missing.

- **Commit(s):**
  - `afb4eac`: renumber D-70–72 → D-87–89.
  - `e71a814`: ruleset identity (P61 §3).
  - `7cad915` + `1a03c64`: pure gate.
  - `9b5dea7`: steps 1–2 settlement pipeline.
  - `61872f7`: recovery tests.
  - `aceb76d`: pre-deploy backup.
  - `05a1c49`: first-compile fixes.
  - `e6888d9`: step 4 shadow evaluation.
  - `deeeb6f`: ranked queue wait.
  - `79b5a1d`: stale v6 schema pin in a unit test.
- **Step 0 reconciliation.**
  - Main equals the plan's audited base, so the protocol is still v17 and
    the schema still v6. The browser HTML lobby is the only ranked
    surface on main (no native shell is merged).
  - Differences from the plan:
    - (a) D-70–D-86 had since been claimed (P59 PR #22, the native
      preview, the P62–P64 plans). All three entries became D-87/D-88/D-89
      together, as the program requires.
    - (b) Other fronts insert a GAME-DESIGN §13 (P58 roster), so §14/§15
      here must be renumbered positionally at merge.
    - (c) The ranked queue-wait gap was confirmed (`ranked()` never set
      `queued_at`) and is fixed in step 3.
- **Implemented (shadow only; nothing gates, holds or sanctions).**
  - *Step 1.* A 16-byte OS-random `MatchKey` is drawn at pairing,
    separate from `LiveId`. `RankedDone` now carries the receipt (frozen
    account ids + settlement sequence), and the lobby refreshes whichever
    session holds that account id, never a token's current owner. A
    receipt is applied only if its sequence is newer than what the cached
    profile contains: login reads the committed high-water mark in the
    same store-worker turn. Equal frozen accounts are refused in the match
    task and in the store; a duplicate key returns the committed receipt.
    Wire protocol unchanged (v17).
  - *Step 2.* The v7 SQL is exactly as in §6. New registrations get a
    baseline row in their transaction. `settle` writes match + integrity +
    two sides, two accounts and meta in one transaction (7 row mutations
    plus one history read), with `ended_unix` clamped to
    `ranked_meta.last_utc`. Each finished ranked result goes to one
    finalizer, which:
    - assigns the completion sequence (= row id);
    - journals the result to `ranked-pending/` (temp, `sync_all`, rename,
      directory sync; caps 32 KiB per file, 32 MiB total);
    - settles strictly in order, retrying 1 s, 5 s, then every 30 s;
    - deletes the file after commit and quarantines conflicts.

    Journal recovery runs before the lobby serves commands. While anything
    cannot settle, ranked admission answers `AccountError::Unavailable`;
    Casual is unaffected. New `/metrics` counters:
    `ranked_settlement_failures`, `ranked_pending_files`,
    `ranked_duplicate_keys`, `ranked_key_mismatches`,
    `ranked_notify_failures`, `ranked_journal_failures`.
  - *Step 3.*
    - `ruleset_version` (RULESET_VERSION 1, hash `881301f9c2597ec3`) and
      the catalog are registered once at start; a mismatch is a startup
      error.
    - `end_detail` (stronghold/surrender/disconnect) is latched on the
      first terminal cause, with the legacy `reason` kept.
    - `reached_running` is latched by `PhaseChanged`; combat ticks are
      stored.
    - Fixed counters: purchases, Evolutions, Fortifications, applied
      Augment copies, reconnects, surrender tick, terminal Stronghold HP.
      Damage stays `null`.
    - Ranked queue waits use their own timestamp map, cleared on cancel and
      cleanup, and the `ranked_queue_wait_{n,p50,p95}` metrics. Casual
      state is untouched.
  - *Step 4.*
    - The pure `ranked_integrity::gate` (core).
    - The plan's single history statement (two indexed arms, 10001-row
      bound, fail-open as `history_unavailable`).
    - `ranked::signals::evaluate`: budget facts, would-be bits/applied
      deltas, S1–S4 including the current match. Review only for
      S1 ∧ S2 ∧ S3.
    - `facts_json` records the policy snapshot, budget facts, `would_*` and
      the signals. The `risk` column records Review; `reason_bits` stays 0
      because nothing is applied.
    - `TBG_RANKED_INTEGRITY` (only `shadow`), `TBG_RANKED_PAIR_CAP`,
      `TBG_RANKED_NEW_SOURCE_CAP`, `TBG_RANKED_WINDOW_SECS` are validated
      at start-up (exit 2 on invalid; `enforce` rejected).
    - The policy is read once per process, so each settlement snapshots
      the process policy.
- **Not done.**
  - The step 4 read-only CLI reports and the step 5 correction CLI
    (`tbg-ranked-admin`).
  - Step 6: protocol/receipt UI (owner gate).
  - Step 7: consolidation.
  - The `EXPLAIN QUERY PLAN` evidence for the history statement.
  - P61 E2/E3 (see the P61 report; E0/E1 are delivered).
- **Validation actually run** (shared lock, `CARGO_BUILD_JOBS=2`, reusing
  the merged P47 worktree's target with an identical `Cargo.lock`; no new
  target directory, nothing cleaned):
  - `cargo fmt -p tbg-core -p tbg-server`: ok.
  - `bash -n deploy/backup.sh deploy/deploy.sh`: ok.
  - `git diff --check`: ok.
  - `cargo clippy -p tbg-core --lib --test contract_ruleset_version --test contract_ranked_integrity -- -D warnings`:
    exit 0.
  - `cargo clippy -p tbg-server --lib --bins --tests -- -D warnings`:
    exit 0 at `deeeb6f`. The first completed run, after seven exit-75
    deferrals while the P59 front held the lock, found 5 errors, fixed in
    `05a1c49`.
- **Targeted tests run** (owner request; shared lock, `CARGO_BUILD_JOBS=2`,
  `--test-threads=1`, at `deeeb6f` + `79b5a1d`):
  - `cargo test -p tbg-core --test rating --test contract_ranked_integrity --test contract_ruleset_version`:
    10 passed, exit 0. The hash computed in Python matches Rust's.
  - `cargo test -p tbg-server --no-fail-fast --test ranked --test ranked_integrity --test ranked_recovery --test accounts --test leaderboard --test contract_challenge --lib`:
    61 passed, exit 0.
    - lib 26, accounts 12, contract_challenge 2, leaderboard 4, ranked 6,
      ranked_integrity 7, ranked_recovery 4.
    - The first run failed one lib test,
      `store::matches::tests::the_arena_is_recorded_and_older_rows_stay_null`:
      it pinned `MAX(schema_version) == 6` (left 7, right 6). The pin
      became `>= 6`, with its arena assertions unchanged.
  - The existing ranked/accounts/leaderboard end-to-end suites now run
    through the finalizer and the v7 settlement.
- **Still deferred:**
  - Desktop browser checks of ordinary ranked results and reconnects.
  - `EXPLAIN QUERY PLAN` evidence for the history statement.
  - The protocol contracts (no wire change was made).
  - The full suite (not requested).
- **Measurements:** none yet (no query plans, storage, CPU or latency
  figures).
- **Attack fixtures:** A2/A3/A12, the exact limit, invalidated and
  NoChange exclusion, history-unavailable and NotStarted pass in
  `tests/ranked_integrity.rs`. The remaining catalog rows (A1, A4–A11) are
  covered only by the core contract or not yet written.
- **Migration/recovery/rollback:** v6 → v7 twice, idempotence,
  conflict/same-account quarantine, rollback on insert failure and journal
  ordering pass in `tests/ranked_recovery.rs`. The
  deploy now takes an integrity-checked pre-migration backup with the
  server stopped; rollback limits are in `deploy/README.md`. Nothing was
  run against production.
- **Deviations.**
  - Recovery is awaited before the lobby loop, so ranked is never offered
    before the journal has drained.
  - A stuck head keeps retrying every 30 s instead of stopping.
  - A failed start-up read keeps ranked unavailable (fail-closed).
  - `ranked_seen` keeps one entry per account seen in the process.
  - The policy is a per-process snapshot (recorded in every record), not
    stored per match in the journal.
- **Owner decisions:** none adopted. O1–O6 remain as recommended in the
  program: shadow first, no automatic hold, frozen-delta corrections, the
  O4 publication floor, bounded bookkeeping, no IP persistence.
