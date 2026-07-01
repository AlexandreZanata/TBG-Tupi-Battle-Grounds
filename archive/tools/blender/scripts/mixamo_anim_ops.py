"""Retarget Mixamo mocap FBX onto an existing TBG_* armature."""

from __future__ import annotations

from pathlib import Path

import bpy

from tools.blender.animation.retarget.mixamo_bone_map import load_mixamo_bone_map, tbg_bone_for_mixamo
from tools.blender.scripts.import_mixamo_ops import clear_scene


def import_skinned_glb(glb_path: Path) -> tuple[bpy.types.Object, bpy.types.Object]:
    clear_scene()
    bpy.ops.import_scene.gltf(filepath=str(glb_path))
    meshes = [obj for obj in bpy.context.scene.objects if obj.type == "MESH"]
    armatures = [obj for obj in bpy.context.scene.objects if obj.type == "ARMATURE"]
    if not armatures or not meshes:
        raise RuntimeError(f"expected skinned GLB: {glb_path}")
    return meshes[0], armatures[0]


def import_mixamo_anim_armature(fbx_path: Path) -> bpy.types.Object:
    before = {obj.name for obj in bpy.context.scene.objects}
    bpy.ops.import_scene.fbx(filepath=str(fbx_path))
    new_arms = [obj for obj in bpy.context.scene.objects if obj.type == "ARMATURE" and obj.name not in before]
    if not new_arms:
        raise RuntimeError(f"no armature in Mixamo anim FBX: {fbx_path}")
    return new_arms[0]


def retarget_mixamo_walk(source_arm: bpy.types.Object, target_arm: bpy.types.Object, clip_name: str = "Walk") -> bpy.types.Action:
    bone_map = load_mixamo_bone_map()
    pairs = _build_retarget_pairs(source_arm, target_arm, bone_map)
    if len(pairs) < 8:
        raise RuntimeError(f"too few mapped bones ({len(pairs)}) — rig Mixamo character first")

    source_action = source_arm.animation_data.action if source_arm.animation_data else None
    if source_action is None:
        raise RuntimeError("Mixamo FBX has no action — download With Skin off, keyframes on")

    frame_start = int(source_action.frame_range[0])
    frame_end = int(source_action.frame_range[1])
    scene = bpy.context.scene
    scene.frame_start = frame_start
    scene.frame_end = frame_end

    target_arm.animation_data_create()
    if target_arm.animation_data.action:
        bpy.data.actions.remove(target_arm.animation_data.action)
    action = bpy.data.actions.new(clip_name)
    target_arm.animation_data.action = action

    for frame in range(frame_start, frame_end + 1):
        scene.frame_set(frame)
        source_arm.update_tag(refresh={"DATA"})
        bpy.context.view_layer.update()
        for src_name, tgt_name in pairs:
            src = source_arm.pose.bones.get(src_name)
            tgt = target_arm.pose.bones.get(tgt_name)
            if not src or not tgt:
                continue
            tgt.rotation_mode = src.rotation_mode
            if src.rotation_mode == "QUATERNION":
                tgt.rotation_quaternion = src.rotation_quaternion.copy()
                tgt.keyframe_insert(data_path="rotation_quaternion", frame=frame)
            else:
                tgt.rotation_euler = src.rotation_euler.copy()
                tgt.keyframe_insert(data_path="rotation_euler", frame=frame)
            if src.location.length_squared > 1e-8 and tgt_name == "TBG_Root":
                tgt.location = src.location.copy()
                tgt.keyframe_insert(data_path="location", frame=frame)

    target_arm.animation_data.action = None
    source_arm.hide_set(True)
    source_arm.hide_viewport = True
    return action


def export_walk_clip_glb(mesh_obj: bpy.types.Object, arm_obj: bpy.types.Object, action: bpy.types.Action, output: Path) -> Path:
    arm_obj.animation_data_create()
    arm_obj.animation_data.action = action
    output.parent.mkdir(parents=True, exist_ok=True)
    bpy.ops.object.select_all(action="DESELECT")
    mesh_obj.select_set(True)
    arm_obj.select_set(True)
    bpy.context.view_layer.objects.active = arm_obj
    kwargs = {
        "filepath": str(output),
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
    arm_obj.animation_data.action = None
    return output


def _build_retarget_pairs(
    source_arm: bpy.types.Object,
    target_arm: bpy.types.Object,
    bone_map: dict,
) -> list[tuple[str, str]]:
    pairs: list[tuple[str, str]] = []
    for bone in source_arm.data.bones:
        tbg = tbg_bone_for_mixamo(bone_map, bone.name)
        if tbg and tbg in target_arm.data.bones:
            pairs.append((bone.name, tbg))
    return pairs
