# P13 — Global ranking and player statistics

**Depends on:** P12 reviewed and merged.
**Branch:** `phase/p13-global-ranking`.
**Difficulty:** medium.

## Goal

Everyone can see the global ranking and each player's statistics — inside
the game and on a light public web page that loads fast on any phone and
can be shared as a link. Reading the ranking must cost the server almost
nothing, no matter how many people refresh it.

## Deliverable

- Read-only JSON API: `GET /api/leaderboard`, `GET /api/players/{name}`,
  served from an in-memory snapshot (leaderboard) and indexed queries
  (player), with per-IP rate limits.
- Protocol v4: `Leaderboard` request/reply for the in-game panel.
- In-game **Ranking** panel (top players, your position, paging).
- Static web pages `/ranking/` and `/ranking/player.html?name=…`
  (plain HTML + CSS + vanilla JS, no framework, < 40 KB total).
- Deploy kit routes `/api/*` and `/ranking/*`.

## Read first (only these)

- `AGENTS.md`
- `docs/GAME-DESIGN.md` BR-R07 (ordering) and BR-P05 (public profile)
- `docs/DECISIONS.md` D-21 (ranking page)
- `docs/ARCHITECTURE.md` §5, §8, §9, §11
- Server: `src/lib.rs`, `src/store/*`, `src/limits.rs`, `src/lobby.rs`
- Client: `src/ui/menu.rs`, `src/ui/mod.rs`, `index.html`
- `deploy/Caddyfile.template`, `deploy/README.md`

## Do NOT

- Expose account ids, key hashes, IPs, or creation timestamps finer than
  the day.
- Use `innerHTML` (or any HTML-string building) with server data in the
  web pages — only `textContent` and DOM element creation.
- Add a frontend framework, bundler, or npm.

## Dependencies you may add

None (`serde_json`, `axum`, `tokio::sync::watch` are already available).

## API (implement exactly)

`GET /api/leaderboard?offset=0&limit=50` — `limit` 1..=100 (default 50),
`offset` 0..=100 000 (default 0); out-of-range values are clamped.
```json
{ "total": 1234, "updated_at": "2026-11-02T18:00:00Z",
  "rows": [ { "rank": 1, "name": "Aldric", "rating": 1432, "games": 61,
              "wins": 40, "losses": 20, "draws": 1, "win_rate": 65.6 } ] }
```
Only accounts with `ranked_games >= 1` (BR-R07); order `rating DESC, wins
DESC, name_lower ASC`; `rank` is the 1-based position in that order.

`GET /api/players/{name}` — `name` must match the PlayerName format, else
404 without a query. Unknown → 404.
```json
{ "name": "Aldric", "rating": 1432, "peak_rating": 1460, "rank": 1,
  "games": 61, "wins": 40, "losses": 20, "draws": 1, "win_rate": 65.6,
  "joined": "2026-10-30",
  "factions": { "Humans": 41.2, "Undead": 20.0, "Machines": 18.8, "Plants": 20.0 },
  "favorite_faction": "Humans",
  "recent": [ { "date": "2026-11-02", "opponent": "Brienne", "result": "W",
                "delta": 12, "reason": "StrongholdDestroyed", "duration_s": 171 } ] }
```
`rank` is `null` with 0 ranked games. `factions` are purchase shares (%)
over all ranked matches; `recent` holds the last 10 ranked matches, newest
first.

Headers: `Content-Type: application/json`, `Cache-Control: public,
max-age=10`. Errors: 404 `{"error":"not_found"}`, 429
`{"error":"rate_limited"}`.

## Steps

### 1. Leaderboard snapshot
- The store keeps an ordered `Vec<LeaderRow>` of every ranked account and
  publishes `Arc<Snapshot { rows, total, updated_at }>` through a
  `tokio::sync::watch` channel. It rebuilds at startup and at most every
  10 s, only when a ranked result was committed since the last build.
- Snapshot readers (HTTP and WS) only clone the `Arc` — no SQL per request.
- **Check:** unit test on a temp DB: 5 accounts with fixed stats → exact
  order and ranks, including a `wins` tiebreak and a `name_lower` tiebreak.

### 2. HTTP API (`src/api.rs`, ≤ 200 lines; split if needed)
- Routes above, added in `lib.rs`.
- `/api/players/{name}`: validate format, then `StoreCmd::PlayerStats` (the
  store runs at most 3 indexed queries: the account row, faction sums over
  `ranked_matches`, the last 10 rows joined with the opponent name). Rank
  comes from the snapshot.
- Per-IP limit for `/api/*`: env `TBG_API_PER_IP_MIN`, default 120 requests
  per minute (reuse `limits.rs`).
- **Check:** integration tests: after one ranked match both players appear
  with correct ranks; `limit=1000` returns 100 rows max; bad name → 404
  without a store call; 121st request in a minute → 429.

### 3. Protocol v4 and in-game panel
- Append `ClientMsg::Leaderboard { offset: u16 }` and
  `ServerMsg::Leaderboard { total: u32, offset: u16, rows: Vec<LeaderRow>,
  you: Option<u32> }` with `pub struct LeaderRow { rank: u32, name: String,
  rating: u16, wins: u32, losses: u32, draws: u32 }` (20 rows per page;
  `you` = your rank if logged in and ranked). One request per second per
  connection; extra requests are ignored. `PROTOCOL_VERSION = 4`.
- Client `src/ui/ranking.rs`: **Ranking** button in the menu → panel with
  rank, name, rating, W-L-D; your row highlighted; Prev/Next; **Open full
  ranking** opens `/ranking/` in a new tab (`window.open`).

### 4. Web pages
Folder `crates/tbg-client/ranking/` copied by Trunk
(`<link data-trunk rel="copy-dir" href="ranking">` in `index.html`):
- `index.html` — header "Tupi Battle Grounds — Global Ranking", a name
  search box (`<form>` → `player.html?name=`), the table (rank, name,
  rating, games, W/L/D, win rate), Prev/Next, "Play now" link to `/`.
- `player.html` — the player's card (name, rank, rating, peak, record, win
  rate, joined), faction share bars, last-10 table; "Back to ranking".
- `ranking.css` — all colors, fonts, radii, and spacing as CSS custom
  properties on `:root` (P20 restyles by changing these tokens); mobile-first
  layout, readable at 320 px, `prefers-color-scheme` aware, focus outlines
  visible, contrast ≥ 4.5:1.
- `ranking.js` — `fetch` + DOM building with `textContent`; loading and
  error states; `name` read from `URLSearchParams` and passed through
  `encodeURIComponent`.
- **Done when** both pages work on desktop and a phone against a local
  server with real ranked data, and Lighthouse (Chrome devtools) gives
  Accessibility ≥ 95 and Performance ≥ 95 on mobile.

### 5. Deploy kit
`deploy/Caddyfile.template`: `handle /api/* { reverse_proxy
127.0.0.1:8080 }` before the static handler; `/ranking/*.html` gets
`Cache-Control: no-cache`. Update `deploy/README.md` and ARCHITECTURE §9.

### 6. Docs
ARCHITECTURE §5 (WS messages), §8 (snapshot), §11 (API limits, no-HTML
rule). README: link to `/ranking/`.

## Acceptance checklist

- [ ] All verification commands from `AGENTS.md` pass; `trunk build --release` passes
- [ ] API and snapshot tests pass
- [ ] `oha -z 30s -c 200 http://127.0.0.1:8080/api/leaderboard` (tool:
      `cargo install --locked oha`; server started with
      `TBG_API_PER_IP_MIN=100000000`) — record requests/s and server CPU;
      expect thousands of requests/s per core
- [ ] Pages pass the Lighthouse targets; XSS check: a player named
      `Bob_script` and an injected `?name=<img>` render as plain text/404
- [ ] In-game panel shows the same order as the web page

## Risks and what to do

| Problem | Action |
|---------|--------|
| Snapshot rebuild gets slow with many accounts | It is one indexed `SELECT` ordered by the index; measure at 50 000 synthetic rows and record |
| The page is scraped heavily | 10 s cache headers + per-IP limits; a CDN proxy can cache `/api/leaderboard` |

## Reviewer checklist

- No SQL on the leaderboard path per request.
- No private field in any JSON or WS reply.
- Web pages contain no `innerHTML`, `eval`, or inline event handlers.

## Phase report

_Executor fills this in: commands run with real output, load numbers,
Lighthouse scores, screenshots, deviations, blocked items._
