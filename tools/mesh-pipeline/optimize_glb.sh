#!/usr/bin/env bash
# Optimize sculpt GLBs for web/Unity preview and mobile game meshes.
# visual (default): Draco only — same geometry + textures as source.
# preview: simplify + 1024 textures + Draco — smaller showcase preview.
# game: simplify to tri budget + resize textures + Draco — mobile runtime mesh.
set -euo pipefail

usage() {
  echo "Usage: $0 [--mode visual|preview|game] [--tri N] [--tex N] <input.glb> <output.glb> [simplify_ratio]" >&2
  exit 1
}

MODE="visual"
TRI_TARGET=8000
TEX_SIZE=1024

while [[ $# -gt 0 && "${1:-}" == --* ]]; do
  case "$1" in
    --mode)
      MODE="${2:?}"
      shift 2
      ;;
    --tri)
      TRI_TARGET="${2:?}"
      shift 2
      ;;
    --tex)
      TEX_SIZE="${2:?}"
      shift 2
      ;;
    *)
      usage
      ;;
  esac
done

[[ $# -ge 2 ]] || usage

INPUT="$1"
OUTPUT="$2"
RATIO="${3:-0.012}"
TMP="$(mktemp -d)"
trap 'rm -rf "$TMP"' EXIT

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"

if [[ "$MODE" == "visual" ]]; then
  npx --yes @gltf-transform/cli draco "$INPUT" "$OUTPUT"
elif [[ "$MODE" == "preview" ]]; then
  npx --yes @gltf-transform/cli simplify "$INPUT" "$TMP/a.glb" --ratio "$RATIO" --error 0.001
  npx --yes @gltf-transform/cli resize "$TMP/a.glb" "$TMP/b.glb" --width 1024 --height 1024
  npx --yes @gltf-transform/cli draco "$TMP/b.glb" "$OUTPUT"
elif [[ "$MODE" == "game" ]]; then
  CURRENT_TRIS="$(python3 "$SCRIPT_DIR/validate_game_glb.py" "$INPUT" --triangles-only)"
  RATIO="$(python3 -c "target=${TRI_TARGET}; current=max(1, int('${CURRENT_TRIS}')); print(min(1.0, target / current * 0.95))")"
  npx --yes @gltf-transform/cli simplify "$INPUT" "$TMP/a.glb" --ratio "$RATIO" --error 0.001
  npx --yes @gltf-transform/cli resize "$TMP/a.glb" "$TMP/b.glb" --width "$TEX_SIZE" --height "$TEX_SIZE"
  npx --yes @gltf-transform/cli draco "$TMP/b.glb" "$OUTPUT"
else
  echo "Unknown mode: $MODE (use visual, preview, or game)" >&2
  exit 1
fi

ls -lh "$INPUT" "$OUTPUT"
