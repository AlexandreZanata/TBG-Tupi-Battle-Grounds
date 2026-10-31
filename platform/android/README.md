# P69 Android host and package tooling

**Partial implementation.** The owner asked P69 to wait for P68 to release
shared client files. This directory prepares the independent host. The Rust
library/bootstrap, Android services consumer and touch UI are not integrated
yet. There is no playable APK or device-gameplay acceptance in this delivery.
See [execution evidence](../../docs/phases/P69-execution.md).

## Pinned inputs

[toolchain.json](toolchain.json) freezes Rust 1.99.0, Bevy 0.20.0, winit
0.30.13, android-activity 0.6.1/JNI 0.22.4, cargo-ndk 4.1.2, API 31/36,
SDK platform revision 2, build tools 36.0.0, NDK r28c 28.2.13676358,
Temurin JDK 21+35, Gradle 8.14.3, AGP 8.11.0, Kotlin 2.1.20 and
GameActivity 4.4.0 and AppCompat 1.7.1. These are prototype inputs, not older-Android certification.
Keep an existing installed SDK/license state; request authority for any
new license. No purchase or global SDK/toolchain change is needed here.

The wrapper JAR SHA-256 is
`7d3a4ac4de1c32b59bc6a4eb8ecb8e612ccd0cf1ae1e99f66902da64df296172`,
matching Gradle's published wrapper digest. The distribution checksum is
pinned in `gradle-wrapper.properties`. GameActivity's AAR digest is also
recorded. No Prefab/CMake/second activity glue is enabled: android-activity
already compiles its matching native glue.

The exact Bevy source sample was obtained from commit
`b3bdd75becaffe9a16677cf5bae5bd6d198f052b`,
[`examples/mobile/src/lib.rs`](https://github.com/bevyengine/bevy/blob/b3bdd75becaffe9a16677cf5bae5bd6d198f052b/examples/mobile/src/lib.rs).
Its `#[bevy_main]` initialization agrees with the installed macro,
`bevy_android::ANDROID_APP` and Bevy winit integration. The future Android
library must use this path once, calling the common app builder.

## Host service contract to integrate after file release

`MainActivity` loads `libtbg_client.so`, uses `sensorLandscape`, preserves
GameActivity inset/lifecycle callbacks, and never starts another game loop.
Rust calls the Activity instance through the AndroidApp/JNI reference:

- `viewportSnapshot()` returns physical window/surface bounds, surface
  origin, density, cutout/system-gesture insets and IME bottom. Subtract
  the surface origin before applying safe insets; measure the conversion
  against Bevy's actual window scale. No 48 dp claim is yet verified.
- `drainHostEvents()` consumes bounded JSON events: `back`, `paste`,
  `text-commit`, `text-cancel` and `link-unavailable`. No event is logged.
  Rust owns navigation/confirmation, form validation and game actions.
- `isHostResumed()`/`hasAudioFocus()` report presentation lifecycle only.
  Rust must pause/resume audio, cancel stale gestures and restore the
  surface. Never change online time or reconnect grace.
- `showTextForm(json)` opens the permitted system EditText modal. Supply
  `id`, `limit`, `secret`, `value` and localized `title/done/cancel/reveal`.
  It supports password reveal, native paste, Done/Back, disables saved
  field state/autofill/learning, and clears text on dismissal. A commit
  is submitted to shared Rust validation, never directly to the network.
- `copyText`, `requestPaste`, `openPublicPage` use Android APIs. Clipboard
  export requires a deliberate game action; plain text only on paste.
  Public navigation accepts HTTPS without credentials/query/fragment and
  only public game paths. No deep-link input or external executable updater.
- `readPreference`/`writePreference` allow only the existing nonsecret
  language/loadout/arena/audio/motion/quality/difficulty keys. AccountKey
  and session credentials must remain in process memory in the Rust client.

Back handling and text controls require the Rust consumer before launch.
The host alone establishes none of the AN03–AN13 runtime gates. Backup
and transfer are excluded in the manifest; that does not prove the eventual
Rust integration avoids secret persistence.

## Explicit build/package sequence (pending shared Rust integration)

Use the same execution SHA for Android and the separate Linux reference.
Preserve the installed game. Commit reviewed source first; the future
Android library must retain `TBG_ANDROID_SOURCE=<40-character-source-SHA>`
as a used static string. The staging script searches for it without executing
the ARM64 library. It rejects missing/mismatched markers and dirty source.

Configure `ANDROID_HOME`, `ANDROID_NDK_HOME` (the pinned r28c), and `JAVA_HOME`.
All compiling/packaging uses this nonblocking lock and two jobs/workers:

```bash
flock -n -E 75 "/tmp/tbg-validation-$(id -u).lock" \
  env CARGO_BUILD_JOBS=2 bash -eu -c '
    export TBG_SOURCE_COMMIT="$(git rev-parse HEAD)"
    cargo ndk -t arm64-v8a -P 31 build --locked --release -p tbg-client --lib
    python3 scripts/package-android.py prepare \
      --source "$TBG_SOURCE_COMMIT" \
      --library target/aarch64-linux-android/release/libtbg_client.so
    cd platform/android
    ./gradlew :app:assembleDebug --no-parallel --max-workers=2 \
      -PprototypeVersionCode=1
  '
```

Exit 75 means deferred; never poll, delete caches or bypass via another
target directory. If ELF imports require `libc++_shared.so`, supply that
exact NDK ARM64 input using `prepare --dependency PATH`. Every library
must have valid ARM64/16 KB LOAD alignment and resolved imports.
Game assets are copied from tracked native assets into generated
`app/build/native-input/assets`, at the APK AssetManager root, preserving
the engine's asset names. JS/CSS/HTML are excluded; models, art, sounds,
catalogs and fonts are shared with Linux. No runtime source-checkout fallback.

Gradle consumes staged inputs and never invokes Cargo. Missing or changed
inputs fail before native-library merge. `:app:compileDebugKotlin` can
validate only the host independently; it does not build or validate a game.

Supply private test signing using `TBG_ANDROID_KEYSTORE`,
`TBG_ANDROID_STORE_PASSWORD`, `TBG_ANDROID_KEY_ALIAS`, and
`TBG_ANDROID_KEY_PASSWORD` in the local environment. Keep signing material
outside tracked sources and reuse the same key. No default debug-key fallback.
Choose a versionCode strictly greater than the installed prototype's value;
never uninstall or downgrade to work around a mismatch.

After signing:

```bash
python3 scripts/package-android.py inspect-apk \
  --apk platform/android/app/build/outputs/apk/debug/app-debug.apk \
  --report .local/p69-evidence/artifact.json
```

Inspection compares assets/libraries/metadata against current staged source,
checks every ELF/import and actual ZIP offset, verifies signature/cert,
application ID/permissions and zipalign using pinned build tools. Final
APK hash/cert live in the external report, never inside the hashed APK.
Physical 16 KB runtime and store readiness remain separate gates.

The debug-only Android network policy allows literal `127.0.0.1`; the Rust
transport must enforce that same restriction independently before local
cross-play is enabled. Public/release transport requires WSS. A debug APK
alone never authorizes cleartext in a release Rust library.

Do not install this host without the Rust integration and reviewed package.
On resumption, select exactly one authorized POCO by model/codename in memory;
do not record its serial. Download/verify the final handed-over APK before
same-key in-place install. Capture only the game viewport and filtered app
logs, preserve global logcat/mappings/settings and remove only owned probes.

## Focused host/tool checks

```bash
python3 scripts/test-android-package.py
python3 scripts/package-android.py verify-stage
```

The second command must refuse missing/stale native input; that refusal is
not a successful APK build. Use the phase runbook for Linux/ARM64 traces,
cross-play, screen/input/lifecycle coverage and the single ten-minute device
observation after P68 releases shared files. No full suite is authorized.
