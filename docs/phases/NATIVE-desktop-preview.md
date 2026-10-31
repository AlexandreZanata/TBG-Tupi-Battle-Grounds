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
