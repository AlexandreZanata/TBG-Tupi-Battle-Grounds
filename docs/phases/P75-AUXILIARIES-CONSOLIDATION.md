# P75 and tiered War Auxiliaries consolidation

Owner request, 2026-11-02: integrate the attached War Auxiliary handoff into
P75, resolve failures, push, verify GitHub CI and merge when validations pass.
The owner acknowledges that main triggers automatic deployment. Schema 12
still requires attended migration; no deployment guard may be bypassed.

## Integrated sources

- P75 S0–S14: `854391b0`, PR #59.
- Main / protected backups and updater: `130cadc1`, PR #57.
- War Auxiliary tiers and presentation: `5c8b9ae2`, three original commits retained.
- Combined protocol: wire 40, append-only P75 tags; auxiliary R16 fingerprint
  `f2e253b98ccb313a` retained. P75 itself introduces no simulation change.
- Client release candidate: `0.1.4-colossus.5`.

## Validation in progress

Level: targeted consolidation and the explicitly requested eight-Arena/both-side
captures and localhost preview. No local full suite is authorized. Every
compilation uses the shared nonblocking host lock and at most two Cargo jobs.
Previous source-branch evidence remains in the P75 report and handoff; this
report records only checks on the combined tree once completed.

## Pending acceptance and activation

- Windows device capture and exact Linux/Windows/desktop-web visual/motion
  correspondence require observed evidence; compilation is not parity.
- Proposed auxiliary values require competitive D-33 acceptance and owner fun
  sign-off; bounded measurements must not be represented as the full gate.
- Production activation: verified backups, copied-database migration rehearsal,
  mail configuration, privacy approval and coordinated compatible client/server
  rollout. Automatic deployment refuses schema changes. No VPS, DNS, real email,
  secrets or production database operation has been performed.

## Combined-tree checks and corrections (2026-11-02)

The initial combined revision was `8ef53b39`; subsequent focused corrections
are ordinary commits, never a rewrite. Main and the auxiliary source worktree
remain untouched. `HANDOFF-war-auxiliaries.md` is untracked only in its source
worktree and is absent from this PR.

The following batch used the required host lock, `CARGO_BUILD_JOBS=2` and
`CARGO_TARGET_DIR=/data/tbg-p75-target`:

```text
cargo test -p tbg-server --no-fail-fast --test contract_p75_identity --test contract_p75_recovery --test contract_p75_admin --test contract_p75_moderation --test contract_p75_reports --test contract_p75_chat --test contract_p75_emoji --test contract_registration --test schema_upgrade --test revocation_feed --test history_privacy --test auth_http --test presence_global --test player_challenges -- --test-threads=1
```

Final deduplicated result: 80 server scenarios passed. All P75 scenarios passed
on the first combined run; the legacy `presence_global::actual_two_slot` test
initially failed parsing its registration response and passed in isolation
(7.11 s) after adding a status/error-only assertion for diagnosis. No credential
is printed by that assertion. The transient first response had not been
captured, so its cause is not claimed resolved by the added diagnostic alone.
The GitHub suite must confirm the whole target.

Additional completed scopes under that prefix:

- `cargo clippy -p tbg-server --lib -- -D warnings`: passed, 5.25 s after fixing
  borrowed references, formatting the permission checks, naming the report row
  type and grouping operator session context. Auth issuance boundaries retain
  individually justified argument-count exceptions.
- `cargo test -p tbg-core --lib auxiliary::tests -- --test-threads=1`:
  6 passed, 0 failed, 0.04 s.
- `cargo test -p tbg-core --features lab --test contract_protocol --test contract_determinism --test contract_lab --test contract_ruleset_version --test contract_release_identity -- --test-threads=1`:
  determinism 2 passed; lab C2/C10 passed. C1 initially misidentified the emoji
  filename `1f64f` as a floating-point type. The frozen asset table moved to the
  existing wire-only module `protocol.rs`; the simulation guard is unchanged.
  The remaining targets in this batch were not run after the failure.
- Static inspection of the exact C1 forbidden-token list and classification of
  every core source: passed after that move and the explicit wire-only
  `protocol/social.rs` registry entry. The R16 fingerprint is unchanged.
- `node --test crates/tbg-client/tests/auth-session.test.cjs`: 5 passed.
- `cargo fmt -p tbg-core -p tbg-client -p tbg-server -p tbg-balance -- --check`,
  `python3 web/scripts/check.py`, `python3 scripts/check-desktop-ui.py`,
  `python3 scripts/check-patch-notes.py --base origin/main`, native version
  policy and `git diff --check`: passed at their recorded revisions.

Two combined core/client follow-ups returned exit 75 while another front owned
the compilation slot. They were deferred without waiting, polling, bypassing
or changing that front. Commands were the C1/protocol/registry/release targets
above and `cargo clippy -p tbg-client --bin tbg-client -- -D warnings`.
Subsequent CI-specific source changes require fresh scoped evidence; old
results do not validate those changes.

GitHub first-round logs identified stale public release metadata, server
Clippy diagnostics, a test hardcoding the previous latest patch entry and the
C1 lexical collision. The next round passed quality, desktop-ui and WASM;
client Clippy identified four additional source diagnostics, now corrected.
No workflow, CI scope, required check or gate threshold was changed. Native
packaging, Windows and all final CI results must be reported separately when
complete. A successful build is not device or motion correspondence.

## Local evidence delivery

The 16 reviewed tier-3 Linux source-branch captures are served at
`http://127.0.0.1:8615/`, with a JSON manifest recording source commit
`5c8b9ae284e40c554f335d7be0964fe7cb4079c2`. Forest, Tundra, Volcanic, Ruins,
Storm, Celestial, Clockwork and Desert each have blue and red views. Bridges,
portraits and compact cards are visible; game look/fun sign-off remains the
owner's. Source captures are not represented as combined-tree or cross-platform
proof. The combined `trunk build --release` is running under the global lock;
its result and the playable localhost URL will be recorded after verification.
