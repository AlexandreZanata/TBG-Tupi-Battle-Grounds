#!/usr/bin/env bash
# Open a .blend file in interactive Blender (native or Flatpak).
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/../.." && pwd)"
BLEND="${1:?usage: open_blend.sh path/to/file.blend}"

if command -v blender >/dev/null 2>&1; then
  exec blender "$BLEND"
fi

if command -v flatpak >/dev/null 2>&1 && flatpak info org.blender.Blender >/dev/null 2>&1; then
  exec flatpak run --filesystem="$ROOT" --filesystem=home org.blender.Blender "$BLEND"
fi

echo "error: Blender not found (install native blender or Flatpak org.blender.Blender)" >&2
exit 1
