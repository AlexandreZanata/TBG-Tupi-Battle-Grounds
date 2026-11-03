# P69 — Android landscape prototype acceptance and runbook

This is a future execution contract, not runtime evidence. Read
[P69](../phases/P69-android-native-prototype.md), GAME-DESIGN §22 and D-98.
The workstream is independent of P68 Windows/authentication. All core rules
and online authority come from the selected installable Linux version.

## 1. Physical reference and orientation

The owner supplied a POCO X6 Pro connected through authorized USB debugging.
Read-only ADB confirmed Android 16/API 36, arm64-v8a, 1220×2712 px physical
panel, 480 dpi and 4096-byte pages. The [device JSON](P69-DEVICE-REFERENCE.json)
contains only nonsecret facts. Landscape reference is 2712×1220 px; usable
app bounds/insets and Bevy scale must still be observed in the actual APK.

**Native Android uses horizontal/landscape play throughout. Existing web
mobile stays portrait.** Configure sensorLandscape/both horizontal directions,
but verify actual behavior on the POCO/Android 16. A manifest declaration
alone is not evidence. Resize/recreate/insets/IME must not lose an online
seat or restart the simulation. Unexpected portrait/unsupported windows
need a safe rotate/viewport message, not a hidden browser layout change.

## 2. Required acceptance

| ID | Contract | Required evidence |
|---|---|---|
| AN01 | Independent source | P69 branch from reviewed Linux base; no P68 Windows/auth work or dirty files included; exact SHA/wire/ruleset/content |
| AN02 | Native Android package | arm64 APK and native ELF libraries, no WebView game wrapper, source/build/signing identity and final SHA-256 |
| AN03 | Real-device launch | Downloaded APK installed/launched on the identified POCO, actual arena/art/sound/touch; no checkout/runtime desktop dependency |
| AN04 | Landscape and safe bounds | Horizontal UI in both directions, real usable viewport and dp/Bevy conversion, cutout/gesture/IME bounds, safe unexpected resize handling; web portrait unchanged |
| AN05 | Same deterministic rules | Existing trace probe's five seed outputs match between Linux and Android ABI; same core/content/features, no rewritten stats or golden updates |
| AN06 | Equivalent action semantics | Tap/inspection/scroll/cancel/multitouch route existing UiAction/PlayerIntention once and preserve sequence/ack/rejection/seat; no mobile cooldown or auto-purchase |
| AN07 | Playable offline | Practice/bots and training behave like the same Linux mode; canonical Energy/Shop/Evolution/Arcanas/Augments/tower actions and result cleanup |
| AN08 | Complete content identity | 46 champions/51 presentation kinds and actual forms/abilities share resolvers; representative walk/attack/Victory/effect paths observed |
| AN09 | Frontend reachability | Screen matrix below in English/pt-BR, touch scrolling/details/navigation, no keyboard/hover-only critical flow |
| AN10 | IME and Android services | Real software keyboard, commit/paste/masking/Done/Back, nonsecret settings restart, safe links/clipboard, audio focus and volume |
| AN11 | Cross-play | Packaged Android versus same-source Linux against compatible test server, valid actions/rejections, authoritative outcome, result/rematch cleanup |
| AN12 | Reconnect/lifecycle | Network interruption, background/return, screen lock, surface rebuild and process death classification; existing 30-second reconnect rule, no pause/grace extension |
| AN13 | Account/security boundary | Existing manual credential protocol only; memory-only AccountKey, explicit backup action, no plaintext persistence/logs/URLs, no new token system/P68 dependency |
| AN14 | Artifact/update safety | Hash/signature/cert and asset inventory; same-key in-place upgrade preserves nonsecret preferences; no uninstall/downgrade/wipe to bypass a failure |
| AN15 | Scoped performance | One controlled ten-minute POCO observation: reference scene/counts/settings, frame times/FPS, memory/thermal/battery and failures; no blanket low-end/device-capacity claim |
| AN16 | 16 KB compatibility evidence | Every .so checked for ELF/package alignment with actual tool versions; 16 KB runtime remains separately pending if tested only on this 4 KB phone |
| AN17 | Shared-client regression | Linux desktop behavior and web portrait/input still work after library/feature refactor; actual test targets/counts and focused browser evidence |
| AN18 | Delivery and limits | Execution report, owner look/fun decision, APK link and PR, independent passed/failed/deferred items; no public release/deploy claim |

A single compiled APK or emulator scene satisfies neither AN03 nor AN07/
AN11. If required functionality cannot be validated, deliver a clearly
labeled partial milestone and exact next action. Unavailable 16 KB/older
phones are separate limits; do not label them tested from ELF checks alone.

## 3. Landscape screen and input coverage

Preserve the same art/content/information, with a mobile layout. Desktop
pixel equality is not the goal for Android: equivalent actions and game
state must be readable and reachable with thumbs. Linux/Windows's separate
desktop visual contract is not changed by this phase.

```text
Landscape game viewport, inside actual safe insets
  Compact names / Stronghold health / Energy / connection status
  Arena and BattleLine, with clear targeting and readable units
  Five Shop offers + primary actions, reachable by touch
  Details and settings open as dismissible panels when requested
```

This is a layout intention, not fixed pixel geometry. Measure it on the
POCO; never scale all desktop content down until it fits. Use at least
48×48 dp effective interactive targets and verify the real physical/logical
coordinate mapping. Keep five Shop offers accessible without hiding costs
or state. Reading details must not unintentionally buy, evolve or fortify.

| Surface | Exercise |
|---|---|
| Startup/loading | Wordmark/icon distinction, real asset progress/errors, no black/blank window, correct landscape |
| Lobby/navigation | Guest/account states, modes, drawer/back behavior, readable creator attribution and public links |
| Arcanas/Strongholds/Arenas | Current selections/details and their exact Linux rules, full touch/scroll/confirm/return paths |
| Troops | Full catalog/lore/abilities/form stats, model walk/attack/Victory and usable search/selection |
| Practice/training | Existing bot difficulties, chosen teams/dummies, reset/exit and no old corpses/effects left behind |
| Ranking/profile | Existing public data/loading/empty/error states; readable details and no new secret fields |
| News/patch notes | Existing native.3 feed/content and available release notes remain readable; no new CMS/admin backend is introduced |
| Account/room | Existing manual key/name/room forms, masked key/paste, keyboard/Done/Back, temporary login and deliberate backup |
| Battle | Shop costs/offers, reserve/Reroll only where the chosen Linux mode enables them, Energy and accepted/rejected intentions |
| Details/tower | Select champion/enemy/tower, abilities/statuses, Evolution/Fortification and close without accidental action |
| War Council | Current pause/selection/hidden-choice rules, all options/details/confirm reachable in landscape |
| Settings | English/pt-BR, sound/volume, current visual settings/Low motion, safe phone controls and persisted nonsecret preferences |
| Result | Winner's complete living/dead cast/current Victory, pages/actions/rematch, arena/camera cleanup before next match |

Required gestures: press/release, quick repeated deliberate taps, long press
inspect without purchase, drag/scroll without activation, second finger,
pointer cancel, UI outside tap and modal occlusion. Retain useful real-mouse
behavior while suppressing generated duplicate mouse events from touch.
Do not add mobile automation or delay intention sending beyond current rules.
Desktop window-size/F11/desktop installer actions do not become Android
controls. Keep relevant build/about information and mobile-safe settings;
do not run a Linux/Windows updater or download executable patches on Android.

## 4. Build/package and device safety runbook

1. Recheck current `adb devices` state and properties; select the owner's
   authorized phone explicitly. Do not log/publish its serial. Multiple or
   unauthorized devices require selection/authorization, not guessing.
2. Verify the toolchain tuple, SDK license state and exact engine sample.
   Local Bevy metadata points to source `b3bdd75...`; do not substitute an
   unrelated current-main sample. Pin cargo-ndk/Gradle/AGP/GameActivity/NDK.
3. Acquire the shared nonblocking host compilation lock and two-job cap.
   Build only ARM64 client/core examples plus the necessary Linux comparison;
   use existing caches. Gradle consumes the verified .so with no parallel
   task execution or hidden Cargo rebuild. Defer lock contention, do not poll.
4. Inspect APK members, ARM64 ELF dependencies, asset paths/manifests,
   app ID, permissions, signing certificate and versionCode. Internet is
   required; camera/microphone/location/broad storage/background services
   are not. Keep exported components minimal and validate any public intent.
5. Download/verify the same APK that is handed over. Inspect the prototype
   app ID already on the phone, if any. Install only that owned package with
   an in-place same-key upgrade where possible. Signing/version mismatch is
   a reportable error: no uninstall, downgrade or phone-data wipe as a fix.
6. Launch and exercise the grouped journeys. Capture only the game viewport
   and filtered app logs; synthetic accounts only. No global logcat clearing,
   unrelated app dumps, user rotation/battery setting changes or root.
7. For a local test server, a scoped debug-only `adb reverse` mapping is
   permitted during execution. Restrict cleartext loopback to that debug
   configuration in the Rust transport as well as manifest/network policy;
   release/public endpoints require WSS. Remove only the owned mapping.
8. Preserve the owner's installed desktop game and personal phone data.
   Stop the owned prototype/test server normally; remove only disposable
   task-owned probe files if no longer needed. App uninstall requires an
   explicit cleanup request, not a validation habit.

Use Android's actual package asset API and app-private preferences. A
successful launch beside the source checkout cannot conceal an incomplete
APK; no source asset fallback or downloaded executable patcher is permitted.
An APK is the prototype deliverable; a Play AAB/publisher setup is future
work. Never claim store acceptance from internal APK installation.

## 5. Focused command queue for later execution

These are examples of the named consolidation scope, not commands run by
this planning task. Resolve the selected tool versions/SDK floor first.
Wrap all compiling/packaging commands in the shared Linux lock with two
Cargo jobs; run tests serially. Gradle has at most two workers.

```text
cargo fmt -p tbg-client -- --check
cargo ndk -t arm64-v8a -P 31 check --locked -p tbg-client --lib
cargo ndk -t arm64-v8a -P 31 build --locked --release -p tbg-client --lib
cargo build --locked --release -p tbg-core --example trace_canon
cargo ndk -t arm64-v8a -P 31 build --locked --release -p tbg-core --example trace_canon
./gradlew :app:assembleDebug --no-parallel --max-workers=2
```

Run Gradle in `platform/android/` after syncing the verified library and
build metadata. The trace probe's existing Linux output must match the
Android ARM64 executable's output from an owned `/data/local/tmp/` location.
It has no Bevy/UI/network/secret requirements and does not validate the final
APK by itself. Do not change its existing seeds or expected hashes.

Write focused tests in the affected mobile layout/gesture/platform modules;
record their actual names/targets in the report. After a bin-to-lib move,
use `cargo test --locked -p tbg-client --lib ui::mobile:: -- --test-threads=1`
for the new selectors and include the affected existing net/training/UI
tests once. Check nonzero counts. Check Linux and WASM compatibility once
for the shared bootstrap/feature diff, with browser portrait observation;
defer cold additional targets in front mode. No workspace suite or balance
gate is authorized. Android emulator success is supplementary, not POCO proof.

## 6. Device observation and lifecycle

One ten-minute consolidated observation uses the final APK, reference
30-unit scenario and a representative real match; record actual maximum
counts without limiting gameplay. Log settings, cold/warm load, frame-time
distribution/FPS, memory and available thermal/battery information. Minimum
reference goal is 30 fps; 60 fps is a measured optimization goal, not a
claim supported by the POCO model name. Investigate thermal degradation,
crashes, repeated texture loading and input stalls before adding features.

Test Home/app switch, lock/unlock, short network loss and a return beyond
the existing reconnect window. Online battle time is server-owned and does
not pause when the app is suspended. Do not promise a persistent background
socket or change forfeit timing. Classify OS process death separately from
a reconnect within the current session. Restore surface/audio/input without
duplicate cameras or stale gestures. No full device stress/soak campaign.

## Official references checked on 2026-10-31

- [Bevy mobile examples](https://github.com/bevyengine/bevy/tree/main/examples/mobile) and the locally installed 0.20 README/VCS metadata: obtain matching source before implementing version-sensitive APIs; current main is not an automatic compatibility guarantee.
- [android-activity upstream](https://github.com/rust-mobile/android-activity): native Rust activity/lifecycle glue, one compatible runtime lineage.
- [GameActivity setup](https://developer.android.com/games/agdk/game-activity/get-started): thin Android host for a native game.
- [Android orientation/resizing](https://developer.android.com/develop/ui/compose/layouts/adaptive/app-orientation-aspect-ratio-resizability): OS/form-factor restrictions and game exceptions vary; verify the actual targeted behavior.
- [Cutouts](https://developer.android.com/develop/ui/views/layout/display-cutout), [touch targets](https://developer.android.com/guide/topics/ui/accessibility/apps#large-controls) and [IME visibility](https://developer.android.com/develop/ui/views/touch-and-input/keyboard-input/visibility): safe landscape interaction and real keyboard behavior.
- [Activity lifecycle](https://developer.android.com/guide/components/activities/activity-lifecycle?hl=en): suspension/recreation/process death need explicit handling.
- [16 KB pages](https://developer.android.com/guide/practices/page-sizes): inspect all native/package alignment and separately test 16 KB runtime; the inspected POCO uses 4 KB pages. Store policy dates must be rechecked when a public launch is planned.
