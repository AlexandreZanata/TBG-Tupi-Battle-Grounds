#!/usr/bin/env python3
"""Mixamo Edit CLI — Mixamo FBX + optional custom GLB → TBG skinned GLB with clips."""

from __future__ import annotations

import argparse
import json
import sys
from pathlib import Path

try:
    import bpy
except ImportError as exc:
    raise SystemExit("mixamo_edit_build requires Blender bpy") from exc

_SCRIPT_DIR = Path(__file__).resolve().parent
if str(_SCRIPT_DIR) not in sys.path:
    sys.path.insert(0, str(_SCRIPT_DIR))

from mixamo_edit_ops import build_mixamo_edit


def _parse_vec(text: str) -> tuple[float, float, float]:
    parts = [float(x.strip()) for x in text.split(",")]
    if len(parts) != 3:
        raise ValueError("expected x,y,z")
    return parts[0], parts[1], parts[2]


def _cli(argv: list[str] | None = None) -> int:
    args_list = argv if argv is not None else sys.argv
    if "--" in args_list:
        args_list = args_list[args_list.index("--") + 1 :]
    parser = argparse.ArgumentParser(description="Mixamo FBX + optional mesh → TBG GLB with animation.")
    parser.add_argument("--mixamo-fbx", type=Path, required=True)
    parser.add_argument("--output", type=Path, required=True)
    parser.add_argument("--custom-mesh", type=Path, default=None)
    parser.add_argument("--height-scale", type=float, default=1.0)
    parser.add_argument("--rig-position", type=str, default="0,0,0")
    parser.add_argument("--rig-scale", type=float, default=1.0)
    parser.add_argument("--bone-proportions", type=str, default="")
    parser.add_argument("--manual-align", action="store_true")
    parser.add_argument("--report-json", action="store_true")
    args = parser.parse_args(args_list)

    bone_props = json.loads(args.bone_proportions) if args.bone_proportions else None
    report = build_mixamo_edit(
        args.mixamo_fbx.resolve(),
        args.output.resolve(),
        custom_mesh=args.custom_mesh.resolve() if args.custom_mesh else None,
        height_scale=args.height_scale,
        rig_position=_parse_vec(args.rig_position),
        rig_scale=args.rig_scale,
        manual_align=args.manual_align,
        bone_proportions=bone_props,
    )
    print(json.dumps(report))
    return 0


if __name__ == "__main__":
    raise SystemExit(_cli())
