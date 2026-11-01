# Laboratory courtyard and desktop usability

Owner request: use the supplied Bone Brute laboratory reference as visual guidance,
build a real procedural 3D stage in the existing arena style, polish the laboratory,
and reinstall the local native Linux client. The reference image stays local.

## Scope

- Preserve the shared header, all champion forms and animation actions, facts,
  exact-team editor, local trial controls and Practice rules.
- Three columns: fixed search/filter header and independently scrolling champion
  list, central real model and compact selected controls, inspector/team/bot pane.
- Match the arena mesh vocabulary: painted bevelled stone, faceted columns,
  stepped circular plinth, blue/gold banners, braziers and distant greenery.
- Reuse the existing rendering pipeline and settings; no new dependency, downloaded
  backdrop, game rule, balance value or server change.
- Search matches both localized and canonical champion names. Empty filters explain
  the result. Selected forms, animation, difficulty and reduced motion stay visible.
- Drag/wheel input follows the actual center viewport rather than fixed percentages.
  Leaving Troops restores the scene, lighting, camera and fog; the stage is removed.

## Validation and delivery

Level: work front, with the owner-requested native release build/install and focused
render observations. All compilation uses the shared host lock with two build jobs.
The owner has explicitly allowed waiting for that lock on requested local deliveries.

Runtime revision: `a7d7995f937d0947d35640f54e2f1c138e4d7ec0`, based on the exact
previously installed runtime `e1017fc55ed069e23d4df04d46b81aa2e12dac17` so this
visual delivery retains its authentication, reconnect and shared-menu integration.

Completed checks:

- `cargo fmt -p tbg-client -- --check`: pass.
- `git diff --check`: pass.
- JSON parsing and exact EN/pt-BR key parity, including the five added laboratory
  messages: pass. Search includes both the canonical and localized name.
- Release build, with the existing host cache and maximum two jobs:

  ```bash
  flock "/tmp/tbg-validation-$(id -u).lock" env CARGO_BUILD_JOBS=2 \
    CARGO_TARGET_DIR="$TBG_EXISTING_TARGET" PKG_CONFIG_PATH="$TBG_NATIVE_PKGCONFIG" \
    TBG_DEFAULT_SERVER=wss://api.tupibg.com/ws \
    TBG_SOURCE_COMMIT="$(git rev-parse HEAD)" \
    cargo build --offline --locked --release -p tbg-client --bin tbg-client --features ui-acceptance
  ```

  Final build output: `Finished release profile [optimized] target(s) in 3m 28s`.
  The executable was copied to a commit-specific local candidate while still
  holding the build lock. The first compilation exposed a moved `DummyMode`
  import; it was corrected before the successful native builds. No failure
  remains from that compile.
- Real Vulkan renderer on isolated Xvfb displays: Bone Brute / Practice / Idle
  at 1920x1080, Trident / Catalog / Walk at 1366x768, Bonehorn / Editor / Walk
  at 1920x1080. Panels fit, model and long weapon remain visible at default zoom.
  Initial render review led to stronger navy contrast, larger champion framing,
  a recessed back wall and ASCII rotation controls supported by the shipped font.
- Real X11 pointer/keyboard observations: searching `bruto` isolates Bone Brute;
  a conflicting faction displays the empty-results message; A2 updates the model
  and facts; dragging rotates it; the right pane scrolls independently of the
  stage; all three tabs open their corresponding content. Hard difficulty and
  the Tundra thumbnail/name reflect the selected settings.
- Actual Editor `Start` opens a running local trial; `Back to catalog` restores
  the stage. Actual Practice starts a local bot match in Tundra. The laboratory
  disappears and the arena camera, colors and lighting return correctly in both
  the same-arena trial and changed-arena bot transition. These are focused
  presentation transitions, not multiplayer or balance acceptance.
- `scripts/package-native.py --platform linux --binary <frozen-candidate>
  --output <local-output> --no-archive`, under the same host lock: pass.
  All 355 manifest entries match their sizes and SHA-256 digests.
- The package's `install.sh` completed successfully. Installed version:
  `0.1.2-rc.1-a7d7995f937d`, release, x86_64 Linux, `ui-acceptance`, protocol 31,
  contract `w31-r6-097385fee9a10b0f`. Both user launchers and the application-menu
  desktop entry point to this installation. Player data and earlier installs
  were preserved.
- Opened the installed executable and inspected its 1920x1080 laboratory render;
  rechecked its complete installed manifest and launcher/desktop targets: pass.
  Installed binary SHA-256:
  `35b6f3bdc44cdb62a8a5d9748ff15bf950ee2ea47b2c1aa7f366e9f0d4728102`.

Local render evidence is under `.local/laboratory-stage-output/`, including
`installed-linux/laboratory.png`, representative desktop captures and
`interaction-final/` transition images. Neither generated captures nor the
owner's reference image are versioned. The source is pushed on
`phase/laboratory-stage`; main and production services are unchanged.

Deferred, not passed:

- Desktop browser compilation/interaction at consolidation:
  `flock -n -E 75 "/tmp/tbg-validation-$(id -u).lock" env CARGO_BUILD_JOBS=2 cargo check
  --locked -p tbg-client --bin tbg-client --target wasm32-unknown-unknown`, then
  inspect Troops and the training transitions in the supported desktop browser.
  This Linux-specific delivery does not justify an additional Bevy target build.
- Windows runtime observations and a complete 46-champion/form/clip gallery;
  only representative Linux framing is claimed here.
- Full suite, balance gates, load tests and multiplayer acceptance were neither
  requested nor run. No competitive simulation, protocol or backend source changed.
- Final subjective approval of appearance remains with the owner.
