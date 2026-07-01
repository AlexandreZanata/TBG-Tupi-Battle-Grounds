#!/usr/bin/env python3
"""Import CMU walk BVH → unit Walk.glb (open-source; Linux + Blender)."""

from __future__ import annotations

import argparse
import json
import sys
from pathlib import Path

try:
    import bpy
except ImportError as exc:
    raise SystemExit("import_cmu_walk_bvh requires Blender bpy") from exc

_REPO = Path(__file__).resolve().parents[3]
if str(_REPO) not in sys.path:
    sys.path.insert(0, str(_REPO))

from tools.blender.package_path import REPO_ROOT, ensure_repo_paths, resolve_repo_path  # noqa: E402

ensure_repo_paths()

from tools.blender.scripts.cmu_anim_ops import import_cmu_walk_to_glb  # noqa: E402


def _default_output(unit_id: str) -> str:
    return f"content/characters/animations/clips/units/{unit_id}/Walk.glb"


def main(argv: list[str] | None = None) -> int:
    args_list = argv if argv is not None else sys.argv
    if "--" in args_list:
        args_list = args_list[args_list.index("--") + 1 :]
    parser = argparse.ArgumentParser(description="CMU walk BVH → unit Walk.glb")
    parser.add_argument("--unit-id", required=True, help="e.g. demo_guard_01")
    parser.add_argument("--cmu-bvh", type=Path, required=True, help="CMU walk BVH")
    parser.add_argument("--unit-glb", type=Path, help="Skinned game GLB (default: meshes/game/{unit_id}.glb)")
    parser.add_argument("--output", type=Path, help="Output clip GLB")
    parser.add_argument("--report-json", action="store_true")
    args = parser.parse_args(args_list)

    unit_glb = resolve_repo_path(args.unit_glb or f"content/characters/meshes/game/{args.unit_id}.glb")
    cmu_bvh = resolve_repo_path(args.cmu_bvh)
    output = resolve_repo_path(args.output or _default_output(args.unit_id))

    if not cmu_bvh.is_file():
        raise SystemExit(
            f"CMU BVH not found: {cmu_bvh}\n"
            "Run: python3 tools/blender/scripts/download_reference_mocap.py"
        )
    if not unit_glb.is_file():
        raise SystemExit(f"Unit GLB not found: {unit_glb} — run export_game_unit.py first")

    report = import_cmu_walk_to_glb(unit_glb, cmu_bvh, output)
    if args.report_json:
        print(json.dumps(report))
    else:
        print(f"Wrote {output}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
