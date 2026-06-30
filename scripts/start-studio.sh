#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT"

HOST="${STUDIO_HOST:-127.0.0.1}"
PORT="${STUDIO_PORT:-8765}"

python3 -m pip install -q -r tools/requirements.txt
echo "TBG Character Studio → http://${HOST}:${PORT}"
export PYTHONPATH="${ROOT}/tools:${PYTHONPATH:-}"
exec python3 -m uvicorn character_studio.web_app:app --app-dir "$ROOT/tools" --host "$HOST" --port "$PORT" --reload
