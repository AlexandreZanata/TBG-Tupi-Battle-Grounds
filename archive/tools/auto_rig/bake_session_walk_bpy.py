#!/usr/bin/env python3
"""Bake procedural Walk clip from a Rig Studio session armature + bones.json."""

from __future__ import annotations

import argparse
import json
import sys
from pathlib import Path

try:
    import bpy
except ImportError as exc:
    raise SystemExit("bake_session_walk_bpy requires Blender bpy") from exc

_REPO = Path(__file__).resolve().parents[3]
for path in (_REPO, Path(__file__).resolve().parent):
    if str(path) not in sys.path:
        sys.path.insert(0, str(path))

from rig_studio_bones import apply_bones_payload, read_bones_json  # noqa: E402
from tools.blender.animation.walk.bake import bake_walk_action  # noqa: E402
from tools.blender.animation.walk.export import default_output_path  # noqa: E402
from tools.blender.animation.walk.profiles import resolve_walk_profile  # noqa: E402
from tools.blender.package_path import resolve_repo_path  # noqa: E402


def _find_mesh_and_armature() -> tuple[bpy.types.Object, bpy.types.Object]:
    meshes = [obj for obj in bpy.context.scene.objects if obj.type == "MESH"]
    arms = [obj for obj in bpy.context.scene.objects if obj.type == "ARMATURE"]
    if not arms:
        raise RuntimeError("session needs an armature")
    return meshes[0] if meshes else None, arms[0]


def bake_session_walk(
    session_blend: Path,
    bones_json: Path,
    *,
    unit_id: str,
    body_class: str,
    root_motion: bool = False,
    output_path: Path | None = None,
) -> dict:
    bpy.ops.wm.open_mainfile(filepath=str(session_blend))
    mesh_obj, arm_obj = _find_mesh_and_armature()
    apply_bones_payload(arm_obj, read_bones_json(bones_json))

    profile = resolve_walk_profile(unit_id, body_class)
    bake_walk_action(arm_obj, profile, root_motion=root_motion)

    out = output_path or default_output_path(body_class, unit_id, root_motion=root_motion)
    out.parent.mkdir(parents=True, exist_ok=True)
    bpy.ops.object.select_all(action="DESELECT")
    arm_obj.select_set(True)
    if mesh_obj is not None:
        mesh_obj.select_set(True)
    bpy.context.view_layer.objects.active = arm_obj
    bpy.ops.export_scene.gltf(
        filepath=str(out),
        export_format="GLB",
        use_selection=True,
        export_apply=False,
        export_skins=mesh_obj is not None,
        export_animations=True,
        export_yup=True,
    )
    clip = "WalkRoot" if root_motion else "Walk"
    return {
        "kind": "session_walk",
        "unitId": unit_id,
        "bodyClass": body_class,
        "walkProfile": profile,
        "clip": clip,
        "output": str(out),
        "bonesJson": str(bones_json),
        "sessionBlend": str(session_blend),
    }


def _cli(argv: list[str] | None = None) -> int:
    args_list = argv if argv is not None else sys.argv
    if "--" in args_list:
        args_list = args_list[args_list.index("--") + 1 :]
    parser = argparse.ArgumentParser(description="Bake Walk from Rig Studio session.")
    parser.add_argument("--session-blend", type=Path, required=True)
    parser.add_argument("--bones-json", type=Path, required=True)
    parser.add_argument("--id", dest="unit_id", required=True)
    parser.add_argument("--body-class", default="HumanoidHeavy")
    parser.add_argument("--root-motion", action="store_true")
    parser.add_argument("--output", type=Path, default=None)
    parser.add_argument("--report-json", action="store_true")
    args = parser.parse_args(args_list)

    report = bake_session_walk(
        resolve_repo_path(args.session_blend),
        resolve_repo_path(args.bones_json),
        unit_id=args.unit_id,
        body_class=args.body_class,
        root_motion=args.root_motion,
        output_path=resolve_repo_path(args.output) if args.output else None,
    )
    if args.report_json:
        print(json.dumps(report))
    else:
        print(f"Exported {report['clip']}: {report['output']}")
    return 0


if __name__ == "__main__":
    raise SystemExit(_cli())
