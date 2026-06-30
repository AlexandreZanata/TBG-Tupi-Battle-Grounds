"""Contract: Perry/Winter biomechanics — dual knee lobe, double pelvis bob, contralateral arms."""

import math

from tools.blender.animation.walk import cycle_frames_for_profile, root_bob_y, sample_pose_at_frame
from tools.blender.animation.walk.biomech_curves import KNEE_CURVE_DEG
from tools.blender.animation.walk.curves import sample_cycle_curve
from tools.blender.animation.walk.constants import cycle_frames


def test_knee_curve_has_stance_and_swing_lobes():
    stance_peak = max(sample_cycle_curve(KNEE_CURVE_DEG, t) for t in (0.05, 0.08, 0.10))
    swing_peak = max(sample_cycle_curve(KNEE_CURVE_DEG, t) for t in (0.65, 0.73, 0.80))
    assert 12.0 <= stance_peak <= 22.0
    assert 50.0 <= swing_peak <= 65.0
    assert swing_peak > stance_peak * 2.5


def test_sampled_knee_shows_dual_flexion_peaks():
    frames = cycle_frames_for_profile("humanoid_light_v1")
    knee_vals = [sample_pose_at_frame("humanoid_light_v1", f)["TBG_LowerLeg.R"][0] for f in range(frames + 1)]
    stance_max = max(knee_vals[: frames // 5])
    swing_max = max(knee_vals[frames // 2 :])
    assert stance_max > math.radians(8.0)
    assert swing_max > math.radians(40.0)
    assert swing_max > stance_max * 1.8


def test_pelvis_vertical_double_support_is_lowest():
    frames = cycle_frames()
    low_0 = root_bob_y(0)
    low_half = root_bob_y(frames // 2)
    high_quarter = root_bob_y(frames // 4)
    assert low_0 < high_quarter
    assert low_half < high_quarter


def test_arms_swing_over_cycle():
    frames = cycle_frames_for_profile("humanoid_light_v1")
    quarter = frames // 4
    pose_a = sample_pose_at_frame("humanoid_light_v1", 0)
    pose_b = sample_pose_at_frame("humanoid_light_v1", quarter)
    assert abs(pose_a["TBG_UpperArm.L"][0] - pose_b["TBG_UpperArm.L"][0]) > 0.1
