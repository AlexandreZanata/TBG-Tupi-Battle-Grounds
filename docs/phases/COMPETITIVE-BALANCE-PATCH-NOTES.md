# Competitive balance audit, scalability and public patch notes

Owner request: 2026-10-31. Branch: codex/competitive-balance-patch-notes.
Base: PR #29 including its CI follow-up; main was 9008f117 at checkout.
No main merge or history rewrite is authorized by this implementation.

## Scope and evidence rules

Audit actual online CANON, preserve simulation ticks and server authority,
measure worker scaling on identical seat-swapped seeds, correct coverage and
bounded-memory issues, execute discovery/screening/held-out balance evidence,
and apply only justified numeric changes. Values remain Proposed for owner
playtesting. A failed gate is evidence, never a reason to weaken a threshold.
The owner explicitly requested heavy balance/performance execution; compilation
still uses the shared host lock and two Cargo jobs. Full unrelated workspace
verification is not authorized. Raw runs stay ignored; summaries are committed.

Every gameplay revision must provide structured, bilingual public patch notes,
exact before/after values, evidence and limitations. CI checks the contract.
Serve notes through public read-only HTTP and expose their page from the
shared native lobby news panel and browser counterpart. Three generated WebP
banners share the existing medieval card palette. Forum artwork announces
community work without claiming that a forum has already shipped.

## Initial findings

- The lab defaulted to Ruleset::preview(), while manifest output labeled it
  canon. Preview enables the Practice-only Shop; online CANON does not.
- Cost search coverage used UNIT_KINDS (ten originals), excluding costs of
  36 active conventional champions. Other knobs already included their stats.
- A fixed 2.5% purchase-share floor is mathematically impossible for 46
  kinds (115% total). The owner approved the proportional floor 0.625 / N active kinds
  (1.3587% for 46); all other thresholds remain unchanged.
- Full-pool evaluation retains large combat counter arrays for every game,
  although reports only need their sums. This grows into tens of GB at the
  default 1000 pairs; bounded batches and lossless aggregation are required.
- Bootstrap intervals run serially despite independent deterministic streams.
- C2 per-tick fixtures still describe the old Shop roster, while the other
  canonical determinism fixtures already describe the 46-kind activation.
- The PR #29 timing failure measured 2.412043ms against an unchanged 2ms
  budget on the shared runner. e7d681e preserves that explicit measurement
  probe and adds a deterministic 300-actor CI regression (5 pass, 2 ignored).

## Baseline hardware and scaling

Intel i7-13620H, 16 logical CPUs, 31 GiB RAM; about 13 GiB available when
inspected. No host configuration changes. Legacy preview baseline, 32 paired
seeds: 1/2/4/8/16 workers produced approximately 54/108/195/295/334 games/s;
all result arrays were identical. This short baseline is exploratory, not the
final benchmark or online balance acceptance. Its manifest incorrectly said
canon, which is part of the audit finding above.

## Validation and release ledger

In progress. Record executed commands, exact failed and deferred checks,
seed partitions, measured effects and release/deployment/installation identity
before delivery. GitHub CI will not be awaited; the owner merges manually.

## Audit implementation and targeted consolidation

- CANON is the explicit default; preview remains opt-in and manifests state
  the actual profile, Shop availability, fingerprint and pool size.
- All 46 active costs are registered; two-kind policy keys no longer collide
  beyond the old twelve-digit encoding (1,035 distinct pairs covered).
- Retained MatchResult is 760 bytes; its removed inline combat payload was
  3,288 bytes (the replacement optional Box occupies eight bytes):
  combat totals are aggregated losslessly in batches of eight matchups.
  At 1,000 pairs / 1,654 matchups, observations retain about 2.52 GB instead
  of about 13.36 GB before allocator overhead. This is an estimate from
  measured struct sizes, not an observed peak-RSS measurement.
- Workers write indexed slots; bootstrap streams are independent and parallel.
  Missing observations now have infinite loss rather than rewarding NaN.
- Base-only duels and suite-only gate screening are explicit. Strict gates
  reject suite scope because it lacks the full-pool Nash evidence.
- Scoped Clippy (balance all targets) passed. Five audit regressions passed
  in 1.11 s; C3 worker-count invariance passed in 2.08 s. No workspace suite.

## Executed exploration

Commands use `.local/validate.sh`, the shared nonblocking host lock and
CARGO_BUILD_JOBS=2. Simulation concurrency was 16 workers, independent of
Cargo jobs. The frozen `.local/balance-audited/tbg-balance` was built from the
working-tree audit diff before the normalized threshold was added.

- `throughput --ruleset preview --pairs 32 --seed-start 0 --threads 16`:
  old 1/2/4/8/16-worker rates 54/108/195/295/334 games/s; audited rates
  54/108/196/302/328. Exact outputs matched across worker counts. CPU timing
  is within noise; only the memory reduction is substantiated.
- `gate --pairs 64 --seed-start 0 --threads 16`: 211,712 complete games,
  473.48 s, 447 games/s, CANON d365d09fafcb66a4, 46 kinds, no preview Shop.
  Discovery only. Machines-majority score 78.29%, heavy 65.63%, cheap 60.94%,
  13 oppressive bases and Nash support 1 / largest weight 0.99999.
- Twenty-four candidate runs plus the control used
  `run --a faction-machines --b bot --pairs 256 --seed-start 200000
  --threads 16 --apply <spec>`. They test independent HP, damage, range,
  cost, Synergy and Oilpress Evolution effects, then small combined groups.
  The control scored 97.3%; even eight-field Machine HP/damage reductions
  scored 95.1% / 94.9%. A diagnostic Siege Walker damage=1 canary scored
  89.5%; this deliberately extreme canary was never a release proposal.
  Candidate files and exact measurements are summarized in the final report.

The owner approved preserving all gameplay numbers and publishing the audit
and tooling (2026-10-31). No candidate is accepted; no numeric balance patch,
new mechanic, rating change, protocol bump or ruleset revision is shipped.
The banked-wave / concentration finding needs a separate rule proposal and
owner decision. Existing required gate thresholds are not weakened.

## Independent confirmation and owner decision

Runtime source: 8614fc6 (later audit test/comment additions do not alter the
simulation or runner). Command:

`.local/validate.sh .local/balance-final/tbg-balance gate --pairs 128
--seed-start 1000000 --threads 16 --strict
--out .local/balance-final/confirmation`

Actual result: **exit 1**, 423,424 complete games / 1,654 policy matchups,
1,029.85 s, 411 games/s. CANON fingerprint d365d09fafcb66a4, 46 kinds,
preview Shop disabled, reaction 5 views, neutral Arcanas, Classic Strongholds,
Forest. All 230 conventional forms were included in the duel matrix at
20/40/60 Energy, with both seat orders. This is offline evidence, not a
sample of human ranked matches or a claim about every arena/tower/build.

118 reported rows, 18 failing. Required failures: heavy vs Bot 68.75%
[60.55–76.56], cheap vs Bot 68.36% [60.55–76.17], 13 oppressive bases,
Nash support 1 and largest weight 0.999985. Machines-majority score was
77.10% [75.45–78.74]. Seat A point score 51.17% [48.05–54.30] is within
the unchanged point band, but this interval is too wide to certify a
1.5-point bias bound. Median duration 169.55 s; comebacks 23.81%
[22.42–25.23]. 82.07% of evolved forms lost their Energy-equal base duel.
Intervals are 95% paired-cluster bootstrap intervals, not human causal data.

Total execution here: 211,712 discovery + 12,800 diagnostic screening +
423,424 confirmation = **647,936 complete matches**, plus throughput runs
and deterministic Energy-equal duels. No confirmation seed selected a value.
The owner explicitly chose: preserve numbers and publish audit/tooling.
No candidate was accepted. Gameplay, rating, rewards, protocol 29,
RulesetVersion 2/source hash 3d56156049e626c5 and DB schema 8 are unchanged.

Committed safe aggregates are in docs/media/competitive-audit-20261031.
The original discovery CSV retains the former 2.5% classification; its
separate normalized CSV derives only the owner-approved 0.625/46 band from
identical observations. Confirmation uses the normalized criterion directly.

## Deferred competitive acceptance

`gate --pairs 1000 --seed-start 1000000 --strict`, a frozen numeric
candidate's multi-reaction confirmation, broader arena/Stronghold/Arcana
experiments and human playtests remain pending a separate rule-design phase.
No number is being accepted here, and a larger sample cannot turn the clear
required failures above into acceptance. Historical P16-C already identified
banked waves/concentration as a mechanic-level concern; this task does not
invent a deploy cooldown, cap or scaling cost without the owner's decision.
The owner's request authorizes this audit execution, not a full workspace
suite, extensive visual gallery or unrelated performance benchmarks.

## Focused integration checks

Level: targeted consolidation plus the owner's explicitly requested balance
and throughput runs. All Rust commands use `.local/validate.sh` (shared host
lock, CARGO_BUILD_JOBS=2); tests use --test-threads=1.

- `cargo clippy --offline -p tbg-client -p tbg-server -p tbg-balance --bins
  -- -D warnings`: passed (4.78 s on the final interface working-tree diff).
- `cargo test --offline -p tbg-balance --test audit`: 7 passed, 1.06 s;
  includes expanded-roster real-simulation HP, 1,035 composition identities,
  compaction/worker parity, normalized floor, missing-data rejection and
  rejection of strict acceptance on partial screening.
- `cargo test --offline -p tbg-balance --test contract
  c5_every_tunable_number_is_registered -- --exact --test-threads=1`:
  1 passed, 0.08 s.
- `cargo test --offline -p tbg-server --lib patch_notes::`: 1 passed.
- `cargo test --offline -p tbg-server --test patch_notes`: 1 passed; actual
  HTTP collection/detail, caching, unknown-id 404 and POST 405. Initial
  sandbox execution could not bind loopback: `Operation not permitted`;
  rerunning only this check with the required sandbox permission passed.
- `cargo test --offline -p tbg-server --lib version::`: 1 passed.
- `cargo test --offline -p tbg-client --bin tbg-client
  ui::shell::patch_notes::`: 1 passed (snapshot and malformed/duplicate/
  oversized-field feed rejection).
- `cargo test --offline -p tbg-client --bin tbg-client
  ui::glyphs::tests::ui_strings_are_drawable -- --exact --test-threads=1`:
  1 passed, including the public catalog's player-visible strings.
- `python3 scripts/test-patch-notes.py`: 5 passed; valid revisions plus
  missing notes, missing bump, duplicate IDs, unsafe document paths and
  unaccepted numeric evidence are covered.
- `python3 scripts/check-patch-notes.py --base origin/main`: passed,
  two bilingual notes matching current revision 2 / 3d56156049e626c5.
- `cargo fmt --all -- --check`, `git diff --check` and
  `node --check crates/tbg-client/assets/lobby/lobby.js`: passed.

No new dependency was introduced. The existing test-only Layout::clear
unused-code warning remains; production scoped Clippy has no warnings.
Native/server production, WASM/browser observations, package verification,
TBG-only deployment and local installation are recorded below when executed.
Full workspace tests/Clippy, the unrelated layout timing benchmark and broad
multiplayer/audio/reduced-motion galleries remain deferred by owner policy.

## Native visual review

`cargo build --offline -p tbg-client --bin tbg-client` passed (13.47 s).
The frozen debug binary captured the patch page at 1440x900 and lobby at
1280x720 under Xvfb, with Wayland variables removed and isolated data folders.
Both exited successfully through the opt-in capture plugin. Logs reported
boot_ready=true, scene_waiting=false, zero pending images/texts and zero failed
images. The patch hero preserves its 3:1 artwork ratio; the five change panels
use common dark metal/gold tokens and scroll below the shared header. At the
minimum desktop size, the news banner and patch CTA remain inside their panel.
No owner window was closed or preference directory modified by these captures.
The offline snapshot was observed with the server deliberately unavailable.
