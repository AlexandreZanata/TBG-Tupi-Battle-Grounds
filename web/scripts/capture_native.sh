#!/usr/bin/env bash
# Real screenshot from an installed native Linux client, isolated from the desktop session:
# a private Xvfb display, throwaway HOME/data/runtime directories and no session D-Bus (so no
# keyring access). The bot plays a Practice match through the client's own --capture mode.
# Usage: capture_native.sh <tbg-client> <out.png> <arena slug> <pt-BR|en> <seconds> [client args...]
# Example: capture_native.sh ~/.local/bin/tupi-battle-grounds /data/shots/forest.png forest en 55 --capture_on running
# Captures are local evidence; publish only cropped derivatives (media_from_capture.sh).
set -euo pipefail
[ $# -ge 5 ] || { sed -n '2,7p' "$0"; exit 2; }
client=$(readlink -f "$1") out=$2 arena=$3 lang=$4 after=$5
shift 5
[ -x "$client" ] || { echo "not an executable client: $client" >&2; exit 2; }
# Large scratch space lives on /data when present (the root disk is small on the owner's machine).
root=$(mktemp -d "${TBG_CAPTURE_TMP:-/data}/tbg-capture.XXXXXX" 2>/dev/null || mktemp -d)
trap 'rm -rf -- "$root"' EXIT
mkdir -p "$root"/{home,data,run,config,cache,share}
chmod 700 "$root/run"
printf '{"tbg.arena":"%s","tbg.lang":"%s","tbg.stronghold":"Bastion","tbg_hints_seen":"1","practice.difficulty":"2"}\n' \
	"$arena" "$lang" >"$root/data/preferences.json"
# --server points at a closed loopback port: Practice runs locally and production is never contacted.
env -i PATH=/usr/bin:/bin HOME="$root/home" TBG_DATA_DIR="$root/data" XDG_RUNTIME_DIR="$root/run" \
	XDG_CONFIG_HOME="$root/config" XDG_CACHE_HOME="$root/cache" XDG_DATA_HOME="$root/share" \
	${VK_ICD_FILENAMES:+VK_ICD_FILENAMES=$VK_ICD_FILENAMES} RUST_LOG=warn \
	xvfb-run -a -s '-screen 0 1920x1080x24 -nolisten tcp' \
	timeout 180 "$client" --autoplay practice --capture "$(readlink -f "$(dirname "$out")")/$(basename "$out")" \
	--capture_after "$after" --capture_size 1920x1080 --lang "$lang" --server ws://127.0.0.1:9/ws "$@"
