#!/usr/bin/env bash
# Shared art pipeline; optional local tools only, no Cargo dependencies.
# uv venv --system-site-packages .local/art-tools/venv
# uv pip install --python .local/art-tools/venv/bin/python vtracer==0.6.15 pillow numpy scipy cairosvg
# ART_SHARP may select a local sharp module when CairoSVG is unavailable.
# Usage: scripts/arcana-art.sh [arcana|augments|strongholds] [asset ...] [--workers 3]
# Normalize generated originals: scripts/arcana-art.sh --normalize
set -euo pipefail
cd "$(dirname "$0")/.."
if [[ $# -gt 0 && "$1" != "arcana" && "$1" != "augments" && "$1" != "strongholds" && "$1" != -* ]]; then
  set -- arcana "$@"
fi
export PYTHONDONTWRITEBYTECODE=1
exec "${ART_PYTHON:-${ARCANA_PYTHON:-.local/art-tools/venv/bin/python}}" scripts/art_pipeline.py "$@"
