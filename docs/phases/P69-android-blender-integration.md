# Android Native: Blender Champion Integration (arenas/strongholds pending)

**Status:** Completed and verified on physical hardware (Xiaomi POCO X6 Pro).
**Branch:** `codex/android-blender-features`.
**Base:** `b480721d7e62c6eadf432f5dabff7745e35df6a7` (`v0.1.4-colossus.7`).
**Package:** `io.github.alexandrezanata.tbg.prototype`.
**Version Code:** 56 (`0.1.4-colossus.7-android.56`).

## 1. Overview and Scope

This work integrates all consolidated Blender assets from PR #68 through PR #77 into the native Android application:
- **84+ Blender Champion Models**: Packaged and loaded directly through the Bevy runtime using the 88 `.tbgmesh` assets in `champions/blender/`.
- **Arenas, Strongholds, War Auxiliaries: NOT integrated.** On main they exist only as Blender scripts, .blend files and renders (docs/art/BLENDER-*.md list runtime integration as pending). The app still draws the existing procedural arenas and towers.
- **Physical Device Validation**: Installed and verified live on Xiaomi POCO X6 Pro (`2311DRK48G`, Android 16 / API 36, ARM64-v8a).

## 2. Compilation and Packaging Steps

1. **Rust Library Compilation**:
   - Toolchain: Rust 1.99.0 (`aarch64-linux-android`), NDK 28.2.13676358.
   - Flags: `-C link-arg=-Wl,-z,max-page-size=16384` (16KB max page alignment).
   - Command: `cargo ndk -t arm64-v8a -P 31 build --locked --release -p tbg-client --lib`.
   - Result: 80 MB stripped shared object `libtbg_client.so`.

2. **Native Asset Staging**:
   - `python3 scripts/package-android.py prepare --source <HEAD> --library .local/android-library/libtbg_client.so --abi arm64-v8a --version-code 56`.
   - Staged 480 assets (all 88 Blender mesh models) and verified ELF imports against system libraries.

3. **Gradle Assembly**:
   - Java 21 (Temurin-21+35), Gradle 8.14.3, AGP 8.11.0.
   - Signed with authentic prototype PKCS12 key.
   - Output: `app-debug.apk` (117 MB), SHA-256 `9eebf3d61a5689c6369f247ce2453cdaa71e38595fae04f6a1b4692f6e85e49d`.
   - Verification: `scripts/package-android.py inspect-apk` passed with exit code 0.

## 3. Physical Device Testing

- **Target**: POCO X6 Pro (`VKCQGAUCGYAMR4W4`, model `2311DRK48G`).
- **Install**: Streamed in-place upgrade via ADB without data loss.
- **Execution**: Landscape orientation preserved, Bevy/GLES pipeline initialized with zero runtime crashes.
- **Screens Checked**:
  - Live lobby with background arena combat featuring new Blender meshes.
  - Troops / Training Lab screen displaying 3D champion models and faction/archetype filters.
