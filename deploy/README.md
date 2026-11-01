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
`https://<domain>/health`. This is the single-slot fallback: the stop stops
new matches and waits, with no time limit, until running matches end and
their results are stored (P66). New play pauses for that wait; for a
release that keeps admitting, use the two slots below.

## Releases without stopping matches (P66, D-92)

Two release slots run one immutable server generation each on loopback:
slot `a` on port 8080 (the single server's port) and slot `b` on 8082,
behind the fixed routes `/ws/slot-a` and `/ws/slot-b`. Both share the
database, whose release directory decides where new matches start. A
promotion or rollback is one audited database change: no proxy reload, no
restart, and a running match never moves. Run `tbg-release-admin` as
`tbg` (`sudo -u tbg /opt/tbg/bin/tbg-release-admin --db /var/lib/tbg/tbg.sqlite3 …`,
shortened to `admin` here).

One-time switch from the single server, in a quiet window with no open
sockets: install the units with `provision.sh`, then
`sudo systemctl disable --now tbg-server`,
`sudo /opt/tbg/bin/release-slot.sh start a <generation> /opt/tbg/bin/tbg-server`,
`admin promote a` and `admin ranked a`. Until `promote`, a slot admits
nothing (fail closed).

Each release, into the idle slot (here `b`):

1. Build `tbg-server` and copy it to the host. Schema changes are additive
   only (expand first; both generations must read and write it).
2. `sudo /opt/tbg/bin/release-slot.sh start b <generation> ~/tbg-server`
   installs `/opt/tbg/releases/<generation>/` (never overwritten) and waits
   for `ready`. A slot that fails to become ready admits nothing.
3. Same match contract (`tbg-server --version` and `admin status` show it):
   `admin canary b 5`, then 25, then `admin promote b` after the indicators
   in docs/planning/P66-RELEASE-CONTRACTS.md §3 hold. A new contract is
   switched with `admin promote b` at its scheduled time (BR-UP09).
4. `promote` drains `a`: its running matches, reconnects and pending
   results finish there; new players get `b` from `/api/admission`.
   Ranked settlement moves to `b` on its own once `a` has no ranked result
   in flight (one writer at a time).
5. `sudo /opt/tbg/bin/release-slot.sh retire a` stops `a` only when
   `admin retire-check a` passes (drained, nothing running or pending, no
   ranked authority). Otherwise it reports Pending and leaves `a` running:
   run it again later. A stuck drain delays the next release; it is never
   killed to make room.

Rollback of new admissions while the previous slot still runs:
`admin rollback a` makes `a` active again and drains `b`; matches on both
keep running to their end. Stopping a canary without promoting it:
`admin hold b`. Every change accepts `--expect <version>` (from `admin
status`) so two operators cannot overwrite each other, and `admin audit`
lists the history. A rollback never restores a database backup (that would
lose newer results) and never replaces a running binary.

Legacy clients (before P66) keep using `/ws`, which reaches the first live
slot. During an overlap their reconnect may land on the other slot and
lose its match; P66 clients pin the slot that admitted them. Browser tabs
pick up the P66 client on their next load.

## Update repository (P66)

`/updates/` serves the signed TUF repository from `/srv/tbg/updates`
(Caddy answers 404 for missing files, which TUF needs). Build the tools
with `cargo build --release -p tbg-update`, then, on a trusted machine that
holds the keys (never the VPS):

```bash
tbg-release init-keys --keys <offline-dir>            # once; never overwrites
tbg-release init-repo --keys <offline-dir> --repo <repo-dir>
python3 scripts/package-native.py --platform linux --binary target/release/tbg-client \
  --managed --build-id <id> --sequence <n> --notes <notes.json> --channel stable
tbg-release publish --keys <offline-dir> --repo <repo-dir> --package target/native/package-<id>.tar.gz \
  --notes target/native/notes-<id>.json --descriptor target/native/descriptor-<id>.json
rsync -a <repo-dir>/ <host>:/srv/tbg/updates/          # targets first, then metadata; no --delete
```

`publish` refuses notes that break the content contract (P66 §3.4). Keys
made by `init-keys` are test keys until the owner approves custody,
threshold and rotation (O02); do not publish a real release with them.
Players adopt the managed install once with the bootstrap archive from
`scripts/managed-bootstrap.sh` (launcher plus the trusted `root.json`);
later releases arrive in game.

## Operate

| Need | Command |
|------|---------|
| Logs | `ssh <host> journalctl -u tbg-server -f` |
| Live metrics (never public) | `ssh <host> curl -s localhost:8080/metrics` |
| Backup now | `ssh <host> sudo /opt/tbg/bin/backup.sh` |
| Off-site weekly copy | put `TBG_BACKUP_REMOTE=user@backuphost:path/` in `/etc/default/tbg-backup` |
| Rollback (single slot) | `git checkout <previous tag>` then `bash deploy/deploy.sh` |
| Release state | `sudo -u tbg /opt/tbg/bin/tbg-release-admin --db /var/lib/tbg/tbg.sqlite3 status` |
| One slot's view | `ssh <host> curl -s localhost:8080/api/release` (loopback only) |

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
