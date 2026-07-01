#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT"

python3 -m pip install -q -r tools/requirements.txt
python3 -m pytest tests/ -q

echo "verify: OK"
