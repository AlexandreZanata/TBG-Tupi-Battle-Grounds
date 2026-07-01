#!/usr/bin/env bash
# Install vendored MB-Lab into the user's Blender add-ons directory.
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
VERSION="1.8.1"
TAG="1_8_1"
SRC="$ROOT/third_party/MB-Lab"

echo "MB-Lab ${VERSION} (tag ${TAG}) → Blender add-ons"

if [[ ! -f "$SRC/__init__.py" ]]; then
  echo "Vendored add-on missing. Running ./scripts/vendor-mblab.sh …" >&2
  "$ROOT/scripts/vendor-mblab.sh"
fi

_resolve_addon_dir() {
  if [[ -n "${BLENDER_USER_ADDONS:-}" ]]; then
    echo "$BLENDER_USER_ADDONS"
    return 0
  fi
  local base ver
  for base in \
    "$HOME/.config/blender" \
    "$HOME/.var/app/org.blender.Blender/config/blender"; do
    [[ -d "$base" ]] || continue
    ver="$(find "$base" -maxdepth 1 -type d -name '[0-9]*' 2>/dev/null | sort -V | tail -1)"
    if [[ -n "$ver" ]]; then
      echo "$ver/scripts/addons"
      return 0
    fi
  done
  return 1
}

ADDON_DIR=""
if ADDON_DIR="$(_resolve_addon_dir)"; then
  mkdir -p "$ADDON_DIR"
else
  ADDON_DIR="$ROOT/.local/blender-addons"
  mkdir -p "$ADDON_DIR"
  echo "note: no ~/.config/blender found — using project path: $ADDON_DIR" >&2
  echo "      export BLENDER_USER_ADDONS=$ADDON_DIR for native Blender" >&2
  echo "      Docker/Flatpak: enable from $ROOT/third_party/MB-Lab in Blender UI" >&2
fi

DEST="$ADDON_DIR/MB-Lab"
rm -rf "$DEST"
cp -a "$SRC" "$DEST"

echo "Installed: $DEST"
echo "Vendored source (always current): $SRC"
echo "Enable in Blender → Edit → Preferences → Add-ons → search MB-Lab"
echo "Next: author templates per content/characters/mblab/projects/README.md"
