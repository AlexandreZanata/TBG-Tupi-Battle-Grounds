"""Bake parametric walk cycles onto TBG armatures (Blender bpy)."""

from __future__ import annotations

from typing import TYPE_CHECKING

from tools.blender.animation.walk.api import cycle_frames_for_profile, sample_pose_at_frame
from tools.blender.animation.walk.constants import WALK_FPS

if TYPE_CHECKING:
    import bpy


from tools.blender.animation.root_motion.bake import clip_name_for_root_motion


def bake_walk_action(
    arm_obj: bpy.types.Object,
    profile_id: str,
    clip_name: str = "Walk",
    *,
    root_motion: bool = False,
) -> bpy.types.Action:
    import bpy as _bpy

    if clip_name == "Walk":
        clip_name = clip_name_for_root_motion(root_motion)

    _clear_pose(arm_obj, _bpy)
    arm_obj.animation_data_create()
    action = _prepare_action(clip_name, _bpy)
    arm_obj.animation_data.action = action
    scene = _bpy.context.scene
    walk_frames = cycle_frames_for_profile(profile_id)
    scene.render.fps = WALK_FPS
    scene.frame_start = 0
    scene.frame_end = walk_frames

    for frame in range(walk_frames + 1):
        scene.frame_set(frame)
        _apply_sampled_pose(
            arm_obj,
            sample_pose_at_frame(profile_id, frame, root_motion=root_motion),
        )
        _keyframe_pose(arm_obj, _bpy)

    _set_smooth_interpolation(action)
    _mark_cyclic(action, walk_frames)
    arm_obj.animation_data.action = None
    return action


def _apply_sampled_pose(arm_obj: bpy.types.Object, table: dict) -> None:
    for bone in arm_obj.pose.bones:
        bone.rotation_mode = "XYZ"
        if bone.name in table:
            bone.rotation_euler = table[bone.name]
        else:
            bone.rotation_euler = (0.0, 0.0, 0.0)
        if bone.name == "TBG_Root":
            root = table.get("TBG_Root", (0.0, 0.0, 0.0))
            bone.location = (0.0, root[1], root[2])
        else:
            bone.location = (0.0, 0.0, 0.0)


def _clear_pose(arm_obj: bpy.types.Object, bpy_mod) -> None:
    bpy_mod.context.view_layer.objects.active = arm_obj
    bpy_mod.ops.object.mode_set(mode="POSE")
    bpy_mod.ops.pose.select_all(action="SELECT")
    bpy_mod.ops.pose.transforms_clear()
    bpy_mod.ops.object.mode_set(mode="OBJECT")


def _prepare_action(name: str, bpy_mod) -> bpy.types.Action:
    existing = bpy_mod.data.actions.get(name)
    if existing is not None:
        bpy_mod.data.actions.remove(existing)
    action = bpy_mod.data.actions.new(name)
    if hasattr(action, "use_cyclic"):
        action.use_cyclic = True
    return action


def _keyframe_pose(arm_obj: bpy.types.Object, bpy_mod) -> None:
    for bone in arm_obj.pose.bones:
        frame = bpy_mod.context.scene.frame_current
        bone.keyframe_insert(data_path="rotation_euler", frame=frame)
        bone.keyframe_insert(data_path="location", frame=frame)


def _set_smooth_interpolation(action: bpy.types.Action) -> None:
    for fcurve in _iter_action_fcurves(action):
        for kp in fcurve.keyframe_points:
            kp.interpolation = "BEZIER"
            kp.handle_left_type = "AUTO_CLAMPED"
            kp.handle_right_type = "AUTO_CLAMPED"


def _iter_action_fcurves(action: bpy.types.Action):
    legacy = getattr(action, "fcurves", None)
    if legacy is not None:
        yield from legacy
        return
    for layer in getattr(action, "layers", []):
        for strip in layer.strips:
            for bag in strip.channelbags:
                yield from bag.fcurves


def _mark_cyclic(action: bpy.types.Action, walk_frames: int) -> None:
    if hasattr(action, "frame_start"):
        action.frame_start = 0
        action.frame_end = walk_frames
