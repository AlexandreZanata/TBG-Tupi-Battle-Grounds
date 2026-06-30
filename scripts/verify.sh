#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT"

python3 -m pip install -q -r tools/requirements.txt
python3 -m pytest tests/ -q

GAME_MESH_DIR="$ROOT/content/characters/meshes/game"
MAX_TOTAL_GAME_MESH_BYTES=$((80 * 1024 * 1024))
if compgen -G "$GAME_MESH_DIR/*.glb" > /dev/null; then
  TOTAL_BYTES=0
  for glb in "$GAME_MESH_DIR"/*.glb; do
    python3 tools/mesh-pipeline/validate_game_glb.py "$glb"
    SIZE=$(stat -c%s "$glb")
    TOTAL_BYTES=$((TOTAL_BYTES + SIZE))
  done
  if (( TOTAL_BYTES > MAX_TOTAL_GAME_MESH_BYTES )); then
    echo "verify: game mesh total ${TOTAL_BYTES} bytes exceeds ${MAX_TOTAL_GAME_MESH_BYTES}" >&2
    exit 1
  fi
fi

echo "verify: OK"
