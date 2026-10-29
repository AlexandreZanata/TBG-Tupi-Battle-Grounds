#!/usr/bin/env bash
# Capacity run (P10, extended by P30-A): for each level of concurrent matches,
# start a fresh server, fill it with tbg-bots, and measure server CPU (cores),
# RSS, late ticks, RTT, view traffic, and connection gauges over a steady
# window. One table row per level.
#
#   PROFILE=release LEVELS="100 250" WINDOW=40 bash scripts/capacity.sh
#   PROFILE=release LIMIT_CPU=200% LIMIT_MEM=1G TBG_WORKERS=2 \
#     LEVELS="250 500" WINDOW=40 bash scripts/capacity.sh
#
# LIMIT_CPU is a systemd CPUQuota (100% = one CPU, 200% = two). LIMIT_MEM is
# a systemd MemoryMax (e.g. 1G). When systemd-run --user --scope is
# unavailable the script falls back to taskset (whole cores only) and
# prlimit --as for memory, which bounds virtual address space, not RSS.
# The bots always run outside the server's limit.
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
# P10 step 3 measured `opt-level = 3` (profile `server`) vs `release`
# (`opt-level = "s"`) at 1000 matches: 0.99 vs 0.97 cores - no gain, so the
# release profile serves both the WASM bundle and the server.
PROFILE="${PROFILE:-release}"
LEVELS="${LEVELS:-250 500 1000 1500}"
WINDOW="${WINDOW:-40}"
PORT="${PORT:-18181}"
ADDR="127.0.0.1:$PORT"
LIMIT_CPU="${LIMIT_CPU:-}"
LIMIT_MEM="${LIMIT_MEM:-}"
TBG_WORKERS="${TBG_WORKERS:-}"
WORK="$(mktemp -d)"
trap 'kill "${BOTS:-}" "${SRV:-}" 2>/dev/null || true; rm -rf "$WORK"' EXIT

cargo build --quiet --profile "$PROFILE" -p tbg-server --bins
BIN="$ROOT/target/$PROFILE"
ulimit -n 65536

USE_SYSTEMD=0
if [ -n "$LIMIT_CPU$LIMIT_MEM" ]; then
	if systemd-run --user --scope --quiet -p CPUQuota=1% true >/dev/null 2>&1; then
		USE_SYSTEMD=1
	fi
fi

to_bytes() {
	case "$1" in
	*[Gg]) printf '%s\n' "$((${1%[Gg]} * 1024 * 1024 * 1024))" ;;
	*[Mm]) printf '%s\n' "$((${1%[Mm]} * 1024 * 1024))" ;;
	*[Kk]) printf '%s\n' "$((${1%[Kk]} * 1024))" ;;
	*) printf '%s\n' "$1" ;;
	esac
}

# The server under the requested cgroup limit; stdout/stderr go to one log.
start_server() {
	local env_args=(
		"TBG_ADDR=$ADDR"
		"TBG_MAX_CONNECTIONS=20000"
		"TBG_MAX_PER_IP=20000"
		"TBG_DB=$WORK/db.sqlite3"
	)
	if [ -n "$TBG_WORKERS" ]; then
		env_args+=("TBG_WORKERS=$TBG_WORKERS")
	fi
	if [ "$USE_SYSTEMD" = 1 ]; then
		local scope=(systemd-run --user --scope --quiet)
		[ -n "$LIMIT_CPU" ] && scope+=(-p "CPUQuota=$LIMIT_CPU")
		[ -n "$LIMIT_MEM" ] && scope+=(-p "MemoryMax=$LIMIT_MEM")
		env "${env_args[@]}" "${scope[@]}" "$BIN/tbg-server" >"$WORK/server.log" 2>&1 &
	else
		local prefix=()
		if [ -n "$LIMIT_CPU" ]; then
			local cores=$((${LIMIT_CPU%\%} / 100))
			[ "$cores" -lt 1 ] && cores=1
			prefix+=(taskset -c "0-$((cores - 1))")
		fi
		if [ -n "$LIMIT_MEM" ]; then
			prefix+=(prlimit --as="$(to_bytes "$LIMIT_MEM")")
		fi
		env "${env_args[@]}" "${prefix[@]}" "$BIN/tbg-server" >"$WORK/server.log" 2>&1 &
	fi
	SRV=$!
}

metric() { curl -fsS "http://$ADDR/metrics" | sed -E "s/.*\"$1\":([0-9]+).*/\1/"; }
cpu_ticks() { awk '{print $14 + $15}' "/proc/$1/stat"; }
rss_mb() { awk '/VmRSS/ {printf "%.1f", $2 / 1024}' "/proc/$1/status"; }

echo "| matches | server cores | matches/core | RSS MB | KB/match | late ticks | RTT p95 ms | lobby max | conns | matches d | KB/s out | KB/s in | KB/s/player | views/s | view B avg |"
echo "|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|"
for level in $LEVELS; do
	start_server
	for _ in $(seq 1 100); do curl -fsS "http://$ADDR/health" >/dev/null 2>&1 && break; sleep 0.1; done
	if ! curl -fsS "http://$ADDR/health" >/dev/null 2>&1; then
		echo "capacity: server did not start on $ADDR" >&2
		tail -20 "$WORK/server.log" >&2 || true
		exit 1
	fi
	"$BIN/tbg-bots" --url "ws://$ADDR/ws" --matches "$level" --ramp-ms 5 >"$WORK/bots.log" 2>&1 &
	BOTS=$!
	for _ in $(seq 1 120); do
		[ "$(metric matches_running)" -ge $((level * 95 / 100)) ] && break
		sleep 0.5
	done
	sleep 3
	t0=$(cpu_ticks "$SRV")
	late0=$(metric late_ticks)
	out0=$(metric bytes_out)
	in0=$(metric bytes_in)
	conn0=$(metric connections)
	run0=$(metric matches_running)
	views0=$(metric view_messages)
	vbytes0=$(metric view_bytes)
	sleep "$WINDOW"
	t1=$(cpu_ticks "$SRV")
	late1=$(metric late_ticks)
	out1=$(metric bytes_out)
	in1=$(metric bytes_in)
	conn1=$(metric connections)
	run1=$(metric matches_running)
	views1=$(metric view_messages)
	vbytes1=$(metric view_bytes)
	rss=$(rss_mb "$SRV")
	rtt=$(metric rtt_p95_ms)
	inbox=$(metric lobby_inbox_max)
	cores=$(awk -v a="$t0" -v b="$t1" -v w="$WINDOW" 'BEGIN {printf "%.2f", (b - a) / 100 / w}')
	per_core=$(awk -v m="$run1" -v c="$cores" 'BEGIN {printf "%.0f", (c > 0 ? m / c : 0)}')
	kb=$(awk -v r="$rss" -v m="$run1" 'BEGIN {printf "%.0f", (m > 0 ? r * 1024 / m : 0)}')
	server_kb=$(awk -v a="$out0" -v b="$out1" -v w="$WINDOW" 'BEGIN {printf "%.1f", (b - a) / 1024 / w}')
	in_kb=$(awk -v a="$in0" -v b="$in1" -v w="$WINDOW" 'BEGIN {printf "%.1f", (b - a) / 1024 / w}')
	player_kb=$(awk -v s="$server_kb" -v m="$run1" 'BEGIN {printf "%.2f", (m > 0 ? s / (2 * m) : 0)}')
	views_s=$(awk -v a="$views0" -v b="$views1" -v w="$WINDOW" 'BEGIN {printf "%.0f", (b - a) / w}')
	view_avg=$(awk -v a="$views0" -v b="$views1" -v va="$vbytes0" -v vb="$vbytes1" 'BEGIN {n = b - a; printf "%.0f", (n > 0 ? (vb - va) / n : 0)}')
	echo "| $run1 | $cores | $per_core | $rss | $kb | $((late1 - late0)) | $rtt | $inbox | $conn1 | $((run1 - run0)) | $server_kb | $in_kb | $player_kb | $views_s | $view_avg |"
	# conn0 is sampled to make the connection gauge's growth explicit in the log.
	: "$conn0"
	kill "$BOTS" "$SRV" 2>/dev/null || true
	wait "$BOTS" "$SRV" 2>/dev/null || true
	BOTS="" SRV=""
done
