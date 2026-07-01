#!/usr/bin/env python3
"""Import Mixamo walk mocap FBX → unit Walk.glb (dev-only; replaces procedural bake)."""

from __future__ import annotations

import argparse
import json
import sys
from pathlib import Path

try:
    import bpy
except ImportError as exc:
    raise SystemExit("import_mixamo_walk_fbx requires Blender bpy") from exc

_REPO = Path(__file__).resolve().parents[3]
if str(_REPO) not in sys.path:
    sys.path.insert(0, str(_REPO))

from tools.blender.package_path import REPO_ROOT, ensure_repo_paths, resolve_repo_path  # noqa: E402

ensure_repo_paths()

from tools.blender.scripts.mixamo_anim_ops import (  # noqa: E402
    export_walk_clip_glb,
    import_mixamo_anim_armature,
    import_skinned_glb,
    retarget_mixamo_walk,
)


def import_mixamo_walk(
    unit_glb: Path,
    mixamo_fbx: Path,
    output_glb: Path,
    *,
    clip_name: str = "Walk",
) -> dict:
    mesh_obj, target_arm = import_skinned_glb(unit_glb)
    source_arm = import_mixamo_anim_armature(mixamo_fbx)
    action = retarget_mixamo_walk(source_arm, target_arm, clip_name)
    export_walk_clip_glb(mesh_obj, target_arm, action, output_glb)
    return {"clip": clip_name, "output": str(output_glb), "frames": int(action.frame_range[1])}


def _default_output(unit_id: str) -> str:
    return f"content/characters/animations/clips/units/{unit_id}/Walk.glb"


def main(argv: list[str] | None = None) -> int:
    args_list = argv if argv is not None else sys.argv
    if "--" in args_list:
        args_list = args_list[args_list.index("--") + 1 :]
    parser = argparse.ArgumentParser(description="Mixamo walk FBX → unit Walk.glb")
    parser.add_argument("--unit-id", required=True, help="e.g. demo_guard_01")
    parser.add_argument("--mixamo-fbx", type=Path, required=True, help="Mixamo Walking FBX")
    parser.add_argument("--unit-glb", type=Path, help="Skinned game GLB (default: meshes/game/{unit_id}.glb)")
    parser.add_argument("--output", type=Path, help="Output clip GLB")
    parser.add_argument("--report-json", action="store_true")
    args = parser.parse_args(args_list)

    unit_glb = resolve_repo_path(args.unit_glb or f"content/characters/meshes/game/{args.unit_id}.glb")
    mixamo_fbx = resolve_repo_path(args.mixamo_fbx)
    output = resolve_repo_path(args.output or _default_output(args.unit_id))

    if not mixamo_fbx.is_file():
        raise SystemExit(
            f"Mixamo FBX not found: {mixamo_fbx}\n"
            "Download Walking from mixamo.com (FBX, without skin) and place it at:\n"
            f"  {REPO_ROOT / '.local/tmp/mixamo_walk.fbx'}"
        )
    if not unit_glb.is_file():
        raise SystemExit(f"Unit GLB not found: {unit_glb} — run export_game_unit.py first")

    report = import_mixamo_walk(unit_glb, mixamo_fbx, output)
    if args.report_json:
        print(json.dumps(report))
    else:
        print(f"Wrote {output}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
