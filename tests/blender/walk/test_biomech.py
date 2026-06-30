"""Contract: symmetric gait — knee flexion and pelvis bob."""

import math

from tools.blender.animation.walk import cycle_frames_for_profile, root_bob_y, sample_pose_at_frame
from tools.blender.animation.walk.constants import cycle_frames


def test_sampled_knee_has_stance_and_swing_peaks():
    frames = cycle_frames_for_profile("humanoid_light_v1")
    knees = [sample_pose_at_frame("humanoid_light_v1", f)["TBG_LowerLeg.R"][0] for f in range(frames + 1)]
    stance_max = max(knees[: max(2, frames // 6)])
    swing_max = max(knees[frames // 2 :])
    assert stance_max > 0.04
    assert swing_max > 0.35
    assert swing_max > stance_max * 1.5


def test_pelvis_vertical_double_support_is_lowest():
    frames = cycle_frames()
    low_0 = root_bob_y(0)
    low_half = root_bob_y(frames // 2)
    high_quarter = root_bob_y(frames // 4)
    assert low_0 < high_quarter
    assert low_half < high_quarter


def test_arms_swing_over_cycle():
    frames = cycle_frames_for_profile("humanoid_light_v1")
    half = frames // 2
    pose_a = sample_pose_at_frame("humanoid_light_v1", 0)
    pose_b = sample_pose_at_frame("humanoid_light_v1", half)
    assert abs(pose_a["TBG_UpperArm.L"][0] - pose_b["TBG_UpperArm.L"][0]) > 0.06
