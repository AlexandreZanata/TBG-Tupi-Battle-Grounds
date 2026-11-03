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

## Validation level

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

All 16 tier-3 Linux captures were regenerated from the combined development
binary and are served at `http://127.0.0.1:8615/`. Forest, Tundra, Volcanic,
Ruins, Storm, Celestial, Clockwork and Desert each have blue and red views.
Four compact review sheets and the source-overlay/binary manifest are in
[the evidence directory](../evidence/p75-auxiliaries/). The binary honestly
reports source `development`; the manifest records the committed base plus
hashes of the compiled client overlay. This is development reference evidence,
not signed-package identity or Windows/web correspondence. The later sprite
binding/control-height fixes affect the online social dock, absent in Practice;
these unchanged auxiliary-scene captures remain applicable.

The combined release bundle passed `trunk build --release` under the required
lock (optimized client compilation: 5m 08s, existing WASM dead-code/import
warnings). Its first attempt had terminated with exit 143 without a Rust error;
the cache was retained. A fresh relevant-code build completed. The bundle is
served at `http://127.0.0.1:8617/?server=ws://127.0.0.1:8617/ws&auxiliaries=3&lang=pt-BR`
with an owned loopback backend, fresh synthetic database and same-origin local
proxy. No real mail or production service is contacted.


## Scoped completion evidence

The final deduplicated local Rust coverage before the last sprite regression
is 152 passing tests: 90 server integrations, 38 core contracts/unit scenarios
and 24 client scenarios. Five JS session tests also passed. This count reuses
unchanged successful cases and never counts a deferred check as a pass.

Exact additional scopes (all under the same nonblocking host lock and two
Cargo jobs):

```text
CARGO_TARGET_DIR=/data/tbg-p75-target cargo test -p tbg-core --features lab --test contract_lab c1_simulation -- --test-threads=1
CARGO_TARGET_DIR=/data/tbg-p75-target cargo test -p tbg-core --test contract_protocol --test contract_ruleset_version --test contract_release_identity -- --test-threads=1
CARGO_TARGET_DIR=/data/tbg-war-aux-target PKG_CONFIG_PATH=/tmp/p75-native-deps/usr/lib/x86_64-linux-gnu/pkgconfig cargo test -p tbg-client --bin tbg-client --features ui-acceptance -- auxiliar br_ax br_rc02 auth::tests patch_notes::tests p75 --test-threads=1
CARGO_TARGET_DIR=/data/tbg-war-aux-target PKG_CONFIG_PATH=/tmp/p75-native-deps/usr/lib/x86_64-linux-gnu/pkgconfig cargo build -p tbg-client --bin tbg-client --features ui-acceptance
CARGO_TARGET_DIR=/data/tbg-p75-target cargo test -p tbg-server --test ranked_recovery --test release_slots --test contract_p75_moderation -- --test-threads=1
CARGO_TARGET_DIR=/data/tbg-p75-target cargo clippy -p tbg-server --lib -- -D warnings
CARGO_TARGET_DIR=/data/tbg-p75-target PKG_CONFIG_PATH=/tmp/p75-native-deps/usr/lib/x86_64-linux-gnu/pkgconfig cargo clippy -p tbg-client --bin tbg-client -- -D warnings
```

Results: C1 1 passed; protocol 23 passed; ruleset/release identity 2 each passed;
client 24 passed (one layout-search test ignored, deliberately not executed);
ranked recovery 4 passed; release slots 6 passed (deployed probe ignored);
moderation 7 passed; both scoped Clippy checks passed. The native link initially
failed because the extracted ALSA development symlink pointed to an absent
relative runtime file. Only the temporary symlink was corrected to the actual
installed runtime; no system library was installed or changed.

The GitHub `test` lane exposed the old schema-10 assertion in `ranked_recovery`
and two retirement tests sampling permanent background backlog. The fixture
now checks the ordered current schema while preserving every ranked baseline
assertion. The moderation follower now reads and performs bounded maintenance
in one writer turn, preventing a second queued maintenance job from aliasing
with every release heartbeat. Retirement still counts every queued store job;
no observation exemption, timeout, deployment guard or assertion was weakened.
The full affected release target and cross-slot ban/switch scenarios passed.

The actual native-provider journey completed against owned synthetic loopback
servers with:

```text
python3 scripts/check-native-auth.py --client /data/tbg-war-aux-target/debug/tbg-client --server /data/tbg-p75-target/debug/tbg-server --report /tmp/p75-native-auth-combined.json --key-backup --confirmation-outage
```

Protected backup roundtrip, secure persistence, same-family restore,
confirmation-outage and stale-vault recovery, remote revocation, offline logout,
process coordination, token-free metadata and synthetic vault cleanup all
passed. The JSON evidence contains booleans only; no credentials are committed.
`legacy_preserved_in_temporary_mode=false` is expected in this successful secure
migration path; the harness checks that alternative only in temporary mode.

## Browser observation and remaining focused check

The web client loaded in pt-BR and paired two real guest clients against the
owned loopback backend. The compact dock and guest read-only central log are
shared with the native client. Actual 1280×720 inspection exposed clipped kit
controls and first-frame unbound emoji images during frequent HUD rebuilds.
Controls now use two 32 px rows; mute is available in the expanded log. Standard
handles stay resident and bind in PostUpdate after UI commands flush and before
layout. A scheduled Bevy regression asserts the image is bound on that frame.
Its local test/Clippy attempt returned exit 75 and is deferred to the final
GitHub lanes, not recorded as passed. The updated bundle/browser observation
must be confirmed before merge. All final GitHub results are recorded in the
PR conversation against its exact final head, avoiding a self-referential
report commit loop.

> blocked (resolved): the original proxy process terminated with exit 143 and
> left browser tab 3 on an error-page data URL. Two operations returned
> `Blocked browser navigation by Browser Use URL policy: data:text/html;charset=utf-8,...`.
> That tab was abandoned. After an independent HTTP-200 probe, a fresh tab
> through the documented browser recovery loaded the real localhost URL.
> No browser security warning was bypassed. The proxy now runs in its own
> detached process and remains bound only to loopback.


The two guest browser clients completed their actual Casual match against the
owned backend (4:05, Stronghold destroyed). The pt-BR Result screen correctly
hid account-only reporting. The lobby and result screenshots in the evidence
directory document that observed bundle, before the final compact-height/sprite
binding adjustment. They are not presented as final visual correspondence.

The final `cargo test ... p75_rebuilt_hud` / scoped client Clippy and the
subsequent updated `trunk build --release` each returned exit 75. Per
`docs/VALIDATION.md`, neither was retried automatically. The owner was asked
whether to authorize one further locked attempt. This local preview currently
serves the prior successfully built bundle; the final source patch must pass
GitHub test, native, Clippy and WASM lanes regardless. The visual/device
acceptance and attended production activation remain separately pending.
