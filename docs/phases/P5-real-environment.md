# P5 — Real-environment validation (owner's VPS + domain)

**Depends on:** P4 reviewed and merged.
**Branch:** `phase/p5-real-environment`.
**Difficulty:** medium. Scripts run on a real server: read every command
before running it, and never run anything not written in this file.

## Goal

Run the skeleton game on the owner's VPS behind the owner's domain with
HTTPS/WSS, and prove with measurements that it works for real players on
real networks: latency, reconnects, capacity, restarts, persistence,
backups. Still placeholder visuals — no 3D work yet.

## Inputs from the owner (required before step 3)

| Input | Example | Where it goes |
|-------|---------|---------------|
| VPS public IP | `203.0.113.10` | `deploy/.env.deploy` (never committed) |
| SSH user with sudo and key-based login | `ubuntu` | `deploy/.env.deploy` |
| SSH port | `22` | `deploy/.env.deploy` |
| Domain or subdomain with an **A record** pointing to the IP | `play.example.com` | `deploy/.env.deploy` |

Assumed OS: Ubuntu 24.04 LTS or 22.04, or Debian 12; x86_64 or arm64;
≥ 2 vCPU, ≥ 2 GB RAM. If the OS differs, stop and ask the reviewer.

**Never commit the IP, domain, user names, keys, or passwords.** Before
every commit run `git grep -n "<the real IP>"` and `git grep -n "<the real domain>"`
— both must print nothing.

## Read first (only these)

- `AGENTS.md`
- `docs/ARCHITECTURE.md` §8 (persistence), §9 (deployment), §10 (budgets), §11 (security)
- `crates/tbg-server/src/{lib.rs,config.rs,match_task.rs,metrics.rs}`

## Do NOT

- Install Docker, Kubernetes, Nginx, databases other than SQLite, or
  monitoring stacks.
- Expose port 8080 publicly; the server listens on `127.0.0.1:8080` and
  only Caddy is public.
- Store personal data or IP addresses in SQLite.

## Steps

### 1. Match records (`records.rs`)

- Add `rusqlite = { version = "0.41", features = ["bundled"] }` (latest on
  2026-10-27).
- Env `TBG_DB` (default `tbg.sqlite3`). On start: open, `PRAGMA
  journal_mode=WAL`, create table:
  ```sql
  CREATE TABLE IF NOT EXISTS matches (
    id INTEGER PRIMARY KEY,
    ended_at TEXT NOT NULL,            -- RFC 3339 UTC
    duration_s INTEGER NOT NULL,
    winner TEXT,                       -- 'A', 'B', or NULL for a draw
    reason TEXT NOT NULL,              -- 'StrongholdDestroyed' | 'TimeUp' | 'Forfeit'
    seed INTEGER NOT NULL,
    seat_a TEXT NOT NULL,              -- JSON: rtt_p50, rtt_p95, intents, rejections, bytes_out,
    seat_b TEXT NOT NULL,              --       purchases {kind: count}, evolutions, reconnects
    step_us_mean INTEGER NOT NULL,
    step_us_max INTEGER NOT NULL,
    late_ticks INTEGER NOT NULL,
    max_lateness_ms INTEGER NOT NULL
  );
  ```
- The match task fills a `MatchRecord` (count purchases from `Spawn`
  events and evolutions from `Evolved` events per seat) and hands it to
  `records::insert` through `tokio::task::spawn_blocking`. One
  `rusqlite::Connection` behind `Arc<Mutex<..>>`. A failed insert is
  logged, never fatal.
- Test: `records_insert_and_count` in `tests/online.rs` (temp DB file).

### 2. Graceful shutdown

On SIGTERM or Ctrl-C: the lobby answers new `QuickMatch`/`CreateRoom`/`JoinRoom`
with `Error{ServerFull}`; running matches continue; the process exits when
no match is running or after 300 s. Use `tokio::signal`. Log
`shutdown: waiting for N matches`.

### 3. Deploy kit (`deploy/`)

Create exactly these files. Placeholders use `__NAME__` and are replaced
by `sed` in the scripts.

- `deploy/.env.deploy.example`:
  ```
  TBG_HOST=203.0.113.10
  TBG_SSH_USER=ubuntu
  TBG_SSH_PORT=22
  TBG_DOMAIN=play.example.com
  ```
  Add `!.env.deploy.example` to `.gitignore` (the `.env.*` rule would
  otherwise hide it). `deploy/.env.deploy` stays ignored.
- `deploy/Caddyfile.template`:
  ```
  __TBG_DOMAIN__ {
  	encode zstd gzip
  	handle /ws {
  		reverse_proxy 127.0.0.1:8080
  	}
  	handle /health {
  		reverse_proxy 127.0.0.1:8080
  	}
  	handle {
  		root * /srv/tbg/web
  		@hashed path *.wasm *.js
  		header @hashed Cache-Control "public, max-age=31536000, immutable"
  		@html path / /index.html
  		header @html Cache-Control "no-cache"
  		file_server
  	}
  }
  ```
- `deploy/tbg-server.service`:
  ```ini
  [Unit]
  Description=TBG game server
  After=network-online.target
  Wants=network-online.target

  [Service]
  User=tbg
  Group=tbg
  WorkingDirectory=/var/lib/tbg
  Environment=TBG_ADDR=127.0.0.1:8080
  Environment=TBG_DB=/var/lib/tbg/tbg.sqlite3
  ExecStart=/opt/tbg/bin/tbg-server
  Restart=always
  RestartSec=2
  KillSignal=SIGTERM
  TimeoutStopSec=330
  LimitNOFILE=65536
  NoNewPrivileges=true

  [Install]
  WantedBy=multi-user.target
  ```
- `deploy/provision.sh` (runs **on the VPS** with sudo, idempotent; the
  deploy script copies and runs it): `set -euo pipefail`; `apt-get update`;
  install `caddy build-essential pkg-config curl sqlite3 ufw rsync` (if no
  `caddy` package exists — e.g. Ubuntu 22.04 — first add the official
  Caddy apt repository exactly as documented at
  `https://caddyserver.com/docs/install`, Debian/Ubuntu section); if total
  RAM < 3 GB and no swap, create a 2 GB `/swapfile`; create system user
  `tbg` (no login shell) and directories `/opt/tbg/bin`, `/var/lib/tbg`
  (owner tbg), `/srv/tbg/web`, `/var/backups/tbg`; install rustup for the
  SSH user if `cargo` is missing (`curl https://sh.rustup.rs -sSf | sh -s -- -y --profile minimal`);
  `ufw allow OpenSSH`, `ufw allow 80,443/tcp`, `ufw --force enable`;
  install the service unit and `systemctl daemon-reload && systemctl enable tbg-server`;
  render the Caddyfile to `/etc/caddy/Caddyfile` and `systemctl reload caddy`;
  install `/etc/cron.d/tbg-backup`: `30 3 * * * root /opt/tbg/bin/backup.sh`.
- `deploy/backup.sh` (on the VPS): `sqlite3 /var/lib/tbg/tbg.sqlite3
  ".backup /var/backups/tbg/tbg-$(date +%F).sqlite3"` and delete all but
  the 7 newest files.
- `deploy/deploy.sh` (runs **locally**): `set -euo pipefail`; source
  `deploy/.env.deploy`; `(cd crates/tbg-client && trunk build --release)`;
  `rsync` the repository (excluding `target/`, `dist/`, `.git/`,
  `archive/`, `deploy/.env.deploy`) to `~/tbg-src/` on the VPS; `rsync`
  `crates/tbg-client/dist/` to `~/tbg-web/`; then over `ssh`:
  first run only `sudo bash ~/tbg-src/deploy/provision.sh <domain>`;
  `cd ~/tbg-src && cargo build --release -p tbg-server --bin tbg-server`;
  `sudo install -m 755 target/release/tbg-server /opt/tbg/bin/`;
  `sudo install -m 755 deploy/backup.sh /opt/tbg/bin/`;
  `sudo rsync -a --delete ~/tbg-web/ /srv/tbg/web/`;
  `sudo systemctl restart tbg-server`; finally
  `curl -fsS https://$TBG_DOMAIN/health` must print `ok`.
  Flag `--provision` runs provisioning; without it, skip it.
- `deploy/README.md` (≤ 60 lines): prerequisites, first deploy, later
  deploys, logs (`journalctl -u tbg-server -f`), metrics
  (`ssh … curl -s localhost:8080/metrics`), the three SQL queries below,
  rollback (`git checkout <previous tag>` then deploy).

SQL queries for the README:
```sql
SELECT count(*), round(avg(duration_s)), sum(reason='Forfeit') FROM matches;
SELECT reason, count(*) FROM matches GROUP BY reason;
SELECT json_extract(seat_a,'$.rtt_p95'), json_extract(seat_b,'$.rtt_p95') FROM matches ORDER BY id DESC LIMIT 20;
```

Check locally before touching the VPS: `bash -n deploy/*.sh` and
`shellcheck deploy/*.sh` if available.

### 4. First deployment

1. Owner confirms DNS: `dig +short <domain>` prints the VPS IP.
2. `bash deploy/deploy.sh --provision`.
3. `curl -I https://<domain>/` → `200`, certificate valid (Caddy obtains it
   automatically; first request may take ~30 s).

### 5. Validation runbook on the real environment

Raise the per-IP cap only while bot tests run (all bots share one IP):
```bash
ssh <host> 'sudo mkdir -p /etc/systemd/system/tbg-server.service.d && printf "[Service]\nEnvironment=TBG_MAX_PER_IP=10000\n" | sudo tee /etc/systemd/system/tbg-server.service.d/bots.conf && sudo systemctl daemon-reload && sudo systemctl restart tbg-server'
```
Remove it after R6 (`sudo rm …/bots.conf`, daemon-reload, restart). Locally
run `ulimit -n 8192` before big bot runs.

| # | Test | Pass when / record |
|---|------|--------------------|
| R1 | `curl -s https://<domain>/health`; `curl -s -o /dev/null -w '%{http_code}' https://<domain>/metrics` | `ok`; metrics is **not** 200 |
| R2 | Desktop Chrome opens `https://<domain>/` with DevTools Network open, cache disabled | loads; record transferred bytes and time until the menu is visible |
| R3 | `cargo run --release -p tbg-server --bin tbg-bots -- --url wss://<domain>/ws --matches 1` | completed; record RTT p50/p95 |
| R4 | same with `--matches 50` | all completed; view-gap p95 ≤ 200 ms |
| R5 | same with `--matches 50 --chaos-pct 20` | all completed; reconnects all succeed |
| R6 | capacity: `--matches 100`, then 250, then 500 (stop early if failures or CPU > 85%) | for each: completed/failed, server CPU % and RSS (`ssh <host> 'top -b -n1 -p $(pidof tbg-server)'` at peak), `/metrics` at peak, bytes_out per player per second |
| R7 | Two people on **different networks** (one on mobile 4G/5G) → Quick Match | match completes; record both overlays' RTT and view p95 |
| R8 | During a match the 4G phone enables airplane mode for 10 s | reconnects and continues |
| R9 | Same, airplane mode for 40 s | both see the result; phone loses by Forfeit |
| R10 | Start one bot match, then `ssh <host> 'sudo systemctl restart tbg-server'` | restart waits; the running match finishes; new connections work after |
| R11 | `ssh <host> "sqlite3 /var/lib/tbg/tbg.sqlite3 'SELECT count(*) FROM matches'"` | equals completed matches since deploy; run `sudo /opt/tbg/bin/backup.sh`, copy the file to `/tmp`, query it successfully |
| R12 | Owner reports RTT from home Wi-Fi and 4G in the overlay | if p95 > 200 ms for the target audience, the report recommends a closer region |
| R13 | `ssh <host> 'sudo ufw status; sudo ss -tlnp'` | only 22/80/443 open; tbg-server bound to `127.0.0.1:8080` |

## Acceptance checklist

- [ ] All verification commands from `AGENTS.md` pass; `bash -n deploy/*.sh` passes
- [ ] R1–R13 pass; numbers recorded in the report
- [ ] `docs/ARCHITECTURE.md` §10: "Measured" values added for load time,
      bandwidth, server memory, concurrent matches (from R2, R6)
- [ ] `git grep` for the real IP and domain prints nothing
- [ ] The bots drop-in file is removed from the VPS

## Risks and what to do

| Problem | Action |
|---------|--------|
| Caddy cannot get a certificate | DNS not pointing yet or port 80 blocked by the provider firewall; check the provider panel |
| `cargo build` killed on the VPS | Out of memory; confirm the swapfile exists, retry with `cargo build -j 1` |
| WebSocket fails behind Caddy but `/health` works | Check the `handle /ws` block and that the client uses `wss://<domain>/ws` (rule in P1 `server_url`) |
| R6 fails early | Record where it failed and the `/metrics` + `journalctl` lines; do not tune code in this phase |

## Reviewer checklist

- Scripts are idempotent, use `set -euo pipefail`, contain no secrets.
- Server bound to localhost; `/metrics` not public; firewall correct.
- The record rows contain no IP or personal data.
- Capacity numbers are consistent with `/metrics` and `top` output.
- Decide whether the region is acceptable (R12) before P6.

## Phase report

*(Executor fills this in at the end. Replace the real domain/IP with `<domain>`/`<host>`.)*

- Commands run and results:
- R1–R13 table with numbers:
- Capacity summary (matches vs CPU/RSS/bandwidth):
- Deviations:
- Blocked items:
