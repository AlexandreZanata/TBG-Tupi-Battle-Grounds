# Deploy — one VPS, Caddy + systemd

Prerequisites (owner): a VPS with Ubuntu 22.04/24.04 or Debian 12 (≥ 2 vCPU,
≥ 2 GB RAM), SSH key login for a sudo user, and a domain/subdomain whose
**A record** points to the VPS IP. Locally: `rustup`, `trunk`, `rsync`, `ssh`.

## First deploy

```bash
cp deploy/.env.deploy.example deploy/.env.deploy   # fill in host, user, port, domain
dig +short "$(. deploy/.env.deploy; echo $TBG_DOMAIN)" # must print the VPS IP
bash deploy/deploy.sh --provision
```

`--provision` installs Caddy, build tools, the `tbg` system user, firewall
(22/80/443 only), the systemd unit, the Caddyfile, a swapfile on small
machines, and the nightly backup cron. Caddy fetches the TLS certificate on
the first HTTPS request (can take ~30 s).

## Later deploys

```bash
bash deploy/deploy.sh
```

Builds the WASM bundle locally, rsyncs source + bundle, builds `tbg-server`
on the VPS (avoids glibc mismatches), installs it, restarts it, and checks
`https://<domain>/health`. A restart waits up to 5 minutes for running
matches to finish (graceful shutdown).

## Operate

| Need | Command |
|------|---------|
| Logs | `ssh <host> journalctl -u tbg-server -f` |
| Live metrics (never public) | `ssh <host> curl -s localhost:8080/metrics` |
| Backup now | `ssh <host> sudo /opt/tbg/bin/backup.sh` |
| Off-site weekly copy | put `TBG_BACKUP_REMOTE=user@backuphost:path/` in `/etc/default/tbg-backup` |
| Rollback | `git checkout <previous tag>` then `bash deploy/deploy.sh` |

Public ranking (P13): Caddy proxies `/api/*` to the server and serves the
static pages at `https://<domain>/ranking/` (all `no-cache`, revalidated by
ETag; only the hashed WASM bundle is immutable). Check after a deploy:

```bash
curl -s https://<domain>/api/leaderboard?limit=3
```

`TBG_API_PER_IP_MIN` (default 120) limits `/api/*` per IP; raise it only on
a test server for load runs (`oha`).

Match records:

```sql
-- sqlite3 /var/lib/tbg/tbg.sqlite3
SELECT count(*), round(avg(duration_s)), sum(reason='Forfeit') FROM matches;
SELECT reason, count(*) FROM matches GROUP BY reason;
SELECT json_extract(seat_a,'$.rtt_p95'), json_extract(seat_b,'$.rtt_p95') FROM matches ORDER BY id DESC LIMIT 20;
```

## Load and latency tests from your machine

```bash
ssh <host> 'sudo mkdir -p /etc/systemd/system/tbg-server.service.d && printf "[Service]\nEnvironment=TBG_MAX_PER_IP=10000\n" | sudo tee /etc/systemd/system/tbg-server.service.d/bots.conf && sudo systemctl daemon-reload && sudo systemctl restart tbg-server'
ulimit -n 8192
cargo run --release -p tbg-server --bin tbg-bots -- --url wss://<domain>/ws --matches 50
ssh <host> 'sudo rm /etc/systemd/system/tbg-server.service.d/bots.conf && sudo systemctl daemon-reload && sudo systemctl restart tbg-server'
```

Never commit `deploy/.env.deploy`, IPs, domains, or keys.
