#!/usr/bin/env python3
"""Import Mixamo FBX and rename skeleton + weights to TBG_* bones."""

from __future__ import annotations

import argparse
import json
import sys
from pathlib import Path

try:
    import bpy
except ImportError as exc:
    raise SystemExit("import_mixamo_fbx requires Blender bpy") from exc

_SCRIPT_DIR = Path(__file__).resolve().parent
if str(_SCRIPT_DIR) not in sys.path:
    sys.path.insert(0, str(_SCRIPT_DIR))

from import_mixamo_ops import export_skinned_glb, import_mixamo_fbx, retarget_mixamo_to_tbg


def import_mixamo_to_tbg(fbx_path: Path, output_path: Path, *, export_animations: bool = False) -> dict:
    mesh_obj, arm_obj = import_mixamo_fbx(fbx_path)
    renamed = retarget_mixamo_to_tbg(arm_obj, mesh_obj)
    export_skinned_glb(mesh_obj, arm_obj, output_path, export_animations=export_animations)
    return {"renamed_bones": renamed, "output": str(output_path), "animations": export_animations}


def _cli(argv: list[str] | None = None) -> int:
    args_list = argv if argv is not None else sys.argv
    if "--" in args_list:
        args_list = args_list[args_list.index("--") + 1 :]
    parser = argparse.ArgumentParser(description="Mixamo FBX → TBG-named skinned GLB.")
    parser.add_argument("input_fbx", type=Path, help="Mixamo FBX with skin")
    parser.add_argument("output_glb", type=Path, help="Output GLB path")
    parser.add_argument("--animations", action="store_true", help="Bake Mixamo action clips into GLB")
    parser.add_argument("--report-json", action="store_true")
    args = parser.parse_args(args_list)
    report = import_mixamo_to_tbg(
        args.input_fbx.resolve(),
        args.output_glb.resolve(),
        export_animations=args.animations,
    )
    if args.report_json:
        print(json.dumps(report))
    else:
        print(f"Wrote {report['output']} ({len(report['renamed_bones'])} bones renamed)")
    return 0


if __name__ == "__main__":
    raise SystemExit(_cli())
