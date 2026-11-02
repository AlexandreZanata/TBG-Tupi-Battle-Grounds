#!/usr/bin/env bash
# P66 two-slot release on the host, as root (deploy/README.md "Releases").
#   release-slot.sh start  <a|b> <generation> <tbg-server binary> [build]
#   release-slot.sh retire <a|b>
# `start` installs a new immutable generation into the idle slot and waits
# until it is `ready` (it admits nothing until an operator promotes it).
# `retire` stops a drained slot only after its retirement gate passed and
# otherwise reports Pending: it never kills a slot that is still busy.
# Running slots are enabled and retired ones disabled, so a reboot restarts
# exactly the generations the directory still knows; it decides which admits.
set -euo pipefail

DB="${TBG_DB:-/var/lib/tbg/tbg.sqlite3}"
# The automatic controller runs sandboxed as root without privilege escalation, where sudo refuses.
if [ "$(id -u)" = 0 ]; then AS_TBG=(runuser -u tbg --); else AS_TBG=(sudo -u tbg); fi
ADMIN=("${AS_TBG[@]}" /opt/tbg/bin/tbg-release-admin --db "$DB")
usage="usage: release-slot.sh start <a|b> <generation> <binary> [build] | retire <a|b>"
cmd="${1:?$usage}"
slot="${2:?$usage}"
case "$slot" in
a) port=8080 ;;
b) port=8082 ;;
*)
	echo "$usage" >&2
	exit 2
	;;
esac

start() {
	local generation="${1:?$usage}" binary="${2:?$usage}" build="${3:-$1}"
	[[ "$generation" =~ ^[A-Za-z0-9._-]{1,64}$ ]] || { echo "bad generation id" >&2; exit 2; }
	if systemctl is-active --quiet "tbg-server@$slot"; then
		echo "slot $slot is running: retire it before reusing it" >&2
		exit 1
	fi
	local dir="/opt/tbg/releases/$generation"
	if [ -e "$dir" ]; then
		echo "$dir exists: a generation is never rebuilt in place" >&2
		exit 1
	fi
	install -d -m 755 "$dir" "/opt/tbg/slots/$slot" /etc/tbg
	install -m 755 "$binary" "$dir/tbg-server"
	ln -sfn "$dir/tbg-server" "/opt/tbg/slots/$slot/tbg-server"
	printf 'TBG_ADDR=127.0.0.1:%s\nTBG_GENERATION=%s\nTBG_SERVER_BUILD=%s\n' \
		"$port" "$generation" "$build" >"/etc/tbg/slot-$slot.env"
	systemctl enable --now "tbg-server@$slot"
	for _ in $(seq 1 60); do
		if "${ADMIN[@]}" status | grep -Eq "^$slot +ready"; then
			echo "slot $slot ready with $generation: canary or promote it with tbg-release-admin"
			return
		fi
		sleep 2
	done
	echo "slot $slot is not ready and admits nothing; see journalctl -u tbg-server@$slot" >&2
	exit 1
}

retire() {
	for _ in $(seq 1 "${TBG_RETIRE_CHECKS:-30}"); do
		if "${ADMIN[@]}" retire-check "$slot"; then
			systemctl disable --now "tbg-server@$slot"
			"${ADMIN[@]}" retire "$slot"
			return
		fi
		sleep 10
	done
	echo "slot $slot: retirement pending (still busy); run retire again later" >&2
	exit 3
}

case "$cmd" in
start) start "${@:3}" ;;
retire) retire ;;
*)
	echo "$usage" >&2
	exit 2
	;;
esac
