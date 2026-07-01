#!/usr/bin/env python3
"""Export faction PBR textures to content/characters/textures/."""

from __future__ import annotations

import argparse
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
TOOLS_DIR = Path(__file__).resolve().parent
sys.path.insert(0, str(TOOLS_DIR))

from faction_graphs import PILOT_UNITS, load_graph  # noqa: E402
from texture_generator import export_unit_textures  # noqa: E402

DEFAULT_TEXTURES_DIR = ROOT / "content" / "characters" / "textures"
DEFAULT_SIZE = 512


def parse_args(argv: list[str] | None = None) -> argparse.Namespace:
    parser = argparse.ArgumentParser(description="Export TBG character PBR textures.")
    parser.add_argument("--id", help="Unit id prefix for output files (e.g. tank_human_01)")
    parser.add_argument("--faction", choices=sorted({f for f in PILOT_UNITS.values()}))
    parser.add_argument("--all-pilots", action="store_true", help="Export all Phase 02 pilots")
    parser.add_argument("--size", type=int, default=DEFAULT_SIZE, help="Texture resolution")
    parser.add_argument(
        "--output-dir",
        type=Path,
        default=DEFAULT_TEXTURES_DIR,
        help="Destination directory for PNG files",
    )
    return parser.parse_args(argv)


def main(argv: list[str] | None = None) -> int:
    args = parse_args(argv)
    if args.all_pilots:
        _export_pilots(args.output_dir, args.size)
        return 0

    if not args.id or not args.faction:
        print("error: --id and --faction required (or use --all-pilots)", file=sys.stderr)
        return 1

    graph = load_graph(args.faction)
    paths = export_unit_textures(args.id, graph, args.output_dir, args.size)
    for path in paths:
        print(_display_path(path))
    return 0


def _export_pilots(output_dir: Path, size: int) -> None:
    for unit_id, faction in PILOT_UNITS.items():
        graph = load_graph(faction)
        paths = export_unit_textures(unit_id, graph, output_dir, size)
        for path in paths:
            print(_display_path(path))


def _display_path(path: Path) -> str:
    try:
        return str(path.relative_to(ROOT))
    except ValueError:
        return str(path)


if __name__ == "__main__":
    raise SystemExit(main())
