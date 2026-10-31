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
- Retained MatchResult is 760 bytes rather than 760 + 3,288 combat bytes:
  combat totals are aggregated losslessly in batches of eight matchups.
  At 1,000 pairs / 1,654 matchups, observations retain about 2.52 GB instead
  of about 13.39 GB before allocator overhead. This is an estimate from
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
