#!/usr/bin/env bash
# One-time / CI Blender setup for TBG game mesh pipeline.
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/../.." && pwd)"
cd "$ROOT"

BLENDER_SH="$ROOT/tools/blender/blender_cmd.sh"
TEMPLATE="$ROOT/tools/blender/templates/HumanoidHeavy_template.blend"
BPY_SCRIPT="$ROOT/tools/blender/scripts/build_rig_template.py"

chmod +x "$BLENDER_SH"

echo "setup: resolving Blender..."
if ! "$BLENDER_SH" --version | head -1; then
  echo "setup: FAILED — install Blender (see tools/blender/README.md)" >&2
  exit 1
fi

if [[ ! -f "$TEMPLATE" ]]; then
  echo "setup: generating HumanoidHeavy rig template..."
  "$BLENDER_SH" --background --python "$BPY_SCRIPT"
else
  echo "setup: template exists — $TEMPLATE"
fi

echo "setup: OK"
echo "  Export pilot: python3 tools/blender/export_game_unit.py demo_sentinel_01"
