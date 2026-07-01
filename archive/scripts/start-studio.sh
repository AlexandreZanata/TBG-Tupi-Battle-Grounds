#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT"

HOST="${STUDIO_HOST:-127.0.0.1}"
PORT="${STUDIO_PORT:-8765}"
URL="http://${HOST}:${PORT}"

if command -v ss >/dev/null 2>&1 && ss -ltn 2>/dev/null | grep -q ":${PORT} "; then
  echo "TBG Character Studio already running → ${URL}"
  echo "MB-Lab Studio → ${URL}/mblab-studio.html"
  exit 0
fi

python3 -m pip install -q -r tools/requirements.txt
echo "TBG Character Studio → ${URL}"
echo "MB-Lab Studio → ${URL}/mblab-studio.html"
export PYTHONPATH="${ROOT}/tools:${PYTHONPATH:-}"
exec python3 -m uvicorn character_studio.web_app:app --app-dir "$ROOT/tools" --host "$HOST" --port "$PORT" --reload
