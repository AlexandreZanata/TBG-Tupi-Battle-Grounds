"""Walk pose sampling — held-weapon in-place gait."""

from __future__ import annotations

from tools.blender.animation.walk.constants import WALK_FPS, cycle_frames
from tools.blender.animation.walk.held_arm_gait import sample_held_arm_pose
from tools.blender.animation.walk.style import WalkStyle
from tools.blender.animation.walk.types import GaitPose


def sample_walk_pose(frame: int, cycle_frame_count: int, style: WalkStyle) -> GaitPose:
    return sample_held_arm_pose(frame, cycle_frame_count, style)


def cycle_frames_for_style(style: WalkStyle) -> int:
    return cycle_frames(style.cadence_spm)


def walk_cycle_duration_s(style: WalkStyle) -> float:
    return cycle_frames_for_style(style) / WALK_FPS
