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
