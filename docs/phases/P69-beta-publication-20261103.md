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
