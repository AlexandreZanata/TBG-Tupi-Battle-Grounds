# Consolidated Linux candidate — 2026-10-31

## Scope and authority

The owner requested all open work fronts, their native counterparts, fresh
3D Shop portraits, one PR to main, a downloadable Linux version, a TBG-only
VPS deployment and local installation. Normal push and release publication
are authorized; main remains under manual owner review. Other worktrees,
local policy edits and unrelated VPS projects are preserved.

## Integration inventory

Integrated tips: #18 (P57 spacing), #20 (roster foundation), #21 (champion
changes and preview Shop), #22 (P59 complete authored roster), #23 (five
arena refinements), #24 (P62 training kernel), #25 (P60/P61 ranked storage),
#26 (Linux UI), #27 (P64 localization), #28 (P63 celebration).

Protocol 29 combines the P59 effects/special fields and P63 result identity
and complete winner cast. Every consumer shares the same core. Ranked
journals/statistics accept historical four-Faction arrays and current seven-
Faction arrays; summary schema 2 retains schema 1 read compatibility.

The owner's explicit activation includes all 46 conventional champions.
The 51-kind presentation inventory also contains Beetling, Crate and the
three drill ranks; summons and objects are never sold by the Shop. Numeric
content remains Proposed. Activation is not a claim of competitive balance;
the P59 W10 balance acceptance and owner look/fun sign-off remain open.

## Native feature integration

The shared shell exposes Troops from both header and lobby. It offers actual
3D form/animation previews, catalog filters, authored lore, canonical stats,
exact teams with duplicates and positions, inert wooden targets and isolated
trials. Trials provide identity-checked pause/resume, one-tick step, reset,
seed repetition, paged per-instance and team meters including dead instances.
Training does not produce competitive outcomes, rewards or ranked records.

Practice offers Easy/Medium/Hard and a visibly assisted Impossible profile.
Medium delegates to the existing Bot. Other policies use public MatchView
only. Impossible receives one additional normal regeneration stream on seat
B in the client-owned local match; the online ruleset is unchanged.
Difficulty ordering is not represented as a measured win-rate guarantee.

Result and Dance preview share the kit-aware Victory resolver. Existing
Victory movements are reused, with a per-kind/form signature extension
point; no claim of exclusive dances for every champion. Complete cast
aggregation includes dead participants, repeats, last forms and animated
summons, with explicit pages of up to 64 figures. Server authority and
immediate result actions remain intact.

## Validation ledger

Level: targeted consolidation. Commands use the shared host lock,
CARGO_BUILD_JOBS=2 and one test thread. Evidence is being collected on the
consolidated candidate; historical front reports are not treated as proof
for this tree. Final executed commands, observed screens, package checks,
deploy identity and deferred acceptance are recorded below before delivery.

Known CI sources addressed: P59's adjacent if formatting; native Bevy system
signature lints; P63 envelope-cache type; complete-roster/summary dimensions;
and a persistence test that incorrectly depended on bot combat ending within
60 wall-clock seconds. That test now finishes through authoritative surrender.

Deferred unless explicitly requested: full workspace suite, long balance
gates/benchmarks, full animation gallery and extensive multiplayer matrix.
Native validation alone never establishes browser compatibility.

## Targeted evidence collected

- Native Clippy: `cargo clippy --offline -p tbg-client -p tbg-core -p tbg-server
  --bins -- -D warnings` passed on the integrated runtime.
- Core targeted binaries: celebration_cast (6), champion_siphon (10),
  contract_protocol (13), contract_protocol_server (2), contract_ranked_integrity
  (3), shop_reserve_reroll (9), training_isolation (1), training_session (9),
  contract_determinism (2), contract_roster (7), contract_ruleset_version (2),
  practice_bot (2): passed. Training includes all 46 kinds in all five forms;
  its complete-roster HP reconciliation also passed after integrating P59
  Drillnest production/fusion into the training pass.
- Server targeted binaries: ranked_integrity (7), ranked_recovery (4),
  ranked_stats (6), ranked_stats_recovery (2): passed. The specific
  `online records_insert_and_count` case passed over real local sockets (0.31s).
  Its first sandboxed attempt was denied socket creation; the authorized
  host execution passed. Unknown summary schema test now uses 99 because
  schema 2 is intentionally supported.
- Client filtered tests: `i18n` (16) and `celebration` (9): passed. A test-only
  unused P57 reference `Layout::clear` warning remains outside production.
- `cargo check --offline -p tbg-client --target wasm32-unknown-unknown`: passed.
- Server production binary: `cargo build --offline --release -p tbg-server
  --bin tbg-server`: passed (18.40s).
- Native laboratory journey observed running, paused, single-stepped, reset,
  rejecting an old-session action, resumed, with a wooden target and HP
  meters independent of competitive state. Initial desktop captures were
  constrained to 1280x636 by inherited Wayland. The exporter now removes
  WAYLAND_DISPLAY/WAYLAND_SOCKET; the final catalog capture is actual
  1440x900 on Xvfb, with zero missing images and settled loading masks.
- 50 actual-model portraits exported: all 400x400 WebP under 40 KiB (largest
  initial export 6,320 bytes), covering every conventional Shop kind plus
  animated summons. Longaxe uses a body-focused high-resolution crop so its
  tall weapon does not make the body unreadable. All five arena thumbnails
  were captured from the refined scenes and encoded at 768x382 (decoded dimensions).

Local logs/captures, databases, packages and build directories are ignored;
they are not part of the PR. Final browser, native package and deployment
observations are appended after the remaining release checks.

The Impossible credit is calculated from the pre-step Interest/arena rate and
applied after the canonical step. The Countdown-to-Running transition receives
its matching credit only when core regeneration begins; prior Countdown ticks
and Council selection produce none. A dedicated regression compares every
credit with actual canonical Energy gains through that transition.

> blocked (capture harness, initial A/B cycle attempt): both seats reported
> `celebration leaked entities across Menu cycles`, left 396 / right 398.
> The assertion counted the entire application, including asynchronous native
> loading/menu entities, so a decrease was incorrectly classified as a leak.
> The independent capture harness now records every cosmetic root and descendant
> before exit and asserts each exact entity identity is absent afterwards;
> camera uniqueness and zero gameplay-metric checks remain. Re-run evidence
> follows below. The original invariant was not a reliable native leak measure.

Browser observation found nested training panes lacked ScrollPosition, leaving
controls below the fold unreachable. Each pane now owns its scroll position,
preserved across catalog rebuilds independently of the page scroll.
The full-roster two-bot WebSocket match passed in 36.25s. The separate
reconnect journey hit the inherited 60s test-client deadline; its test harness
now allows 180s while preserving every outcome and reconnect assertion.

The local Trunk 0.22.1 offline resolver checks PATH before its download cache.
The required wasm-bindgen 0.2.129 already existed in the cache, but offline
builds did not discover it there. The release command now prepends the verified
cached wasm-bindgen and wasm-opt directories to PATH; no system tool is replaced.
The online build completed before this resolver diagnosis. Final artifacts use
the final source tree and the explicit local tool paths.

The affected core library target exposed a stale lab-test assumption that
CANON excludes Bonehorn. The test now explicitly uses the original ten-kind
reference pool and retains its exact off-pool damage/accounting assertions;
all 100 core library tests then passed. The native client target found four
integration issues: Pyre Warden recovery exceeded its 0.50s basic interval,
two kit tests applied player-Evolution constraints to fixed-rank summons,
and a literal >= symbol was missing from the font subset. Pyre recovery is
0.48s with event times unchanged; timing tests inspect actual kit durations;
every form retains the same geometry budget, conventional Evolutions retain
growth/distinctness guards, and a separate guard covers distinct drill ranks.
The unsupported symbol is rendered with ASCII >=. Final re-run follows.

Browser WebGL2 verification exercised nested-pane scrolling, Start, finite
wooden-target completion, Reset and an exact one-tick advance (0.05s, Paused).
The reset observation exposed a fading corpse from the previous session:
dead actors had already left the live-id map. Match/session boundary cleanup
now queries all actor/corpse roots plus transient Vfx/projectiles and resets
the impact FIFO; it is shared by ordinary match exit and laboratory reset/exit.
Training's private presentation counters also reset with the new session.

Final client consolidation: 147 tests passed, zero failed, one performance test
remained ignored. The final runtime change (8873422) passed scoped native
Clippy and native/server production builds. Its browser build completed and
WebGL2 repeated finite-target completion (8.70s, 1000 HP), Reset (0.00s,
zero meters, no previous corpse) and exactly one tick (0.05s, Paused).
The A/B P63 captures each passed 64 figures, two Menu cleanup cycles,
zero gameplay metrics and a single camera. The reconnect re-run passed in
19.03s; finished-result reconnect recovery passed for both seats.

The additional affected balance-library check found that Default cannot be
derived for the new 51-entry metric arrays. Explicit zero initialization
preserves all counters and covers every kind. After the fix,
`cargo clippy --offline -p tbg-balance --lib -- -D warnings` and
`cargo test --offline -p tbg-balance --test contract
c5_every_tunable_number_is_registered -- --exact --test-threads=1` passed
(the contract took 0.07s). No balance gate or simulation benchmark ran.

Package preflight generated both Linux formats. Inspection found linuxdeploy
patches the executable's ELF library paths, so the original portable checksum
must not be presented as the bundled executable's checksum. AppImage packaging
now deploys libraries first, retains the unbundled digest, calculates the actual
bundled digest, and invokes the official bundled appimagetool with two squashfs
processors. The final AppDir executable matched its embedded digest. ShellCheck
and shell syntax checks passed. Both launchers are named TBG.


## Published candidate and deployment

Release: [v0.1.0-native.2](https://github.com/AlexandreZanata/TBG-Tupi-Battle-Grounds/releases/tag/v0.1.0-native.2),
source 1815eb70c691e36b39b6995e6f62e8e135334e3e, protocol 29.
The tar.gz and AppImage plus their SHA-256 files were published, downloaded
back from GitHub, and both archive digests passed. The actual extracted
AppImage executable matched its embedded digest and source identity.
Native production build passed in 4m28s; the server build reused its valid
cache (0.21s). Packaging ran under the shared lock; Cargo and squashfs
parallelism were limited to two.

The final WASM build was produced at 8873422. The following commits change
only the separate balance library, packaging and reports: an explicit
`git diff --exit-code 8873422..1815eb7 -- crates/tbg-server crates/tbg-core
crates/tbg-client Cargo.toml Cargo.lock` passed. The unchanged browser/server
runtime evidence was reused rather than recompiling those targets by habit.
The final server binary digest also matched the binary deployed below.

The portable client ran from /tmp for 18 seconds with isolated preferences,
created window preferences, and emitted no panic or missing-asset errors.
Only its own process group was stopped. Its first harness incorrectly
expected an INFO window line filtered by the logger; the subsequent artifact,
private-window-state and runtime-log checks passed. AppImage --version worked
without FUSE via --appimage-extract-and-run.

TBG-only deployment used the VPS control repository's ignored task directory
`.local/tbg-temp/consolidated-linux-20261031`. No application source, secrets
or deployment dumps were committed there. Script syntax/ShellCheck and
`npm run verify` passed; the latter explicitly skipped unavailable yamllint.
Image `docker.io/library/tbg-server:linux2-1815eb70c691` replaced only the
`tbg-temp/tbg-server` Deployment. Its Recreate strategy restarted only TBG.
The old `pr21-8d2c48efd43b` image remains available.

Before migration, SQLite backup integrity was verified and the recovery
folder made private (0700, database 0600). Backup and previous web files remain
under `/opt/tbg-temp/backups/linux2-1815eb70c691/`; versioned web assets are under
`/opt/tbg-temp/releases/linux2-1815eb70c691/web`. Schema migrated 6 -> 8 with
quick_check=ok and the same 5 accounts, 40 matches and 9 ranked_matches.
The before/after image inventory for every unrelated Deployment was identical.
No image pruning, other namespace change, ingress reload or VPS reboot ran.
The task-specific ROLLBACK.md explains restoring TBG without discarding data.

Public HTTPS health and /api/version passed, reporting client native.2 and
protocol 29. The public 3D catalog rendered in WebGL2 at the deployed domain.
All five arena images and the Pyre Warden/Longaxe portrait sentinels matched
local hashes through HTTPS with the browser's same-origin Referer. A bare
urllib image request returned 403; adding the normal Referer through curl
worked without changing the site's image protection. The initial asset probe
omitted the assets/ prefix and reached the existing SPA fallback; corrected
paths came from the actual dist inventory.

Deployed server SHA-256:
`7941d2fd92276061a7e9309015eda5c72df5eb353eec8f327ddea76d612f6ca1`.
Deployed WASM SHA-256:
`7ac0fcd1b7108d6718b7ca1faad9f09e01b249b9506043525bb6fc68552e7209`.

The downloaded tar.gz was safely extracted, its binary digest verified, and
its included installer installed TBG into
`~/.local/opt/tupi-battle-grounds/0.1.0-native.2-1815eb70c691`.
`~/.local/bin/tbg --version` reported native.2, protocol 29 and the exact source
commit. The desktop entry is named TBG. Byte-for-byte preference preservation
and preservation of every earlier installation directory were verified.
Existing user windows and sessions were not terminated.

## Review status and deferred acceptance

[Consolidation PR #29](https://github.com/AlexandreZanata/TBG-Tupi-Battle-Grounds/pull/29)
contains all ten original PR tips; the final GitHub inventory still showed the
same heads and main 9008f117c4289122ddff978c5d25720d2bd79687. Original PRs,
worktrees and owner policy edits were preserved. Normal pushes and an additive
release tag were used; no force-push, history rewrite or merge ran.

Implementation/package/deploy/install delivery is complete. The PR remains a
Draft for explicit competitive/content acceptance: P59 W10 balance evidence,
owner look/fun acceptance and measured Practice difficulty ordering are not
claimed. Shop reserve/reroll remains the original labeled Practice preview;
no additional online economy rule was activated. Existing Victory motions
are standard fallbacks with future per-kind/form hooks, not exclusive dances.

Deferred commands/scenarios: full workspace check/clippy/test and unrelated
acceptance targets (explicit full-suite request required); D-33 long balance
gate and benchmarks (explicit heavy-run request required); extensive full
animation/audio/reduced-motion galleries and multiplayer matrices (broader
manual/targeted acceptance, not repeated automatically). Windows/macOS/mobile
builds are outside this Linux release. The native binary's maximum observed
GLIBC symbol is 2.39, reflected in the published requirements.
GitHub CI was not awaited; local targeted evidence is not a claim that every
unexecuted remote job will pass. No known executed targeted check remains
failing after the recorded fixes.

## CI follow-up (owner request, 2026-10-31)

Fetched completed job logs without waiting for any running workflow:
124630974233 (8873422) failed on Default for [u16; 51] and [[u16; 2]; 51],
already fixed by b181a12. Job 124626194278 (3aeff7d) reported the four art/font
failures fixed by 10cd8cf plus one shared-runner timing failure:
`visuals::layout::tests::three_hundred_units_within_budget` measured
`2.412043ms per frame`, above its unchanged 2ms limit. This test exercises the
legacy P57 reference implementation, not the active runtime crowd solver.

The timing probe remains intact as an explicitly ignored release benchmark;
no threshold changed. A deterministic 300-actor regression now checks exact
reproducibility, preserved identities, finite coordinates and valid file/back
offsets in ordinary CI. Focused layout validation passed: 5 passed, 2 ignored,
zero failed. The two ignored probes were not executed. Latest workflows were
still running when inspected, so no new remote pass is claimed.

The requested balance audit then exercised contract_lab directly and found
C2 still pinned to pre-46-kind traces. The actual canonical collector printed
all five current traces; final hashes agree with contract_determinism. This
follow-up records the already documented pool activation in GAME-DESIGN and
refreshes C2 in the same commit, with no new simulation change. The full
contract_lab target passed: C1, C2 and C10 (3 passed, 0 failed, 2.17s).
