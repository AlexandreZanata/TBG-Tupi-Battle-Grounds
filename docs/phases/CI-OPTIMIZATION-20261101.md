# CI diagnosis and optimization — 2026-11-01

Part of the [consolidation](CONSOLIDATION-20261101.md). Owner rule: same
validation, less waste; no reduced coverage, `continue-on-error`, removed
asserts, retries, disabled Clippy/tests, raised timeouts or fake success for
skipped jobs. Branch protection requires exactly one context, `check`
(GitHub Actions); it was not changed.

Data: read-only `gh api` over every run from 2026-10-31T18:39Z (first run of
PR #29) to 2026-11-01T02:45Z on PRs #29–#33, their tags and main: 55
completed runs, **1,563.6 runner-minutes**. **Measured** = GitHub timestamps or
job logs. **Estimated** = derived from measured parts; not yet observed.

## 1. Where the time went (measured)

| Workflow | Event | Runs | Runner-min |
|---|---|---:|---:|
| ci | push (branches + tags) | 17 | 722.4 |
| ci | pull_request | 17 | 501.1 |
| Native Linux release | pull_request | 16 | 295.2 |
| Native Linux release | push (tags) | 2 | 44.2 |
| Desktop UI contract | pull_request | 3 | 0.7 |

- Queue time never exceeded 0.2 min: runners were not the bottleneck.
- **47% avoidable** (739.4 runner-min): a second run of the same workflow on
  the same head SHA (565.5) or a superseded PR run nobody cancelled (483.4;
  the sets overlap). Main's old `ci.yml` runs on every branch push *and* on
  `pull_request`; tag pushes re-ran CI on commits a PR had validated.
- Critical path of the split `ci` (#29 tip, #31–#33) is always
  `quality` → `native` → `check`, and `native` is serial: setup, cache restore,
  apt, Clippy, workspace test build, then about 60 test binaries one by one.
  Warm green run: **10.4 min** ([run 41258861477](https://github.com/AlexandreZanata/TBG-Tupi-Battle-Grounds/actions/runs/41258861477)).
  Cold: **22.0–26.3 min** to a failing `check`.
- Main's old single job: 40–75 min; the balance contract alone took 32–65 min
  without `.github/cargo-ci.toml`.
- Native release on PRs: 5.0–6.7 min warm, 19.8–31.0 min cold; 14 of 18 runs
  missed the cache.

| Phase | Warm | Cold |
|---|---|---|
| Clippy, whole workspace | 15–16 s | 4m21s–5m28s |
| Test compile, whole workspace | 46–57 s | 11m32s–13m14s (≈10.5–12.4 min dependencies) |
| Test execution, serial | 204–479 s | same |
| WASM check | 7–13 s | 3m32s–4m47s |
| Cache restore / save | 4–28 s / skipped on exact hit | — / 8–20 s |

In the green run, the balance contract took 274.7 s, `spectate` 74.2 s,
`online` 44.8 s, `leaderboard` 40.6 s; every other binary under 25 s. The
balance contract needs only tbg-core but waited behind the Bevy build.

## 2. Root causes, by wall-clock impact

1. **Cold caches.** Main never ran the current workflow, so it held no entry
   for the current keys and PRs/tags (which can restore only their own or the
   base branch's entries) started cold. A PR #33 run cancelled 17 s after it
   started still saved 150 MB under the exact key (`cache-on-failure: true`
   also saves on cancellation); the next run restored it as a full match,
   recompiled for minutes and never refreshed it. The floating `stable`
   toolchain (1.99.1 on runners vs 1.99.0 pinned by #33's release build) and
   tag-scoped entries added more misses. Usage reached 11.24 GB, over the
   10 GB limit, and an entry was evicted.
2. **One serial lane** (above).
3. **Main's old single-job file still live.**
4. **Duplicate and uncancelled runs**, which also churned the cache budget.
5. `CARGO_BUILD_JOBS=2` on 4-vCPU hosted runners (estimated 1.3–1.8× on cold
   compiles; no four-job cold build exists in the window to measure).

Other measured findings: no job had `timeout-minutes` (one native-release job
sat in apt for over 2.5 h); `v0.1.0-native.2`, `native.3` and `p68.1` were
uploaded manually while CI on those commits failed, and the #31/#32 publish
step reported success for an already existing release (#33's file already
refused that); the split `ci.yml` cancelled in-progress runs for every event,
which would also cancel main runs.

## 3. Changes (commit `cc078a0`)

- **Parallel lanes**: `quality`, `clippy`, `test`, `balance`, `wasm`,
  `desktop-ui`; `check` keeps its id, `if: always()` and the fail-closed
  `scripts/ci-required-checks.py` over all six. Failure, cancellation,
  skipping, a missing lane, an extra lane or malformed input fails `check`.
- **Test split, same coverage**: `test` runs
  `--workspace --exclude tbg-balance --features tbg-core/lab`; `balance` runs
  `-p tbg-balance --features tbg-core/training`. Together they select the same
  targets as `cargo test --workspace` with the same tbg-core features
  ({lab, training}); the balance scope differs only by two additive
  build-time third-party features that cannot change results.
  `scripts/ci-test-lanes.py` recomputes both scopes from the manifests on every
  run and fails on drift (13 positive/negative unit tests). Each lane compiles
  with `--no-run` first; `--no-fail-fast` reports every failing binary.
- **Unchanged validation settings**: `.github/cargo-ci.toml` (debug assertions
  and overflow checks on, opt-level 2 only for tbg-core/tbg-balance tests),
  `--locked`, Clippy `-D warnings`, the client/core WASM check, patch-note and
  required-check guards. No benchmark exists in the workspace; none moved.
- **Desktop UI contract** (P68's separate optional workflow) is now the
  required `desktop-ui` lane: stricter, about 15 s.
- **Cancellation** only for superseded runs of the same pull request.
- **Caches**: explicit runner-image/target/profile keys (rust-cache also hashes
  rustc, env, manifests and Cargo.lock); tags never save; test lanes re-arm
  saving on failure only after their tests compiled, so a cancelled or broken
  compile cannot poison a key. Main pushes seed the release cache.
- **Pinning**: rustc 1.99.0 in every lane and the release build; every action
  pinned to a full commit SHA with its version.
- **Timeouts** of about 2–3× the measured cold worst case, lowering GitHub's
  360-minute default; nothing is raised.
- **Release gate**: a `v*-native.*` tag runs the complete `ci.yml` as
  `validate` in parallel with the build; `publish` needs both. Artifacts come
  only from the same run (the binary embeds its source commit). Existing
  releases are still refused.
- `CARGO_BUILD_JOBS` is no longer forced to 2 on hosted runners; the two-job
  limit remains the local-desktop policy of `docs/VALIDATION.md`.

Not proposed, on purpose: path-based skipping of Rust lanes (the code embeds
`index.html`, `assets/**` and `docs/patches/catalog.json` through `include_*!`
and runtime reads, so no path set is provably irrelevant), new test runners or
name-sharding (risk of silently matching zero tests), cross-run artifact reuse.

## 4. Coverage map

| Check | Before | After |
|---|---|---|
| `cargo fmt --all -- --check` | `quality` | `quality` |
| Required-check guard tests | `quality` | `quality` (now also: every lane gated, pinned actions, no `continue-on-error`, PR-only cancellation) |
| Test-lane split guard | — | `quality` (new) |
| Patch-note tests and gate | `quality` | `quality` |
| Clippy, workspace, `-D warnings` | `native` | `clippy` |
| `cargo test --workspace` (cargo-ci.toml) | `native` | `test` + `balance` (same targets/features) |
| WASM check | `wasm` | `wasm` |
| Desktop UI source routing, comparator tests, browser auth coordinator tests | optional workflow | required `desktop-ui` |
| Aggregator | `check` over 3 | `check` over 6 |
| Native client tests (shipped features), release build, packages, AppImage digest, upload | release `linux` | release `linux` (test and build in separate steps) |
| Publication | needs `linux` | needs `linux` and `validate` |

## 5. Expected critical path (estimated, not yet observed)

| Scenario | Before (measured) | After (estimated) |
|---|---|---|
| Warm PR, all green | 10.4 min | ≈5.5–6 min (balance contract ≈4.6 min is the longest lane) |
| Cold PR | 22.0–26.3 min to a failing `check` | ≈12–17 min, rarer once main seeds caches |
| First tbg-core failure visible | ≥3.4 min into test execution, behind the balance contract | ≈1 min into the `test` lane |
| Release tag | `ci` and release independent; publish ungated | max(validate, build) + publish |
| Cache usage | 11.24 GB | ≈2.4 GB main + ≈1.3 GB per open PR |

To measure after merge: the first main `ci` and native-release runs (seeding),
a PR's restore line from main's key, lane durations, the cold compile at four
jobs and `gh api repos/AlexandreZanata/TBG-Tupi-Battle-Grounds/actions/cache/usage`.

## 6. Validation and limits

Executed locally: both workflows parse; the required-check tests (9) and the
lane-split tests (13) pass on the final consolidated manifests; the lane guard
reports the expected scopes; aggregator probes return 0 only when all six lanes
succeed (cancelled, failed, skipped, missing, extra, malformed and empty input
return 1). The required context name and branch protection are unchanged.
The toy-workspace check of the exact split `cargo test` invocations is recorded
in the consolidation report. Not verifiable before a GitHub run: `workflow_call`
with concurrency and the cache re-arming step; if the re-arming did not take
effect, the lanes would behave like today's `cache-on-failure: true`. Deciding
whether `desktop-ui` stays required and whether main should seed the release
cache are owner choices; both are reversible workflow edits.
