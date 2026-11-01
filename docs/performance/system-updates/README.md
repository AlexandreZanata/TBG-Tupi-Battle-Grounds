# Exploratory server profile comparison

Hardware: Intel Core i7-13620H, CPU affinity 2, mixed scenes with 30 and
100 target units, 600 ticks, seeds 1/7/42. Other development builds were
active; runs were not interleaved. `release-size` is workspace release;
`release-speed` tested opt-level 3, fat LTO, unwind. The candidate did not
show the required >=5% gain and was removed. Deploy is unchanged. These
samples are not a controlled capacity or general profile recommendation.

CSV rows and metadata retain binary SHA-256 and arguments. The working
source was an intermediate tree; a full reproducible confirmation and
1,000/2,000-match cgroup comparison are deferred under the owner's updated
validation policy. No allocator, target-cpu, PGO, shader warmup, dynamic
resolution, blob shadows, VFX pooling, LOD or WASM profile change was adopted.
The client now provides repeatable bench scenes and opt-in metrics for that
future round. CI budgets remain uncalibrated; existing CI is unchanged.
