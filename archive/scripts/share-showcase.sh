#!/usr/bin/env bash
# Share the TBG character showcase carousel with other viewers.
#
# The carousel is a live web app (FastAPI + Three.js). Viewers need network access
# to this machine — it is not a single exportable HTML file (showcase GLBs ~600 MB).
#
# Usage:
#   ./scripts/share-showcase.sh              # same Wi‑Fi / LAN
#   ./scripts/share-showcase.sh tunnel       # public URL via cloudflared (internet)
#   ./scripts/share-showcase.sh local        # localhost only (your machine)
#
# Env:
#   STUDIO_PORT=8765
#   STUDIO_HOST=0.0.0.0   # forced in lan mode
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT"

MODE="${1:-lan}"
PORT="${STUDIO_PORT:-8765}"
SHOWCASE_DIR="$ROOT/content/characters/meshes/showcase"
MANIFEST="$ROOT/content/characters/fixed_test_imports.json"

die() {
  echo "share-showcase: $*" >&2
  exit 1
}

count_showcase_meshes() {
  find "$SHOWCASE_DIR" -maxdepth 1 -name 'demo_*.glb' 2>/dev/null | wc -l | tr -d ' '
}

preflight() {
  if [[ ! -d "$SHOWCASE_DIR" ]]; then
    die "missing $SHOWCASE_DIR — import sculpts first (tools/mesh-pipeline/import_showcase_sculpts.py)"
  fi
  local count
  count="$(count_showcase_meshes)"
  if [[ "$count" -lt 1 ]]; then
    die "no demo_*.glb in $SHOWCASE_DIR"
  fi
  python3 -m pip install -q -r tools/requirements.txt
  export PYTHONPATH="${ROOT}/tools:${PYTHONPATH:-}"
  echo "share-showcase: found $count showcase GLB(s) ($(du -sh "$SHOWCASE_DIR" | awk '{print $1}') total)"
}

lan_ip() {
  hostname -I 2>/dev/null | awk '{print $1}'
}

start_server() {
  local host="$1"
  echo ""
  echo "=== TBG Character Showcase ==="
  echo "Carousel:  http://${host}:${PORT}/showcase.html"
  echo "Animation: http://${host}:${PORT}/animation-studio.html"
  echo "Health:    http://${host}:${PORT}/api/health"
  echo ""
  if [[ "$host" == "0.0.0.0" ]]; then
    local ip
    ip="$(lan_ip)"
    if [[ -n "$ip" ]]; then
      echo "Share with others on your network:"
      echo "  http://${ip}:${PORT}/showcase.html"
      echo ""
      echo "If the link fails: allow TCP port ${PORT} in the firewall."
    fi
  fi
  echo "Press Ctrl+C to stop."
  echo ""
  exec python3 -m uvicorn character_studio.web_app:app \
    --app-dir "$ROOT/tools" \
    --host "$host" \
    --port "$PORT"
}

start_with_tunnel() {
  local tunnel_cmd=""
  if command -v cloudflared >/dev/null 2>&1; then
    tunnel_cmd="cloudflared tunnel --url http://127.0.0.1:${PORT}"
  elif command -v ngrok >/dev/null 2>&1; then
    tunnel_cmd="ngrok http ${PORT}"
  else
    die "tunnel mode needs cloudflared or ngrok installed.
  cloudflared: https://developers.cloudflare.com/cloudflare-one/connections/connect-networks/downloads/
  ngrok:       https://ngrok.com/download"
  fi

  python3 -m uvicorn character_studio.web_app:app \
    --app-dir "$ROOT/tools" \
    --host 127.0.0.1 \
    --port "$PORT" &
  local server_pid=$!
  trap 'kill "$server_pid" 2>/dev/null || true' EXIT INT TERM

  sleep 1
  if ! kill -0 "$server_pid" 2>/dev/null; then
    die "server failed to start on port ${PORT}"
  fi

  echo ""
  echo "=== TBG Character Showcase (internet tunnel) ==="
  echo "Local carousel: http://127.0.0.1:${PORT}/showcase.html"
  echo "Waiting for public URL from tunnel…"
  echo ""
  $tunnel_cmd
}

preflight

case "$MODE" in
  local)
    start_server "127.0.0.1"
    ;;
  lan)
    start_server "0.0.0.0"
    ;;
  tunnel|public|internet)
    start_with_tunnel
    ;;
  *)
    die "unknown mode: $MODE (use: local | lan | tunnel)"
    ;;
esac
