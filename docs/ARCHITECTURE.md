# Architecture — TBG 2.0

> The definitive technical design. Game rules live in
> [GAME-DESIGN.md](GAME-DESIGN.md); the reasons behind each major choice live
> in [DECISIONS.md](DECISIONS.md); build order lives in [ROADMAP.md](ROADMAP.md).
> Numbers marked *hypothesis* must be measured before anyone relies on them.

## 1. System at a glance

```
 Browser (desktop + mobile)                     One small VPS
 ┌──────────────────────────────┐   WSS    ┌──────────────────────────────┐
 │ tbg-client (Bevy → WASM,     │◄────────►│ Caddy: TLS, static files,    │
 │ WebGL2)                      │  binary  │        /ws reverse proxy     │
 │  render · animate · HUD      │ postcard │ tbg-server (Axum + Tokio)    │
 │  sends PlayerIntentions      │          │  lobby · match tasks running │
 │  practice mode runs tbg-core │          │  tbg-core at 20 Hz           │
 │  locally against the bot     │          │ SQLite file: match records   │
 └──────────────────────────────┘          └──────────────────────────────┘
```

One language (Rust), one shared rules crate, one server binary, one static
bundle. No Docker, Kubernetes, Redis, queues, or managed game services.

## 2. Stack

| Concern | Choice | Version / note |
|---------|--------|----------------|
| Toolchain | Rust stable via `rust-toolchain.toml` (installs `wasm32-unknown-unknown`) | env: 1.99.0; MSRV 1.97 (Bevy 0.20) |
| Client engine | Bevy, features `3d` + `ui` (+ `audio` in Phase 6), WebGL2 | 0.20.0, pinned for the whole MVP |
| WASM build | Trunk (`index.html` in `crates/tbg-client/`) + wasm-opt | 0.22.x (`cargo install --locked trunk`) |
| Client WebSocket | `ewebsock` (same API on WASM and native) | 0.8.x |
| Server | Axum (`ws` feature) + Tokio | 0.8.x / 1.x |
| Wire format | `serde` + `postcard` (binary, varint) | 1.x / 1.1.x |
| Persistence | `rusqlite` with bundled SQLite (Phase 7 only) | latest at Phase 7 |
| Edge | Caddy 2: automatic TLS, precompressed static files, WS proxy | distro package |
| Process | systemd unit, non-root user | — |

Versions verified on 2026-10-27 against the local cargo registry and
`cargo info`. Adding a dependency needs a one-line reason in the crate's
`Cargo.toml` comment.

## 3. Workspace and boundaries

```
crates/tbg-core    deterministic rules, content tables, bot, view, protocol
crates/tbg-client  Bevy browser client (depends on tbg-core)
crates/tbg-server  authoritative server (depends on tbg-core; never Bevy)
```

| Crate | May depend on | Must never contain |
|-------|---------------|--------------------|
| `tbg-core` | `serde`, `postcard` only | graphics, async runtime, sockets, filesystem, clocks, floats in simulation state, unseeded randomness |
| `tbg-client` | `tbg-core`, `bevy`, `ewebsock`, small web helpers | outcome decisions in online play |
| `tbg-server` | `tbg-core`, `axum`, `tokio`, `rusqlite` | Bevy, rendering, gameplay rules (they live in core) |

Planned modules (create each only when its roadmap task starts):

| Crate | Module | Responsibility |
|-------|--------|----------------|
| core | `types.rs` | ids, `Faction`, `Archetype`, `SlotCount`, fixed-point helpers |
| core | `content.rs` | roster, evolution trees, synergies, match constants (GAME-DESIGN §5) as `const` tables |
| core | `rng.rs` | PCG32, ~20 lines, no external crate |
| core | `state.rs` | `MatchState`, `PlayerState`, `UnitState`, `Stronghold`, `Phase` |
| core | `intent.rs` | `PlayerIntention`, validation, `Rejection` reasons |
| core | `shop.rs` · `combat.rs` · `synergy.rs` | rule subsystems |
| core | `sim.rs` | `MatchState::step()` — the fixed tick, in a fixed order |
| core | `view.rs` | `view_for(seat) -> MatchView`, redaction, event drain |
| core | `bot.rs` | seeded heuristic opponent (practice, harness, load test) |
| core | `protocol.rs` | `ClientMsg`, `ServerMsg`, `PROTOCOL_VERSION`, encode/decode |
| client | `main.rs` · `camera.rs` · `arena.rs` | app, states, framing, environment |
| client | `art/` | `palette.rs`, `mesh.rs`, `figure.rs`, `anim.rs`, `roster/<faction>.rs` |
| client | `link.rs` · `local.rs` · `net.rs` | `MatchLink` (local or network), view buffer |
| client | `sync.rs` · `vfx.rs` · `audio.rs` | views → entities, effects, sound |
| client | `ui/` | `menu.rs`, `hud.rs`, `evolve.rs`, `result.rs` |
| server | `main.rs` · `ws.rs` · `lobby.rs` · `match_task.rs` · `records.rs` | router/config, connections, pairing/rooms, match loop, SQLite |

Size caps (AGENTS.md) keep every file reviewable in one read. Content
tables are the one allowed exception.

## 4. Simulation model (`tbg-core`)

- **Fixed tick:** 20 Hz (50 ms). All timers are counted in ticks.
- **Integer math only:** positions in milli-tiles (`i32`, 1 tile = 1000),
  HP and damage as `i32`, percentages as basis points, `Energy` in
  milli-energy. No floats in state, so outcomes are reproducible on every
  platform and test assertions are exact.
- **Coordinates:** seat 0's `Stronghold` at x = 0, seat 1's at
  x = 16 000. The client mirrors x for seat 1 at render time.
- **Randomness:** one PCG32 stream per seat per purpose (shop), seeded from
  the match seed. The seed comes from the OS on the server and never leaves
  it (BR-S05). Units, iteration, and targeting use `Vec` order by unit id —
  never `HashMap` iteration.
- **Tick order:** 1) apply queued intentions in arrival order; 2) clock,
  phase, `Energy` regen; 3) shop rotation; 4) synergy recompute;
  5) statuses tick down; 6) targeting + movement; 7) attacks resolve
  (cooldown, windup, splash, statuses, heals); 8) deaths removed, events
  emitted; 9) outcome check.
- **Events:** each step appends `Spawn`, `Attack{from,to}`, `Hit{to,amount}`,
  `Heal`, `Status`, `Death`, `Evolve`, `SynergyChanged`, `StrongholdHit`;
  views drain them so the client animates exactly what happened.
- **Views:** `view_for(seat)` returns tick, phase, clock, both Stronghold
  HPs, all units (id, owner, kind, form, x, hp, max_hp, status), the
  seat's own `Energy`, offers, rotation timer, and `last_seq` (last applied
  intention). Opponent `Energy`, offers, and seeds are never included.
- **Determinism check:** `MatchState::hash()` (FNV over the state) lets
  tests assert that the same seed and intention script give the same
  result.

## 5. Networking

- **Model:** server-authoritative snapshots, not lockstep. The server runs
  the only real simulation; clients render views. This tolerates latency,
  hides private information, and has no desync class of bugs.
- **Transport:** one WebSocket per client at `/ws`, binary frames,
  postcard-encoded `ClientMsg` / `ServerMsg`.
- **Messages (draft):**
  `ClientMsg = Hello{version, resume} | QuickMatch | CreateRoom | JoinRoom{code} | Cancel | Intent{seq, intent} | Ping{t}`;
  `PlayerIntention = Purchase{offer} | Evolve{unit, branch}`;
  `ServerMsg = Welcome{token, name} | VersionMismatch{server} | Queued | RoomCreated{code} | MatchStart{seat, opponent} | View(MatchView) | Rejected{seq, reason} | MatchEnd{outcome, reason} | Pong{t}`.
  Intentions carry no seat: the server derives the seat from the connection.
- **Rates:** simulation 20 Hz; views every second tick (10 Hz) per seat.
  Clients render about 120 ms behind the newest view and interpolate x.
- **Prediction:** UI-level only — a tapped card shows pending and `Energy`
  dims optimistically until `last_seq` confirms or `Rejected` arrives.
- **Sessions:** the server issues a random 128-bit token in `Welcome`; the
  client keeps it in `localStorage`. No accounts; display names are
  generated (`Commander 4821`).
- **Matchmaking:** quick match is a FIFO queue; private rooms use a
  5-character code (unambiguous alphabet, 10-minute expiry), shared as a
  link `?room=CODE`. Bot practice runs locally in the browser.
- **Lifecycle:** paired → `MatchStart` → countdown → running → overtime →
  `MatchEnd` → both sessions return to the lobby. Disconnected seats stay
  in the running match for 30 s; reconnecting with the token resumes with
  a full view; otherwise the seat forfeits (BR-M03).
- **Concurrency:** one task per connection, one lobby task, one task per
  match; they talk through `tokio::sync::mpsc`. No locks on game state.
- **Limits:** handshake within 5 s; inbound frame ≤ 1 KiB; token bucket
  20 msgs/s (burst 40) per connection, then disconnect; ≤ 8 sockets per IP;
  ping every 2 s, idle timeout 15 s; global connection cap from config.
- **Bandwidth (hypothesis):** about 400 B per view → about 4 KB/s down and
  under 0.5 KB/s up per player; about 2 MB per 4-minute match for both.

## 6. Client

- **States:** `Menu → Queue → InMatch → Result → Menu`. The same `InMatch`
  code serves practice (`MatchLink::Local`: `tbg-core` stepped in Bevy
  `FixedUpdate` at 20 Hz against the bot) and online play
  (`MatchLink::Net`). Both deliver `ServerMsg` values; nothing else in the
  client knows which one is active.
- **Camera:** fixed perspective (~35° FOV, ~55° pitch). The lane runs along
  the screen's longer axis — vertical on portrait phones (own `Stronghold`
  at the bottom), horizontal on landscape (own `Stronghold` on the left).
  Framing recomputes on resize.
- **Input:** tap/click only. Shop cards are `bevy_ui` buttons; units are
  picked through an enlarged invisible pick volume so moving targets are
  tappable on phones.
- **UI:** `bevy_ui` for every screen (menu, HUD, evolve panel, result),
  sized in viewport units, thumb-reachable on portrait.
- **Build:** one WebGL2 release artifact; `opt-level="s"` vs `3` decided by
  measurement in Phase 1; `wasm-opt -Oz`; brotli/gzip precompressed at
  deploy. WebGPU is a later optional build, not an MVP target.

## 7. Art direction and asset production

**Style:** a tabletop diorama of chunky toy figurines. Big heads and hands,
oversized props that define each silhouette (shields, bows, saws, leaves),
flat-shaded low-poly surfaces, saturated faction palettes, soft warm sun,
gentle fog, readable from a phone held at arm's length.

**Production: code-built, no Blender, no skinning.**

- **Figurines:** every unit shares one 12-joint body plan (root, hips,
  torso, head, upper arm/forearm/hand ×2, leg ×2). Parts are low-poly
  meshes from `art/mesh.rs` (Bevy primitives plus rounded box, lathe, and
  extrude builders) parented to joints. A `FigureSpec` (data) lists
  proportions, palette, and parts per joint; evolutions are small spec
  diffs (added prop, +8% scale per tier, glowing accent at tier 2).
  Two-slot units use the same plan at about 1.5× scale.
- **Batching:** at spawn, the parts of each joint are merged into one mesh
  with vertex colors, cached per (unit form, joint) and shared by every
  instance; all figurines share one material. Hit flash swaps to a shared
  flash material for about 80 ms.
- **Animation:** `art/anim.rs` holds keyframed joint rotations/offsets with
  easing, shared by every figure: `idle`, `walk`, `attack_melee`,
  `attack_ranged`, `cast`, `hit`, `death`, `spawn`, `evolve`, `victory`.
  Clips crossfade over 0.1 s and are triggered by simulation events.
- **Arena:** generated in `arena.rs`: an elongated grass island with a dirt
  lane and tile markers, two stone `Stronghold`s with three damage states,
  low-poly trees/rocks/ferns at the edges, a water skirt, sky clear color,
  and distance fog. Static parts are merged into a few vertex-colored meshes.
- **Readability layer:** team ring under each unit (ally `#3FA9F5`, enemy
  `#F2545B`), billboard HP bars, hit flash, spawn pop, death puff.
- **Starting palette** (approve in Phase 3): Humans `#8B7355` `#C4A882`
  `#D9B44A` (legacy Iron Guard tones) · Undead `#D8D2C0` `#4B3B5C`
  `#7CF29A` · Machines `#4A5058` `#B5893B` `#FF8A3D` · Plants `#4E9F3D`
  `#6B4F2E` `#F28AB2` · grass `#7BB661`, lane `#C9A46A`, water `#3FA7B5`,
  rock `#8C8A86`, sky `#A9D8F0`, stone `#B9B2A3`.
- **Validation:** a gallery mode (`?gallery`) renders every unit form
  playing every clip on the real arena in the game camera. Agents capture
  desktop and phone-viewport screenshots; a human approves silhouette,
  proportion, color, motion readability, and consistency.
- **Audio and fonts (Phase 6):** CC0 sound packs and one OFL display font,
  each recorded in `THIRD_PARTY_NOTICES.md`.
- **Escape hatch:** if a part cannot look right from builders, an offline
  Blender script may export that *static* part as a small GLB. Rigs,
  skinning, and per-character animation stay out of scope.

## 8. Persistence

- **MVP:** guest sessions in memory; one SQLite file (WAL) with one
  `matches` table — start time, duration, outcome, reason, and per-seat
  purchases/evolutions as JSON. No personal data and no IP addresses. It
  feeds the fun metrics in GAME-DESIGN §6. Writes run off the tick loop
  (`spawn_blocking`).
- **Boundary:** only `records.rs` touches SQL, behind a few plain functions.
- **Move to PostgreSQL when** two or more server instances need shared
  writes (accounts, rankings, `Inventory`), or analytics queries start
  competing with match writes. Swap the module then — not before.

## 9. Deployment and operating cost

- Caddy terminates TLS, serves the precompressed static bundle with
  immutable cache headers for hashed files, and proxies `/ws` to
  `tbg-server` on localhost.
- `deploy/deploy.sh` (Phase 7) builds, rsyncs, and restarts the systemd
  unit. Firewall opens 22/80/443 only. Secrets live in an env file outside
  the repository.
- Backups: nightly `sqlite3 .backup`, 7 local copies, weekly off-site copy.
  Match records are non-critical; a lost day is acceptable.
- Monitoring: `/health` endpoint + a free external uptime checker +
  structured logs (`journalctl`). Nothing more until there is a need.
- Optional: put a free Cloudflare proxy in front for edge caching of the
  static bundle; WebSockets pass through.

| Item | Monthly estimate (verify current prices) |
|------|------------------------------------------|
| VPS, 2 vCPU / 2–4 GB RAM / ≥ 1 TB traffic | US$5–12 |
| Domain | about US$1 (annual fee spread monthly) |
| TLS, CDN proxy, uptime checker, CI (public repo) | US$0 |
| Off-site backup storage (MBs) | under US$1 |

**Latency:** pick the VPS region closest to the first players. An
auto-battler stays fair and readable up to about 200 ms RTT because no
input is twitch-timed; measure real RTT from the target audience before
buying.

## 10. Performance budgets (hypotheses until measured)

| Metric | Target | Measured in |
|--------|--------|-------------|
| WASM download (brotli) | ≤ 4 MB (hard ceiling 6 MB) | Phase 1 |
| Other assets on first load | ≤ 1 MB | Phase 6 |
| Time to menu | ≤ 5 s desktop broadband; ≤ 10 s mid-range phone on 4G | Phase 1, 7 |
| Frame rate | 60 fps desktop; ≥ 30 fps mid-range Android and iPhone 11-class with 30 units | Phase 1, 3, 6 |
| Draw calls in a busy match | ≤ 300 | Phase 3 |
| Triangles | ≤ 1 500 per one-slot unit, ≤ 3 000 per two-slot, ≤ 30 000 arena | Phase 3 |
| Browser memory | ≤ 300 MB | Phase 6 |
| Sim step, 30 units (release) | ≤ 50 µs | Phase 2 |
| Server memory | ≤ 100 KB per match + about 64 KB per socket | Phase 7 |
| Bandwidth per player | ≤ 6 KB/s down, ≤ 0.5 KB/s up | Phase 5 |
| Concurrent matches on 2 vCPU / 4 GB | ≥ 500 | Phase 7 load test |

**Capacity reasoning.** Simulation CPU is tiny (about 20 units, integer
math). The first limits are socket fan-out and bandwidth: 500 matches ≈
1 000 players × 4 KB/s ≈ 32 Mbit/s. Measure with the Phase 7 load test
(bot clients over real WebSockets) at 100/250/500 matches while watching
CPU, RSS, and bytes/s.

## 11. Security baseline

- Server-authoritative combat, `Energy`, shop, evolution, and outcomes.
- Seat comes from the connection, never from message content; tokens are
  random, never shown to the other player, and bound to one seat.
- Every message is decoded with size limits; malformed input closes the
  socket; intentions are validated against current state.
- Rate limits, per-IP socket caps, and timeouts as in §5.
- TLS everywhere public (Caddy); no secrets in git (`.env*` ignored).
- `cargo audit` before each public deploy.
- Not in the MVP: advanced anti-cheat, fraud detection, accounts/MFA.

## 12. Scaling path (build only when metrics demand it)

| Stage | Trigger | Change |
|-------|---------|--------|
| 1. One VPS | launch | everything above |
| 2. Bigger VPS | CPU or bandwidth above 60% at peak | resize; Tokio uses the extra cores |
| 3. Several game servers | one machine is not enough, or a second region is needed | matches are independent tasks, so add servers; the client fetches a `servers.json`; room codes carry a server prefix; quick match per server/region |
| 4. Shared persistence | accounts, rankings, or `Inventory` across servers | `records.rs` moves to PostgreSQL |
| 5. Central matchmaker | cross-server queues become necessary | a small service hands out match tickets; game servers stay the same |

What makes this possible today: no shared state between matches, opaque
session tokens, all SQL behind one module, and static files that are
already CDN-ready.

## 13. Verification commands

```bash
cargo fmt --all -- --check
cargo clippy --workspace -- -D warnings
cargo check --workspace
cargo test --workspace
cargo check -p tbg-client -p tbg-core --target wasm32-unknown-unknown
```

A native check never proves browser compatibility. Rendering claims need
the Trunk build running in a real browser (Phase 1 device matrix).
