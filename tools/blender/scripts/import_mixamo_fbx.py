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

_SCRIPTS_DIR = Path(__file__).resolve().parent
_REPO_ROOT = Path(__file__).resolve().parents[3]
if str(_REPO_ROOT) not in sys.path:
    sys.path.insert(0, str(_REPO_ROOT))
if str(_SCRIPTS_DIR) not in sys.path:
    sys.path.insert(0, str(_SCRIPTS_DIR))

from tools.blender.package_path import ensure_repo_paths  # noqa: E402

ensure_repo_paths()

from import_mixamo_ops import (  # noqa: E402
    export_skinned_glb,
    import_mixamo_fbx,
    retarget_mixamo_to_tbg,
)


def import_mixamo_to_tbg(fbx_path: Path, output_path: Path) -> dict:
    mesh_obj, arm_obj = import_mixamo_fbx(fbx_path)
    renamed = retarget_mixamo_to_tbg(arm_obj, mesh_obj)
    export_skinned_glb(mesh_obj, arm_obj, output_path)
    return {"renamed_bones": renamed, "output": str(output_path)}


def _cli(argv: list[str] | None = None) -> int:
    args_list = argv if argv is not None else sys.argv
    if "--" in args_list:
        args_list = args_list[args_list.index("--") + 1 :]
    parser = argparse.ArgumentParser(description="Mixamo FBX → TBG-named skinned GLB.")
    parser.add_argument("input_fbx", type=Path, help="Mixamo FBX with skin")
    parser.add_argument("output_glb", type=Path, help="Output GLB path")
    parser.add_argument("--report-json", action="store_true")
    args = parser.parse_args(args_list)
    report = import_mixamo_to_tbg(args.input_fbx.resolve(), args.output_glb.resolve())
    if args.report_json:
        print(json.dumps(report))
    return 0


if __name__ == "__main__":
    raise SystemExit(_cli())
