# P69 — Independent native Android landscape prototype

**Status:** resumed by the owner on2026-11-02; native landscape implementation
and emulator consolidation in progress on`codex/p69-mobile-playable`. See the
[current continuation](P69-mobile-20261102.md). The2026-11-01 physical-phone
checkpoint below remains historical, partial device evidence; no current
physical-phone validation is claimed.
**Planning branch:** `codex/p69-android-landscape-plan`.
**Execution branch:** `codex/p69-android-native-prototype`.
**Reviewed Linux base:** `014b0e0c046b38c551ee24534ee00709cde0e493`
(`0.1.0-native.3`, protocol 29, RulesetVersion 2).
**Decision:** D-98. **Architecture:** §29. **Canon:** GAME-DESIGN §23.

## Current owner continuation (2026-11-02)

The owner resumed implementation, selected current `main` as the base and
explicitly requested inheriting open PRs #57 and #58, emulator-based review of
all real screens against local Linux, adaptive landscape composition and a
GitHub PR with verified CI. This instruction supersedes the historical pause,
source-ownership/P68 exclusion and physical-phone requirement for this round.
It does not certify ARM64 runtime, physical performance or store readiness.

Original selected main: `b964875e2219bafe129b19960f7f1369307870d5`.
Current integrated main: `177da4a042260bf044cf7f9e323a66fcb89ecfbc`,
including PR59/60 runtime/auth and PR61 documentation.
Inherited heads: #57 `43d25a8c4845922f43bd74727f6ae6975351d740`,
#58 `b61764dbed4d59e0654a8ced9c865ec68be8ab5c`.
Android checkpoint: `d2a3c6b9d1032133487911f9d88669936bc83bc4`.
The current shared contract is `w40-r16-f2e253b98ccb313a`.
Architecture §29 / GAME-DESIGN §23 hold P69 after resolving the P75 section
number collision; P75 planning and proposed rules remain unchanged.
See [current execution](P69-mobile-20261102.md) for actual evidence.

## 1. Owner contract and independent scope

**The Android native game is played with the phone held horizontally
(landscape). The existing mobile browser experience remains portrait.**
P69 changes neither the browser's orientation nor the desktop layout. It
creates a genuine installed Android application with a touch-oriented
landscape interface and the same gameplay as the installable Linux client.

**P69 is a separate workstream from P68. It has no dependency on completing
Windows packaging or the new authentication module.** Its branch starts
directly from the reviewed Linux native.3 commit above, not P68's planning
or unfinished implementation branch. Do not merge/cherry-pick ongoing P68
work, change its status or modify its Windows/auth artifacts. Reserve its
phase/decision/section numbers without importing its implementation.

Reuse `tbg-core`, `tbg-client`, art, animations, content, messages and the
authoritative server. Touch presentation changes how a player operates the
client; it does not create Android-specific damage, economy, bot rules,
limits, rewards or match outcomes. Kotlin is a thin Android host/service
bridge only. No Kotlin Multiplatform, Compose UI framework, WebView/browser
wrapper, new engine or copied Kotlin simulation is part of this prototype.

Deliver a signed internal arm64 APK that opens on the owner's POCO X6 Pro,
supports offline Practice, useful mobile navigation and a real Android versus
Linux test match with the same wire/ruleset. Do not call an emulator scene,
APK build, logo screen or functioning tap a complete playable prototype.
Google Play launch, billing, a mobile updater, iOS, tablets/foldables,
production deploy and public Android support promises are outside P69.

## Read first

- Current root `AGENTS.md` and the owner's minimum-validation policy; use
  `docs/VALIDATION.md` where present. This task explicitly authorizes Android
  design despite the normal desktop-only focus. Do not copy unrelated dirty
  policy files from other worktrees into the phase.
- This phase, [acceptance/runbook](../planning/P69-ANDROID-ACCEPTANCE.md),
  [device reference](../planning/P69-DEVICE-REFERENCE.json) and
  [source manifest](../planning/P69-ANDROID-SOURCE-MANIFEST.csv).
- GAME-DESIGN §§3/5/22, ARCHITECTURE §§3/6/25 and D-98; the Linux release
  report `docs/phases/CONSOLIDATED-LINUX-RELEASE.md` and `art/branding/README.md`.
- The named source files below and their direct dependencies as needed.
  Read Bevy 0.20's installed examples before coding platform APIs. No archive.

## 2. Audited starting point and actual phone

| Item | Evidence / implication |
|---|---|
| Native Linux identity | Selected reviewed base is native.3/protocol 29/RulesetVersion 2; record actual executable and content identity again at implementation |
| Shared mechanics/content | 46 conventional champions and 51 presentation kinds including summons/objects; use canonical IDs/forms/resolvers, not a reduced Android roster |
| Binary-only entry | `crates/tbg-client/src/main.rs` owns plugin setup; add one shared library/app builder for desktop/web and Android without cloning the game |
| Native is often treated as desktop | `main.rs`, `platform/mod.rs`, native Cargo dependencies and window module use `not(wasm32)`; Android needs explicit platform services |
| Desktop minimum window | `platform/window.rs` clamps to desktop sizes; never apply its 1280×720 logical minimum to a phone surface |
| Linux-only frontend conditions | Shell, HUD, loading, ranking and visuals need presentation-capability review; do not blindly replace every Linux cfg |
| Partial touch handling already exists | `ui/input.rs` consumes touch release/long press, while other shell input uses desktop pointer/keyboard paths; audit pointer IDs, cancel/scroll and synthetic mouse duplication |
| Text entry is desktop-oriented | `ui/textform_bevy.rs` reads keyboard events and clipboard shortcuts; prove Android IME/commit/paste early |
| Asset discovery is not a portable runtime path | `build.rs` generates art paths; its `TARGET.contains("linux")` also happens to match `aarch64-linux-android`. Use explicit target/capability logic, not a misleading substring |
| APK asset access differs | `platform/window.rs` resolves assets beside an executable/source checkout; Android must use the engine's APK AssetManager path |
| Branding exists | `art/branding/android/res/` has launcher resources; reuse them after validation instead of generating another logo |
| Source ownership | P68 was observed working in a separate dirty worktree. Its uncommitted auth code is excluded; coordinate shared-file ownership before implementation |

Read-only ADB inspection on 2026-10-31 confirmed the owner's reference:

- User-reported POCO X6 Pro; reported model `2311DRK48G`, codename `duchamp`,
  manufacturer Xiaomi, platform `mt6897`.
- Android **16 / API 36**, security patch `2026-08-01`, ABI **arm64-v8a**.
- Physical portrait panel **1220×2712 px**; landscape design reference
  **2712×1220 px**; density **480 dpi**. Raw density arithmetic suggests
  roughly **904×407 dp before insets**; this is not a measured app viewport
  or proof that Bevy logical coordinates equal Android dp.
- Memory page size **4096 bytes**. This phone cannot establish 16 KB-page
  runtime compatibility; that needs separate inspection/test evidence.

No device serial, Android ID, IMEI, account or private phone data belongs in
the report. Model/OS/display facts are not GPU performance or battery evidence.
Only properties were read during planning; no APK was built or installed.

## 3. Exact gameplay equivalence with Linux

Freeze one reviewed execution SHA and build both the Android candidate and
its Linux comparison from it, using the same Cargo.lock, core content and
ruleset. Preserve the installed Linux application; use a separate comparison
package/data directory. The initial audited base is explicit, so a later
reviewed Linux change requires a recorded delta rather than a silent rebase.

The Android package must use the same:

- `PlayerIntention` semantics, seat ownership, sequence/ack/rejection logic,
  protocol, matchmaking and authoritative outcomes;
- `Energy`, Interest, Shop offers/prices/reservations, mode-specific Reroll,
  Evolution, Synergy, Arcanas, Augments, Strongholds and Fortification;
- champion kinds/forms/stats, targeting/ranges/statuses, summon behavior,
  seeded bot difficulty and ruleset selection for each matching mode;
- fixed simulation ticks, disconnect window, surrender/result/rating rules,
  training dummy configuration and the existing complete victory cast.

Never compare Android Practice-preview rules against Linux Ranked CANON as
if they were the same mode. Preserve all mode boundaries and existing gated
features. No mobile unit cap, slower combat, aim assistance, auto-purchase,
extra pause, free resources or hidden bot handicap is authorized.

Reuse the existing `tbg-core/examples/trace_canon.rs` as a focused Linux/ARM64
trace probe: same five existing seeds, tick trace/final state and end ticks.
Compile/run it once on each target during consolidation and compare outputs.
This proves a specific native-ABI core scenario, not the whole game or APK.
Complement it with action-to-intention tests and a real packaged cross-play
journey. Do not regenerate deterministic goldens or change thresholds to
make an Android defect pass. No balance gate is needed for unchanged rules.

## 4. Native Android architecture and toolchain

### 4.1 One game library and a small host

Extract the existing app setup into `crates/tbg-client/src/app.rs` and one
`lib.rs`; keep `main.rs` as the desktop/web entry. Add an Android cdylib
entry using the **exact Bevy/winit AndroidApp type and initialization pattern
for the selected 0.20 source**. Share every gameplay/presentation module;
do not create an Android game-code copy or a second Rust workspace/core.

Use `platform/android/` at repository root for the Gradle host, with a thin
`MainActivity.kt` based on compatible GameActivity. Rust/Bevy owns rendering,
world and frame loop. Host callbacks marshal lifecycle/IME/insets/clipboard/
safe URL operations without blocking or touching Bevy world state on the
Java UI thread. Do not start a second competing event loop.

Current installed Bevy metadata identifies upstream source
`b3bdd75becaffe9a16677cf5bae5bd6d198f052b`. Its packaged README documents
mobile builds, but the complete mobile sample is not present in that crate
package and exact-source sample URLs were unavailable during planning.
Resolve matching official source or equivalent installed dependency examples
at Step 0; do not copy unverified main/older-version APIs. A missing compatible
entry/activity integration is an early spike blocker, not grounds to upgrade
the engine or replace it without review.

Keep small OS implementations in `src/platform/android/`. A mobile-landscape
UI profile owns layout/gesture differences under `src/ui/mobile/`, reusing
existing kit/assets/catalogs, `UiAction` and action routing. Prefer functions
and a small profile/resource over a generic platform framework.

Moving the client's modules from bin to lib also moves their tests: update
affected CI/consolidation selectors to `--lib` where appropriate and verify
nonzero test counts. A stale `--bin` filter returning zero tests is not a pass.
Coordinate a narrow ownership window for shared Cargo/bootstrap/UI files
with active fronts; separate worktrees do not justify concurrent conflicting
edits. P69 does not need the rest of P68 to finish.

### 4.2 Build and package inputs

Use the supported wgpu Vulkan path as the primary Android candidate and
record the actual adapter/driver/backend. Any GLES alternative needs an
explicit compatible build/shader check; software rendering or stripping
game information cannot stand in for a playable native result.

Start with **arm64-v8a / aarch64-linux-android**, matching the actual phone.
Proposed prototype floor is Android 12/API 31, with compile/target API 36 for
the reference Android 16 device. A minimum install floor is not a support
claim for untested Android 12–15 phones. Confirm compatibility of Bevy audio,
GameActivity, NDK and Java APIs before freezing those values.

Pin the working Rust toolchain (minimum 1.97 on this base), SDK, NDK,
GameActivity, AGP/Gradle wrapper, JDK and cargo-ndk versions in the Android
build instructions. Prefer NDK r28 or later for modern alignment support;
verify the actual Rust linker's output and every bundled library rather than
assuming an NDK setting covers Rust cdylibs. Keep one compatible
android-activity/winit dependency lineage. A direct JNI dependency, if needed,
must match the reviewed locked adapter (0.21.1/0.22.4 both exist on this base);
do not mix incompatible JNI types or add another Android glue stack.

Split desktop-only arboard/open/window dependencies away from Android, keep
WASM-only libraries on WASM and enable the engine's Android GameActivity
feature only for the Android target. Preserve Linux/Windows/browser feature
selection and compile behavior. No unrelated dependency/toolchain upgrade.

Gradle packages the already built `libtbg_client.so`, its required native
dependencies and the same validated game asset inventory. Do not let a
hidden Gradle task launch unrestricted Cargo or silently package an old .so.
Use one shared build lock around sequential native compilation/packaging,
two Cargo jobs, and at most two Gradle workers/no parallel task execution.
Preserve existing caches; no new target directory to bypass contention.

APK artifacts identify source SHA, Linux base version, Android prototype
revision, wire/ruleset/content identity, ABI, toolchain, signing certificate
fingerprint and final SHA-256. Use a separate prototype application ID,
recommended `io.github.alexandrezanata.tbg.prototype`, and monotonically
increasing versionCode. A stable private debug/test key supports in-place
test upgrades; it is not a public publisher signature. No key/password,
local.properties, .gradle/.cxx/build output or APK enters Git.

### 4.3 File ownership and reproducible artifact layout

- `crates/tbg-client/src/{lib,app}.rs` and the thin existing `main.rs` own
  shared startup; `src/platform/android/{mod,lifecycle,services}.rs` owns
  OS integration; `src/ui/mobile/{mod,layout,input,shell,battle,details}.rs`
  owns cohesive landscape view/gesture differences. Reuse existing forms,
  models/resolvers and action dispatch rather than adding parallel rules.
- `platform/android/` holds the pinned Gradle wrapper/settings/build files,
  `app/build.gradle.kts`, application manifest, thin MainActivity and source
  sets for necessary debug-only loopback policy. Reuse branding resources
  from `art/branding/android/res/` and the shared game assets through one
  defined build input; avoid untracked manual copies.
- A small `scripts/package-android.py` may create/check deterministic asset
  inventories and source metadata using the standard library. It must not
  execute an Android binary on the Linux host or silently reuse a stale .so.
  Build Rust explicitly, then let Gradle package that verified input.
- Embed source/version/wire/ruleset/ABI/asset identity in `build-info.json`.
  Put final APK SHA-256 and signing certificate fingerprint in an external
  artifact report after signing; do not try to embed an APK's own final hash
  inside itself. Verify Android runtime identity against the handoff manifest.
- Add focused ignores for Android build/tool output and private signing/
  local SDK configuration. Keep required pinned wrapper tooling verified,
  game binaries out of Git and signing material outside tracked sources.

No server, schema or core rule change is expected. Any shared source refactor
retains desktop/browser behavior and must be reviewed for integration with
other active fronts without absorbing their unfinished work.

## 5. Landscape interface and Android services

Use `sensorLandscape` for both horizontal directions and identify the app
as a game. Recalculate usable viewport, cutout/gesture/navigation/IME insets
and hit coordinates when the surface changes. Android can override requests
on some form factors/OS versions: keep the game layout horizontal, handle an
unexpected portrait/narrow surface safely with a rotate/restricted-viewport
message, and do not reset a match or change server time. Never change the
phone's global rotation settings. Tablet/foldable certification is deferred.

Keep the same medieval art, champion rigs and readable mechanics, with this
mobile composition: compact status along the top, BattleLine/arena in the
center, five Shop offers and primary actions along the bottom, and an
on-demand details drawer. Critical actions have at least 48×48 dp effective
targets after verifying Android-to-Bevy scale mapping. Support the POCO's
actual usable viewport rather than shrinking a desktop screenshot/canvas.
See the acceptance file for screen/interaction coverage.

Touch ownership must distinguish tap, long press, scroll, cancel and pointer
identity. One tap produces one action; a cancelled/dragged gesture buys
nothing; a long press used for inspection cannot also buy on release.
Suppress synthetic mouse duplication without discarding a real external
mouse or legitimate quick repeated taps. No new gameplay input cooldown.
Provide visible detail/confirm affordances; no essential hover/keyboard-only
flow. Respect UI draw order/modal clipping and empty-space deselection.

Prove system text entry early: name/key/room/search, committed text, accents,
paste, masked/reveal state, Done/Back and keyboard occlusion. Use Bevy/winit
IME where supported; if that path cannot support required editing, a small
native EditText modal is permitted through the host, reusing validation and
submission events. No custom on-screen keyboard or Compose framework.

AssetServer uses Android AssetManager/APK assets and embedded catalogs/fonts;
runtime paths cannot depend on the checkout, current executable directory
or external storage. Private nonsecret preferences/logs use app-internal
storage. Clipboard and public links use Android services/validated intents,
not desktop arboard/open or shell interpolation. Reuse the existing icons,
audio, muted/volume settings and low-motion behavior. Handle audio-focus
loss and system volume normally. New text has English/pt-BR catalog keys.

Back dismisses keyboard/dialog/details first, then follows navigation;
leaving an active online match needs a clear confirmation and the existing
disconnect/surrender semantics. It must not silently surrender. Suspended
surfaces/audio/input are restored without duplicate apps/cameras/units.
The server applies the same BR-M03 connection-pause budget and reconnect
window used by Linux, then decides any forfeit. Clients keep retrying through
the existing terminal replay retention. No background-service trick or
Android-specific pause or reconnect grace is allowed. Offline execution does not gain wall-clock
catch-up rewards; process death may lose an unsaved local session as on the
current unsaved client, and that limitation is stated honestly.

## 6. Authentication boundary: reuse, do not implement P68

Keep the selected Linux base's existing account protocol. Manual AccountKey
login may be reused in process memory for this prototype; never persist it
in Android preferences, files, backups, logs or command lines. The ordinary
game preferences store holds nonsecret settings only. Retain explicit manual
key backup/copy UX with an informed user action; no automatic secret export.

**Do not implement Access/Refresh Tokens, rotation, new HTTP auth routes,
vault architecture, browser-cookie changes or the Windows module in P69.**
The prototype does not wait for P68-A. If a later reviewed common base
already changes authentication, consume its established client contract
through a separately coordinated integration; never copy unfinished work,
downgrade production or silently bypass a new server's auth. A compatible
isolated protocol-29 test endpoint remains valid for this frozen prototype.
Do not claim new persistent login or P68-A acceptance from an Android test.

## 7. Ordered execution and delivery

1. **Freeze/ownership/toolchain:** reconcile source with Linux native.3,
   record actual content/wire, refresh audit manifest, verify the selected
   ADB phone without exposing its serial, and resolve exact 0.20 Android
   entry/Gradle compatibility. Keep P68 code and personal device settings.
2. **Native boot slice:** shared library/bootstrap plus minimal Android host,
   packaged art/audio, landscape viewport and useful log/error behavior.
   First run on POCO: a real arena, one representative champion, sound and
   a working touch control. This is a milestone, not full acceptance.
3. **Playable offline slice:** actual Practice/bot and training controls,
   canonical Shop/Energy/Evolution/Stronghold/Augment behavior and cleanup.
   Add touch/IME/platform handling and the mobile layout using shared actions.
4. **Frontend coverage:** adapt the existing navigation/content/details/
   ranking/selection/result screens; cover all 46 champion/51 presentation
   identities, with representative actual animation/form paths. Record any
   pre-existing Linux limitation, never remove content silently on Android.
5. **Cross-play:** use one compatible test server, synthetic accounts or
   guests as appropriate, Android versus same-source Linux casual/private
   match, control actions, one reconnect and result/rematch/cleanup. No VPS
   deployment, real ranked farming or auth/protocol rewrite.
6. **One consolidation:** run scoped regressions and the existing trace probe
   on Linux/ARM64, then test the downloaded signed APK on the POCO. Group
   functional/rotation/lifecycle/layout checks and one controlled ten-minute
   observation. Inspect APK/ELF alignment and report untested configurations.
7. **Handoff:** fix concrete defects, repeat only affected checks, write
   `docs/phases/P69-execution.md`, update P69 status and deliver the dedicated
   branch/PR to main when the execution handoff is invoked. No automatic
   merge, public store/release, phone-data deletion or production activation.

## 8. Validation, acceptance and decisions

Planning validation is documents/links/JSON/CSV and `git diff --check` only.
ADB property inspection is read-only. No Cargo/Gradle build, SDK install,
APK/device modification, gameplay test or P68 implementation is part of this
planning delivery. Current owner minimum-validation policy governs execution.

Front mode uses the smallest relevant formatting/check; queue runtime tests
once for consolidation. Future invocation of the attached execution prompt
specifically requests Android builds/APK install on the named phone, a
separate Linux comparison, the named trace probe, focused cross-play and
one ten-minute device observation. It never requests the full suite,
balance gate, broad benchmark, repeated gallery or unrestricted soak run.
All local compiling/packaging uses the shared nonblocking host lock and
`CARGO_BUILD_JOBS=2`; exit 75/missing lock/cold front cache means deferred.
Do not wait/poll, kill other builds, clean caches or bypass via another target.

The acceptance matrix is AN01–AN18. A usable native boot slice is partial;
completion requires offline/cross-play/gameplay equivalence and physical
phone evidence. A single POCO does not establish all-Android, older-phone,
16 KB-device or Play readiness. Reference target is at least 30 fps in the
existing 30-unit comparison scenario, with 60 fps as a measured goal; these
are proposed performance inputs, never a gameplay unit cap. Record scene,
counts, settings, thermal/battery/memory/frame-time observations and limits.

- **O01 — Scope/support:** POCO X6 Pro/Android 16 is the primary device;
  minimum API 31 is proposed and untested older versions are not certified.
- **O02 — Look/fun:** owner reviews landscape controls, information density,
  legibility and equivalent access to all combat actions on the real phone.
- **O03 — Build inputs:** freeze compatible SDK/NDK/host tools, identify any
  required new license authorization, and keep signing private. No purchases.
- **O04 — Source/PR ancestry:** retain the independent Linux base and list
  inherited commits if main has not integrated it; do not merge P68 to hide
  ancestry or pretend this is a main-only small diff.

The report records exact source/toolchain/device/artifact identity, command
results, trace comparison, content/action mappings, screenshots, layout and
lifecycle observations, actual test counts, failures/deferred gates and
next steps. Do not include serials, secrets, personal app logs or private
phone screenshots. Never treat unexecuted acceptance as passing.

## Planning verification report

Documentation/link/format checks cover nine documentation files, 13 new or
changed relative links, all 37 source hashes/line counts, device JSON, and
the AN01–AN18/BR-AN01–AN10 registries. Git ancestry verifies the base is
reviewed Linux native.3 and does not inherit the P68 planning commit.
`git diff --check` is the final whitespace check. Read-only ADB properties
confirmed the primary device; its serial/private data were not recorded.
No Cargo/Gradle build, SDK install, APK push/install, trace/gameplay test,
phone setting change, Windows/auth implementation or production deploy ran.
