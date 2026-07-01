#!/usr/bin/env bash
# Convert Mixamo FBX downloads → studio GLB previews (requires Blender).
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
BLENDER="$ROOT/tools/assembly_studio/blender/blender_cmd.sh"
IMPORT_PY="$ROOT/tools/assembly_studio/blender/import_mixamo_fbx.py"
DEST="$ROOT/content/characters/imports/mixamo"
RAW="$DEST/raw"
PREVIEW="$DEST/preview"

mkdir -p "$RAW" "$PREVIEW"

move_if_root() {
  local name="$1"
  if [[ -f "$ROOT/$name" && ! -f "$RAW/$name" ]]; then
    mv "$ROOT/$name" "$RAW/$name"
    echo "Moved $name → $RAW/"
  fi
}

move_if_root "Brute.fbx"
move_if_root "Ch45_nonPBR.fbx"
move_if_root "Defeated.fbx"

import_one() {
  local fbx="$1"
  local glb="$2"
  local label="$3"
  local with_anim="${4:-0}"
  if [[ ! -f "$fbx" ]]; then
    echo "skip: missing $fbx"
    return 0
  fi
  echo "Importing $label…"
  local anim_flag=()
  if [[ "$with_anim" == "1" ]]; then
    anim_flag=(--animations)
  fi
  "$BLENDER" --background --python "$IMPORT_PY" -- "$fbx" "$glb" "${anim_flag[@]}"
}

import_one "$RAW/Brute.fbx" "$PREVIEW/brute.glb" "Brute" 0
import_one "$RAW/Ch45_nonPBR.fbx" "$PREVIEW/ch45.glb" "Ch45" 0
import_one "$RAW/Defeated.fbx" "$PREVIEW/defeated.glb" "Defeated" 1

cat > "$DEST/manifest.json" <<EOF
{
  "characters": [
    {
      "id": "defeated",
      "label": "Defeated",
      "sourceFbx": "raw/Defeated.fbx",
      "meshUrl": "/api/content/characters/imports/mixamo/preview/defeated.glb",
      "fbxUrl": "/api/content/characters/imports/mixamo/raw/Defeated.fbx",
      "defaultClip": "Armature|mixamo.com|Layer0"
    },
    {
      "id": "brute",
      "label": "Brute",
      "sourceFbx": "raw/Brute.fbx",
      "meshUrl": "/api/content/characters/imports/mixamo/preview/brute.glb"
    },
    {
      "id": "ch45",
      "label": "Ch45 (nonPBR)",
      "sourceFbx": "raw/Ch45_nonPBR.fbx",
      "meshUrl": "/api/content/characters/imports/mixamo/preview/ch45.glb"
    }
  ]
}
EOF

echo "Done. Open http://127.0.0.1:8765/mixamo-preview.html"
