#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT"

python3 -m pip install -q -r tools/requirements.txt
python3 -m pytest tests/ -q

GAME_MESH_DIR="$ROOT/content/characters/meshes/game"
if compgen -G "$GAME_MESH_DIR/*.glb" > /dev/null; then
  for glb in "$GAME_MESH_DIR"/*.glb; do
    python3 tools/mesh-pipeline/validate_game_glb.py "$glb"
  done
fi

echo "verify: OK"
