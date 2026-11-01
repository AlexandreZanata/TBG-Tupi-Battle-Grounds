# CI balance C5 reconciliation — 2026-11-01

## Reported failure and cause

Owner request: repair PR #40's failing balance/check jobs and prevent recurrence
without weakening validation; publish the correction without waiting for new CI.

Run `41352567258` tested PR head `275edca145017b97396e616d778d8b96617330ae`
against main `0e23a1ac2f601b182bcfcc03b25acce76e69f962` (merge `07b96aba`).
Balance job `124985937619` failed only C5 at `knobs::apply`: even reapplying
`energy_start` ran the whole role validator, which rejected Farwatch's unchanged
12-tile base range after P71 removed its `NoStronghold` restriction. P71 already
allows it to attack a Stronghold at that range; no Energy parameter was broken.

C3/C6/C7/C8/C9/C11 passed in that run. The contract binary still spent 373.33 s
finishing simulations after the quick C5 failure. Check job `124987725204` then
correctly rejected the failed balance lane; all its other required lanes succeeded.
The aggregator is not independently broken and remains fail-closed.

## Delivered correction

D-102 reconciles the lab invariant with the already documented P71 owner rule.
Farwatch's allowed candidate range is the greater of the candidate Stronghold
range and its registered canonical range. Read that number from CANON, rather
than duplicating a literal or exempting every canonical unit. Preserve the ordinary
Stronghold cap, the original `NoStronghold` exemption and every other role invariant.
Out-of-bound errors include the actual candidate and allowed range.

Four boundary scenarios cover active canonical targeting, the Farwatch ceiling,
all other conventional Stronghold attackers, the ability-based exemption and a
larger experimental Stronghold range. The historical C5 contract is untouched.
No simulation values, core source, protocol, ruleset hash, balance threshold,
client presentation or visual correspondence changes.

CI now runs exact C5 before the multi-minute lab simulations, with the same checked
package scope, unified features and optimized profile. If it fails, the job stops
before C7/C8. If it succeeds, the complete existing lane still runs unfiltered,
including C5; the tiny repeated canary is deliberate, not a coverage exclusion.
The scope guard reuses the required-check parser and rejects missing/reordered
preflight, feature bypass and filtered full tests. No retries, ignored tests or
relaxed required checks were added.

## Validation actually executed

Level: targeted consolidation of this lab/CI correction.
Rust compilation used the shared nonblocking `/tmp/tbg-validation-1000.lock`,
`CARGO_BUILD_JOBS=2`, the existing root target cache and one Rust test thread.

```sh
cargo fmt -p tbg-balance -- --check
flock -n -E 75 "/tmp/tbg-validation-$(id -u).lock" \
  env CARGO_BUILD_JOBS=2 \
  CARGO_TARGET_DIR=/home/iiii/PESSOAL-PROJETOS-ALEXANDRE/TBG-Tupi-Battle-Grounds/target \
  bash .local/check-ci-balance-fix.sh
```

The scoped script ran exactly:

```sh
cargo --config .github/cargo-ci.toml test --offline --locked -p tbg-balance --features tbg-core/training --test contract c5_every_tunable_number_is_registered -- --exact --test-threads=1
cargo --config .github/cargo-ci.toml test --offline --locked -p tbg-balance --features tbg-core/training --test role_invariants -- --test-threads=1
```

Results: C5 **1 passed**, 6 unrelated contracts filtered out; new boundaries
**4 passed**. Compilation took 10.58 s and 0.31 s; C5 execution took 0.02 s.
Local private output: `.local/ci-balance-fix.log` (not committed).

Additional cheap checks:

- `python3 scripts/test-ci-test-lanes.py`: **16 passed**.
- `python3 scripts/ci-test-lanes.py`: exact workspace feature/scope equivalence.
- `python3 scripts/test-ci-required-checks.py`: **9 passed**.
- `python3 scripts/check-patch-notes.py --base origin/main`: **passed**, six entries,
  unchanged ruleset 6 / `097385fee9a10b0f`.
- `actionlint -ignore SC2086 .github/workflows/ci.yml`: **passed**. The existing
  intentional shell scope-expansion diagnostic was excluded only in this local
  syntax invocation; no CI suppression was introduced.
- `git diff --check`: **passed**.

## Deferred and unchanged boundaries

- The complete GitHub balance lane and six-lane `check` outcome are left to CI;
  the owner requested no wait. No claim that the new remote run has passed.
- Do not rerun C7/C8 locally by default: both passed in the reported run, their
  simulation/fixtures/thresholds are unchanged, and each is expensive. No full
  local workspace suite, balance gate, benchmark or extensive gallery was run.
- No Linux/Windows/WASM rebuild, installation, VPS deployment or native publication
  is needed for this lab/CI correction. It changes no game executable inputs.
  A later native publication must still satisfy its own exact-source CI/artifact
  gates; this correction's evidence cannot certify another source's packages.
- Competitive balance acceptance remains pending D-33 evidence and owner decisions.
  A green CI does not mean the competitive balance gate has been accepted.
