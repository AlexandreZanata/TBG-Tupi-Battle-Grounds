# P69 follow-up — Android beta publication and landing media

Owner request: refresh landing game photos from the README/current game and
publish the native mobile beta automatically in the Linux/Windows release format.
Base: main `b9574efaaf58abad0ccd63f2b8965b0f62e17018`, including merged PRs 61/62.
Worktree: `.local/android-beta-landing`; branch: `codex/android-beta-landing`.

## Implemented scope

Shared gameplay and server rules are retained. Android revision 55 adds the Beta
launcher label and a non-debuggable ARM64 release package using official WSS,
no internal loopback feature, stable package/signing identity and monotonic
versionCode. No APK or secret is tracked. Existing toolchain and page checks remain.

Successful exact-source main CI triggers one signed build. Its immutable Android
tag promotes the original inspected bytes, adds hosted-builder OIDC provenance
and publishes draft-first with the existing native publisher. PRs run contract
fixtures without the signing key. Only main may use the protected `android-beta`
signing environment. Changed APK inputs require a new revision; unchanged
docs/static merges reuse the release without replacing bytes or tags.

The existing metadata controller verifies Android and Linux/Windows independently,
preserving compatible entries through one locked atomic writer. It checks
provenance, ancestry, live contract, published size/hash, stable certificate, ABI,
public build mode and upgrade identity. Revocation hides only the affected family.
The landing uses the same verified JSON and requirements/checksum dialog; the
checked-in fallback remains pending.

The gallery has eight README Linux frames and three real Android emulator frames.
Hero/social art uses Linux release frames. Canonical portraits and Arena previews
receive content-hash URLs. See [media evidence](../evidence/landing-android-beta-20261103.md).

## Validation performed

Level: focused packaging/catalog/static consolidation and scoped browser review.

- `python3 scripts/test-android-package.py`: 6 passed.
- `python3 scripts/test-android-release.py`: 5 passed.
- `python3 scripts/test-native-catalog.py`: 7 passed.
- `node scripts/test-native-downloads.cjs`: passed, including Android positives
  and rejection of wrong certificates, debug builds, revisions, URLs and contract.
- `python3 web/scripts/check.py`: passed (translations, references, JavaScript,
  release gates and generated content).
- Actionlint 1.7.7 on Android/CI workflows: passed; official checksum verified.
- Eight new media files decoded, each below 400 KiB. `git diff --check`: passed.
- Actual desktop/390 x 844 PT/EN gallery and pending-dialog observations are in
  the media report. A language selector miss was corrected using the observed
  accessibility control; this was not a product failure.

The first catalog fixture run exposed a local variable shadowing `os` in
`publish`; the loop variable was renamed. Both affected suites were rerun and
passed (5 Android/7 native). Synthetic fixtures test identity/refusal rules, not
real APK builds, OIDC promotion or installation. Media inspection initially used
an unavailable Pillow module; installed ffmpeg/ImageMagick decoded the files.

## Deferred activation and acceptance

The VPS `72.60.147.2:22` timed out; the owner confirmed this network cannot reach it.
Live API inspection showed protocol 38; this source needs
`w40-r16-f2e253b98ccb313a`. Website/controller activation and live download
availability await network access and a compatible live server. No backend
upgrade/migration was attempted; do not bypass compatibility.

Physical ARM64 runtime, 16 KB device runtime, complete Windows pixel/motion
correspondence and owner look/fun retain P69's explicit pending status. The
personal phone was not touched by this follow-up. No local Cargo suite or
balance gate was inferred. Public build, emulator smoke and actual GitHub
CI/publication outcomes are recorded separately when completed.

## First public-mode local package

Source `27a5bed9f381a15b35f006224bf6f74c9f003e39`, revision 55, x86_64 emulator APK:
`c2c854eeb7f4f9ff668afef4386f2f94ef7cc1fe73e5c72a8e050f862c335dc8`.
The common nonblocking lock and two build jobs were used. Rust release completed
in 1m33s; Gradle assembleRelease succeeded in 1m09s. Inspection passed official
WSS/no-loopback marker, non-debuggable mode, stable certificate, source/asset
inventory and ELF/ZIP checks. This x86_64 artifact is private emulator evidence,
not the ARM64 public download. The first attempt selected a Rust alias without
Android std; the already-installed stable alias was checked as exact rustc
1.99.0 with Android targets and used successfully without a fresh build cache.

PR CI first reported seven missing Action version comments in the new workflow;
all actions were already SHA pinned. Added the required version comments without
relaxing the regression guard. Public-mode emulator execution remains separate
from this successful package inspection.

Automatic approval review rejected uploading the existing private signing key,
password and alias to persistent GitHub Secrets without explicit payload/destination
authorization. No key material was sent by that rejected action. The owner then
explicitly authorized the exact payload/destination. Setup completed with an
exact `main` branch policy and the three named environment Secrets; only names
and policy were printed. An HTTP2 policy-creation transport refusal was resolved
by reading state first, then completing the HTTPS request using HTTP1. No secret
values entered arguments, logs or tracked files.

## Public-mode emulator smoke

The x86_64 release APK above upgraded in place successfully on task emulator
`emulator-5582`, API 36. The host renderer exited with signal 139 before boot;
official SwiftShader booted the same retained AVD without wiping data. The app
opened Guest lobby, accepted navigation/Back, opened Practice setup, scrolled
both configuration sections and started a real Tundra match. One ordinary Shop
purchase changed Army 0 to Army 1, spawned the red Archer and renewed all five
offers. Live battles showed unavailable, consistent with the protocol-38 service.
No internal loopback or private backend was used. Software rendering was visibly
slow: this validates navigation/local gameplay, not phone performance or ARM64.

The retained emulator started at forced density 320/font 1.3. The reference smoke
used 240 dpi/font 1.0, matching P69's 1.5 density reference. Early immediate input
captures showed a prior frame; settled frames confirmed settings/scroll actions.
Those early frames are not claimed as input failures. The initial density/font
settings are restored when the task closes.

The signed APK/inspection and safe original PNGs remain in ignored local delivery
storage. Public ARM64 bytes and hosted-builder attestations must still pass the
actual main pipeline. Final CI/publication state is recorded in
[PR 63](https://github.com/AlexandreZanata/TBG-Tupi-Battle-Grounds/pull/63).

Before main publication, workflow inspection found that checkout's default shallow
history would omit the pinned Linux ancestor required by the existing APK
inspector. Android main builds now fetch full history for that ancestry proof;
no source-ancestry guard was removed. Actionlint/workflow policy were rechecked.

- public55-reference-lobby.png: 7986739d34d598116b6c8e4f99d0534df6f393bf1162b067bb0d58057bb818d3

- public55-practice-start-options.png: c62ad52d18ae08aa81448bc69fc142127333ba21cb75eafffc9b8bf9f5e21633

- public55-match.png: 574b7f8c0926223cc44f175fcce3aa3566549e0f4f758d27b7878a6a6deb5258

- public55-after-purchase.png: ed906e4c13d9cba13c91604fc8a948ea71353b2ce09010048776685e1a155344

## First main publication and Java provider correction

PR 63 merged as `c3dbc999c2df4bbacb54c81b72e32d17165549a7` after all seven
PR CI jobs and the separate Android contracts passed. Main CI run 41666585844
also passed all seven jobs. The integrated tree matches the reviewed PR tree
and retains the earlier main as an ancestor.

Static run 41668880875 published the new images as
`static-c3dbc999c2df4bbacb54c81b72e32d17165549a7`. The downloaded tarball digest
is `113431a78fe614908c2d6bc98da37f4d0f7093502d122b68de371044b81527c1`.
It contains all eight new hero/social/Android derivatives. Hosted-builder
attestation verification passed for the static workflow, main ref and exact
source using a task-local official GitHub CLI 2.103.3 archive verified against
cli/cli's release digest. The system CLI 2.45 lacks attestation commands.

Android run 41668880863 passed its source/CI scope and fetched the ancestry, but
failed before compilation: setup-java could not resolve `21.0.0+35`. Its actual
provider inventory lists that same reviewed JDK as `21.0.0+35.0.LTS`. The pin is
corrected to the provider's exact identifier and shared by main/PR jobs. The PR
package-contract job now actually resolves the JDK and prints `java -version`
before running package tests. No signing key is exposed to that PR job.

Full-history checkout also uses the pinned action's supported `blob:none`
filter: commit ancestry stays available for the inspector, while historical
art blobs are fetched only if needed. This addresses the observed multi-minute
cold checkout without reducing source checks. No Android tag or public APK
was created by the failed build; revision 55 remains the first public candidate.
