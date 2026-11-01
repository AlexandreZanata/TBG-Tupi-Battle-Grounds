# CI efficiency and obsolete build cleanup

## Scope and baseline

Owner request, 2026-10-31: clean obsolete local builds first, then make
PR #29 CI efficient without weakening checks. P66 native updates and release
publication belong to another active front and are excluded. Work started
from `55510c5363e3f1e555fa5133208a5faf20c5825b` in an isolated worktree.
Normal PR updates are authorized; no merge or history rewrite is performed.

## Cleanup completed before CI changes

Removed exactly 30 obsolete `0.1.0-native.1` development packaging previews
under `target/native-preview/`, after checking their build metadata and
confirming no process used their executable or working directory. The same
host validation lock protected deletion. Free space increased by
7,248,359,424 bytes (**6.75 GiB**), from approximately 2.0 to 8.8 GiB at the
time of cleanup; subsequent focused compilation uses some of that space.

All shared Cargo compilation caches, installed application versions,
preferences, published release artifacts, source worktrees and active
sessions were retained. No `cargo clean`, global cache purge, GitHub cache
deletion or P66 change was performed. The private local deletion inventory
is retained outside Git under `.local/cache-cleanup-20261031/` in the primary
checkout.

## What actually took time

[CI run 41235942833](https://github.com/AlexandreZanata/TBG-Tupi-Battle-Grounds/actions/runs/41235942833)
finished after approximately **54 min 04 s**, rather than remaining stuck.
Its dependency cache restored successfully in about 21 s. Clippy took
47.53 s and workspace test compilation took 6 min. The seven balance
contracts alone took **2,786.31 s (46 min 26 s)** and all passed. C7 and C8
dominated that time.

The ordinary test profile did not optimize workspace simulation packages.
C8 evaluates 230 forms: 26,335 unordered pairs times three Energy budgets
times both seat orders, or **158,010 duel simulations**. C7 also checks
planted imbalances for the full roster. This is real deterministic work,
not a reason to remove contracts or reduce their sample sets.

The workflow ran the same revision for both feature-branch `push` and
`pull_request`, without superseded-run cancellation. After the balance
contracts passed, `br_ag13_a_replay_keeps_paused_choices_in_order` failed:

```text
assertion `left == right` failed: two paused rounds were replayed
  left: 2
 right: 3
```

Its arbitrary repeated purchases could finish combat before the third
Augment round with the expanded roster. The replay fixture now purchases
one second before that round, retaining live participants without making
pause/replay ordering depend on combat balance. Both pauses, delayed
choices, forgoing, final hash, clock and tick comparisons remain asserted;
recorded intentions must now succeed instead of silently ignoring errors.
Production simulation and balance values are unchanged.

## Delivered workflow

- Run PR CI once per pull request revision; keep `main` pushes, tags and
  manual dispatch. No path filters or documentation shortcuts omit checks.
- Cancel superseded runs only within this CI workflow and the same PR/ref.
  Native release/P66 workflows have a separate identity and are untouched.
- Run native Clippy/workspace tests and WASM checking in parallel after
  formatting and lightweight guards pass.
- Preserve the required `ci / check` status with an always-run aggregator.
  Failed, cancelled, skipped, absent or malformed upstream results fail
  closed. Branch-protection settings are not changed.
- Retain `cargo clippy --workspace -- -D warnings`, the **complete**
  `cargo test --workspace` selection and the client/core WASM check. Keep
  feature unification and all existing contracts, matrices, seeds and
  thresholds. Add `--locked` to prevent dependency lockfile drift.
- Apply package-specific `opt-level = 2` to `tbg-core` and `tbg-balance`
  **tests only**, via `.github/cargo-ci.toml`. Keep debug assertions and
  overflow checks enabled. Local default, development and release profiles
  do not change. No `--release` substitution disables debug guards.
- Separate native/WASM cache keys and allow dependency cache saves after a
  failed check. Cache hits do not skip compilation or tests. Use read-only
  token permissions and do not persist checkout credentials.

No useful test was deleted or ignored. The balance contract file is
unchanged. No new dependency or gameplay/protocol/storage change is added.
`.github/workflows/native-release.yml` is unchanged; its PR #29 baseline
SHA-256 remains
`a4fa4e0d63f7adc0280a6d014aba96dda3d4b9347a4b67e5b3db6424e82130d4`.

## Executed validation

Level: targeted consolidation of the CI configuration and replay fixture,
not a local full workspace run. Rust commands used the shared nonblocking
`/tmp/tbg-validation-1000.lock`, the existing shared target cache and
`CARGO_BUILD_JOBS=2`; tests used `--test-threads=1`.

- `cargo test --offline --locked -p tbg-core --test augment_pause -- --test-threads=1`:
  **9 passed**, 0 failed, 0 ignored; test runtime 0.06 s.
- `cargo --config .github/cargo-ci.toml test --offline --locked -p tbg-balance --test contract -- --test-threads=1`:
  **7 passed**, 0 failed, 0 ignored; compilation 9.89 s, tests **150.12 s**,
  measured command wall time **160.03 s**. C7 and C8 remain included.
- `rustfmt --check crates/tbg-core/tests/augment_pause.rs`: passed.
- `python3 scripts/test-ci-required-checks.py`: **3 passed**, including
  failure/cancellation/skipping/missing-lane cases. Direct all-success and
  skipped-native CLI probes returned 0 and 1 respectively.
- `actionlint .github/workflows/ci.yml`: passed, official actionlint
  v1.7.12 downloaded into ignored local tooling after SHA-256 verification.
- YAML/TOML parsing and `git diff --check`: passed.

The local 150.12 s contract runtime demonstrates the unchanged contracts
complete with the optimized test configuration. Different hardware,
parallel test scheduling, compilation and cache state prevent treating
this as a measured GitHub speedup or promising a specific remote duration.
The first run of each new cache key may be cold.

## Deferred checks and boundaries

- The complete native Clippy/test and WASM jobs execute remotely with the
  next PR revision. The owner requested publication without awaiting CI;
  their outcome and new total duration are **pending**, not passing claims.
- No local `cargo test --workspace`, workspace Clippy, WASM/client release
  build, packaging, balance gate, benchmark, browser gallery or multiplayer
  matrix was run for this change. These unrelated broad checks require the
  appropriate explicit validation scope; a native test is not browser
  compatibility evidence.
- No VPS rollout, installation, release publication, main merge, forced
  update, data deletion outside the specified previews or P66 edit.

## Propagation to the audit/patch-notes PR

PR #31 (audit baseline `c61a744`) integrates the PR #29 corrections with a normal local branch merge,
preserving both histories and the audit's later code. The mandatory patch
comparison-base fetch, all five patch-note regression tests and the public
catalog/ruleset gate remain unconditional in the `quality` lane. Both
compiling lanes depend on that lane; a patch-note failure cannot produce a
green required check. P66 and the native release workflow are not edited.

Additional checks on this integrated tree:

- `python3 scripts/test-ci-required-checks.py`: 3 passed.
- `python3 scripts/test-patch-notes.py`: 5 passed.
- `python3 scripts/check-patch-notes.py --base origin/main`: passed, two
  entries, RulesetVersion 2 / source hash `3d56156049e626c5`.
- `actionlint .github/workflows/ci.yml`: passed.
- `cargo --config .github/cargo-ci.toml test --offline --locked -p tbg-core --features lab --test augment_pause -- --test-threads=1`:
  9 passed, 0 failed, 0 ignored, 0.01 s with the optimized profile and lab
  feature used by workspace feature unification.
- `cargo --config .github/cargo-ci.toml test --offline --locked -p tbg-balance --test contract -- --test-threads=1`:
  final rebuilt-package run **7 passed**, 0 failed, 0 ignored, **148.23 s**;
  compilation 4.83 s, command wall time **153.09 s**. All seven contracts,
  including C7/C8, are unchanged from PR #29. Both Rust commands used the
  same shared host lock, two Cargo jobs and existing target directory.

An initial local helper invocation exited 126 before Cargo (the ignored
helper is not executable); invoking it through `bash` corrected that.
A preliminary contract attempt was interrupted with exit 143 before
completion and is not counted as passing evidence. Its shared cache also
required invalidating the lab package's source timestamp before compiling
the later audit code; no cache deletion or source-content change was used.
The final targeted contract result above is the completed integration evidence.

## References

- [Cargo profiles and package overrides](https://doc.rust-lang.org/cargo/reference/profiles.html)
- [Workflow concurrency and cancellation](https://docs.github.com/en/actions/how-tos/write-workflows/choose-when-workflows-run/control-workflow-concurrency)
- [Workflow event filters](https://docs.github.com/en/actions/reference/workflows-and-actions/events-that-trigger-workflows)
- [rust-cache configuration and failure caching](https://github.com/Swatinem/rust-cache)

## PR #31 follow-up: roster-independent Shop regression

The latest head `014b0e0` failed native run 41242952250 in
`br_s04_a_purchase_redraws_all_five_offers_and_nothing_rotates`:
`Shieldbearer shown 42 of 400`. The test retained a 25-kind assumption
(5/25 inclusion, 80 expected appearances), although online CANON contains
46 kinds (400*5/46 = about 43.48). This is a stale sampling fixture, not a
production Shop bias. The separate `check` failure correctly propagated
`native: failure`; format, Clippy and WASM lanes passed.

Scale the deterministic purchase sample linearly with the actual CANON pool:
16 draws per kind, so each kind and the just-purchased kind still expect
80 appearances. Preserve the original 50..110 bounds, five distinct offers,
pool membership, no timed rotation and greater-than-97.5% redraw checks.
For 46 kinds this is 736 purchases; no gameplay source, balance threshold,
required status or security check changes. Future roster additions no longer
silently invalidate a fixed 25-kind probability. Work remains small and
linear in roster size.

Directed consolidation (shared nonblocking host lock, CARGO_BUILD_JOBS=2):
`cargo --config .github/cargo-ci.toml test --offline --locked -p tbg-core
--test rules -- --test-threads=1` passed all 14 tests (5.26 s build, 0.02 s
execution). Affected package formatting passed;
`python3 scripts/test-ci-required-checks.py` passed all 3 fail-closed guard
scenarios; `git diff --check` passed. No full workspace suite, balance gate,
benchmark or remote CI wait was executed. A normal PR branch push is
explicitly authorized; the owner will review CI and merge manually.
