#!/usr/bin/env bash
# Export all pilot roster demo units to content/characters/meshes/game/*.glb
# One Blender session + parallel post-process (see batch_export_game_units.py).
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/../.." && pwd)"
cd "$ROOT"

ROSTER="$ROOT/content/characters/pilot_roster.json"
BATCH="$ROOT/tools/blender/batch_export_game_units.py"

if [[ ! -f "$ROSTER" ]]; then
  echo "batch_seed_to_game: missing $ROSTER" >&2
  exit 1
fi

# Optional: TBG_BATCH_WORKERS=4 (default) for parallel gltf-transform post-process
# Weight gate: TBG_WEIGHT_GATE=1 (default here) — summary table printed at end
export TBG_WEIGHT_GATE="${TBG_WEIGHT_GATE:-1}"
python3 "$BATCH" "$@"
