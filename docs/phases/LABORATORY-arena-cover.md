# Proportional Practice arena picture

Owner request: remove the flattened arena thumbnail, let the picture occupy the
unused pane area above the selector/Play buttons, and crop the sides when needed.
Also audit that all laboratory improvements and fixes in this chat propagate to
Windows and desktop web; do not duplicate changes if the implementation is shared.

Implementation: a flexible picture replaces the 92-pixel stretched thumbnail;
post-layout centered cover cropping preserves source proportions. Practice content
uses compact consistent gaps and 30-pixel difficulty rows so the image receives
enough height for a lateral center crop at the observed desktop window sizes.
The existing image loading, selector action and Play action are retained.

Propagation: audited build.rs, training plugin, catalog, viewer, preview, stage,
runtime actions and shared locale/arena assets. They are already one implementation
on Linux, Windows and wasm32; no per-platform port is necessary. Shared source does
not prove exact rendered correspondence, which remains a separate contract gate.

Validation level: work front with the owner-authorized native build/reinstall.
Runtime source: `ac50ba6dd1760139a0b91cd156a289f1987705f0`.

Completed:

- `cargo fmt -p tbg-client -- --check`, `git diff --check`, contract JSON parsing,
  unique screen IDs/required platforms/motion-case references, and
  `python3 scripts/check-desktop-ui.py`: pass. The source gate reports `passed:
  true`, `errors: []`, with packaged visual equivalence explicitly unverified.
- Native release build under the shared host lock, maximum two build jobs, reusing
  the existing cache:

  ```bash
  flock -n -E 75 "/tmp/tbg-validation-$(id -u).lock" env CARGO_BUILD_JOBS=2 \
    CARGO_TARGET_DIR="$TBG_EXISTING_TARGET" PKG_CONFIG_PATH="$TBG_NATIVE_PKGCONFIG" \
    TBG_DEFAULT_SERVER=wss://api.tupibg.com/ws \
    TBG_SOURCE_COMMIT="$(git rev-parse HEAD)" \
    cargo build --offline --locked --release -p tbg-client --bin tbg-client --features ui-acceptance
  ```

  Final output: `Finished release profile [optimized] target(s) in 3m 56s`.
  The binary was frozen to a commit-specific candidate before releasing the lock.
- Initial native observations showed that allocating only the old leftover space
  could require a vertical crop in wider windows. Compacting the difficulty rows,
  note padding and gaps gave the image more height. The final observations below
  supersede those first-candidate screenshots.
- Final real-renderer screenshots: Forest at 1920x1080; Volcanic and Celestial at
  1920x970, the wider proportions near the owner's screenshot. Existing 768x382
  and newer 640x360 sources retain their proportions with centered side cropping.
  The image uses the panel's available height; selector and Play are at the bottom,
  both fully visible, without the former unused blue spacer. No image asset changed.
- Actual pointer clicks changed Forest through Tundra to Volcanic, updating name
  and image. A live X11 window resize from 1920x1080 to 1366x768 recomputed the crop
  and layout; inspected the resized screenshot with both buttons visible.
- `scripts/package-native.py --platform linux --binary <frozen-candidate>
  --output <local-output> --no-archive`, under the host lock, and package `install.sh`:
  pass. Verified all 355 installed manifest entries against size/SHA-256, both local
  launcher links and the quoted application-menu Exec. Earlier installations and
  player data are preserved.
- Opened the actual installed executable at 1920x970 with the Volcanic preview.
  Version `0.1.2-rc.1-ac50ba6dd176`, release x86_64 Linux, `ui-acceptance`, protocol
  31 / contract `w31-r6-097385fee9a10b0f`. Binary SHA-256:
  `a39032c17034402801e41cdb881034835357c46cc3b336c99b64df71258094ce`.

Propagation evidence: `build.rs` selects this same presentation for Linux/Windows
and wasm32. `main.rs` installs the same training plugin; `config::bevy_ui()` returns
true; the browser index loads the same `tbg-client` binary target and shared asset
directory, with the old HTML UI removed. Native and web release workflows compile
that same crate. No platform-only laboratory layout/model/action branch was found.
The courtyard, model framing, controls and local reduced-motion correction from
this chat are ancestors of this runtime source. No per-platform duplicate patch
is needed, as the owner explicitly allowed. This is source propagation evidence,
not a Windows/web runtime result or a production deployment claim.

Local screenshots/logs: `.local/laboratory-arena-cover-output/final-*`, `interaction/`
and `installed-linux/`. Reference images, captures, packages and binaries stay local.

Pending, not passed: same-source packaged Windows and deployed desktop-browser
full-frame/motion comparison for the affected contract cases, including the earlier
laboratory and reduced-motion coverage. Queue those scoped checks at consolidation;
Linux observations/shared code do not prove parity. No additional target build,
full suite, whole-roster gallery or public deployment was run. No push under the
latest owner-provided AGENTS.md; the branch is `phase/laboratory-arena-cover`.
