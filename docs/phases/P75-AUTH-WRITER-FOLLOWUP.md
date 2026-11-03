# P75 follow-up: authentication writer contention

## Trigger and scope

PR #59 merged as `23e8097d04acd23ad46917dc89b9741a56de4b12` after all
required PR checks and Linux/Windows packaging succeeded. Its automatic main
run [41562707047](https://github.com/AlexandreZanata/TBG-Tupi-Battle-Grounds/actions/runs/41562707047)
then exposed a transient account-issuance failure in job 125676792055:
`revocation_feed.rs:86:57`, `Unavailable`. The owner explicitly requested that
all failures be resolved and green changes merged. This follow-up uses a new
isolated worktree and branch from that GitHub main, preserving the original
checkout, P75 worktree and auxiliary source front.

## Cause and behavior

`issue_family` and `rotate` opened deferred SQLite transactions and read policy,
account or refresh data before their first write. Another slot can hold the
WAL writer reservation at that point. SQLite rejects the read-to-write upgrade
instead of invoking the busy handler, surfacing `Unavailable` for an otherwise
valid registration or renewal. The legacy `accounts::register` path already
reserves the writer first for this exact two-slot constraint.

Both auth transactions now use `BEGIN IMMEDIATE` before their reads. The
existing five-second SQLite busy handler can wait for the other writer, and
policy checks, account/baseline/email/outbox creation or refresh consumption
share the writable snapshot. No retry loop, timeout increase, rate-limit
change, API widening or deployment-gate exception is introduced. The match
simulation, wire 40, R16 fingerprint, client version and schema 12 are unchanged.

## Regression evidence

Two tests hold a real second connection's write reservation while registration
or refresh runs through the actual store writer. They assert complete atomic
identity rows, retained manual-key access, the same family/account/absolute
expiry, consumed refresh history and exactly one current hash.

The first test location attempted to import the private Enrollment type and
failed compilation with E0603. The tests moved beside the private store module;
no production API was exposed. On the original transaction behavior both
regressions then failed with `Unavailable` (0 passed, 2 failed, 0.36 s). With
the fix they passed (2 passed, 0 failed, 0.41 s).

All compiling commands used the shared nonblocking host lock,
`CARGO_BUILD_JOBS=2` and existing `/data/tbg-p75-target` cache. Exact focused
post-fix scopes, run sequentially in one locked batch:

```text
cargo test -p tbg-server --lib store::auth_sessions::tests:: -- --test-threads=1
cargo test -p tbg-server --test auth_sessions --test auth_http --test contract_registration --test revocation_feed --test contract_p75_identity --test contract_p75_recovery -- --test-threads=1
cargo clippy -p tbg-server --lib -- -D warnings
```

Results: 32 tests passed, 0 failed. Breakdown: writer contention 2 (0.41 s),
auth HTTP 7 (0.25 s), auth session history/migration 3 (0.13 s), P75 identity
7 (7.22 s), recovery/data retention 10 (20.64 s), registration budgets 2
(0.07 s), cross-slot revocation 1 (1.03 s). Scoped Clippy passed (12.12 s).
Formatting and `git diff --check` passed. No full local suite ran.

## Remaining gates

The follow-up PR's exact-head GitHub CI gates its normal merge. The original
failed main run cannot publish a browser/backend candidate; a later green main
run will generate the immutable candidate used to refresh the localhost
preview. Actual final remote results are recorded in the PR conversation.

Production activation remains the attended protocol/schema rollout described
in P75-OPERATIONS. No VPS, DNS, real email, secrets or production migration
was accessed. The original device pixel/motion, Proposed competitive balance
and owner fun acceptance remain pending; a compile does not prove them.
