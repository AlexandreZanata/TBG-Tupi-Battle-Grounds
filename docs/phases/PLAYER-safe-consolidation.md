# Player-safe main consolidation (2026-11-01)

Level: targeted consolidation, not a full-suite or balance acceptance.
Owner scope: consolidate game/backend/desktop/browser fronts; later explicitly
include landing/admin frontends and automatic frontend delivery. Mobile remains
excluded. Normal push and a PR to main are authorized; the owner merges manually.
No force-push or automatic merge is part of this work.

## Integrated source and preservation

Base: main `e2a18af02d809b5926b752ff50e8116fe6d9e63d` (P71/D-100 continuity).
Normal merges preserve ancestry and the verified deployed backend floor:

- #38 `phase/main-auto-deploy`, snapshot `c301397`, includes deployed P70/P71,
  eight Arenas, protocol 31, secure sessions/rotating refresh, schema-10 reconciliation
  and backend release preparation. Deployed runtime is `8be248e025ed8f56290d94604da75536726d765b`.
- #37 `phase/p72-web-native-ui`, snapshot `56074bb`, uses the Linux Bevy desktop UI
  in the browser. Keep the same navigation, illustrated HUD/Shop/Augments, loading,
  training library, roster portraits, winner roster/animations and platform gates.
- #39 `phase/web-landing-admin-foundation`, final snapshot `2000407`, includes the
  landing, safe public admin, fictional local admin preview and static release tools.
- Older #18 (display layout), #22 (full character libraries), #26 (native refinement),
  P63/P66/P68 and earlier consolidation work are ancestry, not re-created patches.
  Current content is 46 conventional champions and eight Arenas, not the stale counts
  written in older PR bodies. No Android/mobile front is merged here.

Work uses an isolated `.local/player-safe-consolidation` worktree. Root policy edits,
other worktrees, running builds, signing material, local accounts and player data
are preserved. No archive import/edit or cache cleanup is involved.

Six modify/delete conflicts combined #37 with newer Arena metadata. Delete the
obsolete DOM CSS/JS/DOM tests as #37 specifies; retain Bevy eight-Arena data and
port the useful description/image/translation coverage into `arena_data_tests`.
The final #39 follow-up only conflicted in a blank line in `deploy/README.md`.

## Delivered behavior

Main now has the deployed gameplay/auth source plus one Bevy desktop presentation
for Linux, Windows and browser. No independent HTML game HUD is reintroduced.
Routine changes add no gameplay numbers or wire versions: exact integrated contract
is `w31-r6-097385fee9a10b0f`, schema 10. Two older schema-9 table families remain
recognized by the existing migration. Two-slot auth revocation remains a D-99 limit.

Three separate CI-gated OIDC artifact families deliver compatible backend, browser
and public statics. Web bundles use immutable `/releases/<source>/` URLs plus an early
document base; pointer promotions retain old assets and open tabs. Static payloads
cannot contain the fictional operator preview or active admin code. Neither frontend
controller restarts the backend or reloads the proxy. Backend drain never force-kills
matches or rolls back player data. All controllers share a host lock, preserve
previous versions, use fsynced transition/rollback records and reject source rollback.
Timers are ten minutes with jitter to avoid exceeding the public GitHub API quota.

See [DEVELOPMENT](../DEVELOPMENT.md), [RELEASES](../RELEASES.md),
[SCALING](../SCALING.md) and [controller runbook](../../deploy/auto/README.md).

## Validation actually executed

Rust source was integrated through `ebe39c9`; subsequent deployment scripts/docs
and source identity used browser build source `71eb81879e8191f74c7dd48f197e94f2f2c380d1`.
The checks below used the shared host nonblocking lock, the existing target cache,
`CARGO_BUILD_JOBS=2`, and one Rust test thread. No unrelated agents were stopped.

```bash
cargo test --offline --locked -p tbg-client --bin tbg-client arena_data_tests -- --test-threads=1
cargo test --offline --locked -p tbg-server --test build_info --test schema_upgrade --test health --test match_continuity --test revocation_feed --test auth_websocket -- --test-threads=1
cargo run --offline --locked --quiet -p tbg-core --example match-contract
cargo check --offline --locked -p tbg-client --target wasm32-unknown-unknown
cargo fmt -p tbg-client -p tbg-core -- --check
```

Results: 2 client metadata tests and 17 server identity/migration/health/continuity/
revocation/auth scenarios passed; contract output is the exact ID above; WASM check
and formatting passed. The release Trunk build below is necessary for the explicitly
requested compatible public browser delivery, not an inferred full-suite permission:

```bash
TBG_SOURCE_COMMIT=71eb81879e8191f74c7dd48f197e94f2f2c380d1 BINARYEN_CORES=2 \
  trunk build --release --offline --skip-version-check \
    --public-url /releases/71eb81879e8191f74c7dd48f197e94f2f2c380d1/
python3 scripts/package-web.py --contract w31-r6-097385fee9a10b0f --output .local/browser-candidate
```

Build/package succeeded (369 files, about 24 MiB compressed). Two pre-existing WASM
warnings remain in gallery motion/polish (`units`, `PAGE_COUNT`); not suppressed.

Focused source/tool checks:

```bash
python3 scripts/test-auto-deploy.py
python3 scripts/test-web-deploy.py
python3 scripts/test-static-deploy.py
python3 scripts/test-ci-required-checks.py
python3 scripts/test-ci-test-lanes.py
python3 scripts/ci-test-lanes.py
python3 web/scripts/check.py
```

The five Python test files passed 53 scenarios (10 + 12 + 9 + 9 + 13): archive
traversal/links/duplicates/budgets, source/hash/workflow binding, operator-preview
rejection, immutable collisions, promotion and rollback, interrupted transitions,
old assets and player data preserved, source scopes and fail-closed CI lane guards.
Web checks passed manifests, JSON, JS syntax, translation parity, availability gates
and generated roster/Arena content. Landing/public-admin/preview exported counts were
115/16/115. New three release workflows passed actionlint. CI YAML passed actionlint
with ShellCheck disabled only for that syntax inspection: its four pre-existing
SC2086 diagnostics on intentionally expanded `$TEST_SCOPE` are not claimed resolved.
No CI validator was weakened. The patch-notes validator and its 5 scenarios also
passed after adding the bilingual delivery candidate (58 Python scenarios overall).
`git diff --check` passed.

Actual Chrome/SwiftShader at 1440x900 loaded the source-qualified game package on an
isolated local protocol-31 server. Lobby and troop/bot configuration rendered, with
zero page errors, failed requests or HTTP errors after selecting the explicit local
server URL. The first trial used the default localhost:8080 URL while the test server
was at port 18789; six API requests failed. Correcting the test URL resolved it without
changing game code. A first launch found no bundled headless browser; the installed
Chrome was used instead, without downloading another browser. Neither event is a
passing check by itself. Private captures/logs stay under the ignored worktree.

## Official host operations and real limits

The retired test TBG was removed separately with private backups and unchanged
other-project inventories/status checks. It remains retired; no new deployment uses it.

Official root/www/admin lacked Caddy host entries/certificates, producing the TLS
alert behind Cloudflare 525. API/app already had Let's Encrypt certificates. Added
only the static hosts/include in a verified zero-connections/results window, validated
Caddy and obtained real Let's Encrypt certificates. Root/admin return HTTPS 200;
www redirects to root; API health stayed 200. No SSL downgrade, DNS edit, credential
publication or database change was made. Config backups and operational receipts are
private. Initial landing/admin release pointers are timestamped and retained.

Installed official GitHub CLI 2.103.3 after checking its official release SHA-256,
then the reviewed controllers/units/private anti-rollback floor. Its declared Git
dependency was absent: initial dpkg configuration and a direct apt install failed.
An inspected `apt-get --simulate --fix-broken --no-install-recommends install`
resolved exactly six new dependencies, no removals/upgrades; applied with `--no-remove`,
completed gh configuration and a clean `dpkg --audit`. No game service was restarted.
All three controller flags/timers are disabled. Verifier required flags are present;
this is not evidence of a real successful Sigstore verification.

The browser was manually bootstrapped at source
`71eb81879e8191f74c7dd48f197e94f2f2c380d1` with all 369 files verified. It uses
`/srv/tbg/web/releases/<source>` and `current`. The initial proxy entry route was
validated/adopted only with zero connections/matches/results; promotion itself did
not reload Caddy. Backend process identity stayed unchanged. Manual receipts have
separate names; no automatic attestation/activation receipt was fabricated.

Two preventive checks stopped publication before a pointer switch: Cloudflare
rejected the generic Python User-Agent with 1010, and its automatic Analytics script
changed the HTML digest. A clear TBG product User-Agent fixed the former. Frontend
Cache-Control now includes `no-transform`, following the official Cloudflare FAQ,
which preserves exact source bytes and prevents automatic beacon injection. WAF,
TLS, private routes and hash checks were not relaxed. Caddy's generated group numbers
and its self-hidden config filename were normalized when comparing adapted API
routes; API/auth source text remained intact. Failed attempts are not counted as
passing checks. Public root/admin/API/game now return HTTPS 200.

Actual public Chrome/SwiftShader at 1440x900 loaded Lobby, bot configuration and
the Forest Practice countdown with the illustrated five-card Shop. No page errors,
failed requests or asset errors occurred. The one HTTP 401 was the expected anonymous
`/api/auth/browser/refresh` response with no refresh cookie; the client continued as
Guest. The initial generic HTTP-status collector exited 1 for that expected response;
it is documented rather than hidden or misreported as an authenticated session test.
This proves public first entry/Practice launch, not a complete match or account recovery.
The landing browser link is enabled only after this observation; incompatible Linux
downloads stay pending.

Final public static releases: landing `20261101T201027Z-e4bdb38c8e72` and admin
`20261101T201032Z-e4bdb38c8e72`, both source `e4bdb38c8e724dba6d12ae037e3a183942ce5acc`.
All five public origins were rechecked: root/admin/game/API HTTPS 200, www redirects
to root, frontend Cache-Control is `no-cache, no-transform`. Public availability JSON
marks the browser live and Linux pending. A focused Chrome check found two enabled
browser CTAs, no failed requests/page errors, and zero scripts/forms in public admin.
These static pointer promotions did not reload Caddy or restart the backend.

Normal branch push and [PR #40](https://github.com/AlexandreZanata/TBG-Tupi-Battle-Grounds/pull/40)
target main; no merge, force-push or CI wait. All current #37/#38/#39 heads and the
original main base are retained as ancestors. Use this consolidated PR rather than
merging the same fronts independently. The source tree is ready for code review;
post-merge activation and the acceptance items below remain explicit.

## Deferred acceptance and open scope

- Real main artifact provenance with an empty GH config, first observed activation
  and rollback on the official host for each family: runbook commands in RELEASES.
  This cannot precede this PR's manual main merge and successful main-push CI.
- Browser two-client online match/auth/reconnect/spectator and end-to-end victory/
  War Council/late-message matrix on protocol 31: scope remains targeted consolidation;
  native/scoped unit checks do not establish these runtime scenarios.
- Real Linux OS-vault/device paths and a newly packaged matching native release;
  real Windows execution/GPU and P68 matched packaged visual matrix. Do not enable
  download links to the existing incompatible protocol-30 prerelease.
- P66 TUF signing custody/bootstrap and cross-slot auth revocation stay gated.
- P67 email/provider integrations remain offline contracts/fixtures. Public admin is
  a preparation page; authenticated permissions and O-W1–O-W3 decisions are not done.
  Privacy/terms/contact and owner visual/fun sign-off remain listed in #39's report.
- D-33 balance gates, load/capacity tests, full workspace Clippy/tests, extensive
  galleries and mobile are not run. They require the owner-approved scope; existing
  competitive failures/Proposed gameplay values are not converted into a pass.

Main delivery is source-prepared and host-staged, not automatically activated. Do
not mark the whole game release-ready or imply all deferred phases are accepted.

## CI Clippy follow-up (2026-11-01)

Owner-reported CI at source `bf01020` failed the workspace Clippy lane with 11
client diagnostics. The integrated P70/P71 art/capture source exceeded argument/type
complexity limits and contained redundant branches/arithmetic/range patterns. This
was not a network, dependency download or server failure. Earlier targeted compilation
and tests were not Clippy evidence; the failing lane is recorded explicitly here.

Fixes group existing Bevy SystemParams and local sweep state without changing their
access or lifetime, name the damage-fragment collection, remove the redundant
`With<UnitVisual>` filter (the required query reference already selects it), and
simplify equivalent color/arithmetic/pattern expressions. The Clockwork value `3.14`
was a scene height, not an angle: `CHIMNEY_CAP_Y = 314.0 / 100.0` preserves the same
f32 geometry instead of incorrectly replacing it with PI. No rule values, protocol,
CI configuration, lint suppression or warning policy changed.

Validation level: work front. On the final nine-file client diff:

```bash
cargo fmt -p tbg-client -- --check
flock -n -E 75 "/tmp/tbg-validation-$(id -u).lock" \
  env CARGO_BUILD_JOBS=2 \
    CARGO_TARGET_DIR=/home/iiii/PESSOAL-PROJETOS-ALEXANDRE/TBG-Tupi-Battle-Grounds/target \
    PKG_CONFIG_PATH=/home/iiii/PESSOAL-PROJETOS-ALEXANDRE/TBG-Tupi-Battle-Grounds/.local/native-deps/root/usr/lib/x86_64-linux-gnu/pkgconfig \
    cargo clippy --offline --locked -p tbg-client --bin tbg-client -- -D warnings
git diff --check
```

All passed. Rust/Clippy 1.99.0; Clippy exited 0 in 7.12 seconds using the existing
cache. No new regression test is warranted for equivalent lint-only refactors.
The workspace Clippy command (`cargo clippy --locked --workspace -- -D warnings`)
and other CI lanes remain for GitHub; no full local suite, additional target build,
rendering gallery, installation or deployment was run. Normal push updates #40;
CI completion and main merge remain with the owner.

## CI canonical-reference follow-up (2026-11-01, D-101)

Run 41346300403 / job 124967085631 failed `contract_determinism`, `contract_lab`,
`contract_roster` and `wave_a`. The first three applied the revision-2 references
or original P59 Trident row to documented P70/P71 candidates in revision 6. The last
fixture let the updated Farwatch legitimately shoot the own Stronghold, attributing
its 70 damage to the 24-damage Bombard blast.

D-101 preserves the old arrays/table, appends revision-6 references keyed by exact
ruleset/source hash and applies the existing Proposed Trident exception. Unknown
source identities fail closed. The Bombard starts nearer to Farwatch than its own
Stronghold, and the fixture additionally requires a real self-blast before asserting
unchanged tower HP. No gameplay implementation, contract or balance value changed.

Collected five complete canonical traces with `cargo run --offline --locked --quiet
-p tbg-core --example trace_canon`, then ran exactly:

```bash
flock -n -E 75 "/tmp/tbg-validation-$(id -u).lock" \
  env CARGO_BUILD_JOBS=2 \
    CARGO_TARGET_DIR=/home/iiii/PESSOAL-PROJETOS-ALEXANDRE/TBG-Tupi-Battle-Grounds/target \
    cargo test --offline --locked -p tbg-core --features lab \
      --test contract_determinism --test contract_lab --test contract_roster \
      --test wave_a -- --test-threads=1
cargo fmt -p tbg-core -- --check
python3 scripts/check-patch-notes.py --base origin/main
git diff --check
```

All 20 tests passed (2 + 3 + 7 + 8). Formatting, patch-note/source identity and diff
checks passed. Full workspace tests remain with GitHub; no local full-suite run or
new D-33 acceptance is claimed. Windows packaging/download work is recorded separately.

Native follow-up in progress: the owner explicitly requested Windows Docker/KVM
runtime comparison, matching Linux/Windows release artifacts and OS-aware landing
downloads. Existing TBG VM data/evidence are preserved. The old GNU/protocol-30 VM
prototype does not constitute acceptance of the new MSVC/protocol-31 packages.
