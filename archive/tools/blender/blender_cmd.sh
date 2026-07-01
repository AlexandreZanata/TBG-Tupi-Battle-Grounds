#!/usr/bin/env bash
# Resolve Blender for TBG headless scripts (native binary, Flatpak, or Docker).
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/../.." && pwd)"

_abs_python_script() {
  local args=("$@")
  local out=()
  local i=0
  while (( i < ${#args[@]} )); do
    local arg="${args[i]}"
    out+=("$arg")
    if [[ "$arg" == "--python" ]] && (( i + 1 < ${#args[@]} )); then
      local script="${args[i + 1]}"
      if [[ "$script" != /* ]]; then
        script="$ROOT/$script"
      fi
      out+=("$script")
      ((i += 2))
      continue
    fi
    ((i += 1))
  done
  printf '%s\0' "${out[@]}"
}

_run_flatpak() {
  # ponytail: mount repo — projects under /data/... are outside default home sandbox
  exec flatpak run \
    --filesystem="$ROOT" \
    --filesystem=home \
    org.blender.Blender "$@"
}

_run_docker() {
  local image="${TBG_BLENDER_DOCKER_IMAGE:?set TBG_BLENDER_DOCKER_IMAGE}"
  exec docker run --rm \
    -v "$ROOT:/workspace" \
    -w /workspace \
    "$image" \
    blender "$@"
}

mapfile -d '' _BLENDER_ARGS < <(_abs_python_script "$@")

if command -v blender >/dev/null 2>&1; then
  exec blender "${_BLENDER_ARGS[@]}"
fi

if [[ -n "${TBG_BLENDER_DOCKER_IMAGE:-}" ]] && command -v docker >/dev/null 2>&1; then
  _run_docker "${_BLENDER_ARGS[@]}"
fi

if command -v flatpak >/dev/null 2>&1 && flatpak info org.blender.Blender >/dev/null 2>&1; then
  _run_flatpak "${_BLENDER_ARGS[@]}"
fi

echo "error: Blender not found." >&2
echo "  Native: install blender and add to PATH" >&2
echo "  Flatpak: flatpak install flathub org.blender.Blender" >&2
echo "  Docker: export TBG_BLENDER_DOCKER_IMAGE=<image> (mounts repo at /workspace)" >&2
exit 1
