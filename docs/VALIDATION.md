# Local validation workflow

**Owner policy, 2026-10-31.** Keep the development computer responsive while
several work fronts run. Use minimal checks per front, consolidate targeted
tests once, and run the full suite only when the owner explicitly requests
it. [AGENTS.md](../AGENTS.md#verification) is the governing instruction.
Older phase command lists describe acceptance coverage; they do not
override this execution policy. Historical reports remain evidence of what
ran at the time, not instructions to repeat those commands.

## 1. Work fronts: minimum useful validation

- Documentation and comments: inspect the changed text and relative links;
  run `git diff --check`. No Rust compilation or tests.
- Assets or scripts: validate the changed format or use an existing cheap,
  focused check. Do not rebuild the game solely to check a file format.
- Rust: check formatting for the affected package. If its current cache is
  usable, run one focused compilation check on the final change, narrowed
  to the affected package, library/binary, features, and actual platform.
  For a browser-only client change, select WASM rather than also building
  the native client. A cold Bevy build or an additional platform build is
  deferred to consolidation.
- Write or update meaningful regression tests where correctness requires
  them. Queue their exact commands; do not run `cargo test` on every front.
  A specific owner request to run a test authorizes that test alone.
- Do not expand a failed focused check into workspace validation. Fix the
  relevant issue, rerun only the affected check when needed, and report
  unrelated failures without changing other fronts' files.

No automatic workspace check/clippy/test, release build, packaging, balance
gate, benchmark, ignored/soak/load test, browser automation campaign, or
multiplayer match matrix belongs in this level. A cheap inspection of an
already running browser is fine; rebuilding for a complete gallery is not
the default. Record unobserved behavior honestly.

## 2. Share one compilation slot across worktrees

All local agents use the **same host lock file per user**, regardless of
branch, checkout, `CARGO_TARGET_DIR`, or target platform:

```bash
# Example only: choose the package and target relevant to the change.
flock -n -E 75 "/tmp/tbg-validation-$(id -u).lock" \
  env CARGO_BUILD_JOBS=2 cargo check -p tbg-core --lib
```

- Acquire this lock for every validation command that compiles Rust or
  runs Rust tests, Clippy, Trunk, packaging, or the balance lab. Hold it for
  the entire command or sequential validation batch. Formatting and static
  text checks need no compilation slot.
- Set `CARGO_BUILD_JOBS=2` explicitly and do not override it with higher
  `-j` values. Test concurrency is separately limited with `--test-threads=1`
  when tests are authorized. Browser clients needed by one multiplayer
  scenario are part of that single validation, not independent campaigns.
- Exit **75** means another validation owns the slot. Record **deferred:
  shared validation slot busy** and continue independent work. Do not
  automatically retry, sleep/poll, or bypass the lock. A different command
  failure keeps its own exit status and must be reported as a failure.
- If `flock` or the shared host lock is unavailable, defer compilation;
  do not substitute an isolated per-worktree lock. Existing builds started
  outside this policy are allowed to finish; do not terminate them to
  acquire capacity.
- Reuse the worktree's existing build cache and target directory. Do not
  run `cargo clean`, delete caches, or create a new target directory just
  to avoid a Cargo lock. Never change another front's active build setup.
- For long-running development servers, build under the lock, then start
  the resulting binary outside it; do not hold the compilation slot for
  the server's lifetime. Preserve existing windows, sessions, and services.

## 3. Consolidation: targeted tests once for the combined changes

When several fronts are brought together, designate one reviewer to own
local validation on one integrated tree. Each front hands over its changed
areas, checks already run, exact deferred commands, and relevant risks.

The reviewer chooses the union of tests relevant to the combined diff,
deduplicates repeated requests, and runs that selection once under the
shared lock. Prefer a named integration-test binary and a test-name filter
over an entire crate; prefer an affected crate over the workspace. For
example, select a real target from `crates/tbg-core/tests/` and run
`cargo test -p tbg-core --test <target> <filter> -- --test-threads=1`
inside the same lock wrapper. A name filter still compiles that target, so
it is not a reason to run tests separately on every front.

Check the relevant platform once. For client/rendering changes, combine the
necessary build and desktop browser observations; for multiplayer changes,
combine the required two-client match and reconnect scenario. Heavy release
builds, full galleries, soak/load runs, and balance experiments remain
deferred unless the owner specifically requests them. A native check never
proves browser compatibility. Keep mobile out unless explicitly requested.

A request to consolidate, review, finish a phase, or merge does **not** mean
"run the full suite". Do not rerun unchanged validation on every source
branch after consolidation. Repeat a check only when relevant code,
dependencies, features, or target settings change, or a failure remains
unresolved. Keep D-33 evidence requirements: balance values cannot be
accepted without the required measurements; deferred evidence is pending,
not a reason to weaken contracts or gate thresholds.

## 4. Full suite: explicit owner request only

An explicit request such as "run the full suite" authorizes the following
baseline once on the consolidated tree. These are reference commands,
**not a work-front checklist**. A request for one named heavy check does
not authorize the rest. Run commands sequentially and stop at a failure:

```bash
flock -n -E 75 "/tmp/tbg-validation-$(id -u).lock" \
  env CARGO_BUILD_JOBS=2 bash -eu -c '
    cargo fmt --all -- --check
    cargo clippy --workspace -- -D warnings
    cargo test --workspace -- --test-threads=1
    cargo check -p tbg-client -p tbg-core --target wasm32-unknown-unknown
    (cd crates/tbg-client && trunk build --release)
  '
```

Add only applicable acceptance scenarios covered by the owner's requested
scope. D-33 balance evidence can require a release lab-equivalence match
and `cargo run --release -p tbg-balance -- gate --pairs 1000`; schedule these
once when explicitly requested, under the same lock. Do not automatically
add ignored tests, long simulations, packaging, or repeated benchmarks to
the baseline. Full-suite permission applies to the requested validation
round, not every subsequent task.

## Evidence and handoff

Every report states:

- Level: work front, consolidation, or explicitly requested full suite;
  for full/heavy runs, cite the owner's request.
- Checks actually run: exact commands, results, and code revision (or
  description of the working-tree diff when changes are uncommitted).
- Deferred checks: exact command or browser scenario, reason, and whether
  it belongs to consolidation or requires an explicit full/heavy request.
- Known failures and limits: keep these distinct from deferred work; do
  not report a skipped check as passing or claim release readiness from it.

New phase specs separate minimum work-front checks from consolidation and
explicit full-suite acceptance. Existing remote CI remains configured as
it is; this local policy neither disables CI nor authorizes agents to
reproduce its full workload locally without the owner's request.
