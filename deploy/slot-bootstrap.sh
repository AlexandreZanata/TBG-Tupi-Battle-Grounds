#!/usr/bin/env bash
# One-time switch from the single tbg-server process to release slot a
# (D-92, deploy/README.md "Releases without stopping matches"), as root.
# It changes the mode, not the code: slot a runs the executable the single
# process runs now, and new code arrives later through the idle slot. Run it
# detached so a dropped SSH session cannot stop it halfway:
#   systemd-run --unit=tbg-slot-bootstrap --wait --collect \
#     /bin/bash slot-bootstrap.sh <generation> [build]
# It refuses unless the server is idle (no connection, match, queue or
# pending result) and restores the single process if slot a does not take
# over. Output goes to /var/lib/tbg-deploy/slot-bootstrap-<time>.log.
set -euo pipefail

usage="usage: slot-bootstrap.sh <generation> [build]"
GEN="${1:?$usage}"
BUILD="${2:-$GEN}"
BIN=/opt/tbg/bin/tbg-server
DB="${TBG_DB:-/var/lib/tbg/tbg.sqlite3}"
ADMIN=(sudo -u tbg /opt/tbg/bin/tbg-release-admin --db "$DB")
LOG="/var/lib/tbg-deploy/slot-bootstrap-$(date -u +%Y%m%dT%H%M%SZ).log"
install -d -m 700 /var/lib/tbg-deploy
exec >>"$LOG" 2>&1
say() { echo "$(date -u +%H:%M:%S.%3N) $*"; }

# Prints the live contract; fails unless the single process is idle.
idle() {
	python3 - "$(curl -sf -m 5 http://127.0.0.1:8080/api/release)" \
		"$(curl -sf -m 5 http://127.0.0.1:8080/metrics)" <<'EOF'
import json, sys
r, m = json.loads(sys.argv[1]), json.loads(sys.argv[2])
load = r['load']
checks = {
    'single process': r['slot'] is None and r['directory'] == [],
    'admitting': r['admitting'] and not r['shutdown'],
    'no connections': m['connections'] == 0 and m['online_players'] == 0 and m['spectators'] == 0,
    'no matches': m['matches_running'] == 0 and m['live_matches'] == 0 and load['matches_running'] == 0,
    'no pending results': load['results_pending'] == 0 and m['result_receipts_pending'] == 0
                          and m['ranked_pending_files'] == 0,
    'no ranked result in flight': load['ranked_inflight'] == 0 and m['ranked_inflight'] == 0,
    'empty queue': m['queue_len'] == 0,
}
bad = [k for k, ok in checks.items() if not ok]
if bad:
    sys.exit('refused: ' + ', '.join(bad))
print(r['contract'])
EOF
}

# Slot a admits new matches, settles ranked results and runs `contract`.
taken_over() {
	curl -sf -m 2 http://127.0.0.1:8080/api/release | python3 -c '
import json, sys
r = json.load(sys.stdin)
sys.exit(0 if r["slot"] == "a" and r["admitting"] and r["ranked"] and not r["shutdown"]
         and r["contract"] == sys.argv[1] else 1)' "$1"
}

restore() {
	say "FALLBACK: $1; restoring the single process"
	systemctl disable --now tbg-server@a || true
	systemctl enable --now tbg-server
	say "single process: $(systemctl is-active tbg-server || true)"
	exit 1
}

[ "$(id -u)" = 0 ] || { say "run as root"; exit 2; }
# Controllers defer while a deployment step holds the shared lock.
exec 9>/var/lib/tbg-deploy/lock
flock -n 9 || { say "deployment lock busy; nothing changed"; exit 75; }
systemctl is-active --quiet tbg-server || { say "single process not active; nothing changed"; exit 1; }
! systemctl is-active --quiet tbg-server@a || { say "slot a already running; nothing changed"; exit 1; }
pid="$(systemctl show tbg-server -p MainPID --value)"
[ "$(sha256sum <"/proc/$pid/exe")" = "$(sha256sum <"$BIN")" ] ||
	{ say "$BIN differs from the running executable; nothing changed"; exit 1; }
contract="$(idle)" || exit 1
/opt/tbg/bin/backup.sh --predeploy
# Re-check right before the stop: a player could have arrived meanwhile.
[ "$(idle)" = "$contract" ] || exit 1

say "stopping the single process $pid ($contract)"
systemctl disable --now tbg-server
say "single process stopped"
/opt/tbg/bin/release-slot.sh start a "$GEN" "$BIN" "$BUILD" || restore "slot a did not become ready"
"${ADMIN[@]}" promote a --actor slot-bootstrap || restore "promote refused"
"${ADMIN[@]}" ranked a --actor slot-bootstrap || say "ranked handoff refused; the active slot takes it when idle"
for _ in $(seq 1 50); do
	if taken_over "$contract"; then
		# Reboots start the active slot, never the retired single unit.
		systemctl enable tbg-server@a
		say "slot a admits new matches with ranked authority"
		"${ADMIN[@]}" audit 4
		exit 0
	fi
	sleep 0.2
done
restore "slot a promoted but not admitting"
