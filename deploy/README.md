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

Backups and restore drill (P14). `/var/lib/tbg` and `/var/backups/tbg` are
mode 700 and the unit sets `UMask=0077`: the database holds names,
ratings, and key hashes. Run the drill after every schema change, on a
scratch copy and a spare port, never on the live file:

```bash
ssh <host> sudo /opt/tbg/bin/backup.sh                     # newest copy in /var/backups/tbg
ssh <host> 'sudo cp /var/backups/tbg/tbg-$(date +%F).sqlite3 /tmp/restore.sqlite3 && sudo chown tbg /tmp/restore.sqlite3'
ssh <host> 'sudo -u tbg env TBG_ADDR=127.0.0.1:8081 TBG_DB=/tmp/restore.sqlite3 /opt/tbg/bin/tbg-server'   # second terminal
ssh <host> curl -s localhost:8081/api/players/<Name>       # same rating and history as the live page
# log in with a real account key in a browser tunnelled to :8081, then stop the scratch server and rm the copy
```

Schema v7 (P60 ranked records). `deploy.sh` now stops the server, runs
`backup.sh --predeploy` (an integrity-checked copy named
`predeploy-<timestamp>.sqlite3`, never rotated, plus a copy of
`ranked-pending/`, the journal of finished ranked results not yet settled)
and only then installs and starts the binary, which migrates v6 to v7 in
one transaction. Before the first v7 deploy, rehearse on a scratch copy:

```bash
ssh <host> 'sudo cp /var/backups/tbg/predeploy-<timestamp>.sqlite3 /tmp/v7.sqlite3 && sudo chown tbg /tmp/v7.sqlite3'
ssh <host> 'sudo -u tbg env TBG_ADDR=127.0.0.1:8081 TBG_DB=/tmp/v7.sqlite3 /opt/tbg/bin/tbg-server'   # migrates the copy
ssh <host> "sqlite3 /tmp/v7.sqlite3 'SELECT max(version) FROM schema_version; PRAGMA integrity_check; PRAGMA foreign_key_check;'"
```

Rollback: v7 is additive, and the previous v6 binary still reads the
old columns, but it ignores and does not write the v7 records (match key,
integrity rows, sides, journal). Restoring the pre-deploy copy after new
ranked matches were accepted loses those matches. Treat a restore as a
maintenance decision for the owner, with an export and replay of the newer
records. Never delete `ranked-pending/` by hand: each file is a finished
match whose rating has not been applied yet. The server replays the files
in order on start.

Ranked review and corrections (P60 §7). Work on a backup copy for anything
that reads, and on the live file only with the server stopped:

```bash
tbg-ranked-admin list-review --db /tmp/copy.sqlite3
tbg-ranked-admin inspect --match <32-hex key> --db /tmp/copy.sqlite3
tbg-ranked-admin plan --action invalidate --match <key> --reason "<why>" --actor <you> \
  --db /tmp/copy.sqlite3 --out plan.json            # shows both accounts' old/new rating
sudo systemctl stop tbg-server
sudo -u tbg /opt/tbg/bin/tbg-ranked-admin apply --plan plan.json --db /var/lib/tbg/tbg.sqlite3
sudo systemctl start tbg-server                      # rebuilds the leaderboard snapshot
```

The plan must be made from a copy that is current. If any settlement
landed since the plan was made, `apply` refuses it and you make the plan
again. `apply` writes `tbg.sqlite3.pre-<operation>.sqlite3` before it
changes anything; running the same plan twice changes nothing. Only the
two accounts of the match are rebuilt, from their frozen deltas;
`restore` undoes an `invalidate`. `--actor` is a label, not
authentication. The binary is built with
`cargo build --release -p tbg-server --bin tbg-ranked-admin`, and
`deploy.sh` does not install it yet.

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
