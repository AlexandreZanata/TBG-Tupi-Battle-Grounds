# Laboratory reduced-motion toggle correction

Owner clarified that the frozen previews came from the reduced-motion button being
active on entry, not from broken champion clips. Scope is limited to that control.

Cause: `Catalog::default` inherited the saved Result/accessibility reduced-motion
preference, and the laboratory toggle wrote back to that same global preference.
This made a temporary preview setting persist and freeze subsequent visits. The
button label did not explain whether reduction was active.

Change: laboratory playback defaults to normal, the toggle stays local to the visit,
leaving Troops clears it, and localized text explicitly labels on/off. Existing
animations and the static reduced pose are preserved. The shared desktop visual
contract and motion matrix are updated in this same change.

Validation level: work front with owner-authorized native build and reinstall.
Runtime source: `547d7aa922ef38cbf80a6b9259f4453c4f7be6c8`.

Completed:

- `cargo fmt -p tbg-client -- --check`, `git diff --check`, locale JSON parsing/key
  parity and `python3 scripts/check-desktop-ui.py`: pass. The source gate reports
  `passed: true`, `errors: []`, and explicitly does not establish visual parity.
- Native build under the shared nonblocking host lock, using the existing target
  cache and `CARGO_BUILD_JOBS=2`:

  ```bash
  flock -n -E 75 "/tmp/tbg-validation-$(id -u).lock" env CARGO_BUILD_JOBS=2 \
    CARGO_TARGET_DIR="$TBG_EXISTING_TARGET" PKG_CONFIG_PATH="$TBG_NATIVE_PKGCONFIG" \
    TBG_DEFAULT_SERVER=wss://api.tupibg.com/ws \
    TBG_SOURCE_COMMIT="$(git rev-parse HEAD)" \
    cargo build --offline --locked --release -p tbg-client --bin tbg-client --features ui-acceptance
  ```

  Output: `Finished release profile [optimized] target(s) in 5m 30s`. The executable
  was frozen to a commit-specific candidate before releasing the lock.
- Real pointer/keyboard checks in the 1920x1080 Linux renderer, isolated from player
  data, with `tbg_reduced_motion` preset to `1`. Bone Brute Base / Dance opens with
  the explicit off label. On gives a static pose; off resumes; after turning it on
  again, Escape and the Troops navigation reopen with off and active playback.
- Lossless 10 fps recordings of the stage, 15 frames per sample, yielded 14 unique
  frame hashes on entry, 1 when reduced, 15 after resuming, 15 after reentry. These
  cropped-stage diagnostics establish playback/freeze for this Linux control check;
  they are not the full-frame cross-platform contract comparison.
- The saved Result preference remained exactly `1` after all toggle/reentry actions.
  No player preferences were modified by the test; it used a private test directory.
- `scripts/package-native.py --platform linux --binary <frozen-candidate>
  --output <local-output> --no-archive` under the host lock: pass.
- Package `install.sh` succeeded. All 355 installed manifest entries match sizes
  and SHA-256 digests; both local launchers and the quoted application-menu Exec
  target resolve to the new installation. Existing player data and previous
  installations remain available.
- Opened the installed executable with separate test data containing the old global
  preference of `1` and inspected its Dance screenshot: explicit off label, normal
  playback mode. Version `0.1.2-rc.1-547d7aa922ef`, release x86_64 Linux,
  `ui-acceptance`, protocol 31 / contract `w31-r6-097385fee9a10b0f`.
  Binary SHA-256: `a8050ee4dcec34f6f34bf948d54bb377a6ec781a20b5347510a4c08394ef152c`.

Local evidence: `.local/laboratory-motion-toggle-output/toggle-check/` contains the
four lossless recordings, full UI state PNGs and frame hashes. `installed/` contains
the installed-executable screenshot. Build outputs and captures stay out of git.

Pending, not passed: Windows and desktop web motion correspondence at consolidation,
with the same-source packaged matrix and full-frame criteria in the visual contract;
other champion/form/clip matrix cases are not claimed by this representative Linux
check. No full suite or whole-roster gallery was requested or run. No competitive
simulation, protocol, animation clip or combat-effect implementation changed.

The latest owner-provided AGENTS.md forbids pushing, so this correction and its
validation receipt are committed locally on `phase/laboratory-motion-toggle`.
