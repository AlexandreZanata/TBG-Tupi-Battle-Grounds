#!/usr/bin/env python3
"""Headless game unit export orchestrator.

Usage (Blender):
  blender --background --python tools/blender/export_game_unit.py -- demo_sentinel_01

Usage (Python fallback — no Blender):
  python3 tools/blender/export_game_unit.py demo_sentinel_01
"""

from __future__ import annotations

import sys
from pathlib import Path

TOOLS_DIR = Path(__file__).resolve().parents[1]
if str(TOOLS_DIR) not in sys.path:
    sys.path.insert(0, str(TOOLS_DIR))

from mesh_pipeline.export_game_unit_core import export_game_unit  # noqa: E402
from mesh_pipeline.game_glb_validate import validate_game_glb  # noqa: E402


def _parse_unit_id(argv: list[str]) -> str:
    if "--" in argv:
        args = argv[argv.index("--") + 1 :]
        if args:
            return args[0]
    if len(argv) >= 2 and not argv[1].startswith("-"):
        return argv[1]
    raise SystemExit("usage: export_game_unit.py <unit_id>  (pilot: demo_sentinel_01)")


def _print_validation(path: Path) -> None:
    result = validate_game_glb(path)
    stats = result.stats
    print(f"Game GLB validation: {path}")
    print(f"  triangles: {stats.triangles}")
    print(f"  bones: {stats.bones}")
    print(f"  skins: {stats.skins}")
    print(f"  animations: {stats.animations}")
    print(f"  file size: {stats.file_bytes} bytes")
    for warning in result.warnings:
        print(f"  warning: {warning}")
    print("  status: PASS" if result.ok else "  status: FAIL")


def main(argv: list[str] | None = None) -> int:
    args = argv if argv is not None else sys.argv
    unit_id = _parse_unit_id(args)
    result = export_game_unit(unit_id)
    repo = Path(__file__).resolve().parents[2]
    output = repo / "content/characters/meshes/game" / f"{unit_id}.glb"
    _print_validation(output)
    return 0 if result.ok else 1


if __name__ == "__main__":
    raise SystemExit(main())
