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
| Client engine | Bevy with an explicit feature list (see `crates/tbg-client/Cargo.toml`; no gamepads, glTF, animation, post-processing, or LUTs; `bevy_audio` + `wav` in the WASM build only, P7), WebGL2, camera tonemapping `AcesFitted` | 0.20.0, pinned for the whole MVP |
| WASM build | Trunk (`index.html` in `crates/tbg-client/`) + wasm-opt | 0.22.x (`cargo install --locked trunk`) |
| Client WebSocket | `ewebsock` (same API on WASM and native) | 0.8.x |
| Server | Axum (`ws` feature) + Tokio | 0.8.x / 1.x |
| Wire format | `serde` + `postcard` (binary, varint) | 1.x / 1.1.x |
| Persistence | `rusqlite` with bundled SQLite (from P5) | 0.41.x |
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
crates/tbg-balance offline balance lab (native binary; depends on tbg-core
                   with its `lab` feature; never a dependency of the others)
```

| Crate | May depend on | Must never contain |
|-------|---------------|--------------------|
| `tbg-core` | `serde`, `postcard` only | graphics, async runtime, sockets, filesystem, clocks, floats in simulation state, unseeded randomness |
| `tbg-client` | `tbg-core`, `bevy`, `ewebsock`, small web helpers | outcome decisions in online play |
| `tbg-server` | `tbg-core`, `axum`, `tokio`, `rusqlite` | Bevy, rendering, gameplay rules (they live in core) |
| `tbg-balance` | `tbg-core` (feature `lab`) and `std` only | network, persistence, anything the online game loads |

**Rulesets (D-33, P16-C).** Every tunable number is a `Ruleset` value;
the simulation reads numbers only through `MatchState::rules()`. The server
and the client only ever use `Ruleset::canon()`, assembled from
`content.rs`; `MatchState::with_rules`, `Bot::with_rules`, and the `lab`
feature (`lab_spawn`) exist for `tbg-balance` alone (contract C10). The
ruleset is not serialized, so hashes, views, and the wire are unchanged.
`tbg_core::headless::play` is the server's loop without a clock, and
`tbg_core::replay` re-runs a recorded match; contract C4 proves the server
at 5 ms and 50 ms ticks equals the headless replay.

**Two clocks (D-41, BR-AG12–AG14).** `MatchState::tick` is the combat
clock; every rule that counts time reads it, and it freezes while an
Augment round pauses the battle. `MatchState::clock` counts `step()`
calls, pauses included: it closes a paused round, paces views (10 Hz),
and keys replay ops. A paused `step()` touches nothing but the selection
clock and the close check, so no extra task or timer exists for pauses.

Planned modules (create each only when its roadmap task starts):

| Crate | Module | Responsibility |
|-------|--------|----------------|
| core | `types.rs` | ids, `Faction`, `Archetype`, fixed-point helpers |
| core | `content.rs` | roster, evolution trees, synergies, match constants (GAME-DESIGN §5) as `const` tables |
| core | `rng.rs` | PCG32, ~20 lines, no external crate |
| core | `state.rs` | `MatchState`, `PlayerState`, `UnitState`, `Stronghold`, `Phase` |
| core | `intent.rs` | `PlayerIntention`, validation, `Rejection` reasons |
| core | `shop.rs` · `combat.rs` · `synergy.rs` | rule subsystems |
| core | `sim.rs` | `MatchState::step()` — the fixed tick, in a fixed order |
| core | `view.rs` | `view_for(seat) -> MatchView`, redaction, event drain |
| core | `bot.rs` | seeded heuristic opponent (practice, harness, load test) |
| core | `ruleset.rs` · `headless.rs` · `replay.rs` · `lab.rs` | numbers as data, the clockless match loop, replays, lab-only duels (P16-C) |
| core | `protocol.rs` | `ClientMsg`, `ServerMsg`, `PROTOCOL_VERSION`, encode/decode |
| client | `main.rs` · `camera.rs` · `arena.rs` | app, states, framing, environment |
| client | `art/` | `palette.rs`, `mesh.rs`, `figure.rs`, `anim.rs`, `roster/<faction>.rs` |
| client | `config.rs` · `overlay.rs` | URL params, server URL; FPS/RTT/view-timing overlay |
| client | `link.rs` · `local.rs` · `net.rs` | `MatchLink` (local or network, a Bevy `NonSend` resource) |
| client | `sync.rs` · `visuals.rs` | views → entities; **the only module that knows how units look** (placeholders in P3, figurines from P6) |
| client | `vfx.rs` · `audio.rs` · `ui/` | effects, sound; `menu.rs`, `hud.rs`, `evolve.rs`, `result.rs`, `toast.rs` |
| server | `lib.rs` · `main.rs` · `config.rs` | `run(config, listener)` (used by tests), env config |
| server | `ws.rs` · `lobby.rs` · `match_task.rs` | connections, sessions/queue/rooms, match loop |
| server | `metrics.rs` · `records.rs` · `names.rs` | counters + `/metrics`, SQLite match records, display names |
| server | `botclient.rs` · `bin/tbg-bots.rs` | WebSocket bot client for tests and load/latency validation |

Size caps (AGENTS.md) keep every file reviewable in one read. Content
tables are the one allowed exception.

## 4. Simulation model (`tbg-core`)

- **Fixed tick:** 20 Hz (50 ms). All timers are counted in ticks.
- **Integer math only:** positions in milli-tiles (`i32`, 1 tile = 1000),
  HP and damage as `i32`, percentages as basis points, `Energy` in
  sub-energy (600 = 1 Energy). No floats in state, so outcomes are
  reproducible on every platform and test assertions are exact.
- **Coordinates:** seat 0's `Stronghold` at x = 0, seat 1's at
  x = 16 000. The client mirrors x for seat 1 at render time.
- **Randomness:** one PCG32 stream per seat per purpose (shop), seeded from
  the match seed. The seed comes from the OS on the server and never leaves
  it (BR-S05). Units, iteration, and targeting use `Vec` order by unit id —
  never `HashMap` iteration.
- **Tick order:** 1) apply queued intentions in arrival order; 2) clock,
  phase, `Energy` regen with the Interest boost; 3) Augment rounds (no shop
  rotation: every purchase redraws the five offers, BR-S04); 4) synergy recompute;
  5) statuses tick down; 6) targeting + movement; 7) attacks resolve
  (cooldown, windup, splash, statuses, heals); 8) deaths removed, events
  emitted; 9) outcome check.
- **Events:** each step appends `Spawn`, `Attack{from,to}`, `Hit{to,amount}`,
  `Heal`, `Status`, `Death`, `Evolve`, `SynergyChanged`, `StrongholdHit`;
  views drain them so the client animates exactly what happened.
- **Views:** `view_for(seat)` returns tick, phase, clock, both Stronghold
  HPs, all units (id, owner, kind, form, x, hp, max_hp, status), the
  seat's own `Energy`, offers, regen multiplier, and `last_seq` (last applied
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
- **Messages** (exact definitions: `docs/phases/P2-core-simulation.md` step 9):
  `ClientMsg = Hello{version, resume} | QuickMatch | CreateRoom | JoinRoom{code} | Cancel | Intent{seq, intent} | Pong{id}`;
  `PlayerIntention = Purchase{offer} | Evolve{unit, branch}`;
  `ServerMsg = Welcome{token, name} | VersionMismatch{server} | Queued | RoomCreated{code} | RoomNotFound | MatchStart{you, opponent} | View(MatchView) | Rejected{seq, reason} | MatchEnd{outcome} | Ping{id} | NetStats{rtt_ms} | Error{code}`.
  Later versions append variants at the end of each enum (`PROTOCOL_VERSION`
  9): accounts v2 (`Register{name} | Login{key} | Logout` →
  `Registered{key, profile} | LoggedIn{profile} | LoggedOut |
  AccountError{code}`), ranked v3 (`RankedMatch`; `MatchStart` gains
  `mode, opponent_rating`; `RatingChanged{before, after, profile}`),
  ranking v4 (`Leaderboard{offset}` → `Leaderboard{total, offset, rows,
  you}`: 20 rows from the in-memory snapshot, `you` = your rank; one
  request per second per connection, extras ignored), takeover v5
  (`ErrorCode::SessionTakenOver`), v6 (`Surrender`, routed like an
  intention with the connection's seat → `MatchCmd::Surrender` →
  `MatchState::forfeit`, idempotent; `MatchStart` gains `you_name` and
  `you_rating` so the HUD names both HP bars), v7 (five Shop offers),
  v8 (D-40: `SetArcanas` → `ArcanasSelected` / `ArcanasRejected`, both
  frozen presets in `MatchStart`), and v9 (P39: `CreateChallenge` /
  `JoinChallenge{token}` / `RequestRematch` → `ChallengeWaiting{token}` /
  `ChallengeUnavailable` / `RematchOpen{opponent}` /
  `RematchState{you_requested, opponent_requested}` / `RematchClosed{declined}`),
  and v11 (P40: the fifteen-rank `SetArcanas`/`ArcanasSelected` payload and
  twelve-point schema; both frozen presets still travel in `MatchStart`).
  P39 appends after the Arcana variants, keeping every existing index and
  all `MatchStart` fields; `MatchMode::Challenge` is appended after Ranked.
  Peers older than v11 are rejected at the handshake.
  Intentions carry no seat: the server derives the seat from the connection.
- **Rates:** simulation 20 Hz; views every second tick (10 Hz) per seat,
  plus an immediate acknowledgement view to the seat whose intention was
  applied or rejected that tick (P9).
  Clients place views on the server clock (`MatchView::clock`), render an
  adaptive 100–350 ms behind the newest one (one view interval plus the
  p95 arrival jitter) and interpolate x; events play when the render time
  reaches their view (P30-C, `tbg-client/src/sync/timeline.rs`).
- **Prediction:** UI-level only — a tapped card shows pending and `Energy`
  dims optimistically until `last_seq` confirms or `Rejected` arrives.
- **Sessions:** the server issues a random 128-bit token in `Welcome`; the
  client keeps it per tab (`sessionStorage`) and opens a listen-only probe
  link on page load, resuming a live match with no tap (P9). Guests get a
  generated display name (`Commander 4821`); accounts (P11) are a unique
  `PlayerName` plus a generated `AccountKey` (name + 80-bit secret, the only
  credential). One active session per account: a new login takes over the
  old one, including a running match, whose seat token is updated so the
  end report finds the new session. The old socket first gets
  `Error{SessionTakenOver}` (the client shows "Your account signed in on
  another device", returns to the menu, and does not reconnect), and its
  session is dropped so a late auto-resume cannot re-enter the match.
  Idle sessions expire after the reconnect window, so a `Welcome` whose
  token differs from the one the client sent is a fresh guest session:
  with a stored key the client logs in again before any mode request
  (`net/login.rs`), otherwise ranked answers `LoginRequired`.
  While logged in, the session carries
  the account name, which opponents see in `MatchStart`.
- **Ranked (P12):** separate FIFO queue (`RankedMatch`; guests are refused
  with `LoginRequired`, private rooms stay casual). Pairing ignores rating.
  Ranked `MatchStart` carries the mode and the opponent's rating; after the
  match the store settles both Elo updates in one transaction and the lobby
  delivers `RatingChanged{before, after, profile}` to each live session.
  Casual matches never touch the rating columns.
- **Matchmaking:** quick match is a FIFO queue; private rooms use a
  5-character code (unambiguous alphabet, 10-minute expiry), shared as a
  link `?room=CODE`. Bot practice runs locally in the browser.
- **Quick Challenge (P39, BR-R08):** `?challenge=<32 hex>` consumes an
  independent 128-bit OS-random invitation, single-use with a 10-minute
  TTL. It pairs the owner and exactly one different session directly,
  through the same `pair` and match task, with no queue or Rating change.
  Creating/joining freezes Arcanas like queue/room entry (BR-AR05).
  An owner reload preserves the invite within the reconnect window; a
  guest cannot consume it while the owner is disconnected. Explicit
  Cancel removes it. Private rooms keep their code and existing behavior.
- **Rematch (P39, BR-R09):** Challenge results open a 120-second in-memory
  ticket for the same sessions. Both must request and be connected before
  expiry; pairing uses a fresh seed and keeps casual rules, no queue and
  no Rating change. Disconnect preserves the offer for reload; Menu/Cancel
  or a new match action declines it. Invites and tickets are bounded by
  the existing 10-second cleanup, with lazy expiry on joins/requests/resume.
  No additional permanent task or timer is needed.
- **Lifecycle:** paired → `MatchStart` → countdown → running (open-ended,
  with Augment pauses; D-41) → `MatchEnd` → both sessions return to the lobby. Disconnected seats stay
  in the running match for 30 s; reconnecting with the token resumes with
  a full view; otherwise the seat forfeits (BR-M03). A seat already
  detached at pairing starts the same reconnect timer.
- **Concurrency:** one task per connection, one lobby task, one task per
  match; they talk through `tokio::sync::mpsc`. No locks on game state.
- **Limits:** handshake within 5 s; inbound frame ≤ 1 KiB; token bucket
  20 msgs/s (burst 40) per connection, then disconnect; ≤ 8 sockets per IP;
  server ping every 1 s plus protocol-level pings every 5 s (any frame —
  binary, ping, or pong — resets the 15 s idle timer, so hidden tabs stay
  connected); account endpoints add per-IP fixed windows (P11: 3 registers
  per hour, 10 logins per minute, tuned by `TBG_REGISTER_PER_IP_HOUR` /
  `TBG_LOGIN_PER_IP_MIN`); Challenge joins add a separate sliding log
  (30 attempts per IP per minute, shared-NAT players allowed); global
  connection cap from config.
- **Latency measurement:** the server sends `Ping{id}` every second,
  computes RTT from `Pong{id}`, records it, and returns `NetStats{rtt_ms}`
  for the client overlay. The client also measures view inter-arrival
  jitter. Each match logs and stores RTT p50/p95 per seat, step time, tick
  lateness, and bytes. `GET /metrics` (loopback only) exposes live
  counters and 60-second RTT percentiles.
- **Validation tools:** `tbg-bots` runs N bot clients over real
  WebSockets with artificial delay/jitter and random disconnects; the
  browser accepts `?lag=MS` to feel added latency.
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
  Framing recomputes on resize. The client is first built with placeholder
  shapes (P3) and gets its final art only after online play is validated
  on a real server (P4, P5).
- **Input:** tap/click only. Shop cards are `bevy_ui` buttons; units are
  picked through an enlarged invisible pick volume so moving targets are
  tappable on phones.
- **UI:** `bevy_ui` for every screen (menu, HUD, evolve panel, result),
  sized in viewport units, thumb-reachable on portrait.
- **Build:** one WebGL2 release artifact; `opt-level="s"` vs `3` decided by
  measurement in P1; `wasm-opt -Oz`; brotli/gzip precompressed at
  deploy. WebGPU is a later optional build, not an MVP target.

## 7. Art direction and asset production

**Style candidate (P15; owner sign-off pending):** medieval tabletop
miniatures. [ART-DIRECTION.md](ART-DIRECTION.md) owns the proposed palette,
proportions, silhouettes, and motion language. D-22 remains proposed until
its owner decision is recorded in the P15 report.

**Production: code-built rigid parts, vertex colors, one shared material.**

- **Rig v2:** the phase's supplied enum names 17 body joints: Root, Hips,
  Spine, Chest, Head, UpperArm/Forearm/Hand ×2, Thigh/Shin/Foot ×2. It
  labels that list "18"; the implementation follows the exact names and
  records the discrepancy for review. Optional Back, HeadGear, and Tabard
  springs use rotation stiffness 60, damping 8, 120 Hz sub-steps and ±35°
  per-axis clamps. WeaponR, OffhandL, and Back are prop sockets.
- **Compatibility:** the preserved twelve-joint `FigureSpec`s go through
  an adapter. Torso maps to Chest, arms to the corresponding v2 joints,
  and old leg geometry is partitioned onto thigh, shin, and foot without
  duplicating triangles. Old units and Archer evolutions remain adapted
  until their refinement phases. Only Archer base has the P15 final-art
  candidate; all eight new body plans are also available as gray blockouts.
- **Body plans:** Humanoid and Large use approximately four-head
  proportions; Large blockouts have 1.5× scale and shoulders 1.4× hips.
  Legacy proportions and evolution scale remain compatible. Existing
  client lane prominence is retained for the Archer candidate.
- **Batching:** parts are merged per joint, cached per form, and shared
  across instances. Archer team-color variants are cached in vertex
  colors, so both teams retain the one shared material. Bow strings and
  arrow geometry are shared assets. Hit flash uses the existing tint material.
- **Color/light:** named palette constants, three vertex values (0.82,
  1.0, 1.12), baked sky/ground fill and a small normal-based rim; one warm
  key, sky ambient, and soft distance fog. No textures or custom unit shader.
- **Animation:** keys select Linear/CubicIn/CubicOut/CubicInOut/Hold;
  clip metadata delivers Strike, Release, and Footstep events. Crossfade
  remains 0.1 s. Walk (gait v2, D-48) advances by the distance the figure
  travels over its stride, on two-bone legs for every figure, with feet
  locked in world X and Z while planted and a 0.15 s stop hysteresis;
  animation runs in `PostUpdate` after all movement. Phase offsets are
  decorrelated by visual id. Archer draw/anchor releases at 0.18 s and
  recovers by 0.50 s; its cosmetic arrows still follow facing for 0.12 s.
  Simulation events still decide damage and outcomes.
- **Unit art kits (D-48, P54):** `art/kits/<unit>/` owns a unit's five
  rig-v2 forms (`model.rs`), its signature pose layer over the shared
  clips (`motion.rs`: keyframe tables and procedural IK), gait, aim and
  attack (`mod.rs`: its own `ShotKind` from a muzzle socket, or a melee
  `Impact` at the blade). Kits aim with the body (an off-center weapon is
  lined up exactly) and raise the weapon to the launch tangent of their
  own shot (`kits::aim`); `visuals/attacks.rs` fires on Release / Strike
  from the real joint transforms. Glow parts merge into one unlit layer
  per joint. Shots (`vfx/shots.rs`) fly a parabola in fixed times that
  keep ART-DIRECTION §4's 0.30 s impact; impacts and muzzles
  (`vfx/impacts.rs`) use shared meshes and materials only.
- **Aiming Strongholds:** the render-delayed `AttackStart` of a tower
  becomes a `TowerOrders` entry; the Artillery carriage and barrel and the
  Sentinel ballista slew onto the target (yaw, elevation along their own
  shot) and fire cannonball or bolt from the muzzle. The other kinds have
  animated defenders (D-49, `arena/strongholds/gunners.rs`): Classic
  crossbowman, Bastion gun port and falconet, Treasury roof mortar lobbing
  gold, Command archer; a fallen defender leaves a bolt from the top.
  Guide: [art/UNIT-ART-KITS.md](art/UNIT-ART-KITS.md).
- **Arena/readability:** the existing generated arena, Stronghold damage
  states, team rings, HP bars, and hit/spawn/death feedback remain integrated.
- **Review tools:** `?gallery=palette`, `?gallery=blockout&silhouette`, and
  `?gallery=<unit>`; `&slow=0.25`, `&turn`, and `&frames`;
  `?gallery=<unit>&range` (five forms attacking dummies on the BattleLine)
  and `?gallery=<unit>&stress=N` (N walking, attacking copies of its forms). Named unit views
  show five forms; contact sheets show the selected base form at four
  quarters of every clip (`&clip=attackranged` can narrow the sheet).
  Neutral studio floors prevent arena props from obscuring review figures;
  the palette board and live practice still demonstrate the game lighting.
- **Measurement:** `?gallery=archer&stress=30&measure` creates 30 walking,
  attacking Archer visuals; `&shadows=0` supplies the current low-cost
  proxy, not the future P17 Low tier. Opt-in browser diagnostics count
  actual WebGL calls, including shadow passes. Physical-phone validation
  and subjective approval remain owner/reviewer gates.
- **Audio (P7):** 11 original synthesized WAV effects (~130 KB), browser
  build only. Font remains Bevy's default font.
- **Fallback:** D-22 permits the reviewer-run Blender-scripted skinned
  glTF comparison after two owner rejections. P15 adds no glTF, textures,
  downloaded models, skinning, or dependencies.

## 8. Persistence

- **MVP (from P5):** guest sessions in memory; one SQLite file (WAL) with one
  `matches` table — end time, duration, outcome, reason, seed, per-seat
  network stats and purchases/evolutions as JSON, server timing. No personal data and no IP addresses. It
  feeds the fun metrics in GAME-DESIGN §6. Match inserts go through the
  store channel (fire-and-forget, never on the tick loop).
- **Accounts (from P11):** a single-writer `store` task owns the SQLite
  connection (schema migrations `v1` matches, `v2` accounts); the lobby
  never awaits it — account commands carry a oneshot for their reply.
  The `accounts` table holds the name, lowercase name (UNIQUE), SHA-256 of
  the key secret, timestamps, and rating columns (defaults 1000/0).
- **Ranked (from P12):** migration `v3` adds `ranked_matches` (one row per
  ranked match: both account ids, winner 0/1/NULL, reason, duration, both
  before/after ratings, both faction-purchase mixes) plus indexes
  `ranked_by_a`, `ranked_by_b`, and `accounts_by_rating` (rating DESC, wins
  DESC, name_lower). The settlement transaction is the only writer of the
  rating columns.
- **Leaderboard snapshot (P13):** the writer thread keeps every ranked
  account (≥ 1 ranked game) in rank order (`rating DESC, wins DESC,
  name_lower ASC`) and publishes it as `Arc<Snapshot>` through a
  `tokio::sync::watch` channel: built at startup, rebuilt at most every
  10 s and only after a ranked result committed. `/api/leaderboard` and the
  WS `Leaderboard` reply only clone the `Arc` — no SQL per request.
  `/api/players/{name}` runs at most three indexed queries (account row,
  faction sums, last 10 ranked matches with the opponent name).
- **Boundary:** only `store/` touches SQL.
- **Move to PostgreSQL when** two or more server instances need shared
  writes (accounts, rankings, `Inventory`), or analytics queries start
  competing with match writes. Swap the module then — not before.

## 9. Deployment and operating cost

- Caddy terminates TLS, serves the precompressed static bundle with
  immutable cache headers for hashed files, and proxies `/ws`, `/health`,
  and `/api/*` to `tbg-server` on localhost. The public ranking pages
  (`/ranking/`, `/ranking/player.html?name=…`, P13) are plain HTML + CSS +
  vanilla JS copied by Trunk (no framework, about 15 KB); they are not
  content-hashed, so all of them are `no-cache` (ETag revalidation).
- `deploy/deploy.sh` (P5) builds the WASM bundle locally, rsyncs source
  and bundle, builds `tbg-server` on the VPS (avoids glibc mismatches),
  installs it, and restarts the systemd unit. `tbg-server` listens on
  `127.0.0.1:8080` only. Firewall opens 22/80/443 only. Secrets live in an env file outside
  the repository.
- Backups: nightly `sqlite3 .backup`, 7 local copies, weekly off-site copy.
  Match records are non-critical; a lost day is acceptable.
- Monitoring: `/health` endpoint + a free external uptime checker +
  structured logs (`journalctl`). Nothing more until there is a need.
- On a 2 vCPU VPS set `TBG_WORKERS=2` in the env file (P10 measured one
  worker per core; unset means all cores).
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
| WASM download (compressed) | ≤ 4 MB (hard ceiling 6 MB) | P1 · measured P4: 4.59 MB brotli, 7.34 MB gzip |
| Other assets on first load | ≤ 1 MB | P7 |
| Total download, lazy assets included | ≤ 100 MB, first load unchanged (**Proposed**, D-26) | P16-A |
| Time to menu | ≤ 5 s desktop broadband; ≤ 10 s mid-range phone on 4G | P1, P5 |
| Frame rate | 60 fps desktop; ≥ 30 fps mid-range Android and iPhone 11-class with 30 units | P1, P3, P6, P7 |
| Draw calls in a busy match | Measured; no fixed maximum (D-28) | P16-A |
| Triangles | Measured and reported per unit form and arena layer; no fixed maximum (D-30) — the frame-rate and memory targets decide | P6 · P16-A |
| Browser memory | ≤ 300 MB | P7 |
| Sim step, 30 units (release) | ≤ 50 µs | P2 · measured: 0.44 µs harness, 4–6 µs in server |
| Sim step, large armies (release) | measured, no fixed cap (D-32) | P16-B: 200 units 45 µs mean / 54 µs p99; full views 28.6 KB/s per player at 100 a side (delta views recommended, P30-B) |
| Server memory | ≤ 100 KB per match + about 64 KB per socket | P10 local: ~300 KB per running match (target missed — sockets and per-connection queues dominate; worst case is bounded by `CONN_QUEUE` and the 64 KiB write cap) |
| Bandwidth per player | ≤ 6 KB/s down, ≤ 0.5 KB/s up | P4 local: 1.46 KB/s down, 3 B/s up · P10 local: 1.29 KB/s down at 250 matches |
| View-gap p95 under 40 ± 20 ms added latency | ≤ 150 ms | P4 local: 128 ms |
| Ack latency p95 (tap to first covering view) | ≤ RTT p95 + 60 ms | P9: measured in the phase report · P10 local: 50 ms at 250 matches, no added delay |
| Concurrent matches on 2 vCPU / 4 GB | ≥ 500 | P10 local: 1000 matches ≈ 1 core, 1500 matches ≈ 1.5 cores, 0 late ticks (16-core dev PC; steady-state `scripts/capacity.sh`) |
| Tokio workers on a 2 vCPU VPS | — | P10: `TBG_WORKERS=2` (one per core); 1 worker also held 1500 matches with 0 late ticks, but 2 leaves headroom for bursts |

**Capacity reasoning.** Simulation CPU is tiny (about 20 units, integer
math). The first limits are socket fan-out and bandwidth: 500 matches ≈
1 000 players × 1.3 KB/s ≈ 10 Mbit/s. Measure with `tbg-bots` (P4 locally, P5 on the VPS)
(bot clients over real WebSockets) at 100/250/500 matches while watching
CPU, RSS, and bytes/s; `scripts/capacity.sh` (P10) automates the steady-state
table at 250/500/1000/1500 concurrent matches.

## 11. Security baseline

- Server-authoritative combat, `Energy`, shop, evolution, and outcomes.
- Seat comes from the connection, never from message content; tokens are
  random, never shown to the other player, and bound to one seat.
- Account keys: the plain key exists only in the `Registered` reply and the
  player's own storage; the server stores SHA-256 of the secret and
  compares hashes in constant shape. Unknown name and wrong secret share
  one `BadKey` error and code path. Logs never contain keys or IPs.
- Every message is decoded with size limits; malformed input closes the
  socket; intentions are validated against current state.
- Rate limits, per-IP socket caps, and timeouts as in §5.
- Read-only API (P13): `/api/*` is limited per IP (`TBG_API_PER_IP_MIN`,
  default 120 requests per minute → 429 `{"error":"rate_limited"}`); bad
  name shapes 404 before any query. Replies never carry account ids, key
  hashes, IPs, or timestamps finer than the day. The ranking pages build
  the DOM with `textContent` / `createElement` only (no `innerHTML`, no
  `eval`, no inline handlers) under a `default-src 'self'` CSP.
- TLS everywhere public (Caddy); no secrets in git (`.env*` ignored).
- Data at rest (P14): the database directory and the backup directory are
  mode 700 and files are created with `UMask=0077` (names, ratings, and key
  hashes are not world-readable on the host).
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

Balance changes add (release builds, D-33):

```bash
cargo test -p tbg-server --test lab_equivalence -- --ignored   # one 1x match
cargo run --release -p tbg-balance -- gate --pairs 1000
```

A native check never proves browser compatibility. Rendering claims need
the Trunk build running in a real browser (P1 device matrix).

## 14. Planned changes (P9–P31)

Planned in `docs/phases/`; each phase updates the sections above when it
lands. Summary: P9 `TCP_NODELAY`, acknowledgement views, protocol pings,
auto-resume (§5); P10 measured server optimizations and a `server` build
profile (§9, §10); P11 accounts (name + key), a single-writer SQLite store
task with migrations (§5, §8, §11); P12 ranked queue and Elo (§5, §8); P13
read-only JSON API + static ranking pages (§5, §8, §9, §11); P15+ the
medieval visual suite described in [ART-DIRECTION.md](ART-DIRECTION.md)
(rig v2 and clip sets replace the 12-joint plan in §7; quality tiers).

## 15. Arcanas extra (active by D-40/D-42; values Proposed)

GAME-DESIGN §9 owns the node table, costs, prerequisites and proposed
values; DECISIONS D-40/D-42 record why the extension is live. P40 expands
the original six-node system in place.

- **Core:** `ArcanaPreset` is fifteen rank bytes. `complete` validates
  per-node maxima, real point cost (supremes cost two) and the shared
  prerequisite table. Per-rank percentages stay in `Ruleset.arcana_bp`;
  the nine conditional values live in `Ruleset.arcana`. Stats still add
  basis points from Evolution, Synergy, Arcanas and Augments before one
  truncation. Conditional checks read `MatchState::army`, a per-tick
  living Tank/DPS count decremented on death, so no hit scans the army.
  No modifier framework, event bus or per-tick allocation is added.
- **Server:** selections are validated only while idle, frozen at
  queue/room/challenge entry, carried through reconnect/account takeover,
  and copied into the authoritative MatchState. Seat identity comes from
  the connection; no client bonus values reach the simulation.
- **Wire:** protocol v11 carries fifteen ranks. MatchStart, replay records
  and ranked rows keep both frozen presets. Six-rank historical rows are
  read through `ArcanaPreset::migrate_legacy` and are never rewritten.
- **Client:** the existing HTML editor keeps the three banners/emblems,
  renders two originals, two children and a centered supreme per family,
  shows cost and prerequisite state, refuses removals with active
  dependents, and fits smaller screens without requiring zoom. The game
  persists schema 2 locally; a schema-1 document is migrated.
- **Store:** account presets accept six or fifteen bytes; a six-byte row
  is migrated in place. Match rows record fifteen bytes from now on.
- **Release:** values stay Proposed. The P34/P40 lab must confirm viable
  family, hybrid and no-supreme builds and reject a compulsory supreme.
  The existing single-VPS/SQLite deployment and Elo/FIFO design remain.

## 16. Strategic expansion (Proposed, P16-F1–F8)

> **Owner revision 2026-10-29 (D-41).** Uncapped `Energy`, Interest and
> infinite Augment rounds are implemented and active; `siege`,
> `ServerLimit`, `SERVER_LIMIT` and the `Siege`/`Collapse` wire variants
> were dropped (the protocol is v10, not an appended v9). The bullets
> below are the original plan, kept for F2–F4.

Planning addendum. GAME-DESIGN §10 owns the
rules and values, D-34–D-38 the reasons, each phase file the exact steps.

- **Ruleset-gated mechanics.** Each mechanic is a `Ruleset` field whose
  canonical value disables it (`energy_cap`, `interest`, `siege`,
  `reserve_ticks`, `fortify_enabled`, `augment_rounds`). The lab measures
  enabled variants; the owner's switch changes `content.rs` once. The
  server and client still only use `Ruleset::canon()` (C10).
- **Core.** Per-seat Stronghold data on `PlayerState` (kind, max HP,
  Fortification tier/branch, shield); one `scale_hp` helper for every max-HP
  change; one `PlayerState::spend` for every Energy cost (Interest
  low-water). One new module, `augment.rs`: the closed `AugmentEffect`
  enum, the mirrored draw (PCG32 stream 3; stream 5 picks the practice
  bot's kind), and fixed-size per-player bp sums. Hooks stay in the module
  they modify (`effective_stats`, `hits.rs`, `combat.rs`, `intent.rs`,
  `summon.rs`). No effect engine, event bus, trait, allocation per tick, or
  dependency.
- **Determinism.** New serialized fields move hash goldens; each phase
  proves unchanged outcomes, events and purchases before refreezing (P16-F1
  step 2). `SERVER_LIMIT` bounds `headless`/`replay`; contracts prove it
  unreachable.
- **Wire.** Appended variants only: `Phase::Overtime` renamed `Siege` at
  the same index; `EndReason` + `Collapse`, `ServerLimit`; intentions
  `Reserve`, `Release`, `Fortify`, `ChooseAugment`, `Forgo`;
  `ClientMsg::SelectStronghold`; `MatchStart` + both kinds; `MatchView` +
  Interest, Collapse rate, hold, Stronghold max, Fortification, kinds,
  round, Augments, shields, Evolution cost. One `PROTOCOL_VERSION` bump per
  phase that changes the wire. Opponent Energy, offers, holds and RNG stay
  hidden; kinds, Fortifications and equipped Augments are public.
- **Server.** Kind chosen while idle, frozen in the queue ticket or room,
  restored on reconnect/takeover. `DRAIN_LIMIT` grows past the longest
  possible match (P16-F1).
- **Store.** Additive migrations: `matches.ruleset_fp` (P16-F1),
  `accounts.stronghold_kind` (P16-F4); seat JSON gains kind,
  Fortifications, Augment offers/choices, Interest and Bounty totals.
- **Client.** Desktop HTML HUD (D-29) and the Bevy fallback both gain the
  Interest meter, Siege clock, hold locks, kind indicators, the Fortify
  panel and the non-modal War Council; four code-built Stronghold
  silhouettes with shared Fortification overlays (D-30 measured triangles).
- **Budgets.** Step time ≤ +10 % at 100 units a side over P16-B; view
  growth of tens of bytes; per-match memory under 1 KB more; measured in
  every phase and re-checked under load in P16-F8.
