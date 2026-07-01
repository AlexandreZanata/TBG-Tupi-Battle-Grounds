#!/usr/bin/env python3
"""Embed animation-set clip names into a skinned unit GLB (Blender bpy)."""

from __future__ import annotations

import json
import sys
from pathlib import Path

_TOOLS_DIR = Path(__file__).resolve().parents[2]
if str(_TOOLS_DIR) not in sys.path:
    sys.path.insert(0, str(_TOOLS_DIR))

from animation_sets.validation import AnimationSet, validate_animation_set

try:
    import bpy
except ImportError as exc:
    raise SystemExit("retarget_clip_library requires Blender bpy") from exc


def retarget_clip_library(
    skinned_mesh_path: Path,
    animation_set_id: str,
    output_path: Path,
    repo_root: Path,
    unit_id: str | None = None,
) -> Path:
    set_path = repo_root / "content/characters/animation_sets" / f"{animation_set_id}.json"
    defn = AnimationSet.from_dict(json.loads(set_path.read_text(encoding="utf-8")))
    validate_animation_set(defn, repo_root)

    bpy.ops.wm.read_factory_settings(use_empty=True)
    bpy.ops.import_scene.gltf(filepath=str(skinned_mesh_path))

    armatures = [obj for obj in bpy.context.scene.objects if obj.type == "ARMATURE"]
    meshes = [obj for obj in bpy.context.scene.objects if obj.type == "MESH"]
    if not armatures or not meshes:
        raise RuntimeError("skinned mesh import requires armature and mesh")

    arm_obj = armatures[0]
    clip_names = list(dict.fromkeys(defn.clips.values()))
    _embed_clip_actions(arm_obj, clip_names, defn.bodyClass, unit_id, repo_root)

    output_path.parent.mkdir(parents=True, exist_ok=True)
    bpy.ops.object.select_all(action="DESELECT")
    arm_obj.select_set(True)
    meshes[0].select_set(True)
    bpy.context.view_layer.objects.active = arm_obj

    export_kwargs = {
        "filepath": str(output_path),
        "export_format": "GLB",
        "use_selection": True,
        "export_apply": False,
        "export_skins": True,
        "export_animations": True,
        "export_yup": True,
    }
    if hasattr(bpy.ops.export_scene.gltf, "export_animation_mode"):
        export_kwargs["export_animation_mode"] = "ACTIONS"
    bpy.ops.export_scene.gltf(**export_kwargs)
    return output_path


def _embed_clip_actions(
    arm_obj: bpy.types.Object,
    clip_names: list[str],
    body_class: str,
    unit_id: str | None,
    repo_root: Path,
) -> None:
    """Walk gets a keyed cycle; other clips stay rest-pose until authored."""
    from tools.blender.package_path import ensure_repo_paths

    ensure_repo_paths()
    from tools.blender.animation.attack.bake import bake_attack_action  # noqa: PLC0415
    from tools.blender.animation.attack.profiles import resolve_attack_profile  # noqa: PLC0415
    from tools.blender.animation.walk.bake import bake_walk_action  # noqa: PLC0415
    from tools.blender.animation.walk.profiles import resolve_walk_profile  # noqa: PLC0415

    walk_profile = resolve_walk_profile(unit_id, body_class)
    unit_walk = (
        repo_root / "content/characters/animations/clips/units" / unit_id / "Walk.glb"
        if unit_id
        else None
    )
    for clip_name in clip_names:
        if clip_name == "Walk":
            if unit_walk and unit_walk.is_file():
                from tools.blender.scripts.cmu_anim_ops import bake_walk_from_unit_clip_glb  # noqa: PLC0415

                try:
                    bake_walk_from_unit_clip_glb(arm_obj, unit_walk)
                    continue
                except RuntimeError as exc:
                    print(f"retarget_clip_library: unit Walk fallback — {exc}")
            bake_walk_action(arm_obj, walk_profile, clip_name)
            continue
        attack_profile = resolve_attack_profile(unit_id, body_class, clip_name)
        if clip_name in ("Attack", "Attack2") and attack_profile is not None:
            bake_attack_action(arm_obj, attack_profile, clip_name)
            continue
        _create_rest_pose_action(arm_obj, clip_name)


def _create_rest_pose_action(arm_obj: bpy.types.Object, clip_name: str) -> None:
    """Rest-pose actions with stable keyframes (no motion — avoids viewer flicker)."""
    existing = bpy.data.actions.get(clip_name)
    if existing is not None:
        bpy.data.actions.remove(existing)
    action = bpy.data.actions.new(clip_name)
    arm_obj.animation_data_create()
    arm_obj.animation_data.action = action
    for pose_bone in arm_obj.pose.bones:
        pose_bone.keyframe_insert(data_path="location", frame=1)
        pose_bone.keyframe_insert(data_path="rotation_quaternion", frame=1)
        pose_bone.keyframe_insert(data_path="scale", frame=1)
    arm_obj.animation_data.action = None
