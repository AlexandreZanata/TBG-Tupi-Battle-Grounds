# P68 — Execution prompt for another agent

Copy the prompt below when implementation, package generation and Wine
installation/testing are intended. This planning branch has not executed
those actions. Resolve real device access, signing and any paid/license
decision with the owner while completing all independent work first.

---

Implement **P68 — Windows release, persistent authentication and Wine acceptance** completely,
using the existing native Bevy client. Read AGENTS.md, the current owner
validation policy and docs/phases/P68-windows-native-release.md, then its
acceptance plan and hashed platform-gate inventory. Follow the phase's
read-first scope and report format; do not inspect or edit archive/.
Also read docs/planning/P68-PERSISTENT-AUTH.md, GAME-DESIGN §21 and D-97.
The owner explicitly added persistent authentication for Windows, Linux
and web to this phase; A01–A14 are required alongside W01–W18.

The required source base is codex/consolidated-linux-release, inspected at
55510c5363e3f1e555fa5133208a5faf20c5825b. Use the P68 planning commit on
codex/p68-windows-native-release-plan as the documentation-bearing descendant
of that base. Create an isolated implementation branch named
codex/p68-windows-native-release. Check ancestry and remote state first. If
the source branch advanced, record the delta and keep one explicitly chosen
reviewed base; do not quietly restart from main, P67 or an old native front.
Preserve all dirty files, other worktrees and running builds/services.

Deliver a genuine x86_64-pc-windows-msvc Windows executable, portable ZIP,
safe per-user installer, complete assets, Windows instructions, PE icon and
version metadata, useful startup diagnostics, file manifests/checksums and
reproducible-input CI configuration. Reuse the current engine, platform
module and packager. Do not introduce an Electron/Tauri wrapper, new combat
rules or an updater service. P68-A needs one coordinated auth wire revision;
update every consumer and package/server identity together, without unrelated
protocol changes or assuming that protocol 29 remains compatible.

Implement the auth module before final packages: opaque random 256-bit
Access Tokens (15 minutes) and rotating Refresh Tokens (30 days absolute
from initial login, never reset by rotation). Persist hashes/history in the
existing SQLite writer, keep Access only in memory, use Linux Secret Service
and Windows Credential Manager Local persistence for native refresh, and
HttpOnly/Secure/SameSite=Strict host cookies for web. Follow current explicit
keyring providers in the plan, not an outdated sample or plaintext fallback.
Use memory-only/session fallback visibly when secure persistence fails.

Implement the specified HTTP route isolation, exact Origin/custom-header
CSRF checks, Web Locks/native process coordination, bounded idempotent
rotation/replay handling, revocation barriers and safe legacy migration.
Remove the old stored AccountKey only after new persistence/confirmation and
backup acknowledgment. Keep valid existing keys/manual recovery and all
account/ranked data. No new automatic key/token storage in preferences,
localStorage/sessionStorage, URLs, logs or caches. A scoped bound reconnect
handle is distinct from account credentials; no secret goes into HTML or
cross-tab notifications.

Preserve one active account controller with separate persistent device
families. Refresh/passive startup never steals another match; explicit login/
Continue here alone authorizes takeover. Reauthenticate an existing socket
without pausing combat. Natural Access expiry must not forfeit an admitted
match; use the specified match-only lease and unchanged reconnect window.
Security revocation invalidates the lease. Test concurrency, lost replies,
restart, expiry, logout/offline pending state, legacy failures, wrong-family
resume and real platform vault/cookie behavior using synthetic accounts and
injected clocks. Do not introduce Redis/JWT/OAuth infrastructure or P67 email/
commerce features. Server/core changes are limited to the named auth, store,
lobby/connection and bounded wire responsibilities; simulation stays intact.

The owner explicitly requires EXACTLY THE SAME FRONTEND APPEARANCE on Linux
and Windows. Match every game screen's layout, sizes, spacing, fonts/text
wrapping, colors, icons/images, models, lighting/shadows, VFX, camera and
animations at equivalent viewport, effective UI scale, language, graphics
settings and scene/time. Similar styling or shared code alone is not enough.
Make platform-specific adjustments where necessary, including DPI/fonts,
assets, renderer configuration or shaders, while keeping shared design and
preserving the approved Linux appearance/quality. Do not downgrade Linux to
match Windows or accept visible differences as normal OS/driver behavior.
Compare packaged builds with matched captures and overlays/diffs. Every
unresolved visible frontend difference blocks W07/W08 and P68 visual
acceptance; document failures honestly instead of waiving this requirement.

Start with the audit blockers: Linux-only guards currently control native
HUD/loading/ranking, many actions, shell pages and art generation. Inspect
both branches of every condition, imports/plugins/resources and build.rs.
Share desktop presentation where appropriate; preserve actual Unix services
and browser routing. Do not do a blind Linux-to-not-WASM replacement.
The frozen base is native.2, protocol 29 and 46 kinds: reconcile actual
source identities rather than hardcoding them or weakening contracts.

Build primarily on a supported pinned Windows CI image using MSVC,
Cargo.lock and two Cargo jobs. Decide CRT/DLL deployment explicitly and
test a clean Windows user. Cross-compilation is optional and cannot replace
actual Windows package validation. Fix packager host-execution assumptions,
stale staging and release-mode source-asset fallback. Do not overwrite any
existing native.2 tag/asset. Use a fresh reviewed candidate identity, and
build the Linux comparison from the same final source SHA. Preserve Linux
release jobs and fix the existing-release false-success path. Public
promotion stays gated; an unsigned internal candidate must say so plainly.

This execution request specifically includes Windows release packaging,
one separate Linux comparison package, installing Wine from reviewed
official distribution packages on this Linux host, and running the focused
P68 acceptance journeys. First recheck OS/package sources and simulate the
package transaction; install only required packages, with no broad OS
upgrade, mixed Ubuntu releases, arbitrary PPA or unreviewed SDK/license
acceptance. Request only necessary system authorization through the tool's
normal mechanism. Do not buy certificates, infrastructure or licenses.

Use an owned isolated Wine prefix and synthetic test data, never ~/.wine or
the installed Linux game's real data. Wine is not a security sandbox.
Download and hash-check the exact Windows candidate, test its actual EXE
from outside the checkout, then install/test/uninstall its installer.
Translate any TBG_DATA_DIR override to a Windows path. Preserve other Wine
processes/prefixes; do not run Wine as root or use global wineserver kills.
Record Wine/GPU/driver/backend, screenshots and each failure honestly.

Compare the grouped screen matrix with native Linux: lobby, account,
Arcanas, Strongholds, Arenas, ranking, Troops, training, settings, battle
HUD/War Council, unit details, result/Victory and next-match cleanup. Cover
pt-BR/English, DPI/fullscreen/focus, audio, clipboard, preferences, links and
TLS. Validate authoritative multiplayer and reconnect against an existing
compatible test endpoint without deploying/restarting production services.
Do not modify gameplay or disable graphics/security features to make Wine
pass. Obtain an actual Windows GPU session for Windows-specific acceptance;
CI and Wine are distinct evidence. If unavailable, finish the candidate and
leave the relevant real-Windows gates explicitly deferred.

Respect minimal local work-front validation. Consolidate affected tests
once. Do not run the full workspace suite, balance gates, broad benchmarks,
repeated galleries or unrelated platform builds. Every Linux compilation/
packaging validation uses the shared nonblocking host lock and
CARGO_BUILD_JOBS=2; exit 75 is deferred, with no polling or bypass. Use the
documented nonblocking host-lock equivalent on separate Windows runners,
also capped at two jobs. Check browser compatibility once where guards
changed. Keep exact commands/results and distinguish failures from deferrals.

Write docs/phases/P68-execution.md and update the roadmap with actual
implementation status, decisions, artifacts/hashes, manifests, environment
versions, screenshots, focused checks and all remaining gates. Do not claim
release readiness from a successful build or Wine launch alone. Signing,
SmartScreen observations, public support and owner look approval are explicit
gates; no secret enters Git or untrusted PR jobs.

When finished, this handoff explicitly authorizes publishing the dedicated
implementation branch and opening a PR on GitHub targeting main. This
owner instruction overrides the repository's default no-push rule for that
branch/PR delivery only. Never force-push or rewrite history; do not merge
the PR, create/promote a public release or deploy production automatically.
If the consolidated base is not yet in main, explain the inherited commits
and broader PR diff instead of merging unrelated work to conceal ancestry.
Give the owner the PR URL, candidate artifact links, tested environments,
known issues, deferred acceptance and exact next steps.
