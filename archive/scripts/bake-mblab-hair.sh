#!/usr/bin/env bash
# Bake MB-Lab-adaptive hair_mblab_01.glb from male.blend template.
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
BLEND="$ROOT/content/characters/mblab/projects/male.blend"
SCRIPT="$ROOT/tools/assembly_studio/blender/bake_mblab_hair.py"
BLENDER="$ROOT/tools/assembly_studio/blender/blender_cmd.sh"
if [[ ! -f "$BLEND" ]]; then
  echo "Missing $BLEND — run ./scripts/bootstrap-mblab-template.sh male first" >&2
  exit 1
fi
exec "$BLENDER" --background "$BLEND" --python "$SCRIPT"
