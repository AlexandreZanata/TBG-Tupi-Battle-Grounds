"""Public attack sampling API — thrust and swing archetypes."""

from __future__ import annotations

from tools.blender.animation.attack.profiles import (
    behavior_attack_hit_ms,
    get_swing_style,
    get_thrust_style,
    is_swing_profile,
)
from tools.blender.animation.attack.swing_sampler import (
    sample_swing_pose_at_frame,
    sample_swing_pose_at_ms,
    swing_frame_count,
)
from tools.blender.animation.attack.thrust_sampler import (
    frame_count as thrust_frame_count,
    sample_thrust_pose_at_frame,
    sample_thrust_pose_at_ms,
)
from tools.blender.animation.attack.timing import impact_ms
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
    if is_swing_profile(profile_id):
        return swing_frame_count(get_swing_style(profile_id))
    return thrust_frame_count(get_thrust_style(profile_id))


def sample_pose_at_frame(profile_id: str, frame: int) -> BonePose:
    if is_swing_profile(profile_id):
        return sample_swing_pose_at_frame(frame, get_swing_style(profile_id))
    return sample_thrust_pose_at_frame(frame, get_thrust_style(profile_id))


def sample_pose_at_ms(profile_id: str, time_ms: float) -> BonePose:
    if is_swing_profile(profile_id):
        return sample_swing_pose_at_ms(time_ms, get_swing_style(profile_id))
    return sample_thrust_pose_at_ms(time_ms, get_thrust_style(profile_id))


def impact_time_ms_for_profile(profile_id: str) -> int:
    behavior_hit = behavior_attack_hit_ms(profile_id)
    if behavior_hit is not None:
        return behavior_hit
    if is_swing_profile(profile_id):
        return impact_ms(get_swing_style(profile_id).timing)
    from tools.blender.animation.attack.constants import impact_time_ms

    return impact_time_ms()
