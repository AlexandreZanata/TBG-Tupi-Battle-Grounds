# P61 — Ranked statistics and evidence for patch notes

**Status:** executable proposal; no implementation or online data collected.
**Base:** P60 steps 0–4 integrated on main; no dependency on cap activation.
**Branch for execution:** `phase/p61-ranked-stats`.
**Decisions:** D-88/D-89. **Rules:** GAME-DESIGN §15, BR-ST01–12.
**Platform:** desktop browser lobby. No player leaderboard, new service,
analytics vendor, ML, mobile work or automatic balance changes.

## Read first

- `AGENTS.md`; [program and code audit](../planning/RANKED-PROGRAM.md);
  [P60](P60-ranked-integrity.md) §§5–7 (shared records and status semantics).
- GAME-DESIGN §§2/5/6/9/10/11/12/14/15; DECISIONS D-33/D-41/D-46/D-55,
  D-87–89; ARCHITECTURE §§3/8/9/11 and ranked addendum.
- Core `types.rs`, `state.rs`, `intent.rs`, `augment.rs`, `synergy.rs`,
  `ruleset.rs`, `content.rs`, `rating.rs`, `summon.rs`; server
  `match_task/{command,report,views}.rs`, `store/{mod,worker,matches,ranked,schema}.rs`,
  `api.rs`, `lib.rs`, `config.rs`; P60's new modules.
- Client `index.html`, `assets/{lobby,ranking,live,arenas,i18n}/`,
  `tests/{ranking,live,lobby}.test.js`; lab `report.rs`, `commands.rs`,
  `docs/PREBALANCE.md`, `docs/media/roster/baseline-roster.csv`.

## 1. What exists and what is missing

Use the program's audited table as the current-state evidence. In
particular, `match_task/report.rs::log_end` already reports purchases,
Evolutions, chosen Augments and Strongholds, but `store/matches.rs::insert`
does not persist all of them. `ranked_matches` has accounts and rating but
no shared key with `matches`, no arena or ruleset revision. Do not join
legacy rows by timestamp/seed/name or manufacture offer histories.

`Ruleset::canon()` uses 25 Shop kinds, while `ALL_KINDS` includes 27
including summons. Collection indexes use enum IDs with catalog metadata,
not a hardcoded assumption that purchasable IDs are contiguous 0..24.
Reconcile a roster expansion before coding; stable IDs are append-only.

## 2. Industry calibration and interpretation

Riot's [Champion Balance Framework](https://www.leagueoflegends.com/en-us/news/dev/dev-champion-balance-framework/)
describes patch-by-patch review with different skill populations. Its
elite population illustrates that even a large game can lack enough
matches for trustworthy win-rate conclusions. TBG should borrow explicit
definitions, cohort labels and investigation thresholds; it should not
copy another game's percentage gates or assume popularity means power.
TBG has no bans, no roles assigned at queue entry and multiple purchases
per participant. Ban rate and pick-or-ban presence are inapplicable.

Riot's [item-balancing explanation](https://www.leagueoflegends.com/en-gb/news/dev/dev-updated-approach-to-item-balancing/)
is the closer analogy: purchases depend on game progress and an existing
advantage. Late purchases can have high win rates because winners can
afford them. TBG therefore needs all-match descriptive presence plus an
early-purchase cohort for investigations, rather than treating a unit's
observed win rate as its causal strength. The proposed 60-second cohort
is a TBG hypothesis, not a recommendation copied from Riot.

Use a [Wilson score interval](https://www.itl.nist.gov/div898/handbook/prc/section2/prc241.htm)
for binary proportions, and disclose that repeated players and paired
sides violate an independent-trials interpretation. These are descriptive
uncertainty bounds, not proof that a unit needs a buff. Account/pair
concentration, an internal leave-one-account-out check, human review and
D-33 lab evidence remain necessary. No league tiers, public top-player
breakdowns or automatic significance sorting at this population size.

Sources consulted 2026-10-31; they are methodological references, not a
claim that these historical Riot policies are its current production rules.

## 3. Ruleset identity shared with P60

Add `tbg-core/src/ruleset_version.rs`:

```rust
pub const RULESET_VERSION: u32 = 1; // first collection revision, not protocol
pub const SUMMARY_VERSION: u16 = 1;
pub fn ruleset_source_hash() -> u64;
```

This is P60 step 3's prerequisite, implemented once and reused here.
Monotonic integer revision names every gameplay-content or mechanic change,
including a mid-patch fix. Cosmetic/UI/server-only fixes keep the revision.
The function computes FNV-1a over a stable sorted manifest of included
gameplay source paths, separators and source bytes, once at server startup;
not per tick/match. Use `include_str!` and std/core facilities, no new crate.
Watch every production core source except presentation/protocol/account,
rating/integrity, bot/headless/replay/lab, root export scaffolding and this
version module; explicitly enumerate paths in `ruleset-sources.txt`.
Include all content submodules, Stronghold/Augment/Arcana tables, structural
constants, targeting, hits, summon rules and `types.rs`. An integration
test walks `src` and fails on any new .rs path not classified into watched
or excluded lists. Exclusions require a documented reason.

Initial watched paths relative to `crates/tbg-core/src`: `afflictions.rs`,
`arcana.rs`, `augment.rs`, `combat.rs`, `combat/formation.rs`, `content.rs`,
`hits.rs`, `intent.rs`, `rng.rs`, `ruleset.rs`, `shop.rs`, `sim.rs`,
`state.rs`, `stronghold.rs`, `summon.rs`, `support.rs`, `synergy.rs`,
`targeting.rs`, `types.rs`. Initial exclusions: `account.rs`, `bot.rs`,
`headless.rs`, `lab.rs`, `lib.rs`, `profiling.rs`, `protocol.rs`, `rating.rs`,
`replay.rs`, `view.rs`, `view/public.rs`, `view/visibility.rs`, plus the new
`ranked_integrity.rs` and `ruleset_version.rs`. Presentation changes that
alter an authoritative mechanical input must be reclassified. New paths
fail classification until reviewed.

`tests/fixtures/ruleset-versions.txt` is append-only `(revision,hash)`;
`contract_ruleset_version.rs` requires the current hash equals the last
entry and the revision is its strictly increasing ID. Old entries cannot
be rewritten to keep a patch ID after a rules change. Conservative source
hashing can split a patch on a pure formatting change; document that
tradeoff, don't silently relax the test. The hash identifies source, not
the behavior of arbitrary binaries. The release manifest also stores Git
commit and deployment date. Server stores both revision/hash at pairing;
a match finishing after an update retains its original stamp. Never mix
different hashes under one revision: fail publication and report collision.

P60's `ranked_rulesets` stores the immutable per-revision catalog once:
numeric ID, stable content slug, comparison identity, purchasable/summon
flag, Faction/Archetype mapping, Evolution form IDs, selectable Stronghold
IDs, Augment/node/Arena IDs and tick frequency. Bound UTF-8 JSON at 64 KiB.
Decode/group old rows with their stored catalog, never today's mapping.
A content redesign changes comparison identity; a numerical tuning keeps
it, so prior-patch comparisons remain meaningful. This registry is server
metadata, not a client-selectable ruleset. No per-match catalog duplication.

Do not reuse `tbg-balance::report::fingerprint` as the online contract: it
hashes Debug-form Ruleset fields and does not cover every mechanic or table.
The offline lab manifest should additionally include the new revision/hash
when next changed for balance work. Do not change or run the lab now.

## 4. Bounded collection and exact data contract

Implement `server/match_task/summary.rs` for accumulation and
`server/stats/{mod,decode,aggregate,privacy,report}.rs` for pure server-side
analytics. SQL stays in `store/stats.rs`. No tbg-balance dependency.
Use fixed arrays indexed by authoritative catalog IDs. No raw event stream,
per-unit lifetime map, per-frame scan, per-tick log or SQL.

```rust
pub struct UnitSummary {
    pub bought: u32,
    pub first_buy_tick: Option<u32>, // combat-relative, not wall time
    pub evolved_forms: u8, // observed paid forms: bits A1,A2,B1,B2
    pub max_tier: u8, // 0,1,2; highest achieved, including dead units
    pub evolved_copies: [u32; 2], // successful tier-1 / tier-2 transitions
}
pub struct SideSummary {
    pub units: [UnitSummary; tbg_core::types::ALL_KINDS.len()],
    pub synergy_max: [u8; 4],
    pub synergy_ticks: [[u32; 2]; 4], // time at level>=1, >=2
    pub augment_offered: [u32; 40], // completed seat-round opportunities
    pub augment_chosen: [u32; 40], // applied purchases at resolution
    pub augment_offered_any: u64, pub augment_chosen_any: u64,
    pub augment_closed_rounds: u32, pub augment_cancelled_rounds: u32,
    pub stronghold: u8, pub fort_tier: u8, pub fort_branch: u8,
    pub arcanas: [u8; 15],
    pub complete: bool,
}
pub struct MatchSummary {
    pub schema: u16, pub arena: u8,
    pub sides: [SideSummary; 2],
    pub combat_ticks: u32, pub wall_ms: u64,
}
pub fn finish_summary(/* authoritative collector and state */)
    -> Result<serde_json::Value, SummaryError>;
pub enum SummaryError { Overflow, InvalidCatalog, TooLarge }
// Store calls this after authoritative settlement COMMIT:
pub async fn save_ranked_stats(&self, match_id: i64, summary: serde_json::Value)
    -> Result<(), StatsWriteError>;
pub enum StatsWriteError { Unavailable, InvalidSummary, ConflictingDuplicate }
```

Make `finish_summary` concrete as
`fn finish_summary(collector: SummaryCollector, state: &MatchState,
wall_ms: u64) -> Result<Value, SummaryError>`, with collector owning two
SideSummary arrays, current Synergy levels/change ticks, current round
ID/options/resolution flags, and saturation flag. All updates are saturating;
overflow sets `complete=false`, excludes that record from every denominator
and records a coverage gap. Existing match duration is unbounded, so a
fixed-size accumulator is mandatory.

Collection rules:

1. Successful `Purchase`: capture kind from the validated offer before
   applying it, increment only after success; persist first combat-relative
   purchase tick once. Rejected/stale commands and generated cargo/summons
   are never Shop purchases. Reuse the existing `count_success` path.
2. Successful `Evolve`: capture the addressed owned unit's kind before
   application; read its accepted resulting tier/branch and OR the bit.
   Increment `evolved_copies[resulting_tier-1]` once. No need to retain dead
   units: each bought copy can transition through each tier once. At this
   baseline generated cargo is a summon (`summon.rs::crate_tick`,
   `cargo_draw`) and cannot evolve (`intent.rs::evolve`, NotEvolvable).
   Thus exact copy conversion follows from successful transitions.
   Require `tier2_count <= tier1_count <= bought`; if a new mechanic allows
   evolving generated units, version the summary and separate that origin
   before collecting it. Never infer Evolution from surviving units alone.
3. `SynergyChanged`: close old-level time at the event's combat-relative
   tick, update maximum/current level and open next interval; close at end.
   Integrate `[change_tick,next_change_tick)` at >=1 and >=2; freezes during
   pauses automatically. Initial level zero. No update for every elapsed tick.
4. `RoundOpened`: remember options once for that round. `AugmentChosen` or
   `Forgone` at resolution adds one opportunity for each offered ID to the
   appropriate seat, and selected count if applicable. Duplicate events
   must not double count. End during an unresolved round increments cancelled
   rounds; it does not imply four declined choices. Hidden pending picks
   remain private and do not count as applied effects.
5. End: copy frozen kind/Arcana/Arena, final Fortification, clocks and
   flags. Units lost earlier remain in accumulated presence/Evolution.
   Both seats must be complete or exclude the whole match, preserving
   paired denominators. P60 shares its terminal/integrity counters.

Fold existing events once in the server before fan-out; do not count them
once per recipient or from client views. This has small nonzero server
bookkeeping cost. `MatchState::step` and simulation state/hash remain
unchanged. Exact per-seat damage and per-unit damage attribution are
explicitly deferred; see program
constraints. Do not enable `lab` online to obtain counters.

### Stored JSON v1

Encode fixed data as compact arrays with documented enum-ID order, using
`serde_json` already present, rather than adding serde/postcard to server:

```json
{"schema":1,"catalog_hash":"hex","arena":0,"combat_ticks":1200,"wall_ms":63000,
 "sides":[{"units":[[0,2,40,1,1,1,0]],"synergy_max":[1,0,0,0],
 "synergy_ticks":[[200,0],[0,0],[0,0],[0,0]],
 "offered":[[3,1]],"chosen":[[3,1]],"rounds":[1,0],
 "stronghold":[2,0,0],"arcanas":[2,2,2,2,2,2,0,0,0,0,0,0,0,0,0],"complete":true},
 {"units":[],"synergy_max":[0,0,0,0],"synergy_ticks":[[0,0],[0,0],[0,0],[0,0]],
 "offered":[[3,1]],"chosen":[],"rounds":[1,0],
 "stronghold":[2,0,0],"arcanas":[2,2,2,2,2,2,0,0,0,0,0,0,0,0,0],"complete":true}]}
```

Shape example only, not a fixture asserting that only one Augment is
offered. `units` entries are
`[kind,bought,first_tick_or_null,form_bits,max_tier,tier1_copies,tier2_copies]`.
Sparse lists are sorted/unique and
bounded by the stamped catalog (27 kinds, 40 Augments at baseline).
Validate form bits, choices<=offers, tick bounds, kind/selectable IDs,
exactly two seats, complete Arcana ranks and <=16384 UTF-8 bytes before
insertion. Historical decoders use their catalog/version; an unknown newer
schema yields a coverage gap, not zero-valued observations. The public
response never contains this internal JSON or account/match identifiers.

## 5. Schema v8, persistence and aggregation

```sql
CREATE TABLE ranked_stats (
  match_id INTEGER PRIMARY KEY REFERENCES ranked_matches(id),
  summary_version INTEGER NOT NULL,
  payload TEXT NOT NULL CHECK(length(payload)<=16384)
);
CREATE INDEX ranked_stats_cohort ON ranked_matches(ruleset_version, ended_unix, id);
INSERT INTO schema_version(version) VALUES(8);
```

One optional raw-summary row per ranked match, written **after** P60's
authoritative transaction commits. SQL length is defense-in-depth; enforce
the UTF-8 byte limit in Rust. Insert-or-verify: identical duplicate is OK;
different content for the same key is an error, never overwrite evidence.
Account/rating outcomes cannot roll back because optional stats failed.
Total: P60's 7 authoritative mutations plus **1 optional row insert**;
no per-unit child-table writes or eager aggregate updates.

Keep optional payload in P60's journal until insert succeeds, or until an
operator explicitly quarantines it as an unrecoverable statistics gap.
Rating acknowledgement is not delayed by a failed stats write. On retry,
match key maps to existing match id; settlement cannot repeat. Malformed/
overflowed summaries are logged as coverage gaps and never retried forever.
Counts of eligible results, complete summaries, gaps and reasons are internal
quality metrics. Missing rows cannot be reconstructed from legacy telemetry;
never claim recovery when authoritative facts were lost.

Alternatives: eagerly maintained counters need inverse corrections and
crash coordination; per-request SQL scans compete with the writer; full
recomputation after every result repeats work. Choose immutable summaries
and a **discardable cached aggregation**, refreshed at most every 300 s
when the database revision changes. Admin invalidation/clear forces a new
generation and invalidates old caches; even a failed refresh must not
serve a cache built before a known invalidation.

One low-priority worker owns a read-only SQLite connection in `store/`.
Read revision plus rows in one bounded snapshot transaction; stream rows,
release snapshot promptly so WAL checkpoints are not held indefinitely.
Cap a rebuild at 100000 matches or 2 s; if exceeded, preserve coverage
diagnostics and return temporarily unavailable for a required new generation,
never publish an undocumented sample. These are proposed budgets to measure.
Use a progress handler only if available without a new feature; otherwise
check elapsed time between fetched rows. One active rebuild; coalesce
requests, no worker per HTTP request. No indefinite per-match in-memory cache.

Finite cache keys: current and immediately preceding complete ruleset,
band=`all`, seven dimensions. Historical CLI reports accept any revision.
API older-than-supported patch returns 404; UI lists only supported patches.
Counters are constant in match count, proportional to catalog dimensions.
Distinct-account/pair quality checks saturate counts at their required
thresholds; an exact concentration maximum uses a temporary SQLite
group-by on the read connection or bounded external iteration, not an
unbounded Rust HashSet. Compute per-account contributions by ordered
`account_id` rows, flush per-account totals, retain only max and up-to-5
distinct-account / up-to-3 pair threshold witnesses. Do not conflate a
bounded collector with constant database/cache size.

Budget estimates, not measurements: sparse summary typically 2–8 KiB,
hard cap 16 KiB; worst-case 1000/day ~15.6 MiB/day or ~5.6 GiB/year before
indexes/WAL, plus P60 facts. Collector <=8 KiB/match, shared serialized
cache <=1 MiB, public JSON <=64 KiB/response. Time target: <=1 ms summary
encode p95, <=5 ms optional insert p95, <=2 s bounded rebuild. Measure
with representative complete arrays, not empty examples. At thousands of
accounts/large history, extend to indexed per-patch derived aggregates and
chunked rebuilds only after this budget fails; raw rows remain authoritative.

## 6. Eligibility and metric definitions

Only post-cutover `Ranked`+`Public` matches with two distinct server-bound
accounts, reached Running, valid authoritative outcome, complete v1
summary/stamped catalog, `risk=Normal`, `disposition=Applied` enter the
main cohort. Exclude Casual, rooms, Challenge/rematch, Practice, pre-Running
abandonments, Review, Invalidated, budget NoChange and missing records.
No minimum duration filter after Running: that would hide legitimate rushes.
An ordinary equal-rating draw with zero points is still Applied and eligible.
Never infer eligibility from nonzero rating change or legacy reason alone.
Publish data coverage only after the same privacy rules; internal reports
always show all exclusion counts. Clearing review can add a match later;
invalidation removes the entire match/two sides on next generation.

Let M be eligible matches in one ruleset, S=2M eligible sides. Each side
has binary win/loss/draw flags and Elo score `s=1,0,0.5` respectively.
For exposure X, `N_X` counts **distinct sides** satisfying X, `W_X` wins,
`D_X` draws, and `L_X` losses. Mirrors count once per side, never once per
copy; match-level statistics use M. Catalog features can coexist, so
presence rates need not sum to 100%.

Universal definitions:

- Presence = `N_X/S`; binary win rate = `W_X/N_X` (draws are non-wins).
  Also report draw rate and score rate `(W_X+0.5*D_X)/N_X`, explicitly named.
  No half-win Wilson interval masquerading as a binomial interval.
- Buyer delta = `WR_X-WR_notX` in **percentage points**, same cohort.
  Both denominators must pass privacy/sample gates; zero denominator => null.
  This is association, **does not isolate causal unit strength or skill**.
- Expected-score residual = `mean(s - rating::expected(own_before,opp_before))`;
  use immutable authoritative settlement-before ratings, never current rating
  or a cached lobby rating. Available in internal E1 now; includes draws.
  It corrects for one coarse skill estimate, not strategy/selection bias.
  No normal/Wilson CI claimed for this residual; public residual deferred.
- Previous-patch delta = current WR minus prior WR only with the same
  definition/cohort and stable semantic content ID and both passing gates.
  Unknown/new/changed-meaning content => null, not zero. Label it descriptive.

| Dimension | Exposure/denominator and companion metrics | Specific limitation |
|---|---|---|
| Unit | At least one successful Shop purchase of kind; copies per all sides=`sum(bought)/S`, per buyer=`sum(bought)/N_X` | Generated units/summons not purchases; all-match WR has survival bias |
| Evolution | For kind buyers, side ever reached tier>=1 or >=2 / kind-buying sides; forms by bit, maximum-stage WR for exactly max=0/1/2; copy conversion=`sum(tierN_copies)/sum(bought)` | Side reach and copy conversion have different denominators; neither is a causal benefit; generated summons are not evolved copies |
| Faction / Archetype | Side bought >=1 kind with that canonical Faction/Archetype; copies summed by that grouping; presence and WR by side | Compositions overlap; no false mutually exclusive team identity |
| Synergy | Side ever reached Faction level>=1 / >=2; uptime=`sum(active_ticks)/sum(combat_ticks)` over exposed sides | One-tick activation counts as presence; uptime identifies transient activation; no inference from total purchases |
| Augment | Conditional pick=`sum(chosen opportunities)/sum(offered opportunities)` over completed seat-rounds; offered-side presence, chosen-side presence and WR of sides with >=1 applied copy | Repeated opportunities from a long match are dependent; affordability and forgone rounds affect choice; no phantom declines on cancellation |
| Arcana | Node rank>0 and each exact rank exposure per side; all-match presence/WR and score. Preset-level public rows deferred | Nodes are selected together; correlated with other nodes/builds; never sum their individual effects |
| Stronghold | Frozen selectable kind per side, exclusive; Fortification tier/branch optional internal exposure | Fortification is selected after play starts and has survival bias |
| Arena | Frequency=`matches_in_arena/M`; mean duration, forfeit rate and **seat A** win/score rates per arena | Both sides share arena, so overall arena win rate is structurally near 50% and uninformative; arena is assigned, not picked |
| General | Mean wall/combat durations over M; Surrender and disconnect-forfeit counts/M separately; seat A wins/M, draws/M, A score/M; matches/day by UTC completion day | No gameplay timeout rate; first waiter is always seat A, so position effect is confounded with queue/skill |

**Primary public unit metric:** descriptive all-match side presence/WR,
with explicit wording. **Primary internal unit investigation metric:**
60-second landmark cohort: retain matches reaching 1200 combat ticks,
include both sides; exposure is purchase at tick<=1200. Compare early
buyers/nonbuyers in that cohort, and report excluded early finishes and
their compositions separately. A first purchase time suffices for early
presence; do not pretend it supplies early copy counts. Sensitivity at
30/90 seconds is deferred, not an extra public filter now.
Landmark analysis limits late-purchase bias but conditions on survival
to the landmark; it cannot measure rush strategy strength alone.

Stronghold x Stronghold matrix: one observation per match, lower kind ID
as focal side, score for focal; same-kind mirror summarized once. Faction
x Faction requires a declared composition rule: dominant purchase Faction
(largest copy count; ties -> Mixed). Mixed is a real bucket, not arbitrary
tie-breaking. Publish a cell only at >=100 matches, >=10 accounts, >=5
pairs and max-account share<=40%; both opposite views share the same data.
These matrices are E1/internal until thresholds are met; not mandatory
empty tabs at E2 launch.

## 7. Sample size, privacy and uncertainty

Proposed per-feature public release: >=50 exposed sides, >=5 distinct
accounts, >=3 distinct unordered account pairs, max account exposure<=40%.
The complement must independently qualify before exposing its WR/delta.
Match-level Arena/general rows need >=25 matches with the same account/
pair/concentration safeguards. Evolution denominators and individual ranks
must qualify separately. Augment WR requires chosen-side gates; conditional
pick requires >=50 offered opportunities from >=25 distinct matches with
the same account/pair safeguards. Keep counts of opportunities, exposed
sides and distinct matches separate internally.

Suppressed rows expose content ID/name and `status=insufficient`, with
rates/intervals/deltas/sample set to null. Do not leak raw small counts
through sorting, totals, residuals or a downloadable response. No public
distinct-account counts or maximum share values. Totals/general rows,
complements and patch comparisons undergo the same checks. Avoid arbitrary
time windows/rating cutoffs that allow differencing. Fixed patch snapshots
refresh only when at least 10 additional eligible matches accumulated since
publication, except invalidation/privacy withdrawal which removes data
immediately. TTL is not a privacy release threshold.

Round exposed public n down to a multiple of 10, rates/CI endpoints to
whole percentage points (internal reports use exact counts). Approximate n
must be labeled. These measures reduce singling out; with public match
histories they do **not guarantee anonymization**. More formal privacy or
finer filters require a separate review. A tiny community may show no
public rows; do not lower safeguards automatically to fill the page.

Wilson binary interval (95%, z=1.96), n>0, p=w/n:
`center=(p+z²/(2n))/(1+z²/n)`;
`half=z*sqrt(p*(1-p)/n+z²/(4n²))/(1+z²/n)`;
clip center±half to [0,1]. It is descriptive under clustering. A 50/100
row is approximately 40.4–59.6%; 30/50 is approximately 46.2–72.4%.
Pairing and repeated accounts reduce effective information further.

Internal leave-one-account-out sensitivity: for each contributing account,
exclude **whole matches involving it**, recompute candidates, report min/max
and whether the direction survives. Offline CLI only; streamed queries,
no always-running per-account engine. Do not treat 25 units x many cuts
as independent significant discoveries. Candidate list is descriptive,
no p-value ranking/multiple-comparison claims. Confirm in held-out lab
seeds and playtests before changing numbers (D-33).

Rating bands: ship `all` only; other band values rejected, selector explains
that bands require more data. Future dynamic tertiles need >=300 distinct
accounts and >=1000 eligible matches per patch plus per-row privacy gates.
Boundaries derive from each account's first eligible pre-match rating in
that patch, frozen at publication, not repeated matches weighted as people.
Use pre-match rating, avoid splitting ties, collapse bands if gates fail;
label boundaries separately per patch, since dynamic bands are not directly
comparable. Implement this only as a later scoped extension.

## 8. Publication contract and lobby integration

`GET /api/stats?patch=1&band=all&dimension=units` with exact allowlist:
`units,factions,synergies,augments,arcanas,strongholds,arenas`. Archetype
rows are the Factions tab's second grouping. No arbitrary player, date,
unit combination, sort SQL or custom rating bounds. At most 256 query
bytes; duplicate/unknown keys or malformed values -> 400. Unknown valid
revision -> 404; cold/invalidated cache -> 503 with Retry-After=5; API
rate limit -> 429. All errors `Cache-Control: no-store`.

Successful JSON <=65536 uncompressed bytes, <=128 rows, finite numbers
or null only; overflow is a 503 + metric, never silent row truncation.
Use existing API rate limiter 120/IP/min and `client_ip` trust policy;
test with a non-loopback spoof too. Headers:
`Cache-Control: public, max-age=60, s-maxage=60, must-revalidate`,
`ETag` of the filtered serialized generation, `Content-Type: application/json`,
`X-Content-Type-Options: nosniff`. Conditional 304 carries matching cache
headers, no SQL. Do not use stale-while-revalidate across invalidations.

```json
{"schema":1,"patch":1,"band":"all","dimension":"units",
 "updated_day":"2026-10-31","generation":"opaque-content-hash",
 "patches":[1],"available_bands":["all"],"sample_approx":100,
 "rows":[{"id":"archer","status":"sufficient","sample_approx":50,
 "presence_pct":50,"win_pct":60,"draw_pct":0,"score_pct":60,
 "win_ci_pct":[46,72],"nonbuyer_delta_pp":null,
 "previous_patch_delta_pp":null,"copies_per_side":1.2}]}
```

Example is illustrative, not real data. Dimensions add only documented
metric keys: Evolution `tier`, `reach_pct`; Synergy `level`,`uptime_pct`;
Augments `conditional_pick_pct`; Arcana `rank`; Arena `assigned_pct`,
`seat_a_win_pct`,`seat_a_score_pct`,`combat_mean_s`,`forfeit_pct`.
Do not publish `account_id`, name, key, IP, raw match IDs, detailed timestamps,
risk facts, seeds or a source JSON blob. Generation is a hash of the public
payload excluding itself, not the internal revision counter.

Cache removal cannot retract a response already downloaded. Known
invalidation removes in-process snapshots immediately, purges configured
edge cache on an authorized deployment/admin procedure, and has a maximum
60-second residual HTTP cache window where respected. A future stricter
removal SLA requires no-store. Verify actual nginx/Cloudflare policy before
publishing; checked-in Caddy alone cannot prove those overrides. Add
unhashed stats JS/CSS/HTML to revalidation rules; current broad `*.js`
immutable policy must not freeze new script filenames.

UI: implement `assets/stats/{data,render,controller}.js`, `stats.css`,
`panel.html`, copied by Trunk as the current Ranking assets are. Reuse
Ranking's illustrated panel typography/table patterns and Live's validated
response/error handling. Use existing lobby navigation state with one
active panel: `lobby`, `ranking`, `arenas` or `stats`. Opening Stats closes
other views/dialogs through their controllers; leaving lobby closes Stats;
Escape/back returns once. Preserve original button nodes/listeners.
No new framework/dependency/art generation is needed.

Seven tabs, patch selector, All rating selector, sortable presence/WR/
previous-patch delta/sample/interval. Suppressed rows sort last and say
Insufficient data. Show draw treatment, collection start, approximate
sample and descriptive-data caveat. Arena labels say assigned frequency
and seat-A score, never pick rate or meaningless pooled winner rate.
Load on open/tab change, reuse cached response; no polling when hidden.
Keyboard focus, table headings, readable loading/error/retry and a proper
close control; inspect 1280x720 and 1920x1080 desktop widths only.

All new visible strings live in both catalogs under `stats.*`, including
`title`, `tab.*`, `metric.*`, `insufficient`, `sample_approx`, `draw_help`,
`bias_help`, `bands_unavailable`, `loading`, `unavailable`, `retry`, `close`.
Source files in this repo are `crates/tbg-client/assets/i18n/en.json` and
`pt-BR.json` (the attachments abbreviated their path). English-only planning
does not waive the executor's Brazilian Portuguese localization requirement.
No postcard change is needed for E0–E3: HTTP and server-only fields do not
alter MatchState/MatchView. Do not bump wire version for JSON schemas.

## 9. Internal report and patch-note workflow

Add `src/bin/tbg-stats.rs` to existing server crate, std CLI parsing:

```text
tbg-stats report --db BACKUP.sqlite3 --patch 1 --compare 0 --out report.md
tbg-stats report --db BACKUP.sqlite3 --patch 1 --leave-one-account-out --out sensitivity.md
```

`--compare` optional; unknown/no data yields an explicit absent comparison,
not division by zero. Report is internal and not served by the HTTP route.
Default output omits account identities even internally. Include source
revision/hash, cutoff, eligibility coverage/exclusions, exact sample and
concentration, metric definitions, Wilson limits, all-match and landmark
rows, residuals, comparable previous revision changes and candidate list.
No command writes to production or changes balance.

Candidate rule (**Proposed**, exploratory): at least 200 exposed sides,
10 accounts and 5 pairs, max share<=25%; Wilson bounds entirely below
45% or above 55%, OR unit presence <2% or >60%. Presence thresholds also
need a denominator of at least 1000 eligible sides; low presence is a
discoverability/investigation signal, not an automatic buff. Form/augment
conditions are recorded contextually; don't apply a unit presence threshold
blindly to every Arcana node or uniformly assigned Arena. Require candidate
direction robust to leave-one-account-out or label concentrated/inconclusive.

Pipeline: online descriptive report -> human hypothesis and plausible
confounders -> controlled candidate -> D-33 lab experiment with one coherent
change family, paired seeds/seat swaps and held-out confirmation -> owner
decision -> gameplay change plus new RulesetVersion/manifest contract in
same commit -> [patch-note template](../patches/TEMPLATE.md) with before/
after, online n/CI and lab artifact -> authorized deploy -> collect the new
revision separately -> comparable follow-up report. Existing roster CSV is
offline evidence; optional spreadsheet export is a derived view of this
report, never an alternative source of rules. No balance lab run here.

## 10. Delivery stages and implementation steps

0. **Integrate contracts.** Confirm P60 shared key, v7, status/revision,
   source stamp and journal. Record any newer roster/UI change. Commit
   adopted BR-ST rules and new tests together with D-88/D-89.
1. **E0 collector.** Add fixed command/event counters and decoder fixtures.
   Test destroyed units, cancelled Augment round, duplicate commands,
   generated units, pause-clock invariants and saturation. No UI/wire change.
2. **E0 persistence.** Add v8 raw row, post-commit insert/recovery and coverage
   metrics. `TBG_RANKED_STATS=collect` default; `off` disables optional
   collection, never P60 settlement. Explain missing intervals in coverage.
3. **E1 aggregation/CLI.** Implement exact definitions, streaming reader,
   privacy gates, Wilson, residual, landmark and correction rebuild. No
   HTTP yet. Report deterministic synthetic examples with known answers.
4. **E2 HTTP cache.** `TBG_RANKED_STATS=public` explicitly enables the public
   route; bounded snapshot/cache/query validation and error headers. Every
   successful result is from a fully qualified privacy-filtered generation.
5. **E2 desktop page.** Integrate one active panel and seven localized tabs;
   client consumes only the allowlisted response, textContent/createElement,
   no innerHTML for API content. Reuse existing CSP and asset pipeline.
6. **E3 patch workflow.** Implement internal report comparison/sensitivity,
   template references and release checklist; record candidate hypotheses
   separately from approved number changes. Do not automate buffs/nerfs.
7. **Consolidation.** Run affected union once and complete report below.
   Owner approves look, public disclosure policy and deployment separately.

Advancement: E0->E1 after complete synthetic coverage, zero effect on rating
under optional write failure, idempotent recovery and measured budgets.
E1->E2 after agreeing metric labels/privacy, zero raw-data leakage, observed
coverage >=99% over at least 50 eligible completions, correct invalidation
rebuild and desktop acceptance. Insufficient population means no published
numeric rows; synthetic evidence cannot substitute for privacy thresholds.
E3 needs CLI/template acceptance; it does not require a real balance patch
to be fabricated. No stage advances on elapsed time alone.

Schema/protocol/rollback by stage: E0 is v8, wire unchanged; E1–E3 no new
migration or wire version. Roll back public->collect to remove route/UI,
or collect->off while preserving rows; P60 settlement continues. Existing
v7-aware code ignores the additive v8 table. Never drop summaries or restore
an older DB over newer ranked results. Use P60's backup/scratch validation
procedure before the authorized migration. Legacy games never backfill.

## 11. Tests, acceptance and deferred work

Work-front follows program policy; tests are written now in implementation
and run once by the consolidation reviewer, not on each step. New targets:

- `tbg-core/tests/contract_ruleset_version.rs`: BR-ST02 append-only revision,
  full source classification, mechanic/content mutation needs a new ID.
- `tbg-server/tests/ranked_stats.rs`: BR-ST01/03–08/12, exact denominators,
  all dimensions, binary draw/score distinction, full-match versus landmark,
  no causal claim, historical dead-unit Evolution, generated-unit exclusion,
  mirror pair counting, known Wilson examples, isolated patch IDs.
- `tbg-server/tests/ranked_stats_recovery.rs`: optional SQL failure after
  rating commit, journal retry, missing blob, conflicting duplicate, v6->v7->v8,
  migrate twice, review->clear->invalidate->restore yields a full rebuild
  equal to a clean scan; no rating mutation by aggregation.
- `tbg-server/tests/contract_ranked_stats.rs`: append-only contracts for
  source authority, privacy gating and no old-data backfill (D-88/D-89).
- `tbg-server/tests/stats_api.rs`: all valid dimensions; malicious/long/duplicate
  filters, body bound, every error header, 304, rate limit, no account data,
  suppressed complements, fixed generation threshold and invalidation race.
- `crates/tbg-client/tests/stats.test.js`: validated data/formatting/sort,
  no NaN/zero for missing metrics, both catalogs' keys, tabs and exclusive
  navigation. Extend existing `lobby.test.js` for Stats/Ranking/Live/Arenas
  opening and close/escape transitions without overlapping interactive panels.

Known synthetic fixture: two matches, one draw and one A win => 4 sides,
1 win/1 loss/2 draws; global binary WR=25%, score=50%; seat-A WR=50%,
score=75%. A feature bought twice by one winning side has presence=25%,
copies/all-side=0.5, copies/buyer=2, WR=100%; it fails public sample gates.
Mirror exposure counts two sides, not two wins. Add qualified fixtures
with exact known groups for every public threshold and one-below edges.

Consolidation commands (under the same host lock; do not run today):

```bash
flock -n -E 75 "/tmp/tbg-validation-$(id -u).lock" env CARGO_BUILD_JOBS=2 \
  cargo test -p tbg-core --test contract_ruleset_version -- --test-threads=1
flock -n -E 75 "/tmp/tbg-validation-$(id -u).lock" env CARGO_BUILD_JOBS=2 \
  cargo test -p tbg-server --test ranked_stats --test ranked_stats_recovery \
  --test contract_ranked_stats --test stats_api -- --test-threads=1
node --test crates/tbg-client/tests/stats.test.js crates/tbg-client/tests/lobby.test.js
```

Merge these with P60's relevant targets, not two repeated suites. Queue one
focused WASM/browser build under the lock for the UI, then inspect two
desktop sizes, both languages, guest/account, suppression, errors, close,
tab changes and overlapping-panel regression. No phone matrix/full suite,
release build, heavy load campaign, lab run or production check is implied.

Acceptance: definitions have reproducible fixture values; all status/patch
filters exact; correction rebuild idempotent; coverage visible internally;
stats failure never affects rating; bounded CPU/storage/query behavior
measured; public errors safe; no unpublished identity/micro-cohort leaks;
all strings localized; panel transitions usable. Owner signs off look.

Remaining risks: small/concentrated samples, unobserved skill/strategy,
survival bias, adaptive opponent choices, mature collusion, publication
differencing and source hash maintenance. Out of scope: causal balance
estimator, bans, ranked bots, per-hit instrumentation, public quantiles,
seasons, warehouse, dashboards, offline lab execution and actual tuning.

## Phase report (executor fills after implementation)

- Commits and stages implemented: only §3 ruleset identity (`e71a814`,
  shared with P60 step 3; `ranked_rulesets` registration in `9b5dea7`).
  E0–E3 (collector, v8 schema, internal report, public API, lobby page,
  patch-note workflow) **not started**. `contract_ruleset_version` passes
  (2 tests).
- Adopted owner decisions / configuration / ruleset identity:
- Validation level and commands/results:
- Coverage and known-data metric evidence:
- Storage/CPU/query/API byte measurements:
- Optional-write failure / invalidation / migration recovery evidence:
- Desktop/i18n evidence and owner look decision:
- Deviations, blocked items and exact deferred checks with reasons:
- Reviewer conclusion / owner publication decision:
