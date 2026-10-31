# Native desktop feedback loop

Owner request, 2026-10-31: install the Linux app as **TBG**, reinstall after
each requested adjustment, and record every change in a local commit.
Linux is the reference implementation for Windows and possibly macOS.
No remote push, CI wait, public release or full-suite validation is requested.

Active branch: `codex/native-desktop-preview`, isolated from the original
checkout's in-progress policy edits. Protocol 20 and the deployed champion,
carrier, shield, Faction and arena changes are merged with the native Bevy
interface. Native decisions are D-70/D-71 after consolidation.

## Update procedure

1. Read the owner's current validation policy and the relevant UI files.
2. Implement the requested adjustment; preserve unrelated files and data.
3. Run formatting/static checks appropriate to the change and commit the
   requested source change locally. Never stage unrelated policy edits.
4. Run `scripts/install-native-preview.sh` with the existing Cargo cache
   and native dependency environment. It uses the shared nonblocking host
   lock and exactly two Cargo jobs, builds the development client, prepares
   an uncompressed local package and installs it as TBG. Busy exit 75 means
   defer; do not automatically retry or wait without an owner exception.
5. Observe the installed build launching outside the repository and report
   its actual commit. The owner reviews appearance using the installed app.

The preview uses a development build for quick iteration, not a public
release artifact. The installer keeps each commit's versioned directory,
updates `~/.local/bin/tbg` and the compatibility launcher atomically, and
leaves account keys/preferences in the existing user data directory.

## Validation

Level: installation work front. The owner explicitly requested the build
and installation. Shell/Python syntax, affected formatting and diff checks
passed before the build. Installation and runtime results are recorded
below after execution.

Deferred: `cargo test -p tbg-client --bin tbg-client -- --test-threads=1`
(targeted union to be selected at consolidation), current WASM/browser UI
consolidation, native/browser full match/reconnect/spectator scenarios,
release packaging/publication and Windows. Workspace tests/Clippy and
balance gates require an explicit full/heavy request. No passing result is
claimed for deferred checks; prior PR21 browser evidence does not validate
this new native combination.

## Installed result — 2026-10-31

- Local integration commit: `60ea1cb`. Installer/feedback-loop commit:
  `97b3aa4`. Installed source commit: `b96d8fab77c96c2aad642617a876a1bc81bbc650`.
- Version `0.1.0-native.1`, protocol 20. Application-menu name **TBG**;
  command `~/.local/bin/tbg`; native window title starts with TBG.
- `scripts/install-native-preview.sh` passed twice after relevant changes,
  with the shared nonblocking lock and two Cargo jobs. Initial native build
  completed in 13.53 s; the endpoint-default update completed in 3.30 s.
  Installed previews explicitly default to `wss://tbg.abastevo.com.br/ws`;
  direct developer Cargo builds may still use localhost unless overridden.
- Both commit-specific installations were retained, proving updates can
  replace the active launcher without deleting the older build or user data.
  The installer checks executable SHA-256 before copying. Installed metadata,
  assets, protocol and runtime shared libraries were checked successfully.
- `desktop-file-validate ~/.local/share/applications/tupi-battle-grounds.desktop`:
  passed. The desktop entry points to the current executable and PNG icon.
- Installed executable launched from `/tmp`, outside the source tree, using
  executable-relative assets. Native capture exited successfully. A fresh,
  isolated X11 capture showed the Portuguese lobby and complete creator
  footer at 1440x900. Its only warnings concern the virtual display's
  XSETTINGS and reported physical dimensions. The earlier Wayland capture
  showed the Ranking panel; no controlled input scenario is claimed for it.
- Captures and isolated profiles are under this worktree's ignored `.local/`;
  the owner's real preferences and account data were not used for smoke tests.
- Active persistent worktree is `.local/native-desktop` inside the original
  checkout. `.local/native-preview.json` in that checkout records its path,
  existing Cargo cache/dependency environment and installed commit. The
  owner's current AGENTS.md/VALIDATION.md are copied here **unstaged**;
  preserve them and never include them in an application-change commit.
- Only local commits were created. No push, CI wait, VPS change or public
  release occurred. Deferred checks above remain pending for consolidation.

## Browser-aligned lobby adjustment — 2026-10-31

Owner supplied Linux/browser screenshots and requested matching visual
positions: modes left, account/presence/navigation above, ranked record
bottom left, four feature cards below the clear center, live battles/news
right. Restore the written Tupi wordmark; keep the new emblem as the app
icon/profile badge. No game rules, server authority or account gates change.

Implemented as lobby-only modules and optional control palette tokens.
Other native panels retain their existing layout. Existing browser vector
UI symbols/news are rasterized to lossless WebP for the existing native
image loader; the written logo is an optimized WebP, never an SVG conversion.

Work-front checks: formatting, diff checks, asset format inspection and the
existing focused lobby-art JavaScript scenario. The owner-requested native
build/reinstallation and concrete desktop capture follow the source commit.
Defer the WASM opt-in Bevy UI check, interaction regression selection and
full suites to consolidation/explicit owner request; no browser build is
needed for the default browser's static wordmark restoration.

### Browser-aligned lobby result

- Installed source: `c172713`; branding restoration: `8b01391`; layout:
  `18c6b53`; profile clearance/presentation: `cdbd097`.
- The owner-requested build/install under the shared nonblocking lock and
  `CARGO_BUILD_JOBS=2` passed. Initial layout compilation took 19.03 s;
  final corrected build 7.71 s. `desktop-file-validate`, formatting and diff
  checks passed. The existing lobby-art JavaScript scenario passed.
- Intermediate failure: E0700 in the news-arrow bundle's borrowed label.
  Owning its text fixed the error; only the relevant native build was
  repeated. No test or compiler gate was weakened.
- Final installed app captures at 1920x972 and 1280x720 were inspected:
  modes left, profile bottom left, four feature cards below, live/news right,
  account/navigation above and no central panel covering the arena. Profile
  title clears the Stronghold badge. Existing runtime data was isolated.
- The written Tupi wordmark is lossless WebP (1024x410, 425330 bytes versus
  the 831090-byte original asset); original artwork is retained. The desktop
  icon is unchanged. The browser's wordmark was also restored and observed
  loaded, using only scoped TBG static-file changes in the deployment repo.
- Captures are `.local/lobby-final-1920.webp` and
  `.local/lobby-final-1280.webp`. Only expected virtual X11 display warnings
  were recorded. Existing windows were left running; reopen TBG to use the
  updated installation, and reload an already-open browser page.
- No full suite, extra gallery/multiplayer campaign, WASM rebuild, remote
  push or CI wait. Deferred interaction/WASM consolidation remains as above.

## Unified Linux header blocks — 2026-10-31

Owner scope from this point: **Linux only; change the web version only on
an explicit request.** Keep native preview changes local and reinstall each
requested application adjustment. No web assets, browser build, deploy or
remote push belongs to this adjustment.

The account/login block and online-player block now sit inside the same
full-height navigation strip as Arcanas/Arenas, using common cell dimensions
and dividers. The account still exposes the existing create/sign-in actions;
presence still opens Live battles and uses the existing public counts.
The redundant sound tag is removed from the header; audio controls remain
in Settings. Other screens and game rules are outside this change.

Validation level: work front. Affected formatting and `git diff --check`
passed before the owner-requested build/install. One installed desktop
capture follows. Auth/presence interaction regression selection remains
for consolidation; no full suite, WASM/browser check or multiplayer run is
authorized or claimed for this layout-only adjustment.

### Unified header result

- Installed application source: `90dfb1893fa307c77cc7a7bfa105422aa7225628`.
  The native build and installation passed under the shared nonblocking
  validation lock with `CARGO_BUILD_JOBS=2`; compilation took 12.14 s.
- Affected formatting, `git diff --check`, desktop entry validation and
  installed executable SHA-256 verification passed.
- Inspected the installed app at 1920x972 using an isolated profile and
  virtual X11 display: account, navigation and presence occupy one strip
  with full-height cells and dividers; the sound tag is absent. Existing
  Settings audio controls remain in source. Capture:
  `.local/header-blocks-1920.webp`. Only the expected virtual display
  XSETTINGS/physical-dimension warnings were recorded.
- The owner's running windows and real account/preferences were preserved.
  Reopen TBG to load this installation. No web, VPS, remote Git or CI action
  was performed. Auth/presence clicks and keyboard regression checks remain
  deferred to consolidation; no full suite or WASM check was run.

## Linux battle HUD reference adjustment — 2026-10-31

Owner requested browser-reference presentation for Linux only: heraldic
Stronghold HP plates and central clock above, Energy/interest/Army strip
above the Shop, Surrender at the upper right, a centered starting countdown
and a persistent arena chip. The larger arena introduction lasts five
seconds of real presentation time and has a close button; the chip can
show it again for five seconds. Presentation reads existing MatchInfo and
LatestView values; no simulation, economy, countdown duration or surrender
confirmation rules change. Other targets retain their prior HUD path.

Level: work front. Affected formatting, asset export and diff inspection
precede the owner-requested native preview build/install using the shared
nonblocking host lock and two Cargo jobs. Phase-specific debug captures
avoid racing startup when inspecting Countdown and Running. Queue manual
click/keyboard checks for Surrender confirmation, arena close/reopen and
Reroll gating for consolidation; do not run a full suite or any web build.

### Battle HUD result

- Installed source: `b007f2eb96d9452e746966c4ef395c9f543ba607`.
  `cargo build --locked -p tbg-client --bin tbg-client` and
  `python3 scripts/package-native.py --platform linux --binary
  "$CARGO_TARGET_DIR/debug/tbg-client" --output
  "$CARGO_TARGET_DIR/native-preview" --no-archive`, followed by the
  generated `install.sh`, passed. The existing cache was reused. Successful
  initial compilation took 13.21 s; final visual correction took 7.02 s.
- The shared nonblocking slot returned 75 once. The owner then explicitly
  authorized waiting/retrying in this chat and later waived the two-job
  limit for this chat. The completed builds waited under the same shared
  host lock and still used `CARGO_BUILD_JOBS=2`; no active build was stopped.
- `cargo fmt -p tbg-client -- --check`, `git diff --check`, Python AST
  inspection, lossless WebP format/dimension verification,
  `desktop-file-validate` and installed executable SHA-256 comparison with
  `build-info.json` passed. Native shape assets total 9942 bytes.
- Initial compiler errors were `E0308: expected FontSource, found
  Handle<Font>`, `E0599: no associated function or constant named
  new_with_justify` and then `type score::Readout is private`. Corrected
  the Bevy 0.20 text APIs and registered readout systems within their own
  modules. The final build has no compiler warnings.
- Installed app captures at 1920x972 were inspected in isolated X11/local
  Practice sessions: `.local/battle-hud-final-countdown.webp` shows the
  heraldic HP plates, clock, upper-right Surrender, centered countdown,
  arena introduction with close control, and a complete Energy frame.
  `.local/battle-hud-final-running.webp` shows the running clock, updated
  Energy/interest/Army values and persistent arena chip with the larger
  introduction gone. Countdown timing and combat rules are unchanged.
- The first capture exposed a missing `native-icons/arenas.webp` path and
  an image-fit issue in the Energy frame. Using the existing `island.webp`
  and stretching the frame over its border box fixed both. Final captures
  contain only expected virtual X11 XSETTINGS/physical-dimension warnings.
  Empty auxiliary labels are hidden, Synergy information is retained and
  Fortification controls follow the existing Stronghold selection state.
- The owner's running app, real preferences/account data and unrelated
  policy edits were preserved. Reopen TBG to load the new installation.
  No web source/assets, VPS, remote Git or CI operation was performed.

Deferred to consolidation: native mouse/keyboard scenarios for closing and
reopening the arena banner (each opening expires after five real seconds),
the two-step Surrender confirmation, Reroll availability across Energy and
War Council states, Stronghold inspection/Fortification and long player
names/ratings. Full suites, other targets and multiplayer campaigns were
not run; they remain outside this work-front validation. These are pending
checks, not passing evidence.

## Linux Stronghold Fortification panel — 2026-10-31

Owner requested the browser-reference Fortification panel in the Linux
client, at the left edge with its top aligned to the player's HP plate.
Both use the same score geometry constants so their alignment follows UI
scaling. The panel replaces the compact row above the Shop and opens via
the existing own-Stronghold selection action. It shows the existing icon,
HP, passive, branch/step/effect/cost details and availability note; close,
outside click and Escape use the existing deselection action. The panel
never predicts a Fortification result or changes prices/branch rules.

The existing Stronghold catalog formatting is shared with this Linux
panel; no web assets or behavior are changed. A debug capture option queues
the normal SelectTower action in an isolated Practice match for visual
inspection. Level: work front. Run formatting/diff inspection followed by
the owner-requested native preview build/install and one focused panel
capture. The owner permits waiting for the shared lock and waives the job
limit in this chat; reuse the existing cache and preserve active work.

Deferred to consolidation: `cargo test -p tbg-client --bin tbg-client
ui::fortify::tests -- --test-threads=1`, the existing
`ui::stronghold_data_tests` scenarios, and mouse/keyboard checks for close,
outside click, Escape, both branch purchases, insufficient Energy,
in-flight rejection/acknowledgment, a paused War Council and max tier.
No full suite, WASM build or multiplayer campaign belongs to this front.

### Fortification panel result

- Installed source: `716f7cc9fd7a67014e3d4bb7a5f74079af92e7a1`.
  `cargo build --locked -p tbg-client --bin tbg-client`, native packaging
  with `--platform linux --no-archive`, and the generated installer passed
  under the same shared host lock with `CARGO_BUILD_JOBS=2`. The first build
  took 11.42 s; the icon correction took 3.64 s; final text clipping took
  11.07 s. No compiler warnings or errors were recorded. Waiting for the
  occupied slot used the owner's explicit exception for this chat.
- `cargo fmt -p tbg-client -- --check`, `git diff --check`, desktop entry
  validation and installed executable SHA-256/source-metadata verification
  passed. The four existing browser Stronghold symbols were rasterized to
  lossless 240x300 WebP (52028 bytes total); the SVG sprite is unchanged.
- Inspected the installed panel in an isolated Command/Forest Practice
  session at 1920x972. The normal SelectTower action opens the left panel;
  its top aligns with the HP plate. The correct Command symbol, current
  HP, passive, branch names, next steps, effects, costs and close control
  are present. Final running capture shows affordable options and the
  branch-selection note. Capture: `.local/fortification-panel-verified.webp`.
- Initial capture review led to using the exact browser symbol instead of
  the lobby illustration and placing passive copy inside a clipped
  two-line parent. An early debug capture preceded 3D scene readiness; the
  final capture at seven seconds shows the loaded arena and corrected
  clipping. Only expected virtual X11 XSETTINGS/physical-dimension warnings
  were recorded.
- Actual mouse/keyboard purchases, close, outside click and Escape remain
  deferred as listed above; queued-action visual observation does not
  claim those scenarios passed. No full suite, web build/assets change,
  remote push, CI wait or VPS action occurred. The owner's existing app
  windows, real preferences and unrelated policy edits were preserved.
  Reopen TBG to load this installation.

## Linux champion inspection and compact Shop — 2026-10-31

Owner requested the reference champion status design at the bottom left,
sharing the Stronghold panel's left edge, and uniform gold Shop frames and
coin/cost slots following the second reference. The Energy strip and all
five cards use one 0.9 transform, including fonts and spacing; a matching
translation keeps their previous bottom center anchor. The unscaled Shop
width remains 862 units and card height 225 units. Champion inspection
shares the side/bottom layout constants and keeps live Report statistics.
The own-unit Evolution choices move into the same styled panel, preserving
existing actions, costs and guards instead of leaving the old central UI.

Level: work front. Affected formatting, frame export/format and diff checks,
then the owner-requested native build/install under the existing shared
lock/cache. The owner's wait/job-limit exceptions remain scoped to this
chat. A debug unit-capture option queues the normal SelectUnit action for
an actual own unit. No web assets or rule changes belong to this request.

Deferred to consolidation: `cargo test -p tbg-client --bin tbg-client
ui::inspect::tests -- --test-threads=1` and the existing `ui::evolve::tests`
regressions. Native mouse/keyboard scenarios: champion selection/close,
enemy/dead/summoned/evolved units, Evolution affordability and War Council,
card buy/hold interactions under the scaled transform, Reroll, and resize
at the supported desktop sizes. No full suite or other-target build runs
on this front; these pending checks are not passing evidence.

### Champion panel and compact Shop result

- Installed source: `8066717506463318ed38ace69697d28efdf4d182`.
  `cargo build --locked -p tbg-client --bin tbg-client`, Linux packaging
  with `--no-archive` and the generated installer passed using the existing
  cache and shared host lock with `CARGO_BUILD_JOBS=2`. Waiting for the
  occupied slot used the owner's exception for this chat. Compilation took
  9.27 s initially and 5.62 s for the final close-control update.
- `cargo fmt -p tbg-client -- --check`, `git diff --check`, the native frame
  exporter, WebP format/dimension verification (332x448, 7410 bytes), desktop
  entry validation and installed executable SHA-256/source-metadata checks
  passed. Initial unused close-action/legacy-palette warnings were resolved
  by wiring both close controls and keeping the legacy palette on its
  remaining targets. The final compilation has no warnings or errors.
- Inspected the installed app at 1920x972 in isolated local Practice with
  an actual own Shatterback selected through the existing SelectUnit action.
  The status panel is at the bottom left on the same side margin as the
  Stronghold panel, with portrait, live HP, effective statistics and own
  Evolution choices. Separate close controls retain the existing inspection
  and Evolution actions. No old central native Evolution panel remains.
- All five Shop cards have uniform gold frames, separate title/role lines
  and coin/cost slots. The Energy strip and cards share one 0.9 transform
  and bottom-center anchor; fonts, gaps and controls scale together.
  Their common bottom aligns with the champion panel. Capture:
  `.local/champion-shop-final.webp`. Only the expected virtual X11
  XSETTINGS/physical-dimension warnings were recorded.
- The capture verifies presentation and queued selection; actual pointer,
  keyboard, purchases and other unit states remain deferred as listed
  above. No Rust tests, full suite, web build/assets changes, remote Git,
  CI wait or VPS action occurred. Real preferences, account data, running
  app windows and unrelated policy edits were preserved. Reopen TBG to
  load the updated installation.
