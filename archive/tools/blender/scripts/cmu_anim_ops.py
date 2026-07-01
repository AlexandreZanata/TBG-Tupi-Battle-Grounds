"""Retarget CMU BVH walk onto TBG_* armatures (open-source, Linux)."""

from __future__ import annotations

from pathlib import Path

import bpy

from tools.blender.animation.retarget.bone_map import load_bone_map
from tools.blender.animation.retarget.curves_from_mocap import detect_walk_cycle
from tools.blender.animation.retarget.import_bvh import pose_from_cmu_armature
from tools.blender.animation.walk.bake import (
    _apply_sampled_pose,
    _keyframe_pose,
    _mark_cyclic,
    _prepare_action,
    _set_smooth_interpolation,
)
from tools.blender.animation.walk.constants import WALK_FPS
from tools.blender.scripts.mixamo_anim_ops import export_walk_clip_glb, import_skinned_glb


def import_cmu_bvh_armature(bvh_path: Path) -> bpy.types.Object:
    before = {obj.name for obj in bpy.context.scene.objects}
    bpy.ops.import_anim.bvh(
        filepath=str(bvh_path),
        rotate_mode="NATIVE",
        global_scale=0.01,
    )
    new_arms = [
        obj for obj in bpy.context.scene.objects if obj.type == "ARMATURE" and obj.name not in before
    ]
    if not new_arms:
        raise RuntimeError(f"no armature after BVH import: {bvh_path}")
    return new_arms[0]


def retarget_cmu_walk(
    source_arm: bpy.types.Object,
    target_arm: bpy.types.Object,
    bvh_path: Path,
    *,
    clip_name: str = "Walk",
    output_frames: int | None = None,
) -> bpy.types.Action:
    bone_map = load_bone_map()
    start, period = detect_walk_cycle(bvh_path, bone_map)
    walk_frames = output_frames if output_frames is not None else max(24, round(period / 4))
    scene = bpy.context.scene
    scene.render.fps = WALK_FPS
    scene.frame_start = 0
    scene.frame_end = walk_frames

    target_arm.animation_data_create()
    if target_arm.animation_data.action:
        bpy.data.actions.remove(target_arm.animation_data.action)
    action = _prepare_action(clip_name, bpy)
    target_arm.animation_data.action = action

    for out_frame in range(walk_frames + 1):
        tau = out_frame / max(walk_frames, 1)
        src_frame = start + int(round(tau * period))
        scene.frame_set(src_frame)
        source_arm.update_tag(refresh={"DATA"})
        bpy.context.view_layer.update()
        pose = pose_from_cmu_armature(source_arm, bone_map)
        _apply_sampled_pose(target_arm, pose)
        _keyframe_pose(target_arm, bpy)

    _set_smooth_interpolation(action)
    _mark_cyclic(action, walk_frames)
    target_arm.animation_data.action = None
    source_arm.hide_set(True)
    return action


def bake_walk_from_unit_clip_glb(
    arm_obj: bpy.types.Object,
    clip_path: Path,
    *,
    action_name: str = "Walk",
) -> None:
    before = {obj.name for obj in bpy.context.scene.objects}
    bpy.ops.import_scene.gltf(filepath=str(clip_path))
    src_arms = [
        obj for obj in bpy.context.scene.objects if obj.type == "ARMATURE" and obj.name not in before
    ]
    if not src_arms or not src_arms[0].animation_data or not src_arms[0].animation_data.action:
        _remove_objects(before)
        raise RuntimeError(f"no action in {clip_path}")

    src_action = src_arms[0].animation_data.action
    walk = src_action.copy()
    walk.name = action_name
    arm_obj.animation_data_create()
    arm_obj.animation_data.action = walk
    arm_obj.animation_data.action = None
    _remove_objects(before)


def import_cmu_walk_to_glb(
    unit_glb: Path,
    cmu_bvh: Path,
    output_glb: Path,
    *,
    clip_name: str = "Walk",
) -> dict:
    mesh_obj, target_arm = import_skinned_glb(unit_glb)
    source_arm = import_cmu_bvh_armature(cmu_bvh)
    action = retarget_cmu_walk(source_arm, target_arm, cmu_bvh, clip_name=clip_name)
    export_walk_clip_glb(mesh_obj, target_arm, action, output_glb)
    return {
        "clip": clip_name,
        "output": str(output_glb),
        "frames": int(action.frame_range[1]),
        "source": str(cmu_bvh),
    }


def _remove_objects(before: set[str]) -> None:
    for obj in list(bpy.context.scene.objects):
        if obj.name not in before:
            bpy.data.objects.remove(obj, do_unlink=True)
