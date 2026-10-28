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
| RTT p95 from the target audience | ≤ 150 ms | |
| ack latency p95 | ≤ RTT p95 + 60 ms | |
| Queue wait p95 with 2+ players online | ≤ 2 s | |
| Concurrent matches at 0 late ticks (this server) | record | |
| Drops per 100 player-minutes (non-chaos) | record | |
| Reconnect success (10 s cases) | 100 % | |
| Ranked results applied exactly once | 100 % | |
| Restore drill | pass | |

## Acceptance checklist

- [ ] Every row of the results table filled
- [ ] Drop and resume matrix passes on a real phone
- [ ] Backup + restore drill passes
- [ ] Security review complete, no open critical item
- [ ] Owner sign-off recorded

## Reviewer checklist

- No host details were committed.
- Load settings were reset after step 4.
- Every failure found has a fix commit or an "Open items" entry.

## Phase report

_Reviewer fills this in: results table, matrix outcomes, drill output,
security notes, open items, owner sign-off._
