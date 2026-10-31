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
- `Archer`: Base, A1, A2, B1, B2; shared Victory fallback; legacy Archer; shared clip with native-Archer layer when selected.
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
