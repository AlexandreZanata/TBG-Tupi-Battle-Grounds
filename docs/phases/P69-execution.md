# P69 execution — Android implementation

**Current continuation (2026-11-01):** the owner released the Linux-base
ownership gate, requested the initial landscape UI and native installation,
then restored POCO ADB access in place of the emulator. See
[device execution](P69-device-20261101.md). The report below preserves the
2026-10-31 host-only checkpoint; it does not describe the current code.

**Date:** 2026-10-31. **Validation level:** work front; owner explicitly
authorized the later Android/Linux builds, named trace, focused cross-play
and one ten-minute observation, but did not request a full suite.

**Not complete.** The owner instructed P69 to wait for P68 to release the
shared files. Cargo/bootstrap/UI/platform client edits are therefore waiting.
This is a file-ownership gate, not a dependency on P68 authentication or
Windows completion. Work outside those files proceeded independently.
No shared client, core, server, Cargo.lock or policy source was changed.

## Source and boundary

- Isolated branch: `codex/p69-android-native-prototype`.
- Worktree: `.local/p69-android`; original dirty checkout, P68 worktree,
  other worktrees/builds and installed Linux application were preserved.
- Start: planning commit `4285013a6203beaea25b7b512ffb740bac81999f`, a direct
  documentation descendant of reviewed Linux native.3
  `014b0e0c046b38c551ee24534ee00709cde0e493`.
- All 37 entries in `P69-ANDROID-SOURCE-MANIFEST.csv` matched both SHA-256
  and line count before implementation. That original snapshot is retained;
  a new runtime snapshot is due after the shared-client integration.
- Selected source identity: native.3, protocol **29**, RulesetVersion **2**,
  canonical ruleset source hash **3d56156049e626c5**. Content SHA-256:
  `6df66afa9f6e7c92a9cbaab822e9e2644ad7acec4d061d4cdbfa693d79dc1b03`.
  No core numbers, fixtures, goldens, wire types, auth or mode gates changed.
- Cached `origin/main..4285013` contains **262 inherited commits**. This
  includes the reviewed Linux ancestry and planning; it is not a small
  main-only Android diff. Refresh the remote comparison before the eventual
  dedicated PR; do not merge P68 or rewrite ancestry to conceal this.

Thread coordination was attempted twice through `send_message_to_thread`;
the tool returned `received invalid arguments` both times. That step stopped
and the owner was asked about the narrow shared-file window. The owner's
wait instruction governs further overlapping edits.

> blocked: Shared-file coordination tried the message tool twice; exact
> error: `send_message_to_thread received invalid arguments`. Suspected
> cause: unsupported message argument schema. The owner explicitly chose
> to await P68's file release. Continue only independent host/tooling work.

## Implemented independent work

`platform/android/` provides a small pinned Gradle/GameActivity host,
separate prototype application ID, ARM64-only packaging and sensorLandscape.
It reuses existing launcher branding by a defined resource source path.
The manifest requests Internet only (plus AndroidX's internal signature
receiver permission after manifest merging), excludes backup/transfer,
exports only the launcher Activity and prohibits public cleartext traffic.
The debug network policy permits only literal loopback; Rust-side transport
enforcement remains pending and is required before any tunnel use.

Kotlin service methods prepare physical surface/inset/density snapshots,
bounded process-only event transfer, Back interception, native EditText
IME/paste/masking/reveal/Done/cancel, safe public HTTPS navigation,
explicit clipboard operations, allowlisted nonsecret preferences and audio
focus. GameActivity's existing event loop/inset callbacks are preserved.
No Kotlin simulation, game UI framework, WebView or second native glue loop
was added. **These services have no Rust consumer yet and are not runtime
acceptance evidence.** In particular, existing Rust AccountKey persistence
must be split before Android can run safely; the host alone does not fix it.

`scripts/package-android.py` stages tracked native assets and verified ELF
libraries with source/build/asset identity, refuses missing/mismatched source
markers and dirty source, and checks staging against tracked source changes.
Final inspection verifies signature/cert, package/permissions, all staged
assets/libraries, every ARM64 ELF/import and actual 16 KB ZIP data offsets.
It never executes an Android library on Linux or starts Cargo from Gradle.
`scripts/android_binary.py` owns ELF/APK inspection. Six focused synthetic
fixture regressions cover invalid ABI/truncation/page alignment, dynamic
string bounds, missing imports and compressed/unaligned/missing game libs.
Synthetic fixtures are never APK/gameplay evidence.

The asset scan identified **331 tracked native assets**, excluding browser
JS/CSS/HTML. No duplicate art/catalog or reduced Android roster was created.
Staging/package outputs, caches, credentials and APKs are ignored.

## Toolchain and physical reference

[Pinned tuple](../../platform/android/toolchain.json): Rust 1.99.0
`b940084d7`, cargo-ndk 4.1.2, Bevy 0.20.0, winit 0.30.13,
android-activity 0.6.1/JNI 0.22.4, GameActivity 4.4.0, AppCompat 1.7.1,
API floor 31/compile-target 36, SDK platform revision 2,
build tools 36.0.0, NDK r28c `28.2.13676358`, Temurin JDK 21+35,
Gradle 8.14.3, AGP 8.11.0 and Kotlin 2.1.20. SDK/NDK/JDK/cargo-ndk and
Android Rust targets already existed. No SDK license acceptance, global
toolchain change, purchase or signing-key creation ran.

The official exact-source Bevy mobile sample was fetched successfully after
the sandbox's network lookup failed. Its `#[bevy_main]` pattern agrees with
the installed 0.20 macro/AndroidApp/winit implementation. The installed
android-activity README explicitly requires GameActivity 4.4.0 and no
duplicate Prefab linkage. The downloaded AAR exposes the expected
`com.google.androidgamesdk.GameActivity` methods; it has no bundled .so.

The Gradle wrapper JAR digest matched the separately published upstream
digest, and its distribution digest is pinned. Wrapper JAR SHA-256:
`7d3a4ac4de1c32b59bc6a4eb8ecb8e612ccd0cf1ae1e99f66902da64df296172`.
GameActivity AAR SHA-256:
`3a8b0887ac55a0a6ab56021ced7353a250539eac811278b74bccd8a5b678d9c9`.
The JAR contains its `META-INF/LICENSE`. Only standard small wrapper tooling
is tracked; the AAR and generated build output are not.

Read-only, explicitly selected-by-model ADB inspection confirmed exactly
one authorized matching POCO: `2311DRK48G`/`duchamp`, Android 16/API 36,
`arm64-v8a`, physical portrait 1220×2712 px, density 480 dpi, pages 4096 bytes.
No serial, Android ID, account, private app log or personal screenshot was
printed or persisted. Automatic approval review rejected an earlier plan
to keep the serial in a temporary file; it was corrected to use process
memory only and the read-only inspection succeeded. No device modification,
install/launch, ADB mapping or global logcat operation ran.

The checkout filesystem had about 6.3 GB free during preflight. Evaluate
capacity before the future cold Bevy build; do not delete unrelated caches
or redirect targets to evade a validation lock.

## Checks actually run

All evidence applies to the isolated working-tree host/tooling diff based
on `4285013`; Rust/game sources remain byte-identical to that source.

1. Python CSV/SHA-256/line-count inspection: **37/37 match**, zero mismatches.
   Pure source identity calculation: protocol 29, ruleset 2, hash and content
   digest above; tracked asset count 331.
2. Host compilation, using the shared nonblocking host lock:

   ```bash
   flock -n -E 75 "/tmp/tbg-validation-$(id -u).lock" \
     env CARGO_BUILD_JOBS=2 bash -c \
     'cd platform/android && ./gradlew :app:compileDebugKotlin --no-parallel --max-workers=2 --console=plain'
   ```

   First attempt failed resource linking: `Theme.AppCompat.DayNight.NoActionBar
   ... not found`. The GameActivity 4.4.0 POM omits its AppCompat superclass
   dependency; adding explicit pinned AppCompat 1.7.1 fixed it. Repeated only
   this affected check: **BUILD SUCCESSFUL in 15s**, 14 actionable tasks,
   8 executed/6 up-to-date, exit 0. The first failed command's log was
   inspected before the focused corrected run replaced the local log.
   Final log: ignored `.local/p69-evidence/host-compile.log`.
   This compiles Kotlin and Android resources, not Rust or an APK.
3. `python3 scripts/test-android-package.py`: **6 tests passed**, 0.003s,
   exit 0. No Cargo/workspace suite or balance gate ran.
4. `python3 scripts/package-android.py --help`: exit 0.
5. `python3 scripts/package-android.py verify-stage`: expected refusal,
   exit 1, `Native input missing; build Rust explicitly, then run prepare`.
   This negative check is not an APK-build pass.
6. Python AST parse of the three scripts; JSON parse of the pinned tuple;
   XML parse of source manifests/resources: passed. Gradle resource/manifest
   processing also passed on the corrected host build.
7. `git diff --check` and staged whitespace/source-boundary inspection:
   final results recorded at this milestone commit.

## AN01–AN18 status

- **AN01 — partial:** correct independent ancestry/source snapshot; the
  common reviewed execution SHA for two game binaries is not built yet.
- **AN02 — partial:** real GameActivity host compiles; no cdylib/signed APK,
  final APK SHA/certificate or native adapter evidence yet.
- **AN03 — deferred:** no APK installed/launched on POCO.
- **AN04 — partial/static:** sensorLandscape and inset snapshot code exist;
  both rotations, real usable bounds and 48 dp/Bevy mapping unobserved.
- **AN05 — deferred:** unchanged core, but Linux/ARM64 traces not executed.
- **AN06 — deferred:** Rust touch ownership/action routing not implemented.
- **AN07 — deferred:** packaged Practice/training/gameplay untested.
- **AN08 — partial/static:** source roster/resolvers retained; actual
  packaged champion/form/animation/Victory/effect paths unobserved.
- **AN09 — deferred:** shared mobile frontend/localization not adapted.
- **AN10 — partial:** host service code compiles; no IME/paste/audio-focus/
  preferences/runtime proof and no Rust bridge yet.
- **AN11 — deferred:** no isolated server/cross-play/result/rematch journey.
- **AN12 — deferred:** no lifecycle/network/surface/process-death journey.
- **AN13 — partial:** host preferences/backup exclude secrets; Rust's
  Android memory-only AccountKey boundary is still required before launch.
- **AN14 — partial:** package verification tooling exists; no final signed
  artifact, downloaded artifact verification or same-key upgrade test.
- **AN15 — deferred:** ten-minute physical observation not started.
- **AN16 — partial/tooling:** six synthetic tests only; every actual .so
  still needs inspection. A 4 KB POCO cannot prove 16 KB runtime.
- **AN17 — deferred:** shared Rust unchanged so far; after library extraction,
  update `--bin` test selectors to `--lib`, prove nonzero focused counts and
  validate Linux/web behavior once. No current regression pass claimed.
- **AN18 — pending:** this partial report exists; playable APK link, complete
  execution PR and owner look/fun approval are not delivered.

## Exact remaining work and deferred checks

Await the owner's/P68 shared-file release. Then implement one Rust library/
app builder and exact Bevy Android entry, split native dependencies, consume
the host services, enforce memory-only AccountKey/debug-loopback boundaries,
and implement the complete landscape touch/frontend through shared actions.
No P68 auth code is needed or authorized for import. Preserve mobile web
portrait/desktop, rules/mode distinctions and the 30-second reconnect window.

After the source is reviewed/frozen, run the named consolidation once:

```text
cargo fmt -p tbg-client -- --check
cargo ndk -t arm64-v8a -P 31 build --locked --release -p tbg-client --lib
cargo build --locked --release -p tbg-client --bin tbg-client
cargo build --locked --release -p tbg-core --example trace_canon
cargo ndk -t arm64-v8a -P 31 build --locked --release -p tbg-core --example trace_canon
python3 scripts/package-android.py prepare --source <execution-SHA> --library <built-cdylib> [--dependency <required-ARM64-so>]
./gradlew :app:assembleDebug --no-parallel --max-workers=2 -PprototypeVersionCode=<greater-than-installed>
python3 scripts/package-android.py inspect-apk --apk <signed-APK> --report <external-artifact-report>
cargo test --locked -p tbg-client --lib ui::mobile:: -- --test-threads=1
cargo check --locked -p tbg-client --lib --target wasm32-unknown-unknown
```

All compiling/packaging commands require the same nonblocking host lock and
two Cargo jobs/Gradle workers; the list is not permission to bypass ownership
or run extra work-front builds. Add the affected existing net/training/UI
selectors after extracting the library, deduplicate them and record nonzero
counts. Run the existing five-seed traces on Linux/phone and compare exact
outputs; do not regenerate fixtures. Build the separate Linux reference
from the same execution SHA and data directory, preserving installed Linux.

Only then download/verify/install the handed-over signed APK on the selected
POCO, run the packaged offline/screens/IME/input/rotations/lifecycle journeys,
Android-versus-Linux match and reconnect/result/rematch, and the single
controlled ten-minute observation. Record screenshots, real viewport/
adapter/frame-time/memory/thermal/battery evidence and failures. Final
handoff publishes only the dedicated branch and opens the authorized PR to
main; never auto-merge, publish a store release or deploy production.

Full workspace suites, balance gates, broad galleries/soaks, other Android
versions/devices and actual 16 KB runtime remain outside this milestone's
executed checks. None is reported as passing.
