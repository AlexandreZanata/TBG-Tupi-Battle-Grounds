#!/usr/bin/env python3
"""Rename TBG bones + export library base from an open MB-Lab .blend."""

from __future__ import annotations

import argparse
import json
import sys
from pathlib import Path

try:
    import bpy
except ImportError as exc:
    raise SystemExit("export_mblab_project requires Blender bpy") from exc

REPO = Path(__file__).resolve().parents[3]
sys.path.insert(0, str(Path(__file__).resolve().parent))

from export_base_body import export_base  # noqa: E402
from export_mblab_studio_preview import export_studio_preview  # noqa: E402
from rename_mblab_bones import rename_armature_bones  # noqa: E402


def _active_armature() -> bpy.types.Object:
    arm = next((o for o in bpy.context.scene.objects if o.type == "ARMATURE"), None)
    if arm is None:
        raise RuntimeError("no armature in scene — open an MB-Lab character .blend")
    return arm


def main(argv: list[str] | None = None) -> int:
    args_list = argv if argv is not None else sys.argv
    if "--" in args_list:
        args_list = args_list[args_list.index("--") + 1 :]
    parser = argparse.ArgumentParser(description="Export MB-Lab project to library base")
    parser.add_argument("--base-id", required=True, choices=["male", "female", "small", "large"])
    parser.add_argument("--height-m", type=float, required=True)
    parser.add_argument("--body-class", default="HumanoidLight")
    parser.add_argument("--label", required=True)
    args = parser.parse_args(args_list)

    renamed = rename_armature_bones(_active_armature())
    report = export_base(
        args.base_id,
        body_class=args.body_class,
        height_m=args.height_m,
        label=args.label,
    )
    report["preview"] = export_studio_preview(args.base_id)
    report["renamed_bones"] = len(renamed)
    print(json.dumps(report))
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
