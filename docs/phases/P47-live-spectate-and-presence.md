# P47 — Live battles in the lobby, players online, and spectating

**Status:** planned; **revised 2026-10-30** for today's code and the owner's
lobby request (see *Revision notes*).
**Base:** `phase/battle-hud-cards` (the live release: consolidation of every
PR, arenas D-55, HUD cards). If `phase/units-archer-crab-selection` (P56,
Cargo Crab, protocol v16) is merged first, rebase on it.
**Branch:** `phase/p47-live-lobby`.
**Platform:** desktop browsers only (AGENTS.md *Platform focus*): no phone
menu work in this phase.
**Difficulty:** medium–high. One protocol bump, a public view in core, a
lobby registry with a snapshot, fan-out in the match task, two read-only
API routes, a spectator render mode and lobby surfaces.

## Goal

Anyone, guest or logged in, opens the game and **immediately** sees in the
lobby:

1. **How many players are online right now** and how many are in a battle,
   so they know at a glance whether the game is active;
2. a **Live battles** panel with the battles happening now (players,
   ratings, arena, Stronghold kinds, HP, match time, spectators) and a
   **Watch** button;

and can **watch any listed battle in real time** with one click. Watching is
listen-only and shows only public information. Players keep today's
experience, plus a small "N watching" badge.

## Revision notes (2026-10-30)

- **Owner request:** a live-battles panel on the lobby home and the online
  player count visible as soon as the lobby opens (not hidden in a tab).
- **Watchable matches widened (O-5):** public **Casual** quick matches and
  **Ranked**; private rooms, challenges, rematches and practice stay private.
  Ranked-only would usually leave the panel empty (guests cannot play ranked).
- **Rooms share `MatchMode::Casual`** (`lobby/matchmaking.rs`: the quick-match
  path and `join_room` both call `pair(…, MatchMode::Casual, …)`), so the
  listing needs an explicit `Listing::{Public, Private}` decided where the
  match is paired, never inferred from the mode.
- **Protocol:** today `PROTOCOL_VERSION = 15` (arenas, D-55) on the live
  branch; P56 uses 16. Take the **next free version** when you start and pin
  it in the contract test.
- **Arenas (D-55):** the public view and `SpectateStart` carry the match's
  `Arena`; the spectator scene switches with the existing `ActiveArena`.
- **i18n (P41) is merged:** every new string goes into `assets/i18n/en.json`
  and `pt-BR.json`; Rust strings follow the existing `i18n` module.
- **Lobby v2 (#14) and the Arenas page** changed `index.html`,
  `assets/lobby/*` and the nav: re-read them; do not regress them.
- **No phone menu** (desktop focus); the ranking-page section is optional
  and last.

## Read first (only these)

- `AGENTS.md`; `docs/ROADMAP.md` (P47 row); this file.
- `docs/GAME-DESIGN.md`: glossary, BR-S05 (private information), BR-M01–M07,
  BR-R01–R09, BR-AG09/12/14 (Augment visibility), BR-SK08, §12 Arenas.
- `docs/ARCHITECTURE.md` §5 (sessions, rates, limits, concurrency), §9,
  §10 (budgets), §11 (security, read-only API), §12.
- `docs/DECISIONS.md` D-06, D-09, D-39, D-41, D-46, D-54, D-55.
- Core: `src/view.rs`, `src/state.rs` (`Event`), `src/protocol.rs`,
  `tests/contract_protocol*.rs`.
- Server: `lobby.rs`, `lobby/{sessions,matchmaking,ranked,leaderboard,rematch,challenge}.rs`,
  `chan.rs`, `ws.rs`, `ws/*`, `match_task.rs`, `match_task/{command,views}.rs`,
  `api.rs`, `lib.rs`, `limits.rs`, `config.rs`, `metrics.rs`,
  `store/worker.rs` (the `watch` snapshot pattern), `botclient*`,
  `tests/online.rs`, `tests/arenas.rs`.
- Client: `src/net/*`, `src/sync/*`, `src/arena/scene.rs` (`ActiveArena`),
  `src/ui/{battle_hud,hud,hud_update,actions,desktop_actions}.rs`,
  `index.html`, `assets/lobby/*`, `assets/arenas/*`, `assets/battle/*`,
  `assets/i18n/*`, `tests/*.test.js`.

## Audit summary (re-verify before coding)

| # | Fact | Where |
|---|------|-------|
| A1 | The lobby keeps no registry of running matches: a session only holds `SessionState::InMatch{tx, seat}`; every pairing goes through `Lobby::pair(tokens, mode, …)`, which also draws the arena | `lobby.rs`, `lobby/matchmaking.rs` |
| A2 | Quick match and private rooms both pair as `MatchMode::Casual`; challenges as `Challenge`; ranked as `Ranked` | `lobby/matchmaking.rs`, `lobby/challenge.rs`, `lobby/rematch.rs` |
| A3 | Views are built per seat (`view_for(seat)`) and sent as `ConnEvent::Send(ServerMsg)`; each connection encodes its own copy; a full outbound queue detaches the **seat** | `match_task/views.rs` |
| A4 | `MatchView` carries private data: `energy`, `regen_bp`, `offers`, `last_seq`, `evolve_costs`, `augment_round.resolved` for `you` | `view.rs` |
| A5 | The read-only API reads an in-memory `watch` snapshot (leaderboard), never SQL per request; per-IP limit `TBG_API_PER_IP_MIN` (120/min); replies carry no ids, IPs or tokens | `api.rs`, `store/worker.rs`, ARCHITECTURE §11 |
| A6 | `/metrics` counts connections and running matches but no presence or spectators, and is not shown to players | `metrics.rs` |
| A7 | Every open tab keeps a listen-only probe link (P9), so connections ≠ distinct people; a tab with the lobby open **is** a person on the site | ARCHITECTURE §5 |
| A8 | No spectator concept exists anywhere | — |
| A9 | The desktop lobby already polls `/api/players/{name}` from `lobby.js`; the proxy already serves `/api/*` in production; JSON is cached `max-age=3600` by nginx unless the route sets its own `Cache-Control` | `assets/lobby/lobby.js`, VPS |

## Non-goals

No replays or VOD, no chat, no betting, no watching rooms, challenges,
rematches or practice, no camera control beyond the fixed arena camera, no
delta views, no persistence (memory only; **no schema change**), no rule or
balance change, no phone UI.

## Owner decisions (defaults used unless the owner says otherwise)

| ID | Question | Default | Why |
|----|----------|---------|-----|
| O-1 | Broadcast delay for spectators | **0 s** | The spectator view is public-only (BR-LV03); a delay would protect nothing private |
| O-2 | List online **account names** | **yes**, name + rating + status; guests only counted | Names and ratings are already public on the leaderboard |
| O-3 | Caps | 200 spectators per match, 2 000 in total | ≈ 2.6 Mbit/s at the measured 1.3 KB/s per view stream |
| O-4 | Players see the spectator count | **yes**, count only, never names | Social proof without exposure |
| O-5 | Which matches are listed and watchable | **public Casual quick matches and Ranked** (revised) | Owner wants a lobby panel of battles happening; rooms, challenges, rematches and practice stay private |
| O-6 | What "online" counts | every session with an attached connection (a tab with the game open), accounts counted once; plus "in battle" = seats in running matches | The owner wants to see whether the game is active, not only logged-in accounts |

## Contracts (freeze before coding)

### Rules (append to GAME-DESIGN, new section "Live battles and spectating", Proposed)

| ID | Rule |
|----|------|
| BR-LV01 | Only live matches paired from the **public queues** (Casual quick match, Ranked) are listed and watchable; private rooms, challenges, rematches and practice never |
| BR-LV02 | Any session (guest or account) may watch while **idle**; watching is listen-only: a spectator has no seat, and any `Intent`/`Surrender` from it is refused like today's `NotInMatch` |
| BR-LV03 | Spectators receive only `PublicView`: no `Energy`, regen, offers, `last_seq`, Evolution prices, RNG, or a pending Augment pick (BR-S05, BR-AG14). Augment options of an open round are public; each seat's *resolved* flag is public, its choice only after resolution |
| BR-LV04 | Spectator delay is 0 s (O-1) |
| BR-LV05 | Both players see the number of spectators, never who |
| BR-LV06 | Presence (O-6): `online` = sessions with an attached connection, accounts counted once; `in_battle` = seats of running matches (practice excluded); online accounts are listed with name, rating and status (`idle`, `queued`, `playing`, `watching`); guests are counted, never listed; no ids, IPs, tokens or the `LiveId` of a private match ever leave the server |
| BR-LV07 | Featured battle: the listed match with the highest mean rating (guests count as 1000), ties by longest running, then lowest `LiveId`. It stays featured for at least 30 s unless it ends |
| BR-LV08 | A listed match has a random, non-sequential 64-bit `LiveId` (OS random), valid only while it is live; it is shareable as `?watch=<16 hex>` |
| BR-LV09 | Caps (O-3); full or unknown answers `SpectateUnavailable`, never a disconnect; a slow spectator is dropped without ever affecting a seat |

### Wire: next free `PROTOCOL_VERSION` (append-only)

```rust
pub type LiveId = u64;
// ClientMsg, appended
Spectate { id: LiveId },
StopSpectating,
// ServerMsg, appended
SpectateStart { id: LiveId, mode: MatchMode, arena: Arena,
                players: [LivePlayer; 2], arcanas: [ArcanaPreset; 2] },
SpectatorView(Box<PublicView>),
SpectateUnavailable { reason: SpectateRefusal },
SpectateEnd { outcome: Option<Outcome> },
pub struct LivePlayer { pub name: String, pub rating: Option<u16> }  // None for guests
pub enum SpectateRefusal { NotFound, NotWatchable, Full, NotIdle, RateLimited }
```

`MatchView` gains `spectators: u16` (BR-LV05).

### Core: `PublicView` (`tbg-core::view`)

`MatchState::public_view(events: &[Event], spectators: u16) -> PublicView`
with `tick, clock, elapsed, phase, phase_ticks_left, arena, stronghold_hp,
strongholds, shields, units, synergy, augments, round: Option<PublicRound>,
events, outcome, spectators`, where `PublicRound { round, options, costs,
ticks_left, resolved: [bool; 2] }`. Step 1 classifies every `Event` variant
as public or private, with a test that fails when a new variant is added
unclassified.

### Server state (memory only)

- `Lobby::pair` takes a `Listing::{Public, Private}`: the quick-match and
  ranked paths pass `Public`; `join_room`, challenges and rematches pass
  `Private`. Only `Public` matches enter `live: HashMap<LiveId, LiveMatch>`
  (lobby state, not simulation, so `HashMap` is allowed); removed on
  `MatchEnded`. `LiveMatch { tx, mode, arena, players: [LivePlayer; 2],
  kinds, started: Instant, progress: LiveProgress, spectators: u16 }`.
- `SessionState::Watching { id: LiveId }`, entered only from `Idle`.
- The match task sends `LobbyCmd::LiveProgress { id, hp_bp: [u16; 2],
  tiers: [u8; 2], elapsed_s: u32, spectators: u16 }` every 20 ticks with
  `try_send` (dropped when the lobby inbox is full: progress is cosmetic).
  Private matches send nothing.
- The lobby publishes `watch::Sender<Arc<LiveSnapshot>>` at most once per
  second and only when something changed: the counts (`online`,
  `accounts`, `in_battle`, `watching`), the sorted live list (≤ 50 rows),
  the featured id and the online account rows (leaderboard pattern: the API
  never touches lobby state).

### Fan-out (match task)

- `MatchCmd::Watch { conn: ConnHandle }` / `MatchCmd::Unwatch { conn_id }`;
  `spectators: Vec<ConnHandle>` (cap per match).
- On each view tick with spectators: build `public_view` **once**, encode
  **once**, send `ConnEvent::Frame(Bytes)` (a new pre-encoded variant) to
  each spectator with `try_send`. `Full` or `Closed` removes the spectator
  and tells the lobby; it never detaches a seat.
- New spectator: `SpectateStart` plus an immediate full `SpectatorView`.
  Match end: `SpectateEnd{outcome}` to every spectator; the lobby returns
  their sessions to `Idle`.

### Lifecycle

`Idle` → `Spectate{id}`, checked in this order: rate limit (1/s per
session, else `RateLimited`); idle (else `NotIdle`); live (else
`NotFound`); public (else `NotWatchable`, unreachable by construction but
tested); caps (else `Full`). Then `Watching` and `MatchCmd::Watch`.
`StopSpectating`, a disconnect or the match end return to `Idle`. A reload
does not auto-resume: the client re-sends `Spectate` from `?watch=` or the
id kept in `sessionStorage` for that tab.

### HTTP (read-only, existing per-IP limit, explicit `Cache-Control`)

- `GET /api/live` → `{"online":{"players":N,"accounts":N,"in_battle":N,"watching":N},
  "featured":"<16 hex>"|null, "matches":[{"id":"<16 hex>","mode":"casual"|"ranked",
  "arena":"<slug>","players":[{"name","rating"|null,"kind","tier"},{…}],
  "elapsed_s":N,"hp_bp":[a,b],"spectators":N}]}`. ≤ 50 rows, featured first,
  then highest mean rating; **`Cache-Control: no-cache, max-age=0`** (the
  route must override nginx's JSON default, A9); ≤ 16 KB.
- `GET /api/online?offset&limit` → `{"total":N,"rows":[{"name","rating","status"}]}`,
  rating descending, `limit` ≤ 100 (existing `paging`), `max-age=5`.

### Config

`TBG_MAX_SPECTATORS` (default 2 000) and `TBG_SPECTATORS_PER_MATCH`
(default 200), read in `Config::from_env` like the existing limits.

### Client behavior (desktop)

- **Online count, always visible in the lobby header** (owner request):
  a compact chip `● N online · M in battle` next to the Sound button,
  filled by the first `/api/live` call as soon as the lobby shows, then
  refreshed every 5 s while the lobby is visible (`visibilitychange` pauses
  it). While loading it shows a neutral placeholder, never "0".
- **Live battles panel on the lobby home** (owner request): a panel in the
  lobby v2 style at the right, above the news panel, aligned with its
  edges, never covering the mode panel or the centre of the arena at
  1920×1080 and 1280×800:
  - header `Live battles · N`;
  - the featured battle first, then up to 3 more rows: both names,
    ratings (or "Guest"), mode badge (Casual/Ranked), arena icon/name,
    two HP bars, match time, `N watching`, **Watch**;
  - **See all** opens a dialog with every listed battle (Watch, Copy watch
    link) and the online accounts list (lazy `/api/online`);
  - empty state: "No battles right now" with a **Play Casual** button that
    triggers the existing quick-match action.
- DOM built with `textContent`/`createElement` only (existing CSP); every
  string through the i18n catalog (en, pt-BR); new `data-*` names must not
  collide with `lobby.js` text projection (`tests/lobby.test.js`).
- **Spectator mode:** the match's arena (`ActiveArena`), neutral camera
  with seat A on the left; the top bar shows both names, ratings,
  kinds/tiers and HP; a `LIVE · N watching` badge; during an Augment pause
  "Players are choosing" with the public round; no Shop, Fortify, Energy
  bar or War Council input; a **Leave** button; at the end the result
  overlay and **Back to lobby**. The client maps `PublicView` into its
  render state internally; it never invents private values.
- **Players:** the battle HUD shows `N watching` when > 0.
- **Ranking page (optional, last):** a "Live now" section with watch links.

## Steps (in order; check and commit each)

0. **Baseline.** `scripts/capacity.sh` at 250 matches (CPU, RSS, bytes/s,
   late ticks) and a `tbg-bots` run; save to `docs/media/p47/baseline.txt`.
1. **Core `PublicView`** and the event classification. Contract tests:
   every private field absent; a pending pick never visible before
   resolution; every `Event` variant classified; same state gives the same
   public view; the arena is present. `cargo test -p tbg-core`.
2. **Protocol (next free version).** Append the messages; bump
   `contract_protocol` (version pin, new variants' bytes) in the same commit
   as the GAME-DESIGN BR-LV rows and a new decision (next free `D-` number;
   D-56 and D-57 are taken by parallel work — check).
3. **Listing, registry, progress and snapshot (server).** `Listing` in
   `pair`; registry; `LiveProgress`; the throttled `LiveSnapshot`; presence
   counts. Unit tests: casual quick match and ranked listed; room,
   challenge, rematch and practice never; removal on end; featured ordering
   and 30 s stickiness; counts after a disconnect and a takeover.
4. **Spectating (server).** Lifecycle, refusals, fan-out with
   `ConnEvent::Frame`, caps, cleanup. Online tests: an anonymous guest
   watches a casual bot match to the end and receives `SpectateEnd`; its
   `Intent`/`Surrender` change nothing; a room match is `NotFound`; a queued
   session gets `NotIdle`; a full match gets `Full`; a spectator that never
   reads is dropped while both seats finish with 0 late ticks and no
   detach; players see `spectators` go up and down; the spectator gets the
   match's arena.
5. **HTTP API.** `/api/live`, `/api/online` from the snapshot only. Tests:
   shapes and limits; no tokens, IPs or account ids (assert on the raw
   JSON); 429 over the per-IP limit; private matches absent; the
   `Cache-Control` header of `/api/live`.
6. **Metrics and load tool.** `/metrics` gains `spectators`,
   `live_matches`, `online_players`; `tbg-bots --spectators N` opens N
   anonymous sockets that read `/api/live` and watch random listed ids.
   Measure 250 matches with 0, 500 and 2 000 spectators into
   `docs/media/p47/capacity.txt`.
7. **Client transport.** `Spectate`/`StopSpectating`, `?watch=<hex>` and
   `sessionStorage` resume, every new `ServerMsg`, Leave.
8. **Spectator render mode.** Arena switch, neutral camera, HUD variant,
   end overlay; every player input system disabled while spectating, with a
   test that no intention is sent.
9. **Lobby surfaces.** Header online chip, Live battles panel, See-all
   dialog with the online list, i18n keys, JS tests (data shapes, empty
   state, polling pauses when hidden, no `data-*` collision).
10. **Canon and docs.** GAME-DESIGN BR-LV01–LV09 (Proposed); DECISIONS;
    ARCHITECTURE §5 (messages, lifecycle, fan-out, limits), §10 (budgets),
    §11 (routes); ROADMAP status.
11. **Verification and phase report.** The AGENTS.md matrix, `trunk build
    --release`, desktop browser checks at 1920×1080 and 1280×800. **Do not
    deploy**: the owner approves the look first, then the reviewer deploys.

## Budgets

| Metric | Target |
|--------|--------|
| Spectator download | ≤ 6 KB/s (same as a player) |
| Encodes per match per view tick | exactly 1 public view, whatever the spectator count |
| 250 matches + 2 000 spectators | 0 late ticks; server CPU ≤ baseline + 15 % |
| Lobby snapshot build | ≤ 1 ms at 5 000 sessions (measured) |
| `/api/live` body | ≤ 16 KB |
| Lobby polling | 1 request per 5 s per visible lobby, 0 when hidden |
| Lobby FPS | unchanged within noise with the panel visible |

## Acceptance matrix

- [ ] An anonymous browser opens the site and, without clicking anything,
      sees `N online · M in battle` and the Live battles panel (or its
      empty state).
- [ ] With a casual bot match running, the panel lists it with arena,
      names, HP and time; Watch plays it in real time in the right arena
      and ends with the result overlay.
- [ ] A logged-in player can do the same; both players see `1 watching`.
- [ ] `?watch=<id>` opens the same match directly and works after a reload.
- [ ] Spectators can never act; private data never reaches them (tests).
- [ ] Rooms, challenges, rematches and practice are never listed or watchable.
- [ ] Caps, `NotIdle`, `RateLimited` and `Full` behave as specified.
- [ ] A dead or slow spectator never affects either seat.
- [ ] Budgets met with recorded numbers.
- [ ] `/api/live` and `/api/online` leak no ids, IPs or tokens.
- [ ] All texts in en and pt-BR; desktop look checked at both sizes.
- [ ] All verification commands pass; native and wasm build.

## Reviewer checklist

- No new state in the simulation; `PublicView` is a pure function of `MatchState`.
- Seat authority untouched: nothing from a spectator reaches `apply_intention`.
- The listing is decided at pairing (`Listing`), never inferred from the mode.
- The lobby inbox never blocks: progress is sent with `try_send`.
- Contract tests appended, never rewritten; protocol bump in the same commit as the decision.
- No SQL on the API path; no new dependency (`bytes` comes through `axum`; confirm, or add a one-line reason).

## Deploy (reviewer, after owner approval of the look)

Follow `.local/tbg-temp/README.md` in the VPS repository with a new
`<name>-<date>/` folder made from the latest one (`hud-20261030/`): backup
with 0 matches running, image tag `prev-vN`, verify health, served WASM,
DB unchanged (no schema change in P47), other pods and nginx unchanged,
`nginx -t`, teste.abastevo 200, a bot match over wss, plus `/api/live` and
`/api/online` within limits, a `tbg-bots --spectators 20` run and a real
browser watching a live bot match. Log the entry with the rollback.

## Phase report

Filled by the executor on 2026-10-30 on `phase/p47-live-lobby`. Protocol
v17, D-59, GAME-DESIGN §13 (BR-LV01–LV09, Proposed). Owner visual sign-off
is still open; nothing is merged, pushed to `main`, or deployed by this
report.

### Commits (oldest first, one per logical step or step pair)

| Commit | Scope |
|--------|-------|
| `7ee2834` | step 1: `PublicView`, event visibility, contract test |
| `47813d8` | step 2: protocol v17 messages, contract bytes, GAME-DESIGN §13, D-59 |
| `d75559a` | steps 3–4: lobby registry, snapshot, progress, fan-out, watch lifecycle |
| `0fc762d` | step 5: `/api/live`, `/api/online`, integration tests |
| `f37eae4` | step 6: `/metrics` gauges, `tbg-bots --spectators N`, capacity script |
| `736775a` | style: rustfmt of the server files |
| `f9a6139` | steps 7–8: client watch transport and spectator render mode |
| `f422e04` | step 9: lobby chip, Live battles panel, See all, JS tests |
| `9dfbfb2` | fix: clippy findings in the spectator client |
| `29558c1` | docs: ARCHITECTURE §5/§11 and ROADMAP |

### Commands and real output

Run on the worktree with `CARGO_TARGET_DIR=$PWD/target CARGO_BUILD_JOBS=6`
under `nice -n 10`. Final tree (after the last client change):

- `cargo fmt --all -- --check`: exit 0.
- `cargo clippy --workspace --all-targets -- -D warnings`: exit 0.
- `cargo test --workspace --exclude tbg-balance`: exit 0; 335 passed, 0 failed (all test binaries, lib, integration and doc tests).
- `cargo check -p tbg-client -p tbg-core --target wasm32-unknown-unknown`: exit 0.
- `(cd crates/tbg-client && node --test tests/*.test.js)`: exit 0; all tests pass (live tests included).
- `(cd crates/tbg-client && trunk build --release)`: exit 0 (final client build, `dist/` is gitignored).
- Balance lab (`tbg-balance`): not run, as the owner asked.


### Capacity (250 matches, release, 40 s window, same host)

Source: `scripts/capacity.sh`, output in `docs/media/p47/`. Baseline is the
pre-P47 commit `13bb789` (run in a detached worktree).

| Run | Server cores | RSS MB | KB/s out | Views/s | Late ticks | RTT p95 ms | Lobby inbox max | Connections |
|-----|--------------|--------|----------|---------|------------|------------|-----------------|-------------|
| Baseline (`13bb789`) | 0.22 | 18.3 | 1171.3 | 5094 | 0 | 2 | 2 | 500 |
| P47, 0 spectators | 0.24 | 19.5 | 1171.7 | 5107 | 0 | 3 | 3 | 500 |
| P47, 500 spectators | 0.33 | 30.2 | 2269.7 | 5093 | 0 | 2 | 29 | 1000 |
| P47, 2000 spectators | 0.57 | 55.3 | 5654.8 | 5094 | 0 | 2 | 74 | 2500 |

Budget check (`Budget` table in the spec):

- Server CPU ≤ baseline +15 % (0.253 cores): **met at 0 spectators** (+9 %),
  **not met at 500 (+50 %) or 2000 (+159 %)**. This is the main open item;
  the cause is not profiled. Suspects: one socket write per spectator per
  view tick, and the spectator frames themselves. The reviewer should decide
  whether to lower the spectator view rate or batch the writes before
  approving.
- Spectator download ≤ 6 KB/s: about 2.2 KB/s per spectator at 2000
  ((5654.8 − 1171.7) / 2000 KB/s). Met.
- 0 late ticks at 250 matches + 2000 spectators: met.
- Encodes per match per view tick = 1: met by construction (one
  `public_view` and one `encode` per tick in `send_spectator_views`); not
  separately instrumented.
- Lobby snapshot ≤ 1 ms at 5000 sessions and `/api/live` ≤ 16 KB: **not
  measured**. A 50-row snapshot is estimated at about 12 KB, not checked.

Spectator load-tool smoke (`tbg-bots --matches 4 --spectators 3`, release,
`docs/media/p47/tbg-bots-spectators-3.txt`): `watch starts 64 | refused 3 |
views 12217 | ended 3 | errors 0`; all 8 bots completed. The capacity script
does not print the spectator summary (its bot log is removed), so the
capacity table has no spectator-side numbers.

### Desktop browser observations (Playwright, Chrome headless, release build)

Server `TBG_STATIC_DIR` = the trunk build, bot matches running. Screenshots
in `docs/media/p47/` (JPG). Language: EN only.

- Anonymous visitor, no clicks: header chip reads `6 online · 6 in battle`
  at 1920×1080 and 1280×800. Live battles panel lists 3 matches with names,
  Guest, Casual, arena, time, `0 watching`, HP bars and **Watch**, above the
  news panel, and does not cover the mode panel or the arena centre (see
  `en-lobby-1920.jpg`, `en-lobby-1280.jpg`).
- Watch (`?server=…&watch=<featured id>`): the watched match loads in the
  right arena (`storm`, `ruins`, `forest` seen in the console as
  `watching arena: …`), the names and HP show, the seat controls are hidden
  (no Shop, Energy, Fortify or War Council on the desktop HUD), a
  `LIVE · 1 watching` badge is at the bottom, and **Leave** is top-right
  (`en-watch-1920.jpg`, `en-watch-1280.jpg`).
- Not verified in the browser: two players seeing `1 watching` (covered by
  the socket-level test `players_see_the_watcher_count_and_the_match_arena`);
  reload with `?watch=` after a watch (the sessionStorage path is unit
  tested only); the result overlay and "Back to lobby" at the end of a
  watched match (socket-level `SpectateEnd` test only); Portuguese (the
  catalog parity is checked by the build, but no PT page was looked at);
  rooms and challenges in the browser (socket-level and lobby unit tests).
- Console: one `Failed to load resource: 404` on every page. Not
  investigated; it predates P47's code paths as far as I can tell.

### Deviations (and why)

- **Protocol v17, not v16.** v16 belongs to the Cargo Crab branch (not
  merged). The spec's "next free version" is 17 in this tree.
- **Decision D-59.** D-56 is the Cargo Crab branch's; D-57 was reserved by
  parallel work per the brief; D-58 went to the C7 balance fix on `main`
  while this branch was in review, so the entry was renumbered.
- **Steps 3–4 and steps 7–8 each landed as one commit** (the step-3 and
  step-4 changes were interleaved in the same files). Steps 1, 2, 5, 6 and
  9 are separate.
- **The spectator chrome lives in the HTML battle HUD, not in Bevy.** On
  desktop, the visible battle controls are the `#tbg-battle` DOM overlay
  (`battle_hud.rs` → `index.html`, `assets/battle/hud.css`). The first
  attempt hid Bevy nodes, which were not the visible controls. The Bevy
  spectator overlay was removed.
- **Phone fallback HUD is not spectator-aware.** Desktop only (the spec
  says so); a watcher on a narrow window would still see the seat HUD.
- **`LobbyCmd::LiveProgress` carries a nested `LiveProgress`** instead of
  flattened fields; same content.
- **Chip numbers.** `N online` = sessions with a connection (O-6); `M in
  battle` = seats of running matches (the spec's `in_battle`).
- **Ranking "Live now" section (optional) not done.**
- **Spectator frames are a new `ConnEvent::Frame`** (pre-encoded bytes). The
  spec named it; no deviation, listed for review.

### Decisions taken without the owner

- Listed matches are both casual quick matches and ranked (O-5).
- Caps: 200 spectators per match, 2000 in total (O-3). Env:
  `TBG_SPECTATORS_PER_MATCH`, `TBG_MAX_SPECTATORS`.
- Featured battle holds 30 s, guests count as 1000 (BR-LV07).
- Spectator refusals return `SpectateUnavailable` and never disconnect.

### Limitations

- CPU budget not met at 500 and 2000 spectators (see the table).
- Snapshot build time and `/api/live` size are not measured.
- Browser checks are partial (see the list above).
- Open, not root-caused: in a debug smoke run with 20 spectators, two
  connections closed with `bad_message`. The current release run with 3
  spectators showed `errors 0`, but the 20-spectator case was never
  reproduced or explained.
- `tbg-bots` process: the spectator counts in the capacity script are not
  printed.
- Nothing deployed. The VPS deploy follows owner sign-off of the look.
