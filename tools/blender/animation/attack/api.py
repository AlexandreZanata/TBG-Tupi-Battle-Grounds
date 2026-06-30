"""Public trident thrust sampling API."""

from __future__ import annotations

from tools.blender.animation.attack.constants import impact_time_ms
from tools.blender.animation.attack.profiles import get_thrust_style
from tools.blender.animation.attack.thrust_sampler import frame_count, sample_thrust_pose_at_frame, sample_thrust_pose_at_ms
from tools.blender.animation.types import BoneEuler, BonePose

__all__ = [
    "BoneEuler",
    "BonePose",
    "frame_count_for_profile",
    "impact_time_ms_for_profile",
    "sample_pose_at_frame",
    "sample_pose_at_ms",
]


def frame_count_for_profile(profile_id: str) -> int:
    return frame_count(get_thrust_style(profile_id))


def sample_pose_at_frame(profile_id: str, frame: int) -> BonePose:
    return sample_thrust_pose_at_frame(frame, get_thrust_style(profile_id))


def sample_pose_at_ms(profile_id: str, time_ms: float) -> BonePose:
    return sample_thrust_pose_at_ms(time_ms, get_thrust_style(profile_id))


def impact_time_ms_for_profile(profile_id: str) -> int:
    _ = profile_id
    return impact_time_ms()
