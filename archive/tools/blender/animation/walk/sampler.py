"""Walk pose sampling — segment gait default; held-arm for weapon scouts."""

from __future__ import annotations

from tools.blender.animation.walk.constants import WALK_FPS, cycle_frames
from tools.blender.animation.walk.eight_phase_gait import sample_eight_phase_pose
from tools.blender.animation.walk.held_arm_gait import sample_held_arm_pose
from tools.blender.animation.walk.segment_gait import sample_segment_pose
from tools.blender.animation.walk.style import WalkStyle
from tools.blender.animation.walk.types import GaitPose


def sample_walk_pose(frame: int, cycle_frame_count: int, style: WalkStyle) -> GaitPose:
    if style.gait_sampler == "held_arm":
        return sample_held_arm_pose(frame, cycle_frame_count, style)
    if style.gait_sampler == "eight_phase":
        return sample_eight_phase_pose(frame, cycle_frame_count, style)
    return sample_segment_pose(frame, cycle_frame_count, style)


def cycle_frames_for_style(style: WalkStyle) -> int:
    return cycle_frames(style.cadence_spm)


def walk_cycle_duration_s(style: WalkStyle) -> float:
    return cycle_frames_for_style(style) / WALK_FPS
