#!/usr/bin/env bash
# Optimize sculpt GLBs for web/Unity preview.
# visual (default): Draco only — same geometry + textures as source (~79MB → ~37MB).
# preview: simplify + 1024 textures + Draco — smaller, lower fidelity.
set -euo pipefail

usage() {
  echo "Usage: $0 [--mode visual|preview] <input.glb> <output.glb> [simplify_ratio]" >&2
  exit 1
}

MODE="visual"
if [[ "${1:-}" == "--mode" ]]; then
  MODE="${2:?}"
  shift 2
fi

[[ $# -ge 2 ]] || usage

INPUT="$1"
OUTPUT="$2"
RATIO="${3:-0.012}"
TMP="$(mktemp -d)"
trap 'rm -rf "$TMP"' EXIT

if [[ "$MODE" == "visual" ]]; then
  npx --yes @gltf-transform/cli draco "$INPUT" "$OUTPUT"
elif [[ "$MODE" == "preview" ]]; then
  npx --yes @gltf-transform/cli simplify "$INPUT" "$TMP/a.glb" --ratio "$RATIO" --error 0.001
  npx --yes @gltf-transform/cli resize "$TMP/a.glb" "$TMP/b.glb" --width 1024 --height 1024
  npx --yes @gltf-transform/cli draco "$TMP/b.glb" "$OUTPUT"
else
  echo "Unknown mode: $MODE (use visual or preview)" >&2
  exit 1
fi

ls -lh "$INPUT" "$OUTPUT"
