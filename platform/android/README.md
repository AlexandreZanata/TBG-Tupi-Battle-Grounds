# Native Android landscape beta

The Rust client is one library shared with desktop/browser. GameActivity is
only the Android host: rendering, core/content, actions and authority are
shared. The landscape lobby uses the original art with touch controls;
Practice, Troops, Arcanas, Strongholds, Arenas and the native HUD retain shared
models/resolvers. The browser's portrait behavior stays separate.

See [device execution](../../docs/phases/P69-device-20261101.md) for actual
checks and outstanding acceptance. A compiled library or emulator boot alone
is not a playable-device pass.

## Toolchain

[toolchain.json](toolchain.json) pins the tested input tuple: Bevy 0.20.0,
android-activity 0.6.1/JNI 0.22.4, GameActivity 4.4.0, AppCompat 1.7.1,
Rust 1.99.0/cargo-ndk 4.1.2, NDK r28c 28.2.13676358, SDK 31/36,
build tools 36.0.0, JDK 21+35, Gradle 8.14.3, AGP 8.11.0 and Kotlin 2.1.20.
The verified wrapper/distribution hashes are pinned. No duplicate Prefab glue,
Compose framework, WebView or game-code copy is used.

## Explicit native build and package

Use a reviewed, committed execution SHA. Configure `ANDROID_HOME`, the pinned
`ANDROID_NDK_HOME` and `JAVA_HOME`. Keep a private stable signing key outside
tracked sources; provide `TBG_ANDROID_KEYSTORE`, `TBG_ANDROID_STORE_PASSWORD`,
`TBG_ANDROID_KEY_ALIAS` and `TBG_ANDROID_KEY_PASSWORD` locally without logging.
Never silently fall back to a different debug signature.

Every compile/package command uses the common nonblocking host lock and two
Cargo jobs/Gradle workers. Busy exit 75 is deferred under normal project
policy. A task-specific owner exception to wait is recorded in its report;
it is not a standing change to that policy.

```bash
flock -n -E 75 "/tmp/tbg-validation-$(id -u).lock" \
  env CARGO_BUILD_JOBS=2 bash -eu -c '
    export TBG_SOURCE_COMMIT="$(git rev-parse HEAD)"
    export CARGO_TARGET_AARCH64_LINUX_ANDROID_RUSTFLAGS="-C link-arg=-Wl,-z,max-page-size=16384"
    cargo ndk -t arm64-v8a -P 31 build --locked --release -p tbg-client --lib
    mkdir -p .local/android-library
    "$ANDROID_NDK_HOME/toolchains/llvm/prebuilt/linux-x86_64/bin/llvm-strip" --strip-unneeded \
      -o .local/android-library/libtbg_client.so target/aarch64-linux-android/release/libtbg_client.so
    python3 scripts/package-android.py prepare --source "$TBG_SOURCE_COMMIT" \
      --library .local/android-library/libtbg_client.so
    cd platform/android
    ./gradlew :app:assembleDebug --no-parallel --max-workers=2 \
      -PprototypeVersionCode=2 -PprototypeAbi=arm64-v8a
  '
```

Use the actual existing shared target directory if the checkout already uses
one. Never create a target directory to evade contention. Gradle consumes
verified staged libraries/assets and never invokes Cargo. If an ELF import
requires an extra native library, pass its exact ARM64 file with
`prepare --dependency PATH`; inspect every library, not just the game .so.
Assets are copied to the APK AssetManager root with one tracked inventory,
not loaded from a checkout or external storage.

For a scoped local test only, compile the explicit `android-loopback` feature.
It permits only literal `ws://127.0.0.1` and only while Android reports a
debuggable APK. The manifest debug policy matches it. Public/release APKs
require WSS even if a library was accidentally built with the feature.
Without an explicit server override, that internal test feature selects
`127.0.0.1:18069`; use a compatible isolated server and only a task-owned
ADB reverse mapping. Never bypass a production protocol/auth mismatch.

After signing:

```bash
python3 scripts/package-android.py inspect-apk \
  --apk platform/android/app/build/outputs/apk/debug/app-debug.apk \
  --report .local/p69-evidence/artifact.json
```

Inspection verifies source marker, tracked code/asset inventories, every
native ELF/import and actual 16 KB ZIP data offset, signature/certificate,
package and permissions. Final APK SHA/cert live in the external report.
The explicit `--abi x86_64`/`-PprototypeAbi=x86_64` path is emulator-only and
must never be presented as a POCO ARM64 artifact or ARM64 runtime proof.

## Services and safety

`MainActivity` fills the native surface and reports cutouts/system gestures/IME
separately from the actual surface/density. Rust insets controls once and compares this with
Bevy coordinates for effective 48 dp controls. It preserves GameActivity's
callbacks and one event loop. Unexpected portrait blocks inputs with a rotate
message; it does not reset state or pause online authority.

System EditText supplies masked/reveal, paste, Done/Back and localized labels;
Rust owns validation/submission. No saved input/autofill/learning is enabled.
Back dismisses local UI first and never silently surrenders. Shared online
combat and the normal reconnect window are unchanged. Audio follows Android
focus and system music volume.

Only allowlisted nonsecret settings reach app-private files. AccountKey,
Access/Refresh and resume credentials stay in memory through the reviewed
common auth coordinator; no Android vault or new auth protocol is introduced.
Manual backup uses explicit clipboard or system document export. Public links use checked HTTPS
intents. Desktop updater/window/vault controls do not execute on Android.
Backup/device-transfer are excluded by manifest policy.

Select exactly one authorized POCO by model/codename in process memory and
never log/persist its serial. Inspect an existing prototype package before
same-key in-place install. Signing/version conflicts are errors; no uninstall,
downgrade, data wipe, global logcat clearing or global ADB mapping removal.
Capture only the game foreground/viewport and filtered game logs. Keep the
installed desktop game and personal phone data intact.

## Automatic public beta publication

`android-release.yml` follows the native desktop pattern: successful exact-source
main CI, a signed production-endpoint build, immutable version tag, promotion of
the original bytes, hosted-runner attestation and a complete GitHub prerelease.
`release.json` supplies the monotonic versionCode and stable public certificate.
Changing shipped Android inputs under an existing revision is refused.

The `android-beta` GitHub environment is restricted to main and contains only
the stable keystore/password/alias secrets. PR jobs never receive that key.
The key is materialized only in private runner storage and removed afterward;
it is never part of an artifact. `build-android-beta.py` compiles without
`android-loopback`, packages the non-debuggable release variant and inspects its
source, WSS endpoint, certificate, permissions and16KB ELF/ZIP alignment.

The VPS's existing native catalog timer verifies Android provenance, immutable
asset digests, increasing source/revision and the live MatchContract. It merges
Android without erasing Linux/Windows. A changed server contract or revoked APK
fails closed. Website availability is independent of a GitHub beta existing:
an incompatible server never enables an online-compatible download claim.

The2026-11-03 publishing continuation is documented in
[P69 beta delivery](../../docs/phases/P69-beta-publication-20261103.md).
