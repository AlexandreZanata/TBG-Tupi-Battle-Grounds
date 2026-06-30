"""Bake in-place walk cycle actions onto TBG armatures (Blender bpy)."""

from __future__ import annotations

try:
    import bpy
except ImportError as exc:
    raise SystemExit("walk_cycle_builder requires Blender bpy") from exc

from walk_cycle_profiles import WALK_FRAMES, WALK_FPS, WALK_PROFILES, root_bob_y


def bake_walk_action(arm_obj: bpy.types.Object, profile_id: str, clip_name: str = "Walk") -> bpy.types.Action:
    poses = WALK_PROFILES.get(profile_id)
    if poses is None:
        raise ValueError(f"unknown walk profile: {profile_id}")

    _clear_pose(arm_obj)
    arm_obj.animation_data_create()
    action = _prepare_action(clip_name)
    arm_obj.animation_data.action = action
    scene = bpy.context.scene
    scene.render.fps = WALK_FPS
    scene.frame_start = 0
    scene.frame_end = WALK_FRAMES

    keyed_frames = sorted({0, *poses.keys(), WALK_FRAMES})
    for frame in keyed_frames:
        scene.frame_set(frame)
        _apply_frame_pose(arm_obj, poses, frame)
        _keyframe_pose(arm_obj)

    _set_linear_interpolation(action)
    _mark_cyclic(action)
    arm_obj.animation_data.action = None
    return action


def _clear_pose(arm_obj: bpy.types.Object) -> None:
    bpy.context.view_layer.objects.active = arm_obj
    bpy.ops.object.mode_set(mode="POSE")
    bpy.ops.pose.select_all(action="SELECT")
    bpy.ops.pose.transforms_clear()
    bpy.ops.object.mode_set(mode="OBJECT")


def _prepare_action(name: str) -> bpy.types.Action:
    existing = bpy.data.actions.get(name)
    if existing is not None:
        bpy.data.actions.remove(existing)
    action = bpy.data.actions.new(name)
    if hasattr(action, "use_cyclic"):
        action.use_cyclic = True
    return action


def _apply_frame_pose(arm_obj: bpy.types.Object, poses: dict, frame: int) -> None:
    key = 0 if frame == WALK_FRAMES else _nearest_pose_frame(poses, frame)
    table = poses[key]
    for bone in arm_obj.pose.bones:
        bone.rotation_mode = "XYZ"
        if bone.name in table:
            bone.rotation_euler = table[bone.name]
        else:
            bone.rotation_euler = (0.0, 0.0, 0.0)
        if bone.name == "TBG_Root":
            bone.location = (0.0, root_bob_y(frame), 0.0)
        else:
            bone.location = (0.0, 0.0, 0.0)


def _nearest_pose_frame(poses: dict, frame: int) -> int:
    return min(poses.keys(), key=lambda k: abs(k - frame))


def _keyframe_pose(arm_obj: bpy.types.Object) -> None:
    for bone in arm_obj.pose.bones:
        bone.keyframe_insert(data_path="rotation_euler", frame=bpy.context.scene.frame_current)
        bone.keyframe_insert(data_path="location", frame=bpy.context.scene.frame_current)


def _set_linear_interpolation(action: bpy.types.Action) -> None:
    for fcurve in _iter_action_fcurves(action):
        for kp in fcurve.keyframe_points:
            kp.interpolation = "LINEAR"


def _iter_action_fcurves(action: bpy.types.Action):
    legacy = getattr(action, "fcurves", None)
    if legacy is not None:
        yield from legacy
        return
    for layer in getattr(action, "layers", []):
        for strip in layer.strips:
            for bag in strip.channelbags:
                yield from bag.fcurves


def _mark_cyclic(action: bpy.types.Action) -> None:
    if hasattr(action, "frame_start"):
        action.frame_start = 0
        action.frame_end = WALK_FRAMES
