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

Local validation follows [AGENTS.md](../../AGENTS.md#verification) and
[VALIDATION.md](../VALIDATION.md): minimal work-front checks, targeted tests
once at consolidation, and full-suite/release checks only on an explicit
owner request. Record exact results, deferred checks, measurements and
deviations below; publication still requires the applicable acceptance.

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

## Work-front implementation report — 2026-10-31

**Level:** work front under the updated owner policy. Commands executed before
adopting that policy are historical evidence only. Other fronts' edits to
AGENTS.md and the validation documentation are preserved.

### Implemented

- TLS-enabled ewebsock; production native default/compile-time override;
  environment and command-line room/challenge/watch/language parameters.
- Shared local/session storage. Native preferences use atomic JSON writes,
  private Linux permissions, corruption fallback and retained account keys.
- Native WAV audio, persistent mute/volume, clipboard copy/paste, public links,
  executable-relative assets, remembered window size/mode, fullscreen/VSync,
  shadows, a stable application ID and five diagnostic log files.
- Shared Bevy lobby, stats, news, play modes, friends, settings, Arenas,
  Strongholds/branch details, Arcanas drafts/prerequisites, live matches and
  account presence. The existing queue, account, Ranking, Evolution,
  Fortification, result and rematch paths are integrated with the kit.
- Bounded text editing, cursor/selection, paste, mask/reveal, keyboard focus,
  shared button states, scrolling retained across screen refreshes, Shop
  portraits, numeric HP/max HP and spectator controls without seat actions.
- `/api/version`, SemVer update discovery, portable archive/AppImage scripts,
  checksums, exact executable/source identity checks, a per-user installer
  and a Linux-only GitHub release workflow. Windows follows Linux acceptance.
- Owner-selected custom visible-attribution LICENSE and persistent creator/
  repository links in Bevy and the existing HTML game/Ranking pages (D-71).

### Minimal checks completed on this front

- `cargo fmt -p tbg-client -p tbg-server -- --check` and `git diff --check`:
  see the final handoff command result; no broad suite is inferred.
- `flock -n -E 75 /tmp/tbg-validation-$(id -u).lock env
  CARGO_BUILD_JOBS=2
  PKG_CONFIG_PATH="$PWD/.local/native-deps/root/usr/lib/x86_64-linux-gnu/pkgconfig"
  cargo check -p tbg-client --bin tbg-client`: passed on the final code,
  `Finished dev profile [unoptimized + debuginfo] ... in 2.08s`.
- Python AST parsing of the packager and embedded installer, JSON catalog
  parsing, HTML origin-link assertions and `bash -n` for both shell scripts:
  passed. These are static checks, not package/install acceptance.

### Historical checks and observations before policy adoption

- `cargo test -p tbg-client`: `105 passed; 0 failed; ... finished in 0.29s`
  on the earlier working-tree implementation. Later button, attribution,
  catalog-formatting and keyboard changes have not had their tests rerun.
- `cargo clippy --workspace -- -D warnings`: passed on the earlier diff
  (`Finished dev profile ... in 4.61s`); not a current-tree/full-suite claim.
- `cargo check -p tbg-client -p tbg-core --target wasm32-unknown-unknown`:
  passed on the earlier diff, with `warning: unused variable: lang` in the
  legacy DOM bridge; the unused binding was corrected afterward.
- An already-started `trunk build --release` completed with exit 0 after
  policy adoption. Its code predates the final attribution/button changes.
- `cargo run -p tbg-client --example native_probe --
  wss://tbg.abastevo.com.br/ws`: `Native handshake OK · protocol 17`.
  This verifies WSS compatibility, not a completed online match.
- Native development builds opened on Pop!_OS 24.04, NVIDIA RTX 4060 Laptop,
  Vulkan; screenshots were saved on Wayland and an isolated X11 display.
  Reviewed lobby, Arcanas, Strongholds, Arenas, settings and live surfaces.
  Captures under ignored `.local/native-captures/` predate the final footer.
- The previously started workspace test was stopped after the new validation
  policy was adopted. It has no passing result. No balance gate was run.

### Resolved failures during implementation

- Old test expected a relative native invite; actual value was
  `http://127.0.0.1:8080/?challenge=0f0f0f0f0f0f0f0f0f0f0f0f0f0f0f0f`.
  Native sharing now requires a full URL; that non-contract test was updated.
- `characters the D5 subset cannot draw` reported U+2212 in two UI sources;
  replaced it with an ASCII minus. Existing glyph tests were preserved.
- `assertion failed: !change(&mut draft, 0, -1)` was an incorrect new test:
  Shieldwall requires Guard rank 1, not rank 2. The test now rejects removing
  the prerequisite's final rank; core rules and contract tests are unchanged.
- `error[E0277]: the trait bound ScrollPosition: Copy is not satisfied`:
  preserving the inner Vec2 fixed the scroll snapshot.
- The initial Trunk dependency download failed with `Read-only file system
  (os error 30)` under the sandbox. The already-started authorized dependency
  download/build then completed. No registry permissions were changed.

### Deferred to consolidation

Run only the selected union, under the shared host lock and two Cargo jobs;
use `--test-threads=1` for tests. Do not run the workspace suite by default.

- `cargo test -p tbg-client --bin tbg-client -- --test-threads=1`: new storage,
  text, Arcanas, live-parser, version and Stronghold-display regression tests.
- `cargo test -p tbg-server --lib version::tests -- --test-threads=1`:
  public discovery response/cache/protocol contract.
- One current WASM check/build and desktop browser observation for the final
  integrated UI. The old web build is not proof of the final attribution code.
- One consolidated online scenario: native/browser, account login, private
  room/challenge, full match, reconnect and spectator exit; separately verify
  background/minimized processing. Current evidence is only a TLS handshake.
- Owner review of the final screens. Keep the HTML default until approval;
  cleanup/default switching and final visual parity acceptance remain open.

### Release work still pending

The original artifact request authorizes the release deliverables, not an
unrelated full suite. After consolidation/visual acceptance, build under the
shared lock with two Cargo jobs and `TBG_SOURCE_COMMIT=$(git rev-parse HEAD)`:
`cargo build --locked --release -p tbg-client`, then package the exact binary
with `scripts/package-native.py` and `scripts/package-appimage.sh`.
Verify archive extraction, launch outside the repository and the installer.
Publish the versioned GitHub artifact, download that release and install it
on the owner's PC, then finish Windows. No release, remote branch/tag push,
production deployment or real installation has happened in this front.

### Limitations

- Full UI acceptance, HTML cleanup/default switch, final captures, artifact
  sizes, cross-play/reconnect and real minimized-window checks are pending.
- Native keys are plaintext with local permissions; system keyrings deferred.
- Mobile work, PWA, itch.io setup, Windows packaging/signing and device QA are
  deferred. No Steam or paid infrastructure is introduced.

## Desktop preview integration — 2026-10-31

Owner requested a locally installed app named TBG, reinstallation after each
requested adjustment, and a local commit for every change. This preview
installation precedes public release acceptance; no GitHub publication or
Windows completion is claimed. PR21 head 0047830 is integrated to preserve
protocol 20 and the current server rules while retaining the Bevy desktop
interface. D-70/D-71 identify the native and license decisions after merging
the already deployed D-60 through D-69. Other active work is preserved.
