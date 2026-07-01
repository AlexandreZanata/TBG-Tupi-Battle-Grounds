#!/usr/bin/env bash
# Export one MB-Lab .blend template → library/bases/{id}/ (rename TBG + GLB).
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
BLENDER="$ROOT/tools/assembly_studio/blender/blender_cmd.sh"
BASE_ID="${1:?usage: export-mblab-base.sh <male|female|small|large>}"
PROJECT="$ROOT/content/characters/mblab/projects/${BASE_ID}.blend"

case "$BASE_ID" in
  male)   HEIGHT=1.8  LABEL="Male body"   CLASS=HumanoidLight ;;
  female) HEIGHT=1.65 LABEL="Female body" CLASS=HumanoidLight ;;
  small)  HEIGHT=1.2  LABEL="Small body"  CLASS=HumanoidLight ;;
  large)  HEIGHT=2.1  LABEL="Large body"  CLASS=HumanoidHeavy ;;
  *) echo "unknown base: $BASE_ID" >&2; exit 1 ;;
esac

if [[ ! -f "$PROJECT" ]]; then
  echo "missing $PROJECT — bootstrapping via MB-Lab …" >&2
  "$ROOT/scripts/bootstrap-mblab-template.sh" "$BASE_ID"
fi

echo "Exporting $BASE_ID from $PROJECT"
"$BLENDER" --background "$PROJECT" --python "$ROOT/tools/assembly_studio/blender/export_mblab_project.py" -- \
  --base-id "$BASE_ID" --height-m "$HEIGHT" --body-class "$CLASS" --label "$LABEL"

echo "Done. Verify: ./scripts/verify.sh"
