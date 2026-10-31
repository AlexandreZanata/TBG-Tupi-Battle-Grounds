# P69 — Execution prompt for an independent Android prototype

Copy this when actual implementation, Android tool setup and testing on the
owner's connected POCO are intended. This planning delivery performed only
read-only device inspection and documentation work.

---

Implement P69 — Independent native Android landscape prototype. Read the
current AGENTS.md/minimum-validation policy, docs/phases/P69-android-native-prototype.md,
its acceptance/runbook, source manifest, device reference, GAME-DESIGN §22
and D-98. Follow that scope and write an execution report with real evidence.

P69 is a separate workstream from P68 Windows and new authentication. Start
from codex/p69-android-landscape-plan, a documentation descendant of reviewed
Linux native.3 source 014b0e0c046b38c551ee24534ee00709cde0e493. Use a new isolated
branch codex/p69-android-native-prototype; preserve dirty files, other
worktrees, builds and personal device data. Do not import/edit ongoing P68
code or wait for Windows/token implementation. Coordinate narrow shared-file
ownership before touching Cargo/bootstrap/UI files used by active fronts.

The Android game is played with the phone HORIZONTAL, in LANDSCAPE, including
both horizontal directions. Keep the existing mobile WEB experience PORTRAIT
and preserve the desktop interface. Create a proper touch layout, not a
scaled screenshot or rotated browser page. Keep the same medieval assets,
champions, animations, information and actions as Linux.

Reuse the exact tbg-core rules/content, canonical IDs, fixed ticks, bots,
PlayerIntentions, rejection/ack sequencing and authoritative server. No
mobile-specific stats, damage, economy, limits, timeouts, automatic actions,
bot assistance or outcome/rating changes. Preserve the difference between
Practice preview and online CANON. Build the Android candidate and separate
Linux reference from the same reviewed execution SHA; never alter the
installed Linux game or regenerate goldens to hide divergence.

Use Rust/Bevy 0.20 for rendering and UI, one shared client library/app builder,
ARM64 cdylib and a small Gradle/GameActivity host with Kotlin only for Android
services. Resolve exact-version engine examples/AndroidApp glue first. Pin
SDK/NDK/JDK/Gradle/AGP/GameActivity/cargo-ndk, split desktop dependencies away
from Android, package real APK assets, and verify every .so/import/alignment.
No KMP/Compose framework, WebView, new engine or duplicate game code.

The owner has a POCO X6 Pro connected through ADB with debugging authorized.
Planning inspection found model 2311DRK48G/duchamp, Android 16 API 36,
arm64-v8a, physical panel 1220x2712, density 480 dpi and 4096-byte pages.
Recheck/select that phone explicitly without publishing its serial. Landscape
reference is 2712x1220 before insets; measure real app/Bevy coordinate mapping.
Use safe cutout/gesture/IME bounds and effective 48dp touch targets.

Implement one-action taps, scroll/cancel/long-press separation, pointer
ownership, synthetic mouse suppression, mobile details/Shop/navigation,
software keyboard/paste/masking/Done/Back, app-private nonsecret preferences,
safe public links and audio focus. Preserve English/pt-BR and existing
Practice/training/War Council/Victory behavior. Handle lifecycle/surface
recreation and network changes without pausing online combat or extending
the normal 30-second reconnect window.

Authentication is out of scope beyond reusing the selected Linux protocol.
Manual AccountKey login may live in process memory only; no secret in Android
preferences/files/backups/logs/URLs. Keep deliberate backup UX. Do not create
Access/Refresh Tokens, HTTP auth routes, vault architecture or browser-cookie
changes; never bypass a newer production server or copy unfinished P68 work.
Use a compatible isolated test endpoint, synthetic accounts/guests and WSS
publicly. A debug-only scoped ADB loopback tunnel is allowed for local tests.

This execution request specifically includes the Android native build,
download/setup of required development tools subject to normal permissions/
license authority, building a signed internal APK, installing/upgrading only
the owned prototype package on the selected phone, a separate Linux build,
the named Linux/ARM64 trace probe, focused Android-versus-Linux test match
and one ten-minute device observation. No paid service/license purchase,
public store/release, production deploy, real ranked farming, root or device
settings/data wipe. Signing conflicts are errors; do not uninstall/downgrade
an existing app to force an install. Never clear global logcat or ADB mappings.

Respect front-mode minimum checks and consolidate affected tests once. All
local compiling/packaging uses the shared nonblocking host lock,
CARGO_BUILD_JOBS=2 and at most two Gradle workers. Busy lock is deferred,
with no polling/cache deletion/alternate target bypass. No full workspace
suite, balance gate, repeated gallery or broad soak/benchmark. Update test
selectors after the library refactor and verify nonzero counts. Record
AN01-AN18, actual protocol/ruleset/content, APK SHA/cert, device/software
versions, exact commands/results, screenshots and failures/deferred checks.
One 4KB POCO does not prove 16KB, older Android or store readiness.

When finished, this owner handoff explicitly authorizes publishing the
dedicated implementation branch and opening a GitHub PR targeting main,
overriding the repository's default no-push rule only for that delivery.
Never force-push/rewrite history, merge automatically or include P68 work.
If main lacks the Linux base, explain inherited commits and the larger diff.
Deliver the PR URL, verified APK link, evidence/report and exact remaining
device/visual gates. Never claim completion from compilation or a boot screen.
