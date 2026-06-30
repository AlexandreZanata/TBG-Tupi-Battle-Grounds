#!/usr/bin/env bash
# Resolve Blender for TBG headless scripts (native binary or Flatpak).
set -euo pipefail

if command -v blender >/dev/null 2>&1; then
  exec blender "$@"
fi

if command -v flatpak >/dev/null 2>&1 && flatpak info org.blender.Blender >/dev/null 2>&1; then
  exec flatpak run --filesystem=home org.blender.Blender "$@"
fi

echo "error: Blender not found." >&2
echo "  Install: flatpak install flathub org.blender.Blender" >&2
echo "  Or add a native blender binary to PATH." >&2
exit 1
