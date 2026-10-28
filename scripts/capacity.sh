#!/usr/bin/env bash
# Capacity run (P10): for each level of concurrent matches, start a fresh
# server, fill it with tbg-bots, and measure server CPU (cores), RSS, late
# ticks, and RTT over a steady window. One table row per level.
#   PROFILE=server LEVELS="250 500 1000 1500" WINDOW=40 bash scripts/capacity.sh
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
PROFILE="${PROFILE:-server}"
LEVELS="${LEVELS:-250 500 1000 1500}"
WINDOW="${WINDOW:-40}"
ADDR="127.0.0.1:8181"
WORK="$(mktemp -d)"
trap 'kill "${BOTS:-}" "${SRV:-}" 2>/dev/null || true; rm -rf "$WORK"' EXIT

cargo build --quiet --profile "$PROFILE" -p tbg-server --bins
BIN="$ROOT/target/$PROFILE"
ulimit -n 65536

metric() { curl -fsS "http://$ADDR/metrics" | sed -E "s/.*\"$1\":([0-9]+).*/\1/"; }
cpu_ticks() { awk '{print $14 + $15}' "/proc/$1/stat"; }
rss_mb() { awk '/VmRSS/ {printf "%.1f", $2 / 1024}' "/proc/$1/status"; }

echo "| matches | server cores | matches/core | RSS MB | KB/match | late ticks | RTT p95 ms |"
echo "|---------|--------------|--------------|--------|----------|------------|------------|"
for level in $LEVELS; do
	TBG_ADDR="$ADDR" TBG_MAX_CONNECTIONS=20000 TBG_MAX_PER_IP=20000 TBG_DB="$WORK/db.sqlite3" \
		"$BIN/tbg-server" >"$WORK/server.log" 2>&1 &
	SRV=$!
	for _ in $(seq 1 50); do curl -fsS "http://$ADDR/health" >/dev/null 2>&1 && break; sleep 0.1; done
	"$BIN/tbg-bots" --url "ws://$ADDR/ws" --matches "$level" --ramp-ms 5 >"$WORK/bots.log" 2>&1 &
	BOTS=$!
	for _ in $(seq 1 120); do
		[ "$(metric matches_running)" -ge $((level * 95 / 100)) ] && break
		sleep 0.5
	done
	sleep 3
	t0=$(cpu_ticks "$SRV") late0=$(metric late_ticks)
	sleep "$WINDOW"
	t1=$(cpu_ticks "$SRV") late1=$(metric late_ticks)
	running=$(metric matches_running) rss=$(rss_mb "$SRV") rtt=$(metric rtt_p95_ms)
	cores=$(awk -v a="$t0" -v b="$t1" -v w="$WINDOW" 'BEGIN {printf "%.2f", (b - a) / 100 / w}')
	per_core=$(awk -v m="$running" -v c="$cores" 'BEGIN {printf "%.0f", (c > 0 ? m / c : 0)}')
	kb=$(awk -v r="$rss" -v m="$running" 'BEGIN {printf "%.0f", (m > 0 ? r * 1024 / m : 0)}')
	echo "| $running | $cores | $per_core | $rss | $kb | $((late1 - late0)) | $rtt |"
	kill "$BOTS" "$SRV" 2>/dev/null || true
	wait "$BOTS" "$SRV" 2>/dev/null || true
	BOTS="" SRV=""
done
