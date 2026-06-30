"""Public walk sampling API."""

from __future__ import annotations

from tools.blender.animation.walk.constants import WALK_FRAMES, WALK_FPS
from tools.blender.animation.walk.profiles import get_walk_style
from tools.blender.animation.walk.sampler import sample_walk_pose
from tools.blender.animation.walk.types import BoneEuler, GaitPose

__all__ = [
    "WALK_FRAMES",
    "WALK_FPS",
    "BoneEuler",
    "GaitPose",
    "sample_pose_at_frame",
    "root_bob_y",
]


def sample_pose_at_frame(profile_id: str, frame: int) -> GaitPose:
    style = get_walk_style(profile_id)
    return sample_walk_pose(frame, WALK_FRAMES, style)


def root_bob_y(frame: int) -> float:
    return sample_pose_at_frame("humanoid_light_v1", frame).get("TBG_Root", (0.0, 0.0, 0.0))[1]
