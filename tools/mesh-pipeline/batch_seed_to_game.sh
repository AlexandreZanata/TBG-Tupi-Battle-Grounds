#!/usr/bin/env bash
# Export all pilot roster demo units to content/characters/meshes/game/*.glb
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/../.." && pwd)"
cd "$ROOT"

ROSTER="$ROOT/content/characters/pilot_roster.json"
EXPORT="$ROOT/tools/blender/export_game_unit.py"

if [[ ! -f "$ROSTER" ]]; then
  echo "batch_seed_to_game: missing $ROSTER" >&2
  exit 1
fi

mapfile -t PILOT_IDS < <(python3 -c "
import json
from pathlib import Path
data = json.loads(Path('$ROSTER').read_text())
for entry in data.get('pilots', []):
    print(entry['id'])
")

FAILED=()
for unit_id in "${PILOT_IDS[@]}"; do
  echo "==> exporting $unit_id"
  if ! python3 "$EXPORT" "$unit_id"; then
    FAILED+=("$unit_id")
  fi
done

if ((${#FAILED[@]} > 0)); then
  echo "batch_seed_to_game: failed units: ${FAILED[*]}" >&2
  exit 1
fi

echo "batch_seed_to_game: OK (${#PILOT_IDS[@]} units)"
