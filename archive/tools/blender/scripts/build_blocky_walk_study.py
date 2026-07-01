#!/usr/bin/env python3
"""Build Minecraft-style block human with TBG Walk for motion study (Linux / Blender only)."""

from __future__ import annotations

import argparse
import json
import sys
from pathlib import Path

try:
    import bpy
except ImportError as exc:
    raise SystemExit("build_blocky_walk_study requires Blender bpy") from exc

_REPO = Path(__file__).resolve().parents[3]
if str(_REPO) not in sys.path:
    sys.path.insert(0, str(_REPO))

from tools.blender.package_path import REPO_ROOT, ensure_repo_paths, resolve_repo_path  # noqa: E402

ensure_repo_paths()

from tools.blender.animation.walk.bake import bake_walk_action  # noqa: E402
from tools.blender.scripts.blocky_human_ops import (  # noqa: E402
    BONE_COLORS,
    BODY_CLASS,
    create_blocky_human,
    prepare_blocky_for_gltf_export,
)

TEMPLATE = REPO_ROOT / "tools/blender/templates" / f"{BODY_CLASS}_template.blend"
DEFAULT_UNIT = "demo_blocky_walk_01"


def build_blocky_skinned_glb(output_path: Path, *, bake_walk: bool = True) -> dict:
    if not TEMPLATE.is_file():
        raise FileNotFoundError(f"missing template: {TEMPLATE} — run tools/blender/setup.sh")

    bpy.ops.wm.open_mainfile(filepath=str(TEMPLATE))
    template_body = f"{BODY_CLASS}_Body"
    old = bpy.data.objects.get(template_body)
    if old is not None:
        bpy.data.objects.remove(old, do_unlink=True)

    arm_obj = next(obj for obj in bpy.context.scene.objects if obj.type == "ARMATURE")
    mesh_obj = create_blocky_human(arm_obj)

    clip_names: list[str] = []
    if bake_walk:
        bake_walk_action(arm_obj, "humanoid_blocky_v1", "Walk")
        clip_names.append("Walk")
        _create_rest_pose_action(arm_obj, "Idle")
        clip_names.append("Idle")

    prepare_blocky_for_gltf_export(mesh_obj, arm_obj)
    output_path.parent.mkdir(parents=True, exist_ok=True)
    _export_glb(mesh_obj, arm_obj, output_path)
    return {"output": str(output_path), "parts": len(BONE_COLORS), "clips": clip_names}


def _create_rest_pose_action(arm_obj: bpy.types.Object, clip_name: str) -> None:
    action = bpy.data.actions.new(clip_name)
    arm_obj.animation_data_create()
    arm_obj.animation_data.action = action
    for pose_bone in arm_obj.pose.bones:
        pose_bone.keyframe_insert(data_path="location", frame=1)
        pose_bone.keyframe_insert(data_path="rotation_euler", frame=1)
    arm_obj.animation_data.action = None


def _export_glb(mesh_obj: bpy.types.Object, arm_obj: bpy.types.Object, output_path: Path) -> None:
    bpy.ops.object.select_all(action="DESELECT")
    mesh_obj.select_set(True)
    arm_obj.select_set(True)
    bpy.context.view_layer.objects.active = arm_obj
    kwargs = {
        "filepath": str(output_path),
        "export_format": "GLB",
        "use_selection": True,
        "export_apply": False,
        "export_skins": True,
        "export_animations": True,
        "export_yup": True,
    }
    if hasattr(bpy.ops.export_scene.gltf, "export_animation_mode"):
        kwargs["export_animation_mode"] = "ACTIONS"
    bpy.ops.export_scene.gltf(**kwargs)


def main(argv: list[str] | None = None) -> int:
    args_list = argv if argv is not None else sys.argv
    if "--" in args_list:
        args_list = args_list[args_list.index("--") + 1 :]
    parser = argparse.ArgumentParser(description="Blocky walk study character (Minecraft-style)")
    parser.add_argument("--unit-id", default=DEFAULT_UNIT)
    parser.add_argument("--output", type=Path, help="GLB path (default: game mesh for unit)")
    parser.add_argument("--showcase-only", action="store_true", help="Export T-pose mesh only")
    parser.add_argument("--report-json", action="store_true")
    args = parser.parse_args(args_list)

    if args.showcase_only:
        out = resolve_repo_path(
            args.output or f"content/characters/meshes/showcase/{args.unit_id}.glb"
        )
        report = build_blocky_skinned_glb(out, bake_walk=False)
    else:
        out = resolve_repo_path(
            args.output or f"content/characters/meshes/game/{args.unit_id}.glb"
        )
        report = build_blocky_skinned_glb(out, bake_walk=True)

    if args.report_json:
        print(json.dumps(report))
    else:
        print(f"Wrote {report['output']} ({report['parts']} cubes, clips={report['clips']})")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
