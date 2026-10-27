# Roadmap — TBG 2.0 to a public multiplayer MVP

> Each task can be handed to one agent on its own branch or worktree. Specs
> live in [GAME-DESIGN.md](GAME-DESIGN.md) and
> [ARCHITECTURE.md](ARCHITECTURE.md); tasks point to them instead of
> repeating them. When a task is done, tick its box and append the commit
> hash. If an agent fails a task twice, it writes a three-line `> blocked:`
> note under the task (what was tried, the exact error, a suspected cause)
> and escalates one tier.

**Tiers:** **C** cheap/fast model · **M** mid model · **S** strongest model ·
**H** human (judging, devices, purchases, legal).

**Next task:** 1.1 (see the end of this file for the ready-to-send brief).

## Dependency map

```
P0 Simplify (done)
 ├─► P1 Browser + network spike ──► P3 Visual slice ──┐
 └─► P2 Core simulation ──────────────────────────────┴─► P4 Local playable match
                                                            ├─► P5 Online 1v1 ───────┐
                                                            └─► P6 Content & feel ───┴─► P7 Public MVP
```

| Can run in parallel | Files they own (no overlap) |
|---------------------|-----------------------------|
| P1 ∥ P2 | P1: `crates/tbg-client/*`, `crates/tbg-server/*`, CI · P2: `crates/tbg-core/*` |
| P3 ∥ P2 | P3: `crates/tbg-client/src/{art,arena.rs,camera.rs}` · P2: `crates/tbg-core/*` |
| P5 ∥ P6 | P5: `tbg-server/*`, `tbg-core/src/protocol.rs`, `tbg-client/src/{net.rs,link.rs,ui/menu.rs}` · P6: `tbg-client/src/{art,vfx.rs,audio.rs,ui/hud.rs,ui/evolve.rs}`, `tbg-core/src/content.rs` |
| 6.1 per faction | one agent per `art/roster/<faction>.rs` |

Rules common to every phase: the global definition of done in
[AGENTS.md](../AGENTS.md); no task adds a crate, a service, or a dependency
that it does not name.

---

## P0 — Repository simplification ✅

- [x] Backup tag `pre-simplify-20261027`; archive untouched.
- [x] Removed placeholder `content/`, `tools/`, `tests/`, `deploy/` (1203eae).
- [x] Workspace collapsed to three crates; rusqlite dropped until P7 (2b0ff00).
- [x] Agent harness submodule, symlinks, `.cursor/rules/`, old doc tree
  replaced by `AGENTS.md` + four docs.

---

## P1 — Browser + network spike

**Objective.** Prove early that Bevy WebGL2 builds, loads, renders, and
takes touch input on real desktop and mobile browsers, and that the
browser reaches the Rust server over WebSocket.
**Deliverable.** A `trunk serve` page with a placeholder arena (lane strip,
two Stronghold blocks); tapping spawns capsules that walk the lane; an
overlay shows FPS and server RTT. Release sizes recorded.
**Depends on.** P0. **Tier mix.** S for the build pipeline, then C/M.

- [ ] **1.1 (S) Web build pipeline.** Enable `bevy` features `["3d", "ui"]`
  with `default-features = false`; confirm WebGL2 is active
  (`cargo tree -p tbg-client -e features | grep webgl`). Add
  `crates/tbg-client/index.html` (full-viewport canvas, `touch-action: none`,
  no page scroll, loading text) and `Trunk.toml`. Add `[profile.release]`
  (`codegen-units = 1`, `lto = true`) and `[profile.dev.package."*"] opt-level = 2`
  to the root `Cargo.toml`. Replace the placeholder `main.rs` with a windowed
  Bevy app (canvas bound to the page, fit to parent).
  *Files:* `crates/tbg-client/{Cargo.toml,index.html,Trunk.toml,src/main.rs}`, `Cargo.toml`.
  *Done when* `trunk build --release` succeeds and the page shows the
  clear color in Chrome with no console errors.
- [ ] **1.2 (M) Camera + placeholder arena.** `camera.rs` implements the
  portrait/landscape framing in ARCHITECTURE §6 and recomputes on resize;
  `arena.rs` spawns ground, a 16-tile lane strip, two Stronghold cuboids,
  a directional light with shadows, and ambient light.
  *Done when* both orientations show the whole lane with margin (devtools
  device mode, 360×640 and 1920×1080).
- [ ] **1.3 (C) Tap to spawn.** A tap/click spawns a capsule at the
  player's Stronghold; it walks to the far end and despawns.
  *Done when* it works with mouse and with real touch.
- [ ] **1.4 (C) Diagnostics overlay.** FPS and frame time in a `bevy_ui`
  text node; `?stress=N` spawns N walking capsules.
- [ ] **1.5 (C) Server ping route.** Enable the axum `ws` feature; serve
  `GET /health` → `ok` and `GET /ws` echoing binary frames (max 1 KiB);
  listen on `TBG_ADDR` (default `0.0.0.0:8080`).
  *Files:* `crates/tbg-server/{Cargo.toml,src/main.rs}`.
- [ ] **1.6 (M) Client ping.** `net.rs` with `ewebsock`; server URL is
  `ws://<page host>:8080/ws` unless `?server=` is given; ping every second
  and show RTT in the overlay; the native build works too.
- [ ] **1.7 (C) CI.** `.github/workflows/ci.yml` running ARCHITECTURE §13
  on push and pull request with cargo caching. Fix any clippy findings.
- [ ] **1.8 (H + C) Device matrix.** Serve the release build on the LAN;
  record in ARCHITECTURE §10: brotli WASM size for `opt-level = "s"` and
  `3`, time to first frame, FPS at `?stress=30` on Chrome, Firefox, and
  Safari desktop, Android Chrome, and iOS Safari. Keep the faster
  opt-level that fits the size budget.

**Tests.** No new automated tests; CI gates only.
**Acceptance.** Renders and takes input on all five targets; ≥ 30 fps
with 30 capsules on both phones; brotli WASM ≤ 6 MB (or a recorded
reduction plan); RTT visible on a phone; all §13 commands pass.
**Risks.** iOS Safari WebGL2 limits → test the iPhone first, reduce MSAA
and shadows. Bundle size → replace feature collections with an explicit
feature list. `ws://` blocked from an `https` page → plain HTTP on the LAN
until P7. Kill criteria in D-01.
**Not yet.** Real art, rules, menus, matchmaking, audio.
**Done when** acceptance holds and measurements are committed.

---

## P2 — Core simulation (`tbg-core`)

**Objective.** The full MVP rule set (GAME-DESIGN §3–5) as a
deterministic, headless, tested library.
**Deliverable.** `cargo run --release -p tbg-core --example balance -- 1000`
prints faction win rates, median duration, Stronghold-kill/overtime/draw
rates, purchase share per unit, and mean step time.
**Depends on.** P0 (parallel with P1 and P3). Follow ARCHITECTURE §4 exactly.

- [ ] **2.1 (S) State skeleton.** `types.rs`, `rng.rs` (PCG32), `state.rs`
  (`MatchState`, `PlayerState`, `UnitState`, `Stronghold`, `Phase`,
  events), `sim.rs` (`MatchState::new(seed)`, `step()`, `hash()`); add the
  `serde` derive dependency. Move the existing vocabulary into `types.rs`.
  *Done when* an empty match steps countdown → regulation → overtime →
  ended and ends in a draw at tick (3 + 180 + 60) × 20 = 4 860.
- [ ] **2.2 (C) Content tables.** `content.rs` holds every number in
  GAME-DESIGN §5 converted to ticks/milli-units: 8 `UnitDef`s, 4 evolution
  forms each, synergy table, match constants.
  *Done when* no other file contains a balance number.
- [ ] **2.3 (M) Energy and clock.** Milli-energy regeneration, cap,
  overtime ×2, phase transitions.
- [ ] **2.4 (M) Shop.** Three offers per seat from a per-seat RNG stream;
  shared rotation every 200 ticks; immediate refill on purchase.
- [ ] **2.5 (M) Intentions.** `intent.rs`: `apply(seat, seq, intent) ->
  Result<(), Rejection>`. `Purchase` checks phase, offer index, `Energy`,
  `SLOT_CAP`, then spawns 0.5 tiles in front of the buyer's Stronghold.
  `Evolve` checks ownership, alive, next tier, branch, `Energy`.
  `Rejection` = `NotRunning | BadOffer | NotEnoughEnergy | SlotCapReached
  | NotYourUnit | UnitGone | MaxTier | WrongBranch`. Track `last_seq`.
- [ ] **2.6 (S) Combat.** `combat.rs`: targeting (nearest enemy ahead,
  Stronghold included, ties by id), movement with slow/root, cooldowns
  with a 0.3 s windup, damage, splash, status replace/refresh rule,
  Support healing, Stronghold attacks, deaths, events.
- [ ] **2.7 (M) Synergy.** `synergy.rs`: per-seat faction counts (a
  two-slot unit counts once), tier bonuses applied to effective stats
  with HP-ratio preservation (floor rounding), Plants regeneration,
  `SynergyChanged` events.
- [ ] **2.8 (M) Outcome.** Stronghold destroyed (both in the same tick =
  draw), regulation comparison, overtime, draw, `forfeit(seat)`.
- [ ] **2.9 (M) View and protocol.** `view.rs` (`view_for(seat)` with
  redaction and per-seat event drain); `protocol.rs` with the enums in
  ARCHITECTURE §5 and size-checked `encode`/`decode` (add `postcard`,
  `alloc` feature).
- [ ] **2.10 (C) Bot.** `bot.rs`: `Bot::new(seed)`, `decide(&MatchView) ->
  Option<PlayerIntention>`. Buys when affordable, keeps a tank alive,
  evolves when `Energy` ≥ 8 or the slot cap is full. Deterministic.
- [ ] **2.11 (C) Tests** in `crates/tbg-core/tests/` (10–15 total):
  determinism (same seed + script → same `hash()`), `Energy` regeneration
  and overspend rejection, shop rotation equal for both seats and refill,
  every `Rejection` path, evolution branch gating, synergy at 2 and 4 and
  loss on death, Stronghold kill, overtime, draw, forfeit, protocol
  round-trip and garbage rejection.
- [ ] **2.12 (C) Balance harness.** `examples/balance.rs` (bot vs bot,
  random seeds) printing the GAME-DESIGN §6 metrics and step time.

**Acceptance.** `cargo test -p tbg-core` green; 1 000 harness matches all
end; mean step ≤ 50 µs with 30 units (release); `tbg-core` passes the
wasm32 check; no `f32`/`f64` in `state.rs`; files within size caps.
**Risks.** Ambiguous rule → take the simplest reading and record it in
GAME-DESIGN as Proposed. Stalemates → the harness asserts every match
ends. Rounding drift → integer math with floor rounding everywhere.
**Not yet.** `Reposition`, bench, reroll, transport, persistence, tuning
beyond "matches end".
**Done when** acceptance holds and the harness output is pasted into the
commit message.

---

## P3 — Visual slice (figurines, arena, animation)

**Objective.** Prove that code-built figurines on a code-built arena look
professional in the real game camera, on phones.
**Deliverable.** `?gallery` page: Shieldbearer in all 5 forms playing all
10 clips, a 30-figure stress row, on the final arena and lighting.
Approved desktop and phone-portrait screenshots.
**Depends on.** 1.1, 1.2 (parallel with P2). Spec: ARCHITECTURE §7.

- [ ] **3.1 (M) Palette and mesh builders.** `art/palette.rs`;
  `art/mesh.rs` with rounded box, lathe, and extruded polygon builders,
  flat normals, vertex colors (darker toward the ground as fake ambient
  occlusion), and a mesh cache resource.
- [ ] **3.2 (S) Figure system.** `art/figure.rs`: `Joint` (12), `BodyPlan`
  per size class, `FigureSpec` data, `spawn_figure(..) -> Entity`, per-joint
  mesh merging cached per (form, joint), one shared material, a
  `PlayClip` component. This API is what P4 and P6 build on — document it
  in the module header.
- [ ] **3.3 (M) Animation library.** `art/anim.rs`: keyframe format with
  easing, the 10 clips, 0.1 s crossfade, a hit-frame marker for attacks.
- [ ] **3.4 (M) Shieldbearer.** `art/roster/humans.rs`: base + 4 evolved
  forms (shield, sword, helmet; tier accents).
- [ ] **3.5 (M) Final arena.** Island, lane with tile markers, Strongholds
  with 3 damage states, edge props, water, fog, sky; static meshes merged.
- [ ] **3.6 (C) Readability layer.** Team ring, billboard HP bars, hit
  flash, spawn pop, death puff.
- [ ] **3.7 (C) Gallery mode.** `?gallery` scene; log triangle counts per
  figure and arena at spawn.
- [ ] **3.8 (H) Visual review.** Silhouette readable at phone size;
  faction obvious by color; props show role; attack windup readable;
  consistent scale; no z-fighting or shadow acne; ≥ 30 fps with 30
  figures on the phones.

**Acceptance.** Human approval in 3.8; triangle and draw-call budgets in
ARCHITECTURE §10.
**Risks.** "Cheap toy" look → spend effort on lighting, palette,
proportions, bevels, and fake AO before adding geometry. Draw calls →
per-joint merging. Phone shadow cost → shadow toggle by device.
Rejected twice → D-07 reopens.
**Not yet.** Other 7 units' final art, projectiles, VFX library, UI, audio.
**Done when** approved screenshots are committed under `docs/media/`
(phone + desktop, ≤ 300 KB each).

---

## P4 — Local playable match (the full loop, vs bot)

**Objective.** A complete match against the bot in the browser using the
real rules.
**Deliverable.** Menu → Practice vs Bot → full match with HUD → Result →
Play again, on desktop and phone.
**Depends on.** P2; 3.2 and 3.3.

- [ ] **4.1 (M) States and screens.** `Menu/InMatch/Result` states;
  `ui/menu.rs` (title, Practice button, online buttons shown disabled);
  `ui/result.rs` (victory/defeat/draw, reason, Play again).
- [ ] **4.2 (S) Match link.** `link.rs` (`MatchLink::{Local, Net}`, Net
  stubbed) and `local.rs`: `MatchState` stepped in `FixedUpdate` at 50 ms,
  bot on seat 1, `ServerMsg::View` every 2 ticks via `view_for`.
- [ ] **4.3 (M) View sync.** `sync.rs`: view buffer, interpolation 120 ms
  behind, figure spawn/despawn, seat mirroring, unit form → `FigureSpec`.
- [ ] **4.4 (M) HUD.** `ui/hud.rs`: fractional `Energy` bar; 3 shop cards
  (name, faction color, archetype glyph, cost, disabled states); rotation
  countdown; clock with overtime banner; both Stronghold HP bars; slot use
  `n/8`; synergy badges.
- [ ] **4.5 (M) Intentions UI.** Card tap → `Purchase` with pending state;
  unit tap (enlarged pick volume) → `ui/evolve.rs` panel listing branch
  names, effects, costs → `Evolve`; rejection toast with a readable reason.
- [ ] **4.6 (M) Events to motion.** Attack/Hit/Death/Spawn/Evolve events
  drive clips, flashes, puffs; ranged attacks show a simple arcing
  projectile.
- [ ] **4.7 (C) Placeholder looks for 7 units.** Base body + faction
  palette + one distinct prop each, in the faction roster files.
- [ ] **4.8 (H) First fun check.** Three matches on desktop, one on a
  phone; write the top five issues under P6 as tasks.

**Acceptance.** Matches finish with the correct result; Play again works;
no console errors or panics; ≥ 30 fps on the phones; a first-time player
finishes a match without instructions.
**Risks.** Small screens → lay out for 360×640 portrait first. Tapping
moving units → enlarged pick volumes.
**Not yet.** Networking, audio, final art for 7 units.
**Done when** acceptance holds on desktop and both phones.

---

## P5 — Online 1v1 (authoritative multiplayer)

**Objective.** Two real players on separate devices complete
server-authoritative matches, including reconnects.
**Deliverable.** Menu: Quick Match, Create Room (code + Copy link), Join
by link or letter grid; full online match.
**Depends on.** P4. Spec: ARCHITECTURE §5 and §11.

- [ ] **5.1 (S) Protocol final.** Messages per ARCHITECTURE §5;
  `PROTOCOL_VERSION = 1`; maximum decoded size; round-trip tests.
- [ ] **5.2 (S) Connections.** `ws.rs`: 1 KiB frame limit, 5 s handshake,
  version check, 128-bit session tokens (`getrandom`), resume, token
  bucket, per-IP cap (trust `X-Forwarded-For` only from localhost), ping
  and idle timeout, bounded outbound queue (64; a slow client is
  disconnected rather than stalling a match).
- [ ] **5.3 (M) Lobby.** `lobby.rs`: one task; FIFO queue; rooms with
  codes and expiry; cancel; pairing spawns a match task.
- [ ] **5.4 (S) Match task.** `match_task.rs`: owns `MatchState`; 50 ms
  interval with `MissedTickBehavior::Burst` capped at 5 catch-up ticks;
  intents tagged with the session's seat; views every 2 ticks;
  disconnect keeps the seat for 30 s, then `forfeit`; `MatchEnd`;
  sessions return to the lobby; a panic ends only that match, logged.
- [ ] **5.5 (S) Client network link.** `net.rs`: connect, `Hello`/resume
  (token in `localStorage`; memory on native), reconnect every 1 s for
  30 s behind a "Reconnecting…" overlay, "New version — reload" screen on
  mismatch.
- [ ] **5.6 (M) Online menus.** Quick Match (cancel, wait timer, "practice
  while waiting"), Create Room (code + Copy link), auto-join from
  `?room=`, on-screen 5-letter grid.
- [ ] **5.7 (M) Server tests.** `crates/tbg-server/tests/online.rs` with
  `tokio-tungstenite` (dev-dependency): two socket bots finish a match;
  evolving the opponent's unit returns `NotYourUnit`; malformed frame
  closes the socket; version mismatch rejected; reconnect resumes;
  forfeit after the window (window shortened through the server config).
- [ ] **5.8 (C) Bandwidth log.** Bytes in/out per connection at match end.
- [ ] **5.9 (H) Real devices.** Desktop + phone on the LAN: full match;
  phone Wi-Fi off 10 s → resumes; off 40 s → both see the forfeit;
  Play again queues again.

**Acceptance.** 5.9 passes; tests green; ≤ 6 KB/s down per player; the
server stays healthy while one client floods garbage at maximum rate.
**Risks.** Tick drift under load → capped burst. Channel deadlocks →
bounded queues, never block the match task on a socket.
**Not yet.** Public deploy, TLS, persistence, accounts, rematch, regions.
**Done when** acceptance holds.

---

## P6 — Content, feel, and balance

**Objective.** The MVP is fun and readable: 8 distinct units with
evolutions, visible synergies, sound, and balance within targets.
**Deliverable.** A playtest-ready build and a balance report.
**Depends on.** P4. Runs in parallel with P5 (file split in the map above).

- [ ] **6.1 (C ×3) Roster art.** `art/roster/{undead,machines,plants}.rs`
  plus Archer in `humans.rs`: 5 forms per unit following the P3 checklist;
  one agent per file; gallery screenshots per faction for review.
- [ ] **6.2 (M) VFX.** Per-unit projectiles, splash ring, slow and root
  visuals, heal sparkles, evolve burst, synergy pulse, Stronghold hit and
  collapse; pooled entities. *File:* `vfx.rs`.
- [ ] **6.3 (C) Audio.** Enable Bevy `audio`; about 12 CC0 effects and one
  CC0 loop, OGG mono, ≤ 600 KB total, each recorded in
  `THIRD_PARTY_NOTICES.md`; unlock on first tap (iOS); mute toggle.
- [ ] **6.4 (M) UX polish.** Card icons; long-press tooltips with stats and
  counters; synergy panel; three first-match hints; result stats.
- [ ] **6.5 (M) Balance pass.** Harness runs across faction mixes; edit
  `content.rs` until the GAME-DESIGN §6 harness targets hold; update
  GAME-DESIGN §5 in the same commit as each change.
- [ ] **6.6 (H + M) Playtest round.** ≥ 4 people, ≥ 10 matches, short form
  (readable? fun? why did you lose? play again?); fix the top three
  issues; confirm or change the D-14 values.

**Acceptance.** §6 harness targets met; playtest averages ≥ 4/5 for
readability and ≥ 50% play-again; ARCHITECTURE §10 budgets still met on
the phones with full VFX and audio.
**Risks.** Inconsistent art across agents → one palette and the P3
checklist. Balance churn → change one table at a time, harness
before/after.
**Not yet.** Units 9–12, cosmetics, new modes.
**Done when** acceptance holds and D-14 is confirmed or updated.

---

## P7 — Public MVP launch

**Objective.** Anyone with the URL can play a fair online match on desktop
or phone.
**Deliverable.** A public HTTPS URL on one VPS, match records, measured
capacity.
**Depends on.** P5, P6. Spec: ARCHITECTURE §8–12.

- [ ] **7.1 (C) Match records.** `records.rs`: `rusqlite` (bundled), WAL,
  one `matches` table, insert at `MatchEnd` via `spawn_blocking`, path from
  `TBG_DB`.
- [ ] **7.2 (M) Production server.** `tracing` logs; graceful shutdown
  (SIGTERM stops new matches, waits up to 5 min); connection cap from env;
  `/health` reports match and socket counts.
- [ ] **7.3 (M) Deploy kit.** `deploy/{Caddyfile,tbg-server.service,deploy.sh,backup.sh,README.md}`:
  release builds, brotli/gzip precompression, rsync, restart, nightly
  backup with 7 copies plus a weekly off-site copy.
- [ ] **7.4 (M) Load test.** `crates/tbg-server/examples/loadtest.rs`: N
  bot clients over WebSocket via quick match; reports completed matches,
  p95 view interval, bytes/s. Run 100/250/500 matches against the VPS from
  another machine while recording server CPU and RSS; update ARCHITECTURE §10.
- [ ] **7.5 (S) Security review.** Limits, seat ownership, tokens, Caddy
  headers, firewall, `cargo audit`; fix anything critical.
- [ ] **7.6 (H) Owner decisions.** Add the root `LICENSE` (MIT intended),
  buy the domain and VPS (region from measured RTT). Then (C) list
  dependency licenses in `THIRD_PARTY_NOTICES.md`.
- [ ] **7.7 (H) Public smoke.** Two players on different networks (one on
  mobile data) finish a match; reconnect works; the match appears in
  SQLite; a backup is restored once successfully.

**Acceptance.** 7.7 passes; load-test capacity recorded (≥ 500 matches
target, or the real number); `cargo audit` has no unaddressed critical
findings.
**Risks.** Regional latency → measure before buying. Proxy idle timeouts
→ 2 s pings. Abuse → caps and rate limits from P5.
**Not yet.** Accounts, cosmetics, store, ranking, multiple servers.
**Done when** the URL is shared with the first external players.

---

## After the MVP (not planned until P7 ships)

Accounts, `Inventory`, cosmetics store (BR-C01–C03) · ranking · units
9–12 · rematch and spectating · `Reposition`/bench design · WebGPU build ·
delta views · additional regions (ARCHITECTURE §12).

---

## Ready-to-send brief for task 1.1

> You are working in the TBG 2.0 repository. Read `AGENTS.md`, then
> `docs/ROADMAP.md` task 1.1 and `docs/ARCHITECTURE.md` §2, §6, §13. Goal:
> the Bevy 0.20 client builds for the browser with Trunk and shows a
> windowed WebGL2 canvas that fills the page. Change only
> `crates/tbg-client/{Cargo.toml,index.html,Trunk.toml,src/main.rs}` and the
> root `Cargo.toml` profiles. Enable bevy features `["3d", "ui"]` with
> `default-features = false`; verify `webgl2` appears in
> `cargo tree -p tbg-client -e features`. Install Trunk with
> `cargo install --locked trunk` if missing. Verify with `cargo fmt --all
> -- --check`, `cargo clippy --workspace -- -D warnings`, `cargo test
> --workspace`, `cargo check -p tbg-client --target wasm32-unknown-unknown`,
> and `trunk build --release` (from `crates/tbg-client/`); then open the
> served page in a browser and confirm a clear-colored canvas with no
> console errors. Report the raw and brotli-compressed `.wasm` sizes. Commit
> locally as one commit; do not push. Tick task 1.1 in the roadmap with
> the commit hash.
