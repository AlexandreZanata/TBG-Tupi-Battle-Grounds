#!/usr/bin/env bash
# Create content/characters/mblab/projects/{baseId}.blend via MB-Lab (headless).
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
BLENDER="$ROOT/tools/assembly_studio/blender/blender_cmd.sh"
BASE_ID="${1:?usage: bootstrap-mblab-template.sh <male|female|small|large|all>}"

_bootstrap_one() {
  local id="$1"
  local out="$ROOT/content/characters/mblab/projects/${id}.blend"
  if [[ -f "$out" ]]; then
    echo "skip: $out exists"
    return 0
  fi
  echo "Bootstrapping $id …"
  if ! "$BLENDER" --background --python "$ROOT/tools/assembly_studio/blender/bootstrap_mblab_template.py" -- "$id"; then
    echo "error: bootstrap failed for $id" >&2
    exit 1
  fi
}

if [[ ! -f "$ROOT/third_party/MB-Lab/__init__.py" ]]; then
  "$ROOT/scripts/vendor-mblab.sh"
fi
"$ROOT/scripts/install-mblab.sh"

if [[ "$BASE_ID" == "all" ]]; then
  for id in male female small large; do
    _bootstrap_one "$id"
  done
else
  _bootstrap_one "$BASE_ID"
fi

echo "Done. Export: ./scripts/export-mblab-base.sh $BASE_ID"
