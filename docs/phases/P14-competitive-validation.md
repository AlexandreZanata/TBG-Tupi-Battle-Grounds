# P14 — Competitive validation in the real environment (gate)

**Depends on:** P9–P13 reviewed and merged; the owner provides a server
(VPS + domain) and SSH access to the reviewer.
**Branch:** `phase/p14-competitive-validation`.
**Difficulty:** medium. Mostly running, measuring, and fixing what breaks.
**Who:** the reviewer agent runs it (it touches a real server); the owner
does the human checks.

## Goal

Prove on a real server, over the public internet, that the competitive
game is fast, fair, robust to drops, and safe with player data — before
any visual work starts. **The visual suite (P15+) starts only when this
phase is signed off.**

## Deliverable

- Deployed build with accounts, ranked, and the ranking pages.
- A filled results table (below) with real numbers.
- Backup **and restore** drill of the accounts database.
- Security review notes; owner sign-off.

## Read first (only these)

- `AGENTS.md`
- `docs/phases/P5-real-environment.md` (runbook R1–R13 and the temporary run)
- `docs/phases/P9`–`P13` reports
- `deploy/README.md`, `docs/ARCHITECTURE.md` §9–§11

## Do NOT

- Commit host names, IPs, domains, SSH users, or keys to this repository
  (the report says "the server"; host details stay in the owner's private
  notes).
- Run load tests against a server shared with other people's production
  services without the owner's written OK in chat.
- Change gameplay values.

## Steps

### 1. Deploy
Deploy `main` with the kit (or the owner's existing setup) using the
server build profile (P10). Run with `TBG_REGISTER_PER_IP_HOUR` at its
default; raise it only during step 4 and set it back after.

### 2. Functional smoke (two people, two networks)
The owner and one other person (different networks; one on 4G):
create accounts (phone + desktop), save keys, log in on a second device
with the key, play 3 ranked matches, check rating changes on both screens
and on `/ranking/`, open each other's player page.

### 3. Drop and resume matrix (phone and desktop)

| Case | Expected |
|------|----------|
| Phone airplane mode 10 s mid-match | `Reconnecting…` countdown, same match resumes |
| Phone airplane mode 40 s | forfeit; rating shows the loss (BR-R04) |
| Phone: switch to another app 20 s, return | resumes (or never dropped) |
| Desktop tab hidden 60 s | still connected, match continues |
| Page reload mid-match | resumes in ≤ 3 s with no tap |
| Same account opened on a second device mid-match | second device takes over; first shows a clear message |
| Server restart (deploy) during a ranked match | drain lets it finish; rating applied once |

### 4. Load and latency over the internet
From one or two outside machines (`tbg-bots`, `wss://`):
- 50 ranked matches with `--ranked` → no store errors; `ranked_matches`
  rows = 50; every bot got `RatingChanged`.
- Capacity: the P10 levels up to the server's limit; record matches, CPU,
  RSS, late ticks, RTT p95, ack p95, view gap p95, drops by `reason`.
- Ranking load: `oha -z 30s -c 100 https://<server>/api/leaderboard` from
  outside; record rate-limited share (expected: most requests 429 from one
  IP — that is the limit working) and server CPU.
- Soak: 2 hours at ~30 % of the measured capacity: RSS growth < 10 %, no
  late-tick bursts except ones explained by host CPU steal (record steal).

### 5. Data safety drill
- Nightly backup job present; run it once by hand.
- Restore drill on a scratch copy: start a second server on another port
  with the restored file, log in with a real key, open a player page —
  same rating and history.
- Off-site copy exists (if the owner configured `TBG_BACKUP_REMOTE`).

### 6. Security review (reviewer)
Check against ARCHITECTURE §11 and P11–P13 rules, with evidence:
keys never in logs (`journalctl | grep -E '[0-9A-Z]{4}-[0-9A-Z]{4}-'` →
nothing), no IPs in the DB, `BadKey` uniform, rate limits for register /
login / API / WS leaderboard observed, `/metrics` not public, API JSON has
no private fields, web pages have no `innerHTML`, `cargo audit` clean,
firewall (`ss -tlnp`, `ufw status`) as expected.

### 7. Results and sign-off
Fill the table below; open issues go to "Open items" with owners. The
owner signs off in the report (date + "OK to start P15").

## Results table (fill in)

| Area | Target | Measured |
|------|--------|----------|
| RTT p95 from the target audience | ≤ 150 ms | **pending (real server)**; loopback 2 ms |
| ack latency p95 | ≤ RTT p95 + 60 ms | loopback 50 ms vs RTT p95 2 ms (**local only**) |
| Queue wait p95 with 2+ players online | ≤ 2 s | 24 ms (50 ranked matches, loopback; **local only**) |
| Concurrent matches at 0 late ticks (this server) | record | **pending (real server)**; dev PC (16 cores): 450 concurrent ranked matches, 0 late ticks over 11 min; P10: 1500 matches at 1.47 cores |
| Drops per 100 player-minutes (non-chaos) | record | 0.0 (100 connections, 242.7 player-minutes, loopback; **local only**) |
| Reconnect success (10 s cases) | 100 % | 40/40 at a 5 s drop (`--chaos-pct 100`, ranked); **10 s / 40 s phone cases pending** |
| Ranked results applied exactly once | 100 % | 100 % (50/50, 20/20, and 1 189 rows in the soak: every account's counters equal `ranked_games`) |
| Restore drill | pass | pass (local scratch copy; **pending on the real server**) |

## Acceptance checklist

- [ ] Every row of the results table filled (**local values only; real-server rows pending**)
- [ ] Drop and resume matrix passes on a real phone
- [x] Backup + restore drill passes (local; repeat on the server)
- [x] Security review complete, no open critical item (server-side items — `ss`, `ufw`, `journalctl` — to repeat on the host)
- [ ] Owner sign-off recorded

## Reviewer checklist

- No host details were committed.
- Load settings were reset after step 4.
- Every failure found has a fix commit or an "Open items" entry.

## Phase report

Executed 2026-10-28 by the reviewer on the dev PC (16 cores) in a separate
git worktree (branch `phase/p14-competitive-validation`). **This is a local
rehearsal, not the gate**: no server, domain, second network, or phone was
available, and nothing here replaces steps 2, 3 (phone rows), 4 (internet),
or the owner sign-off. Host details: none (none were used).

### Verification commands (final tip, all pass)

```text
$ cargo fmt --all -- --check                                  # clean
$ cargo clippy --workspace --all-targets -- -D warnings       # clean
$ cargo clippy -p tbg-client --target wasm32-unknown-unknown -- -D warnings   # clean
$ cargo test --workspace   # core 6+5+15; server 2 unit, 11 account, 4 leaderboard,
                           # 12 online, 5 ranked (5/5 on 5 consecutive runs)
$ cargo check -p tbg-client -p tbg-core --target wasm32-unknown-unknown   # ok
$ trunk build --release    # success; WASM 24 349 033 bytes (+335 vs the P13 build)
$ cargo audit              # 0 vulnerabilities; 1 warning: ttf-parser unmaintained
                           # (client-only via winit; not in tbg-server's tree)
```

### Step 4 — load (loopback, release server, default 50 ms tick)

- **50 ranked matches** (`tbg-bots --ranked --matches 50`): 100/100 bots
  completed, RTT p95 2 ms, ack p50/p95 49/50 ms, view gap p95 100 ms, queue
  wait p95 24 ms, 0 late ticks, 0 rejections. Store: 50 `ranked_matches`
  rows, 100 accounts each with `ranked_games = 1` and `wins+losses+draws =
  ranked_games`, ratings equal the rows' `after` values, sum of ratings
  100 000 (conserved), reasons 40 `StrongholdDestroyed` / 10 `TimeUp`.
- **Drops:** 0 in 100 connections (242.7 player-minutes).
- **Reconnect (chaos):** `--ranked --matches 20 --chaos-pct 100`: 40 bots
  dropped at 20–60 s and rejoined 5 s later: 40/40 reconnected and
  completed, 20 rows added, no forfeit, counters consistent.
- **Soak (shortened; spec asks 2 h on the server):** 450 concurrent ranked
  matches (~30 % of the P10 capacity), 4 waves back to back, 11 min, default
  allocator: 0 late ticks, RTT p95 2 ms, 237 CPU-seconds over 662 s (0.36
  cores average), waves 1–2 900/900 bots each, 1 189 ranked rows and 2 699
  accounts with 0 counter mismatches. **RSS:** flat inside each wave
  (138.5 → 139.8 MB) but the plateau steps up between waves: 136 → 139 → 180 MB
  (+32 % by wave 3) and the idle floor 25 → 124 MB. Repeated with
  `MALLOC_ARENA_MAX=2`: 136 → 136 → 155 MB (+14 %), same CPU (234.6 s). No
  per-wave linear growth, so it looks like heap high-water marks
  (fragmentation) rather than a leak, but 11 minutes cannot prove it. Note
  16 cores give glibc 128 arenas; a 2 vCPU host gets 16. See Open items.
- `/api/leaderboard` load: P13 (≈ 4 940 requests/s, 1.66 cores). Not repeated.

### Step 3 — drop and resume matrix (what a local run can show)

| Case | Result |
|------|--------|
| Phone airplane mode 10 s / 40 s | **pending (phone)**. Covered: bot drop 5 s → resumed 40/40; `br_r04_ranked_forfeit_is_a_loss` and `forfeit_after_window` (1 s window) |
| Phone: other app 20 s | **pending (phone)** |
| Desktop tab hidden 60 s | not run (the in-app pane hides the page and drops it to ~1 FPS but the socket stayed connected for minutes during every browser test) |
| Page reload mid-match | **pass** (in-app browser, ranked and casual): resumed with no tap; after the fixes below no misleading toast and the menu stays logged in |
| Same account on a second device mid-match | **pass after fix**: see Fixes; browser shows "Your account signed in on another device" and returns to the menu as guest |
| Server restart during a ranked match | **pass**: SIGTERM mid-match → process stays up (`shutdown: waiting for matches running=1`), `/health` ok, a new ranked match is refused (2/2 bots failed), the running match finished, process exited, 1 `ranked_matches` row, ratings 1020/980 applied once |

### Step 5 — data safety drill (local scratch copy)

- `deploy/backup.sh` run by hand against the live database while the server
  ran (WAL): backup file mode **600**, `integrity_check` = ok, same row
  counts as the live database (103 accounts, 51 ranked matches); retention
  keeps the 7 newest of 10 files.
- Restore: copied the backup, started a second server (port 8283) on it,
  logged in with the real key of `DrillOne` (rating 980, 1 loss, same as
  live), a wrong key on the restored server → `BadKey`, `/api/players/DrillOne`
  JSON **identical** to the live one (rating, rank 52, last-10), leaderboard
  totals equal (102).
- Off-site copy: not configured (`TBG_BACKUP_REMOTE` is owner-side).

### Step 6 — security review

| Check | Evidence |
|-------|----------|
| Keys never in logs | `grep -E '[0-9A-Z]{4}-[0-9A-Z]{4}-'` on 151 server log lines of the 100-bot run → 0; only IPv4 in the log is the bind address in the startup config |
| No IPs in the DB | schema has no ip/addr/host column; rate-limit logs are memory-only |
| `BadKey` uniform | wrong secret and unknown name share one code path (store test + restored-server run); API 404s identical for unknown and malformed names |
| Rate limits observed | register 3/h → 4th `RateLimited`; `/api/*` 121st → 429; WS `Leaderboard` 1/s (second request ignored) — all by integration tests |
| `/metrics` not public | 200 on loopback, **404 with an `X-Forwarded-For` header**; Caddyfile does not route it |
| API has no private fields | player and board JSON keys listed: no id, key hash, IP, or sub-day timestamp |
| Web pages | 0 `innerHTML`/`eval`/inline handlers; `<img onerror>` as `?name=` renders as text (P13) |
| Firewall / listening sockets | **pending (host)**: unit binds `127.0.0.1:8080`; `provision.sh` opens 22/80/443 only |
| File permissions | **found and fixed**: database, WAL, and backups were 644 in 755 directories (see Fixes) |

### Fixes (each its own commit)

- **be3f603 private data files:** `/var/lib/tbg` and `/var/backups/tbg` mode
  700, `UMask=0077` in the unit, `umask 077` in `backup.sh`; restore drill
  commands in `deploy/README.md`.
- **0f2b04d takeover message (protocol v5):** after a BR-P03 takeover the
  old device reconnected to a fresh guest session and sat on a frozen match
  screen. The server now sends `Error{SessionTakenOver}` before closing the
  old socket; the client shows the message, returns to the menu, stops
  reconnecting, and clears its local profile. Test asserts the message.
- **1bc5ddb auto-login and resume:** (a) the startup probe only opened when a
  *session token* existed, and the token lives per tab (`sessionStorage`),
  so a new tab or a later visit showed Guest although the account key was
  saved (a P11 spec miss): the stored key alone now opens the probe, and the
  browser logs in on a fresh tab; (b) a resumed session that holds an
  account sends `LoggedIn`, and the client no longer toasts "Finish your
  match first" when its auto-login races a resumed match (before, a
  casual-match reload left the menu as Guest).
- **1d11e8e flaky test:** `br_r04_ranked_forfeit_is_a_loss` assumed the second
  socket is `Seat::B`; it failed 1 run in 3 under load. It now reads the
  stayer's real seat.

### Open items

1. **Real-server validation (owner):** deploy, steps 2 and 3 on a phone and a
   desktop on two networks, step 4 from outside (50 ranked, capacity,
   `oha`, 2 h soak with steal recorded), `ss -tlnp` / `ufw status` /
   `journalctl` greps, off-site backup copy, then sign-off. Owner decision
   recorded in chat: proceed to P15 in parallel; the visual suite does not
   touch the server or protocol.
2. **RSS steps between waves (reviewer, on the real host):** re-measure in
   the 2 h steady soak on the real host. If the plateau still climbs, try
   `Environment=MALLOC_ARENA_MAX=2` in the unit (measured −14 % at wave 3
   here, no CPU cost) before anything heavier; P10 already ruled out
   mimalloc (+12 % RSS).
3. **Root-run backup of a stopped database:** `backup.sh` runs as root;
   `sqlite3 .backup` on a WAL database whose server is stopped could create a
   root-owned `-shm`/`-wal` that `tbg` then cannot open. Not reproduced (the
   cron runs while the server is up); worth `sudo -u tbg` if the server is
   ever stopped at 03:30.
4. **Link in a stale tab:** `?room=` links skip the startup probe, so a
   logged-in player who opens a room link plays that room as a guest. Casual
   only; left as is.
5. **Hidden in-app browser pane:** runs at 1–2 FPS and stops taking some
   clicks, so the on-screen highlight of the player's own row in the in-game
   ranking (P13) and a 60 s hidden-tab case were not eyeballed.

### Owner sign-off

Pending. (date + "OK to start P15" goes here after the real-server run.)
