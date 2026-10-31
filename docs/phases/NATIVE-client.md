# Native Linux and Windows client

Owner request, 2026-10-31. Branch: `phase/native-linux-windows`.

## Scope and order

Ship the complete Bevy interface on Linux first, publish a versioned GitHub
Release, install that exact artifact on the owner's Linux PC, then finish
Windows. The owner explicitly chose full UI parity before publication.
The supplied native plan is a reference; it does not authorize paid
services, a license choice, production deployment, or game-rule changes.

Read first: AGENTS.md, ARCHITECTURE sections 6 and 10, DECISIONS D-12,
D-20, D-21, D-27, D-29, the client entry point and the affected UI modules.

## Steps and acceptance

1. Platform: TLS, production endpoint override, atomic private storage,
   native audio, clipboard/links, launch arguments, window controls, logs.
   Check storage isolation/corruption/permissions; compile native and WASM.
2. Shared Bevy kit and text editing; reference captures and parity list.
   Check editing, bounded input, clipboard, focus, scale, browser builds.
3. Account, lobby, friends, settings, Arenas, Strongholds, Arcanas, live
   matches, battle HUD, queue/result/rematch. Reuse existing UiActions and
   server-authoritative intentions. Validate data parsers and editor rules.
   Observe complete practice and online play, spectator and reconnect.
4. Owner reviews concrete screenshots. Keep the HTML path available until
   that approval; the new path is selected with `?ui=bevy` on the web.
5. Version discovery and download link, Linux tar.gz/AppImage packaging,
   checksums and reproducible GitHub Actions. Publish only validated output.
6. Install the released Linux artifact in the user's home, add a desktop
   launcher, and observe startup from outside the repository.
7. Windows portable executable and installer, CI build and checksums.
   Report hardware checks that require a real Windows device.

Standard verification: fmt, workspace clippy/tests, WASM check and Trunk
release build. Record exact failures, measurements and deviations below.

## Report

- Baseline: `9008f11` (origin/main); local main was 263 commits behind.
- Branch created; unrelated untracked `.claude/` preserved.
- GitHub CLI authentication works outside the filesystem sandbox.
- ALSA runtime is installed; development files extracted into ignored
  `.local/native-deps/` without changing system packages (sudo needs a password).
- Implementation and acceptance: in progress.

## Deviations

- One branch for the entire owner-requested Linux-then-Windows effort.
- PWA and itch.io account setup are independent follow-up work; the owner
  requested native executables and GitHub Releases.
