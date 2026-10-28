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

Executed 2026-10-28 on the dev PC. Steps 1–2 by the executor (522e117,
c581c96); the executor stopped there without a report. Steps 3–6, the
acceptance runs, and this report were done by the reviewer on the same
branch (the owner asked the reviewer to finish the phase).

### Verification commands (all pass, final tip)

```text
$ cargo fmt --all -- --check                     # clean
$ cargo clippy --workspace --all-targets -- -D warnings   # clean
$ cargo clippy -p tbg-client --target wasm32-unknown-unknown -- -D warnings  # clean
$ cargo test --workspace   # core 6 account + 5 rating + 15 rules; server 2 unit,
                           # 10 account, 4 leaderboard, 12 online, 5 ranked
$ cargo check -p tbg-client -p tbg-core --target wasm32-unknown-unknown   # ok
$ trunk build --release    # success; dist/ranking/ = 14 808 bytes (< 40 KB)
```

Tests: `store::leaderboard::tests::order_and_tiebreaks` (rating, wins and
`name_lower` tiebreaks); `tests/leaderboard.rs` — both players ranked after
one ranked match with the public profile (no id/hash fields), `limit=1000`
→ 100 rows over 102 bot accounts, bad shape and unknown name → identical
404, requests 1–120 → 200 and the 121st → 429, and the in-game
`Leaderboard` reply (same order as the API, `you = Some(1)`, a second
request within 1 s ignored).

### Load (`oha -z 30s -c 200 http://127.0.0.1:8080/api/leaderboard`)

Release server, `TBG_API_PER_IP_MIN=100000000`, 180 ranked accounts
(default page = 50 rows, 5.17 KiB JSON), oha on the same 16-core machine:

```text
Success rate 100.00 %   Requests/sec 4939.6   148 024 × [200]
p50 35.1 ms   p99 125.7 ms   slowest 210.5 ms
server CPU 49.85 s over 30 s = 1.66 cores → ≈ 2 970 requests/s per core
```

Snapshot rebuild at 50 180 synthetic accounts: the ordered `SELECT` takes
30 ms and runs as `SCAN accounts USING INDEX accounts_by_rating`.

### Browser (in-app browser, local release server, `TBG_TICK_MS=2`)

- Real data: `tbg-bots --ranked --matches 50` (100/100 completed) plus 40
  more (80/80), then one human ranked match against a bot.
- `/ranking/` and `player.html` at 1280×800, 375×812 and 320×640: table,
  pager, search, faction bars, last-10 table; no horizontal page scroll
  at 320 px (Games column hidden under 420 px; tables scroll inside
  their box).
- Lighthouse 12, mobile profile: board page Performance 98 /
  Accessibility 100 / Best Practices 100 (FCP 0.9 s, LCP 1.1 s, CLS
  0.085, TBT 0 ms); player page 98 / 100 / 100 (FCP 0.8 s, LCP 1.1 s,
  CLS 0.088).
- XSS: `player.html?name=<img src=x onerror=alert(1)>` and
  `?name=Bob_script` render as plain text in the status line; no element
  is created. No `innerHTML`, `eval`, or inline handlers in the pages.
- In-game panel: guest opens Ranking → "180 ranked players", rows #1–#20
  (two columns of 10 at 1280×800, one column of 20 in portrait); Next →
  #21–#40 with Prev enabled. Registered `ReviewerOne`, played ranked vs
  one bot → result `Rating 1000 -> 980 (-20)`. Server log has no key text
  and no IP.
- Not seen on screen: the highlighted own row (the pane was hidden and
  stopped taking clicks); covered by the `you == Some(1)` integration
  assertion and the view code. Real phones: owner-pending (P14 matrix).

### Deviations

- Leaderboard request pacing lives per connection in `ws/client_msg.rs`
  (extras ignored before reaching the lobby). The client re-sends a wanted
  page every 1.1 s until its reply arrives, so a fast Prev/Next is never
  lost; Prev/Next are disabled while a page loads.
- `updated_at` is the newest ranked match end time (data freshness), not
  the rebuild time. A result reaches the snapshot within ~10–20 s (10 s
  throttle, plus up to one 10 s idle wake-up).
- The in-game panel shows two 10-row columns where the screen is wide
  but short (a 20-row column does not fit 1280×800 at UI scale 1.5), one
  column on portrait phones. The menu hides under modal panels (ranking,
  save your key) because taps hit-test the first visible button.
- The P12 menu already overflowed 1280×800; paired buttons now share rows.
- Web pages: 50 rows per page; a `default-src 'self'` CSP meta tag; the
  spec asked `no-cache` only for HTML, but the ranking CSS/JS are not
  content-hashed, so all `/ranking/*` files are `no-cache` (ETag) and are
  excluded from the immutable `*.js` rule (it would have pinned
  `ranking.js` for a year).
- `window.open` runs a frame after the tap; browsers allow it as a
  user-activated popup (Chrome checked); strict popup blockers may still
  stop it.
- Lighthouse ran headless from the CLI (`npx lighthouse@12`) instead of
  Chrome devtools; same engine and mobile profile.

### Blocked items

None. Owner-pending: both pages and the panel on a real Android phone and
iPhone (P14 device matrix); Caddy routing on the VPS (Caddy is not
installed on the dev PC, so the template was not run here).
