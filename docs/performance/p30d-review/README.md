# P30-D independent review evidence (2026-10-30)

Same machine as the P30-D report (i7-13620H, 16 logical CPUs). Every
binary was rebuilt from source in its own worktree and target folder:
baseline = `ca5069e` (baseline core plus the benchmark harness; the
1,400-step reference takes `fixture.rs` from `0a5af9e`), P30-D = `feea791`,
review = `phase/p30d-review` head (lazy formation anchors and send_view).

- `timing.csv`: native release ticks on logical CPU 2, seeds 1, 7, 42.
  Small armies: 600 ticks, three interleaved repetitions per version
  (base, P30-D, review, base, ...). Large armies: 120 ticks, one pass.
  Speedups are the median of per-seed paired ratios.
- `allocations.csv`: separate `bench-alloc` builds, 20 ticks, seed 1
  (never compare with native time).
- `capacity-1000.json`: `scripts/performance_capacity.py`, 1,000
  matches, 60-s window, server on CPU 2 under CPUQuota 100 % and
  MemoryMax 1G, generator `tbg-bots` (review build) on CPUs 4-7.
- Exact traces: `scripts/performance.py --compare` at 1,400 ticks, sizes
  20 and 200, seven scenes, three seeds = 58,800 frames, equal for both
  P30-D and the review head.
- Natural games: `tbg-balance run` bot/bot (400 pairs, seeds from 9000),
  heavy/saver and cheap/evolver (300 pairs each, seeds from 12000);
  `roster.csv` and `run.txt` byte-identical between baseline, P30-D and
  the review head (2,000 games each).
