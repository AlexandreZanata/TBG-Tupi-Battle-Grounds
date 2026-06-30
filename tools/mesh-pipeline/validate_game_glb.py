#!/usr/bin/env python3
"""CLI: validate game-tier GLB files against mobile budgets."""

from __future__ import annotations

import argparse
import sys
from pathlib import Path

TOOLS_DIR = Path(__file__).resolve().parents[1]
if str(TOOLS_DIR) not in sys.path:
    sys.path.insert(0, str(TOOLS_DIR))

from mesh_pipeline.game_glb_validate import validate_game_glb
from mesh_pipeline.glb_inspect import inspect_glb


def _format_bytes(size: int) -> str:
    if size >= 1_048_576:
        return f"{size / 1_048_576:.2f} MB"
    if size >= 1024:
        return f"{size / 1024:.1f} KB"
    return f"{size} B"


def _print_report(path: Path, result) -> None:
    stats = result.stats
    print(f"Game GLB validation: {path}")
    print(f"  triangles: {stats.triangles}")
    print(f"  bones: {stats.bones}")
    print(f"  skins: {stats.skins}")
    print(f"  animations: {stats.animations}")
    print(f"  materials: {stats.materials}")
    print(f"  textures: {stats.textures}")
    print(f"  max texture: {stats.max_texture_size}px")
    print(f"  file size: {_format_bytes(stats.file_bytes)}")

    for warning in result.warnings:
        print(f"  warning: {warning}")
    for error in result.errors:
        print(f"  error: {error}")

    if result.ok:
        print("  status: PASS")
    else:
        print("  status: FAIL")


def main(argv: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser(description="Validate game-tier GLB mobile budgets.")
    parser.add_argument("glb", type=Path, help="Path to .glb file")
    parser.add_argument(
        "--triangles-only",
        action="store_true",
        help="Print triangle count only (for optimize_glb.sh).",
    )
    args = parser.parse_args(argv)

    if not args.glb.is_file():
        print(f"error: file not found: {args.glb}", file=sys.stderr)
        return 1

    if args.triangles_only:
        print(inspect_glb(args.glb).triangles)
        return 0

    result = validate_game_glb(args.glb)
    _print_report(args.glb, result)
    return 0 if result.ok else 1


if __name__ == "__main__":
    raise SystemExit(main())
