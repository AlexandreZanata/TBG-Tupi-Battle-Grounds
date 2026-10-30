# P46 — Live ranked matches, spectating and online presence

**Depends on:** PR #10 merged (protocol v13, D-46 Stronghold kinds in
`MatchView.strongholds`). If P41 (i18n) is merged first, every new string
goes through its catalog. Do not run concurrently with any phase that
edits `lobby.rs`, `match_task/*`, `protocol.rs` or the desktop lobby
(`index.html`, `assets/lobby/*`).
**Branch:** `phase/p46-live-spectate`.
**Status:** planned (owner request 2026-10-29).
**Difficulty:** medium–high. One protocol bump, a public view in core, a
lobby registry with a snapshot, fan-out in the match task, two read-only
API routes, a spectator render mode and three lobby surfaces.

## Goal

Anyone, guest or logged in, opens the game and sees at once **how many
players are online**, a **featured live ranked match** ("Live now"), and a
**list of every live ranked match** they can **watch in real time** with
one click, plus a list of online accounts. Watching is listen-only and
shows only public information. Players keep exactly today's experience,
plus a small "N watching" badge.

## Read first (only these)

- `AGENTS.md`; `docs/ROADMAP.md` (P46 row); this file.
- `docs/GAME-DESIGN.md`: glossary, BR-S05 (private information), BR-M01–M07,
  BR-R01–R09, BR-AG09/12/14 (Augment visibility), BR-SK08.
- `docs/ARCHITECTURE.md` §5 (sessions, rates, limits, concurrency), §9,
  §10 (budgets), §11 (security, read-only API), §12.
- `docs/DECISIONS.md` D-06, D-09, D-39, D-41, D-46.
- Core: `src/view.rs`, `src/state.rs` (`Event`), `src/protocol.rs`,
  `tests/contract_protocol*.rs`.
- Server: `lobby.rs`, `lobby/{sessions,matchmaking,ranked,leaderboard}.rs`,
  `chan.rs`, `ws.rs`, `ws/*`, `match_task.rs`, `match_task/{command,views}.rs`,
  `api.rs`, `lib.rs`, `limits.rs`, `config.rs`, `metrics.rs`,
  `store/worker.rs` (the `watch` snapshot pattern), `botclient*`,
  `tests/online.rs`.
- Client: `src/net/*`, `src/sync/*`, `src/ui/{battle_hud,hud,hud_update,menu}.rs`,
  `assets/lobby/*`, `assets/battle/*`, `index.html`, `ranking/*`.

## Audit summary (verified on main 0154dd4 + PR #10; re-verify)

| # | Fact | Where |
|---|------|-------|
| A1 | The lobby keeps no registry of running matches: a session only holds `SessionState::InMatch{tx, seat}`; ranked pairing goes through `pair(tokens, mode, …)` | `lobby.rs`, `lobby/matchmaking.rs` |
| A2 | Views are built per seat (`view_for(seat)`) and sent as `ConnEvent::Send(ServerMsg)`; each connection encodes its own copy; a full outbound queue detaches the **seat** | `match_task/views.rs` |
| A3 | `MatchView` carries private data: `energy`, `regen_bp`, `offers`, `last_seq`, `augment_round.resolved` for `you` | `view.rs` |
| A4 | Read-only API reads an in-memory `watch` snapshot (leaderboard), never SQL per request; per-IP limit `TBG_API_PER_IP_MIN` (120/min); replies carry no ids, IPs or tokens | `api.rs`, `store/worker.rs`, ARCHITECTURE §11 |
| A5 | `/metrics` counts connections and running matches but no presence or spectators | `metrics.rs` |
| A6 | Every tab opens a listen-only probe link (P9), so connections ≠ players | ARCHITECTURE §5 |
| A7 | No spectator concept exists anywhere; "spectating" is listed under "Later" | `ROADMAP.md` |
| A8 | Desktop lobby already polls `/api/players/{name}` from `lobby.js`; nginx/Caddy already proxies `/api/*` (verified in production) | `assets/lobby/lobby.js` |

## Non-goals

No replays or VOD, no chat, no betting or predictions, no casual, room,
challenge or practice spectating, no spectator camera control beyond the
fixed arena camera, no delta views (still "Later"), no persistence (all of
P46 is memory only; no schema change), no rule or balance change.

## Owner decisions (defaults used unless the owner says otherwise)

| ID | Question | Default | Why |
|----|----------|---------|-----|
| O-1 | Broadcast delay for spectators | **0 s** (real time, as requested) | The spectator view is public-only (BR-LV03), so a delay would protect nothing private; a delay is a single constant if ever wanted (ring buffer of encoded frames) |
| O-2 | List online **account names** | **yes**, name + rating + status; guests only counted | Names and ratings are already public on the leaderboard |
| O-3 | Caps | 200 spectators per match, 2 000 in total (`TBG_MAX_SPECTATORS`) | ≈ 2.6 Mbit/s at the measured 1.3 KB/s per view stream |
| O-4 | Players see the spectator count | **yes**, count only, never names | Social proof without exposure |
| O-5 | Which matches are watchable | **ranked only** | Owner request; private and casual stay private |

## Contracts (freeze before coding)

### Rules (append to GAME-DESIGN, new section "Live and spectating", Proposed)

| ID | Rule |
|----|------|
| BR-LV01 | Only live **ranked** matches are listed and watchable; casual, private rooms, challenges and practice never |
| BR-LV02 | Any session (guest or account) may watch while **idle**; watching is listen-only: a spectator has no seat, and any `Intent`/`Surrender` from it is refused like today's `NotInMatch` |
| BR-LV03 | Spectators receive only `PublicView`: no `Energy`, regen, offers, `last_seq`, RNG, or a pending Augment pick (BR-S05, BR-AG14). Augment options of an open round are public (both seats see the same, BR-AG12); each seat's *resolved* flag is public, its choice only after resolution |
| BR-LV04 | Spectator delay is 0 s (O-1) |
| BR-LV05 | Both players see the number of spectators, never who |
| BR-LV06 | Presence: online = sessions with an attached connection, accounts counted once; online accounts are listed with name, rating and status (`idle`, `queued`, `playing`, `watching`); guests are counted, never listed; no ids, IPs, tokens or `LiveId` of a non-ranked match ever leave the server |
| BR-LV07 | Featured match: the live ranked match with the highest mean rating; ties by longest running, then lowest `LiveId`. It stays featured for at least 30 s unless it ends |
| BR-LV08 | A live match has a random, non-sequential 64-bit `LiveId` (OS random), valid only while it is live; it is shareable as `?watch=<16 hex>` |
| BR-LV09 | Caps (O-3); full or unknown answers `SpectateUnavailable`, never a disconnect; a slow spectator is dropped without ever affecting a seat |

### Wire: `PROTOCOL_VERSION = 14` (append-only)

```rust
pub type LiveId = u64;
// ClientMsg, appended
Spectate { id: LiveId },
StopSpectating,
// ServerMsg, appended
SpectateStart { id: LiveId, players: [LivePlayer; 2], arcanas: [ArcanaPreset; 2] },
SpectatorView(Box<PublicView>),
SpectateUnavailable { reason: SpectateRefusal },
SpectateEnd { outcome: Option<Outcome> },
pub struct LivePlayer { pub name: String, pub rating: u16 }
pub enum SpectateRefusal { NotFound, NotWatchable, Full, NotIdle, RateLimited }
```

`MatchView` gains `spectators: u16` (BR-LV05).

### Core: `PublicView` (`tbg-core::view`)

`MatchState::public_view(events: &[Event], spectators: u16) -> PublicView` with
`tick, clock, elapsed, phase, phase_ticks_left, stronghold_hp, strongholds,
shields, units, synergy, augments, round: Option<PublicRound>, events,
outcome, spectators`, where `PublicRound { round, options, costs,
ticks_left, resolved: [bool; 2] }`. Step 1 classifies every `Event` variant
as public or private, with a test. Today all are combat or public events,
but the test pins the rule for future variants.

### Server state (memory only)

- Lobby `live: HashMap<LiveId, LiveMatch>` (lobby, not simulation, so
  `HashMap` is allowed), filled in `pair` for `MatchMode::Ranked` only,
  removed on `MatchEnded`. `LiveMatch { tx, players: [LivePlayer; 2],
  kinds, started: Instant, progress: LiveProgress, spectators: u16 }`.
- `SessionState::Watching { id: LiveId }`, entered only from `Idle`.
- The match task sends `LobbyCmd::LiveProgress { id, hp_bp: [u16; 2],
  tiers: [u8; 2], elapsed_s: u32, spectators: u16 }` every 20 ticks with
  `try_send` (dropped when the lobby inbox is full: progress is cosmetic).
- The lobby publishes `watch::Sender<Arc<LiveSnapshot>>` at most once per
  second, only when something changed: the online counts, the sorted live
  list (≤ 50 rows), the featured id, and the online account rows. This is
  the leaderboard pattern: the API never touches lobby state.

### Fan-out (match task)

- `MatchCmd::Watch { conn: ConnHandle }` / `MatchCmd::Unwatch { conn_id }`;
  `spectators: Vec<ConnHandle>` (cap per match).
- On each view tick with spectators: build `public_view` **once**, encode
  **once**, send `ConnEvent::Frame(Bytes)` (a new pre-encoded variant) to
  each spectator with `try_send`. `Full` or `Closed` removes the spectator
  and tells the lobby; it never detaches a seat.
- New spectator: `SpectateStart` plus an immediate full `SpectatorView`.
  Match end: `SpectateEnd{outcome}` to every spectator, then the lobby
  returns their sessions to `Idle`.

### Lifecycle

`Idle` → `Spectate{id}`, checked in this order:

1. rate limit, 1 per second per session (else `RateLimited`);
2. the session is idle (else `NotIdle`);
3. the id is live (else `NotFound`);
4. the match is ranked (else `NotWatchable`, unreachable by construction but tested);
5. the caps (else `Full`).

Then `Watching` and `MatchCmd::Watch`. `StopSpectating`, a disconnect or the
match end return the session to `Idle`. A reload does not auto-resume: the
client re-sends `Spectate` from `?watch=` or from the last id kept in
`sessionStorage` for that tab.

### HTTP (read-only, existing per-IP limit, `Cache-Control` as given)

- `GET /api/live` → `{"online":{"players":N,"accounts":N,"playing":N,"watching":N},
  "featured":"<16 hex>"|null, "matches":[{"id":"<16 hex>","players":[{"name","rating",
  "kind","tier"},{…}],"elapsed_s":N,"hp_bp":[a,b],"spectators":N}]}`. At most
  50 rows, highest mean rating first; `max-age=2`; ≤ 16 KB.
- `GET /api/online?offset&limit` → `{"total":N,"rows":[{"name","rating","status"}]}`,
  rating descending, `limit` ≤ 100 (existing `paging`), `max-age=5`.

### Config

`TBG_MAX_SPECTATORS` (default 2 000) and `TBG_SPECTATORS_PER_MATCH`
(default 200), read in `Config::from_env` like the existing limits.

### Client behavior

- **Desktop lobby (HTML):**
  - `● N online` in the header;
  - a **Live now** featured card on the home screen with both names, ratings, Stronghold kinds and tiers, HP bars, the match time, `N watching` and **Watch**;
  - a **Live** tab listing every row with **Watch** and **Copy watch link**;
  - an **Online** panel (lazy `/api/online`).
  `/api/live` is polled every 5 s only while the lobby is visible (`visibilitychange` pauses it). The DOM is built with `textContent`/`createElement` only, under the existing CSP. New `data-*` names must not collide with `lobby.js` text projection (`tests/lobby.test.js`).
- **Phone (Bevy menu):** a **Watch live** entry showing the online count, the featured match and up to 10 rows.
- **Spectator mode:**
  - same arena and timeline as a player, with a neutral camera: seat A on the left;
  - the top bar shows both names, ratings, kinds/tiers and HP;
  - a `LIVE · N watching` badge;
  - during an Augment pause, "Players are choosing" with the public round;
  - no Shop, no Fortify, no War Council input; a **Leave** button;
  - at the end, the result overlay and **Back to Live**.
  The client maps `PublicView` into its render state internally; it never invents private values on the wire.
- **Players:** the HUD shows `N watching` when > 0.
- **Ranking page:** a **Live now** section at `/ranking/` with watch links.

## Steps (in order; check and commit each)

### 0. Baseline
`scripts/capacity.sh` at 250 matches (CPU, RSS, bytes/s, late ticks) and
a `tbg-bots` run. Save the results to `docs/media/p46/baseline.txt`.

### 1. Core `PublicView` and the event classification
Add the view, `public_view`, and `MatchView.spectators`. Contract tests:
- every private field is absent;
- a pending pick never appears before resolution;
- every `Event` variant is classified;
- determinism: same state gives the same public view.
Check: `cargo test -p tbg-core`.

### 2. Protocol v14
Append the messages. Bump `contract_protocol` (version pin, new variants'
bytes) in the same commit as the GAME-DESIGN BR-LV rows and a new decision
(the next free `D-` number at execution time; call it D-LV below).

### 3. Live registry, progress and snapshot (server)
Registry in `pair` (ranked only), `LiveProgress`, the throttled
`LiveSnapshot` and the presence counts (accounts deduplicated, probe links
excluded by counting only sessions with a connection).
Unit tests:
- ranked is listed, casual/room/challenge are not;
- removal on end;
- featured ordering and 30 s stickiness;
- counts after a disconnect and a takeover.

### 4. Spectating (server)
Lifecycle, refusals, fan-out with `ConnEvent::Frame`, caps, cleanup on
end, disconnect and stop. Online tests (`tests/online.rs`):
- an anonymous guest watches a ranked bot match to the end and receives `SpectateEnd`;
- its `Intent` and `Surrender` change nothing;
- a room match is `NotFound` (never listed);
- a queued session gets `NotIdle`;
- a full match gets `Full`;
- a spectator that never reads is dropped while both seats finish with 0 late ticks and no detach;
- players see `spectators` go up and down.

### 5. HTTP API
`/api/live` and `/api/online` read the snapshot only. Tests:
- shapes and limits;
- no tokens, IPs or account ids (assert on the raw JSON);
- 429 over the per-IP limit;
- non-ranked matches are absent.

### 6. Metrics and load tool
- `/metrics` gains `spectators`, `live_matches` and `online_players`.
- `tbg-bots --spectators N` opens N anonymous sockets that read `/api/live` and watch random listed ids.

Measure at 250 matches with 0, 500 and 2 000 spectators and record the
results in `docs/media/p46/capacity.txt`.

### 7. Client transport
- `Spectate` / `StopSpectating`, `?watch=<hex>` and `sessionStorage` resume;
- handling for every new `ServerMsg`;
- `Leave` returns to the lobby.

### 8. Spectator render mode
The neutral camera, the HUD variant and the end overlay. Every player
input system is disabled while spectating; add a test that no intention is
sent.

### 9. Desktop lobby, phone menu and ranking page
The online counter, the featured card, the Live tab, the Online panel, the
phone entry, and the ranking-page section. Run the JS tests, plus a
collision test for the new `data-*` names.

### 10. Canon and docs
- GAME-DESIGN BR-LV01–LV09 (Proposed);
- DECISIONS D-LV;
- ARCHITECTURE §5: messages, lifecycle, fan-out, limits; §10: new budget rows; §11: the new API routes;
- ROADMAP status.

### 11. Verification, deploy and phase report
- the full AGENTS.md matrix, `trunk build --release` and browser checks;
- the deploy rules below;
- a filled phase report.

## Budgets

| Metric | Target |
|--------|--------|
| Spectator download | ≤ 6 KB/s (same as a player) |
| Encodes per match per view tick | exactly 1 public view, whatever the spectator count |
| 250 matches + 2 000 spectators | 0 late ticks; server CPU ≤ baseline + 15 % |
| Lobby snapshot build | ≤ 1 ms at 5 000 sessions (measured) |
| `/api/live` body | ≤ 16 KB |
| Lobby polling | 1 request per 5 s per visible lobby, 0 when hidden |

## Acceptance matrix

- [ ] An anonymous browser opens the site, sees `N online`, the featured live match and the list, and clicks Watch. The match plays in real time and ends with the result overlay.
- [ ] A logged-in player can do the same; both players see `1 watching`.
- [ ] `?watch=<id>` opens the same match directly and works after a reload.
- [ ] Spectators can never act; private data never reaches them (tests).
- [ ] Casual, room, challenge and practice matches are never listed or watchable.
- [ ] Caps, `NotIdle`, `RateLimited` and `Full` behave as specified.
- [ ] A dead or slow spectator never affects either seat.
- [ ] Budgets met with recorded numbers.
- [ ] `/api/live` and `/api/online` leak no ids, IPs or tokens.
- [ ] All verification commands pass; native and wasm build.
- [ ] Deployed and verified on the VPS (rules below).

## Reviewer checklist

- No new state in the simulation; `PublicView` is a pure function of `MatchState`.
- Seat authority untouched: nothing from a spectator reaches `apply_intention`.
- The lobby inbox never blocks: progress is sent with `try_send`.
- Contract tests appended, never rewritten; protocol bump in the same commit as D-LV.
- No SQL on the API path; no new dependency (`bytes` comes through `axum`; confirm, or add a one-line reason).

## Deploy rules (VPS, mandatory)

- Repository: `/home/iiii/I9-PROJETOS/VPS-HOSTINGER-I9-32GB`, host `sorrimobi-vps`.
- Touch only namespace `tbg-temp`, `/opt/tbg-temp` and `.local/tbg-temp/`. Never touch other namespaces, images, nginx sites or certificates.
- Keep `.local/tbg-temp/k8s/01-server.yaml` as it is. Add the two new env vars only if the owner changes the defaults.
- Before the deploy:
  - DB backup through the sqlite backup API (`mode=ro`), chmod 600, with integrity and counts printed;
  - web backup;
  - image tag `prev-vN`;
  - pods snapshot;
  - nginx md5.
- After the deploy, verify:
  - health, and the served wasm hash equals the local build;
  - `/api/live` and `/api/online` answer within limits;
  - the DB is unchanged (no schema change in P46);
  - other namespaces are unchanged;
  - nginx is unchanged and `nginx -t` passes;
  - teste.abastevo answers 200;
  - a bot match runs over wss, and a `tbg-bots --spectators 20` run against production completes;
  - a real browser watches a live ranked bot match.
- Log the deploy in `.local/tbg-temp/README.md` with the rollback. Never copy production player data to the PC.

## Phase report

_Executor fills in: baseline and after numbers, every command with its
real output, browser observations (desktop, phone), capacity table with
spectators, deviations, owner decisions taken._
