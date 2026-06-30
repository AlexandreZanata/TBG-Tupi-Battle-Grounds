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
    _create_named_actions(arm_obj, clip_names)

    output_path.parent.mkdir(parents=True, exist_ok=True)
    bpy.ops.object.select_all(action="DESELECT")
    arm_obj.select_set(True)
    meshes[0].select_set(True)
    bpy.context.view_layer.objects.active = arm_obj

    export_kwargs = {
        "filepath": str(output_path),
        "export_format": "GLB",
        "use_selection": True,
        "export_apply": True,
        "export_animations": True,
        "export_yup": True,
    }
    if hasattr(bpy.ops.export_scene.gltf, "export_animation_mode"):
        export_kwargs["export_animation_mode"] = "ACTIONS"
    bpy.ops.export_scene.gltf(**export_kwargs)
    return output_path


def _create_named_actions(arm_obj: bpy.types.Object, clip_names: list[str]) -> None:
    if arm_obj.animation_data is None:
        arm_obj.animation_data_create()
    for clip_name in clip_names:
        if clip_name in bpy.data.actions:
            bpy.data.actions.remove(bpy.data.actions[clip_name])
        action = bpy.data.actions.new(clip_name)
        arm_obj.animation_data.action = action
        arm_obj.pose.bones[0].keyframe_insert(data_path="location", frame=1)
