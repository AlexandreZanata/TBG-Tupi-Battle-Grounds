# P63 execution report

## Step 0 — frozen integration base and decisions

- Fetched main: `9008f117c4289122ddff978c5d25720d2bd79687`, protocol 17.
- Isolated worktree `.local/p63-victory-celebration`, branch
  `codex/p63-victory-celebration`. Imported planning-only commits
  `0249d01` and `5565913` without replacing later files. Local owner policy
  copies remain unstaged. No other implementation front is incorporated.
- P62 PR #24 is draft: its core training slice exists separately, but its
  Dance/model viewer has not landed on main. P59 PR #22 is draft at protocol
  27; staged kinds are not active on this base. Native Linux PR #26 is
  stacked on unmerged PR #21 (protocol 20). Native visuals are not silently
  assumed present. P60/P61 PR #25 remains separate (protocol 17).
- Actual public Shop pool: 25 kinds; eligible animated summon: Beetling;
  Crate is inert and excluded. Five forms each yield 130 eligible current
  kind/form combinations. Cargo uses those same models, with summon class.
- O01 approved by owner: all purchased champions and animated summons,
  alive/dead, duplicates, final Evolution. Losers/watchers keep normal Result.
- O02 approved by owner: all figures through 64; deterministic pages of 64
  thereafter with exact ranges and total; no truncation.
- O03 owner overrides the planned exclusive pilot: use a shared standard now,
  inspect/reuse existing Victory and leave per-character dances extensible.
  No new signature pilot is required or claimed. Visual acceptance remains
  pending; future Dance preview must call the shared resolver.
- Spawn inventory: `MatchState::spawn` (purchases, lab placement), direct
  `spawn_beetling`, `spawn_cargo`, and excluded `cargo_tick` Crates. Evolution
  is one successful mutation in `intent.rs::evolve`; death/expiry removes
  living units but must never decrement the lifetime aggregate. Practice and
  debug army fill use actual Purchase and therefore the same collector.
- Result inventory: online match task emits MatchEnd then exits; LocalMatch
  emits the same message; spectators have their own ordinary end route.
  Existing session resume can recover a running match/rematch, but cannot
  replay a finished cast. Add a compact terminal payload to the session for
  the existing reconnect window, clear on new match/explicit dismissal and
  expiry. Do not retain all dead units or add a database/TTL.
- Compatibility: version 28 is the next unused observed number after the
  separately published v18–20 and v27 contracts. P63 starts from the actual
  v17 schema and adds result identity/summary; it does not pretend to include
  unmerged v20/v27 fields. All consumers and guarded fixtures change together.
- Runtime-only counts stay outside serde/state hashes, combat, events and
  RNG. Wire vectors are bounded at decoding; count expansion is lazy and
  page-limited. Invalid metadata cannot change Outcome or rating.
- FigureLibrary already owns meshes/materials, rig-aware Animator and
  event-free preview sampling. Default forward is +Z in these kits; derive
  every figure yaw from the actual seat-aware result camera, not a fixed
  global angle. Reuse the existing ArenaCamera and restore its projection
  and transform on hide/exit. UI actions stay immediately available.

## Current motion coverage (five forms per row)

This is a motion/dispatch inventory, not owner approval or a gallery result.
Existing kit-specific movement is preserved under the shared standard.
There are no newly authored exclusive dances. Future signatures are a
content backlog, registered once in the shared resolver.

- `Shieldbearer`: Base, A1, A2, B1, B2; Raise; `art/kits/shieldbearer/mod.rs`.
- `Archer`: Base, A1, A2, B1, B2; shared Victory fallback; Native Archer v2 model/layer; shared Victory fallback without a kit-specific override.
- `BoneBrute`: Base, A1, A2, B1, B2; shared Victory fallback; `art/kits/bone_brute/mod.rs`.
- `Hexer`: Base, A1, A2, B1, B2; Dance; `art/kits/hexer/mod.rs`.
- `Sawbot`: Base, A1, A2, B1, B2; Dance; `art/kits/sawbot/mod.rs`.
- `SiegeWalker`: Base, A1, A2, B1, B2; shared Victory fallback; `art/kits/siege_walker/mod.rs`.
- `Bloomkeeper`: Base, A1, A2, B1, B2; Dance; `art/kits/bloomkeeper/mod.rs`.
- `Rootcaller`: Base, A1, A2, B1, B2; shared Victory fallback; `art/kits/rootcaller/mod.rs`.
- `Nester`: Base, A1, A2, B1, B2; Both; `art/kits/nester/mod.rs`.
- `Rampart`: Base, A1, A2, B1, B2; Raise; `art/kits/rampart/mod.rs`.
- `Longaxe`: Base, A1, A2, B1, B2; kit-specific Victory; `art/kits/longaxe/mod.rs`.
- `Farwatch`: Base, A1, A2, B1, B2; kit-specific Victory; `art/kits/farwatch/mod.rs`.
- `Bombard`: Base, A1, A2, B1, B2; kit-specific Victory; `art/kits/bombard/mod.rs`.
- `Rotcap`: Base, A1, A2, B1, B2; Dance; `art/kits/rotcap/mod.rs`.
- `OldFinn`: Base, A1, A2, B1, B2; kit-specific Victory; `art/kits/old_finn/mod.rs`.
- `AntidoteBrewer`: Base, A1, A2, B1, B2; Raise; `art/kits/antidote_brewer/mod.rs`.
- `Shatterback`: Base, A1, A2, B1, B2; Both; `art/kits/shatterback/mod.rs`.
- `BloodFurnace`: Base, A1, A2, B1, B2; Raise; `art/kits/blood_furnace/mod.rs`.
- `Bloodweaver`: Base, A1, A2, B1, B2; Both; `art/kits/bloodweaver/mod.rs`.
- `Oilpress`: Base, A1, A2, B1, B2; Both; `art/kits/oilpress/mod.rs`.
- `Drumguard`: Base, A1, A2, B1, B2; kit-specific Victory; `art/kits/drumguard/mod.rs`.
- `Leechroot`: Base, A1, A2, B1, B2; Both; `art/kits/leechroot/mod.rs`.
- `CargoCrab`: Base, A1, A2, B1, B2; kit-specific Victory; `art/kits/cargo_crab/mod.rs`.
- `Puppetmaster`: Base, A1, A2, B1, B2; Dance; `art/kits/puppetmaster/mod.rs`.
- `Smuggler`: Base, A1, A2, B1, B2; Dance; `art/kits/smuggler/mod.rs`.
- `Beetling`: Base, A1, A2, B1, B2; Dance; `art/kits/beetling/mod.rs`.

## Validation and pending work

Level: work front until Step 5 consolidation. Step 0 inspected source,
planning links, current refs/PR metadata and `git diff --check`; no gameplay
check is claimed. Step 1–4 implementation and A01–A08 evidence follow below.
All compiling checks use the host lock, existing cache and two Cargo jobs.
No full suite, benchmark, broad gallery, install, deploy or main merge.

## Step 1 — exact lifetime roster

Implemented `celebration.rs` with fixed-capacity bucket domains, checked u64
counts, last-form moves, canonical ordering, lazy index lookup and bounded
cosmetic decoding. Attached one runtime-only ledger to MatchState, skipped
by serde/hash. New matches start exact; deserialized snapshots deliberately
cannot claim missing history. Purchases, lab placement, hatching and cargo
record actual initial forms. Death/expiry never decrements participation;
inert Crates are excluded. Counter inconsistency invalidates cosmetics only.

`cargo fmt -p tbg-core` and `git diff --check` passed. Under the shared lock,
existing cache and two jobs, `cargo check --locked -p tbg-core --lib` passed
in 1.35 s. New `celebration_cast` regressions cover early deaths, duplicate
champions, Evolution, expiry/cargo, hash/event/outcome parity against an
untracked deserialized state, invalid buckets/overflow and huge logical
counts. Their execution is queued once for Step 5, not claimed here.

## Step 2 — authoritative end and bounded reconnect replay (D-82)

Protocol 28 binds MatchStart/MatchEnd to a random public identity and sends
last_tick plus an opaque, size-bounded CastPayload. Corrupt, duplicate,
wrong-winner/form or oversized cosmetic content decodes to ordinary Result
without discarding a well-framed Outcome. The server sends the cast only to
the winner. All core/client/server/bot/test consumers are adjusted together;
existing guarded protocol fixtures change with the D-82 canon amendment.

Session-local CompletedResult retains only compact start/end data for the
existing reconnect window. It is cleared by explicit dismissal, next
pairing and expiry. The terminal-delivery race also handles a new connection
that attached as the match task finished. Ordinary healthy connections are
not sent a second Start. No DB, rating path or TTL change. Local Practice
uses the same summary and a distinct local identity. Client acceptance
rejects stale/duplicate end records before playing sound or entering Result;
rematch messages cannot reenter the same Result or interrupt a live match.

Formatting/diff checks passed. Focused server consumer checks exposed and
fixed exhaustive MatchEnd in botclient, a test's inherited ServerMsg import,
and exhaustive MatchStart in contract_challenge. The latter check stopped
with `error[E0027]: pattern does not mention field result_id`; corrected
consumers and new regressions are queued for Step 5 rather than claiming a
pass. Tests cover replay/expiry/dismissal, either-seat real WebSocket result
recovery, idempotent client acceptance and malformed cosmetic isolation.

## Step 3 — cosmetic Result scene and immediate controls

Added separate celebration layout/scene/UI modules. Only an eligible,
validated winning cast is expanded, with at most 64 actual figures and lazy
logical indexing. Reuses FigureLibrary meshes, glow and Archer bow, without
UnitVisual, UnitEntities, combat intents or metric hooks. Conservative rig
chain/mesh envelopes and deterministic staggered slots size the formation;
yaw derives from each slot's direction to the seat-aware camera. One camera
override reserves upper-screen space; compact localized Result controls,
page ranges and keyboard shortcuts remain active below it. Empty casts are
explicit; unusable cosmetics get an ordinary-result diagnostic.

Hide/exit despawns the page hierarchy and restores the camera/projection;
resizing reflows the page. Reduced motion freezes the sampled celebration,
removes secondary springs and keeps a static camera. Browser preference
uses explicit persisted choice, then prefers-reduced-motion; native supports
the explicit environment preference and the in-session M/control toggle.
No audio behavior or timing is added here. UI calls use main's actual
compiled Msg IDs, not the unmerged native Lang::key helper.

Focused client check passed in 2.49 s after correcting that i18n API
assumption. Subsequent localized Result copy is syntax/key-checked and its
final compilation is queued with Step 5. Added pure page/formation/facing
regressions for zero/1/3/64/65 and huge totals, both seats and desktop aspects.
Actual rendering, lifecycle, spacing and browser evidence remain pending;
no visual or frame-time acceptance is claimed from compilation.


## Step 4 — one motion resolver and future signatures

Added `art::celebration::celebration_for(kind,tier,branch)`: one current
Victory clip, real kit duration, loop metadata and optional per-kind/form
signature callback. Scene and ordinary Victory duration lookup use it; P62
has no integrated Dance preview to adapt yet and must call this API when
its viewer lands. No second animation library or new attack clip is created.
Animator keeps its existing native/kit pose layers, then applies the optional
signature and root/facing bounds for cosmetic playback only. Explicit
presentation clocks and preview sampling clear actions, aim, hit-stop and
events; reduced motion samples a stable pose rather than freezing a corpse.

The owner explicitly chose the shared standard now, so no exclusive pilot
or newly unique dance is claimed. Existing Raise/Both/Dance and kit-specific
Victory content remains; all future signatures are still a content backlog.
Regressions enumerate all 130 current eligible kind/form combinations and
sample 16 phases each, checking finite poses, event-free clip metadata and
root bounds; a synthetic callback proves the override seam is executed.
Formatting/diff checks passed. Final compilation and test execution remain
queued with Step 5; A04 visual approval remains distinct from these checks.

## Step 5 — targeted consolidation (implementation draft)

Level: consolidation requested by the owner, not a full-suite run. Tests ran
on the committed Steps 1–4 plus the acceptance fixes described below. The
retry/dismissal fix is `644ce18`; final rendering/cleanup fixes are `2a354fd`.
Main was fetched again before publication and remains `9008f11`. This report
is a subsequent documentation commit. Every compiling command used:

```bash
flock -n -E 75 "/tmp/tbg-validation-$(id -u).lock" \
  env CARGO_BUILD_JOBS=2 \
  CARGO_TARGET_DIR=/home/iiii/PESSOAL-PROJETOS-ALEXANDRE/TBG-Tupi-Battle-Grounds/target \
  <command>
```

Commands actually executed successfully (all tests add `-- --test-threads=1`):

```bash
cargo test --locked -p tbg-core --test celebration_cast
cargo test --locked -p tbg-core --test contract_determinism
cargo test --locked -p tbg-core --test contract_protocol
cargo test --locked -p tbg-core --test contract_protocol_server
cargo test --locked -p tbg-server --lib celebration
cargo test --locked -p tbg-server --test online celebration
cargo test --locked -p tbg-server --test online rematch
cargo test --locked -p tbg-server --test online version_mismatch_rejected
cargo test --locked -p tbg-client celebration
cargo test --locked -p tbg-client ui::result::tests::rematch_controls_follow_the_offer_without_query_conflicts
cargo test --locked -p tbg-client net::socket::tests::challenge_result_reconnects_without_consuming_the_invite_again
cargo test --locked -p tbg-client art::celebration::tests
cargo test --locked -p tbg-client celebration::layout::tests
cargo test --locked -p tbg-client net::socket::tests
cargo test --locked -p tbg-client celebration::scene::tests
cargo check --locked -p tbg-server --lib --tests --bins
cargo build --locked -p tbg-client --bin tbg-client
```

**41 distinct tests passed**, with no failed tests in the final relevant
runs; repeated filters above followed actual code changes or corrected
compilation failures. Core/server account for 27, client for 14. Frozen
determinism hashes remain unchanged. The real-WebSocket acceptance test
uses two connections, buys a participant, disconnects the winner, confirms
the other seat's surrender, then recovers identical result identity and cast
twice without a live view; it exercises each winning seat. Existing rematch
contracts pass. This is not two rendered browser clients finishing a match.

Acceptance fixes: measure actual mesh boxes through 32 celebration poses,
cache envelopes by bounded kind/form domain, stagger rows, use continuous
presentation clocks across looping clips, keep localized controls below the
formation and restore/clear the camera override on exit. A regression checks
hierarchy destruction and a later ordinary Result retaining its own camera.
An explicitly dismissed result is ignored during retry/login and across
browser tab reload via sessionStorage, without changing account credentials.
These fixes do not change the authoritative outcome, match duration or rating.

### Focused native observations

An opt-in debug-only `p63_fixture=1` creates participants using real core
Purchase/Evolution, kills half, confirms a core forfeit outcome, then enters
Result. It does not fake production input or run on ordinary startup. Native
captures used Xvfb/X11, the debug client, existing host graphics driver and
the same actual arena/models; no installed application was replaced.

Representative invocation (each capture uses its own path and log):

```bash
TBG_P63_FIXTURE=1 TBG_P63_COUNT=9 TBG_P63_SEAT=A \
TBG_P63_SIZE=1920x1080 TBG_P63_CAPTURE="$PWD/.local/p63-a9-framed.png" \
TBG_SERVER=ws://127.0.0.1:9/ws WINIT_UNIX_BACKEND=x11 \
timeout 50 xvfb-run -a -s '-screen 0 1920x1080x24 -nolisten tcp' \
  env -u WAYLAND_DISPLAY \
  /home/iiii/PESSOAL-PROJETOS-ALEXANDRE/TBG-Tupi-Battle-Grounds/target/debug/tbg-client
```

Observed captures, all with one camera and zero game-metric changes:

- Seat A, 1920×1080: nine figures (594 total ECS entities), frontal intact
  models, machine and humanoid poses; controls remain unobstructed.
- Seat B, 1366×768, pt-BR: nine figures, same orientation/spacing contract;
  longer translated control copy wraps without hiding Menu/retry.
- Empty cast, 1366×768: zero figures, explicit empty message and range 0–0.
- One figure, seat B, reduced motion, 1366×768: stable pose; 20 Result/Menu
  cycles return owned entities to zero and total ECS count to the first
  warmed-up Menu baseline. Final Result has 387 entities.
- 64 figures, seat A, 1920×1080: 64 cosmetic roots, range 1–64 of 64,
  2,022 total ECS entities; no observed overlap with result controls.
- 65 participants, seat B, 1366×768, page 1: only the final logical figure
  allocated, range 65–65 of 65, 385 total ECS entities. Other logical slots
  remain available through the previous-page control.

These are rendering fixtures, not ranked games or owner visual approval.
The first PNG attempt failed because Bevy's existing PNG encoder feature was
disabled; that zero-byte artifact is not evidence. Enabled `png` with a
Cargo.toml reason and resolved its already cached transitive dependencies
using `cargo build --offline -p tbg-client --bin tbg-client` (3m41s). Later
locked builds/captures succeeded. The initial framing was visibly too small;
the final captures above use sampled mesh envelopes. Test/capture outputs
stay under ignored `.local/`, never in git.

### Coverage limits and follow-up

- A01/A06: aggregate, death/Evolution/summon, bounded wire validation,
  lazy expansion, identity and hash/event isolation pass targeted tests.
  Unknown enum decoding is rejected inside the cosmetic payload; malformed
  entire frames still follow existing transport rejection rules.
- A02: winner A/B, loss/draw/spectator/unusable-cast gating pass tests;
  forfeit rendering and empty cast observed. A naturally completed Practice
  win and P62 controlled drills remain unobserved; P62 drills are unmerged.
- A03/A04: representative native formations observed at both desktop sizes;
  130 form motions are finite/eventless in sampled tests. Full visual review
  of every large rig/form and subjective owner approval remain
  pending. Exclusive pilot is deferred by the owner's explicit O03 override.
- A05: rematch server contracts, client result UI, duplicate/stale result
  acceptance and retry/login regression pass. A rendered online
  retry/rematch journey and delayed-rating/audio playback review remain pending.
- A07: pure tests cover 0/1/3/64/65 and huge u64 totals/page boundaries;
  native observations cover 0/1/9/64/65. No count-sized allocation is used.
- A08: 20 automated native enter/exit cycles and reduced pose observed;
  stable entity/camera assertions pass. Audio resource baseline, listening
  with mute, native keyboard/page cycling and automatic OS preference
  behavior require a focused interactive review. The browser controls were
  observed as described below; tests do not substitute for listening.
- Proposed p95 +10% budget, cold load and real-device frame comparison are
  **not measured**. A screenshot's FPS label is not a performance result.
  Benchmarks, release packaging, full workspace suite, extensive galleries,
  Windows/macOS/mobile and deployment were not run or authorized here.
- Do not deploy protocol 28 to one endpoint only. Reconcile wire changes
  when integrating unmerged P59/native work; their missing implementations
  are not implied by the chosen version number. Native PR #26 remains
  separately stacked on PR #21.

Browser build/observations are recorded separately below. Until the pending
mandatory visual/performance/interactive acceptance is reviewed, publish as
a Draft PR to main rather than declaring P63 production-ready.

### Focused WebGL2 observations

Ran `trunk build` in `crates/tbg-client` under the same lock/cache/two-job
wrapper. The first attempt was interrupted with exit 143 and no compiler
diagnostic. A cache-reusing recovery completed its Rust dev build in 54.34s;
the final fixture-control adjustment rebuilt in 3.50s. Both finished with
one existing WASM warning: unused `lang` at `ui/battle_hud.rs:126`, also
present in main `9008f11`. No unrelated HUD edit was made to hide it.
No release/WASM optimization readiness is claimed from this dev build.

Served only `127.0.0.1:8016` with Python's HTTP server and opened a separate
hidden in-app browser tab at:

```text
http://127.0.0.1:8016/?p63_fixture=1&p63_count=65&p63_seat=B&lang=pt-BR&server=ws://127.0.0.1:9/ws
```

After the debug readiness assertion (`data-p63-ready=true`), observed:

- WebGL2 arena and 64 cosmetic figures at 1366×768, pt-BR; range 1–64 of 65.
- Physical PageDown (`Next`) reaches the final figure, range 65–65 of 65.
- M switches to the stable reduced-motion pose and updates localized copy.
- H removes the cosmetic figure and restores the ordinary arena camera;
  H again restores the figure. PageUp returns to 1–64; M restores motion.
- Resizing the same live scene to 1920×1080 reflows camera/formation and
  leaves the full result controls visible below it. Saved full-viewport
  evidence locally as `.local/p63-web-65.jpg` (not a tracked binary).
- Escape immediately returns to the ordinary DOM lobby. No warning/error
  entries were captured by the browser console before or after the journey.

The opt-in debug driver initially reapplied its requested page every frame;
adjusted it to initialize once so it cannot override real UI navigation.
Reloaded the final build before the interactive observations above. This
fixture has no working server at port 9 and cannot validate online rendered
retry/rematch, rating arrival or sound playback. It is separate browser
evidence, not a claim inferred from native compilation. Closed only the
created tab, reset the temporary viewport and stopped only this local HTTP
process. Existing user windows, native installation and VPS were untouched.
