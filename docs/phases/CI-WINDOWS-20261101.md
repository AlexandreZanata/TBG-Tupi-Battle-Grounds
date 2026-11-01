# Windows native CI: identity correction and build efficiency

Owner request: diagnose PR #40's Windows failure and reduce its elapsed time
without removing validation. Level: focused script/workflow consolidation.

## Observed failure and timings

Native run [41354006947](https://github.com/AlexandreZanata/TBG-Tupi-Battle-Grounds/actions/runs/41354006947),
PR head `2738511`, tested merge `d95d4b265080dc44584f1e264ad1f0451adfd6dc`:

- Linux job `124990081295`: successful, approximately **6 min 26 s**. It restored
  a warm dependency cache; native tests took 55 s and the release build 262 s.
- Windows job `124990081556`: failed after approximately **26 min 10 s**. Its
  dependency cache was cold. Release test compilation took **16 min 15 s**;
  all four Windows platform tests passed. The release application then compiled
  in **9 min 08 s** and passed PE x64, GUI subsystem, resources and import checks.
- Failure was the subsequent Python portable packager's version/source check.
  Source environment and checkout both identify `d95d4b2`; this was not a
  missing MSVC toolchain or failed Rust regression.

Rust emits UTF-8 for `--version`, including the middle-dot separator. Python 3.11
`text=True` inherits the Windows locale encoding; CP1252 turns `C2 B7` into two
characters and makes the old version substring test fail. A focused regression
reproduces the failure with CP1252 and verifies the fix. An actual older Windows
TBG CLI in the existing VM emitted `C2 B7`; UTF-8 decoding matched the separator,
CP1252 decoding did not. That older binary is evidence for the encoding boundary,
not runtime acceptance of the new MSVC candidate.

## Delivered changes

- Decode CLI output and source text explicitly as UTF-8. Read structured
  `--build-info` first, retain exact commit/version/release-profile checks and
  cross-check `--version`. Report the mismatching field and actual value.
  CLI probes have a 30-second timeout; invalid UTF-8 remains an error.
- Run the locale/source regressions and compile the real Inno installer script
  using a tiny fixture before compiling Rust. The fixture uses the real icon and
  reviewed compiler, languages, constants and locking code. It is never installed,
  launched, uploaded or published; its unique temporary directory is removed.
- Let Cargo use the hosted runner's available CPUs, as Linux already does.
  Microsoft's dedicated GitHub runner is not the owner's shared local machine;
  local compilation still requires the shared nonblocking lock and two jobs.
- `.github/cargo-windows-release.toml` splits only the large client application
  and test harness into 16 codegen units for parallel compilation. Dependencies
  keep their existing release settings. Size optimization, ThinLTO, assertions,
  feature selection, MSVC target and static CRT are unchanged. Both test and
  application builds use the same explicit configuration.
- Reuse dependency caches within same-repository PR merge refs. Main and tags
  cannot restore a PR-scoped cache; fork PRs remain restore-only. Cache failure
  saving is disarmed after restore and re-armed only after tests compile, retaining
  completed dependencies if a later test or packaging step fails. No workspace
  binary, package staging directory or credentials are added to the cache.
- Split compilation, regression execution and packaging into observable steps;
  retain Cargo HTML timings for seven days. Publishing selects the existing
  package artifact names, excluding the new timing artifact. Promotion still
  requires successful exact-source builds and all existing required CI lanes.

This changes build/packaging mechanics, not game rules, frontend presentation or
P66 update/signing custody. It does not certify visual parity or ship a new build.
The PR's ordinary six-lane CI on `2738511` passed separately in run `41354006965`.

## Validation executed

- `python3 scripts/test-native-package.py`: **6 passed**, including the CP1252
  reproduction, correct UTF-8 output, stale commit/version/debug rejection,
  inconsistent CLI/invalid-UTF-8 rejection, bounded runner invocation, matching
  Windows build/test options and preserved cache/publication boundaries.
- `python3 scripts/test-native-artifacts.py`: **4 passed**, preserving complete
  source/lock/features/target/checksum requirements for native promotion.
- `python3 scripts/test-ci-required-checks.py`: **9 passed**, all required lanes
  still fail closed; actions remain pinned and only superseded PRs cancel.
- `actionlint -ignore SC2086 .github/workflows/native-release.yml .github/workflows/ci.yml`:
  **passed**. The local exclusion covers the pre-existing intentional shell scope
  expansion; no CI suppression was added.
- `cargo --config .github/cargo-windows-release.toml metadata --offline --locked --no-deps --format-version 1`:
  **passed**, manifest/configuration resolution only, no compilation.
- Actual Windows PowerShell parser: **passed** for `windows-env.ps1`,
  `package-windows-installer.ps1` and `test-windows-installer.ps1`.
- `python3 -m py_compile scripts/package-native.py scripts/test-native-package.py`: **passed**.
- `python3 scripts/check-patch-notes.py --base origin/main`: **passed**, six entries, unchanged ruleset.
- `git diff --check`: **passed**.

The Windows VM CLI encoding probe passed using the existing isolated job agent.
Its raw bytes and command output are retained privately under the worktree's
`.local/windows-ci-optimization` / the VM's own test results, outside Git.

## Local installer-tool limitation

> blocked: provisioning the official Inno 6.7.1 compiler inside the existing VM
> timed out twice (normal mode, then explicit `/CURRENTUSER` with a user-writable
> destination). Both returned `agent: job killed after the 90 s timeout` (exit 124).
> The downloaded installer matched GitHub's SHA-256
> `4d11e8050b6185e0d49bd9e8cc661a7a59f44959a621d31d11033124c4e8a7b0`
> and its Windows Authenticode status was `Valid`. The suspected cause is an
> interactive setup/environment boundary, not the TBG installer compile step;
> neither attempt reached that step. Stop provisioning retries under AGENTS.md.

The GitHub Windows image already has the reviewed compiler: the previous run's
`windows-env.ps1` passed that file/version inventory. The new early fixture compile
must run there. Local compiler installation and fixture compilation are **not**
reported as passing. No TBG executable or installer was installed in these probes.

## Deferred checks and limits

- The next native Windows run must complete release compilation, all four
  platform regressions, PE/resources/import checks and real portable/installer
  packaging. No full local Rust workspace suite, cold Bevy rebuild, balance gate
  or benchmark was started for this script correction.
- **The new elapsed time is unmeasured.** A cold Windows build is not comparable
  to a warm Linux run; 6 min 26 s is a reference, not a promised SLA. The first new
  cache/configuration entry may be cold. Compare the emitted per-stage timings
  and a subsequent warm run before claiming equivalent speed.
- Per the owner request, push the correction without waiting for the new general
  CI to finish. Remote success is pending, not inferred from these local checks.
- New Windows executable runtime, installer lifecycle and Linux/Windows/web visual
  correspondence remain separate acceptance. An older VM executable or passing
  packaging scripts cannot certify the final candidate. No release, VPS deployment,
  merge or Linux reinstallation is required for this CI-only correction.

## References

- [Python subprocess text encoding](https://docs.python.org/3.11/library/subprocess.html).
- [Cargo codegen units and profile overrides](https://doc.rust-lang.org/cargo/reference/profiles.html#codegen-units).
- [GitHub public hosted runner resources](https://docs.github.com/en/actions/reference/runners/github-hosted-runners#standard-github-hosted-runners-for-public-repositories).
- [GitHub cache scope isolation](https://docs.github.com/en/actions/reference/workflows-and-actions/dependency-caching#restrictions-for-accessing-a-cache).
- [Pinned rust-cache restore implementation](https://github.com/Swatinem/rust-cache/blob/d1527c8f67ef9af357290475d4af4b0fab8f12e0/src/restore.ts)
  and [post-step condition](https://github.com/Swatinem/rust-cache/blob/d1527c8f67ef9af357290475d4af4b0fab8f12e0/action.yml).
