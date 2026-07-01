"""Public walk sampling API."""

from __future__ import annotations

from tools.blender.animation.walk.constants import WALK_FPS, cycle_frames
from tools.blender.animation.walk.profiles import get_walk_style
from tools.blender.animation.walk.sampler import cycle_frames_for_style, sample_walk_pose
from tools.blender.animation.walk.types import BoneEuler, GaitPose

SCOUT_PROFILE_ID = "scout_pilot_v1"

__all__ = [
    "WALK_FPS",
    "WALK_FRAMES",
    "BoneEuler",
    "GaitPose",
    "cycle_frames_for_profile",
    "sample_pose_at_frame",
    "root_bob_y",
]


def cycle_frames_for_profile(profile_id: str) -> int:
    return cycle_frames_for_style(get_walk_style(profile_id))


def sample_pose_at_frame(profile_id: str, frame: int, *, root_motion: bool = False) -> GaitPose:
    style = get_walk_style(profile_id)
    frames = cycle_frames_for_style(style)
    pose = sample_walk_pose(frame, frames, style)
    if profile_id == SCOUT_PROFILE_ID:
        from tools.blender.animation.shared.trident_guard import merge_guard_arm

        pose = merge_guard_arm(pose)
    if not root_motion:
        return pose
    from tools.blender.animation.root_motion.bake import merge_root_forward

    return merge_root_forward(pose, frame, frames, style.stride_length_m)


def root_bob_y(frame: int) -> float:
    return sample_pose_at_frame("humanoid_light_v1", frame).get("TBG_Root", (0.0, 0.0, 0.0))[1]


# Default humanoid frame count (110 spm @ 30 fps)
WALK_FRAMES = cycle_frames()
